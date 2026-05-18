use anyhow::{Context, Result, bail};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct ImageMeta {
    pub image_id: String,
    pub filename: String,
    pub url: String,
    pub relative_path: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub alt: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub image_id: String,
    pub filename: String,
    pub relative_path: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub alt: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ImageIndex {
    #[serde(default)]
    images: BTreeMap<String, ImageRecord>,
}

#[derive(Debug, Clone, Default)]
pub struct ImageMetaPatch {
    pub alt: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct PublicImage {
    pub record: ImageRecord,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum ImageHostError {
    InvalidRequest(String),
    Unauthorized,
    NotFound,
    Conflict(String),
    PayloadTooLarge,
    UnsupportedMediaType(String),
    Internal(String),
}

impl ImageHostError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Unauthorized => "unauthorized",
            Self::NotFound => "image_not_found",
            Self::Conflict(_) => "image_conflict",
            Self::PayloadTooLarge => "payload_too_large",
            Self::UnsupportedMediaType(_) => "unsupported_media_type",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidRequest(message)
            | Self::Conflict(message)
            | Self::UnsupportedMediaType(message)
            | Self::Internal(message) => message.clone(),
            Self::Unauthorized => "unauthorized".to_string(),
            Self::NotFound => "image not found".to_string(),
            Self::PayloadTooLarge => "payload too large".to_string(),
        }
    }
}

impl From<anyhow::Error> for ImageHostError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value.to_string())
    }
}

pub struct ImageStore {
    base_dir: PathBuf,
    index: RwLock<ImageIndex>,
}

impl ImageStore {
    pub fn load_or_init(base_dir: impl Into<PathBuf>) -> Result<Self> {
        let base_dir = base_dir.into();
        fs::create_dir_all(base_dir.join("files"))
            .with_context(|| format!("create image dir {:?}", base_dir))?;
        let index_path = base_dir.join("index.json");
        let index = if index_path.exists() {
            let raw = fs::read_to_string(&index_path)
                .with_context(|| format!("read image index {:?}", index_path))?;
            serde_json::from_str::<ImageIndex>(&raw).context("parse image index.json")?
        } else {
            let index = ImageIndex::default();
            let bytes = serde_json::to_vec_pretty(&index).context("serialize image index")?;
            atomic_write(&index_path, &bytes).context("write initial image index")?;
            index
        };

        Ok(Self {
            base_dir,
            index: RwLock::new(index),
        })
    }

    pub async fn create_image(
        &self,
        bytes: Vec<u8>,
        alt: String,
        description: String,
        max_upload_bytes: usize,
    ) -> Result<ImageRecord, ImageHostError> {
        validate_text(&alt, &description)?;
        let info = validate_image_bytes(&bytes, max_upload_bytes)?;
        let now = now_unix_seconds()?;

        let mut guard = self.index.write().await;
        let mut next = guard.clone();
        let image_id = generate_unique_image_id(&next)?;
        let filename = format!("{image_id}.{}", info.extension);
        let relative_path = build_relative_path(now, &filename);
        let target_path = self.record_path_from_relative(&relative_path);
        write_image_file(&target_path, &bytes)?;

        let record = ImageRecord {
            image_id: image_id.clone(),
            filename,
            relative_path,
            content_type: info.content_type,
            size_bytes: info.size_bytes,
            width: info.width,
            height: info.height,
            sha256: info.sha256,
            alt,
            description,
            created_at: now,
            updated_at: now,
        };
        next.images.insert(image_id, record.clone());
        if let Err(err) = self.save_index(&next) {
            let _ = fs::remove_file(&target_path);
            return Err(err.into());
        }
        *guard = next;
        Ok(record)
    }

    pub async fn list_images(
        &self,
        limit: usize,
        offset: usize,
        q: Option<&str>,
    ) -> Result<(Vec<ImageRecord>, usize), ImageHostError> {
        if !(1..=100).contains(&limit) {
            return Err(ImageHostError::InvalidRequest(
                "limit must be between 1 and 100".to_string(),
            ));
        }
        if q.is_some_and(|value| value.chars().count() > 100) {
            return Err(ImageHostError::InvalidRequest(
                "q must be at most 100 characters".to_string(),
            ));
        }

        let query = q.map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty());
        let guard = self.index.read().await;
        let mut items: Vec<ImageRecord> = guard.images.values().cloned().collect();
        if let Some(query) = query {
            items.retain(|item| {
                item.image_id.to_ascii_lowercase().contains(&query)
                    || item.filename.to_ascii_lowercase().contains(&query)
                    || item.alt.to_ascii_lowercase().contains(&query)
                    || item.description.to_ascii_lowercase().contains(&query)
            });
        }
        items.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.image_id.cmp(&left.image_id))
        });
        let total = items.len();
        let paged = items.into_iter().skip(offset).take(limit).collect();
        Ok((paged, total))
    }

    pub async fn get_image(&self, image_id: &str) -> Result<ImageRecord, ImageHostError> {
        let guard = self.index.read().await;
        let record = guard
            .images
            .get(image_id)
            .cloned()
            .ok_or(ImageHostError::NotFound)?;
        let path = self.record_path(&record);
        if !path.exists() {
            return Err(ImageHostError::Conflict(format!(
                "image file missing for {image_id}"
            )));
        }
        Ok(record)
    }

    pub async fn get_public_image(
        &self,
        image_id: &str,
        filename: &str,
    ) -> Result<PublicImage, ImageHostError> {
        let guard = self.index.read().await;
        let record = guard
            .images
            .get(image_id)
            .cloned()
            .ok_or(ImageHostError::NotFound)?;
        if record.filename != filename {
            return Err(ImageHostError::NotFound);
        }
        let path = self.record_path(&record);
        if !path.exists() {
            return Err(ImageHostError::NotFound);
        }
        Ok(PublicImage { record, path })
    }

    pub async fn update_image_meta(
        &self,
        image_id: &str,
        patch: ImageMetaPatch,
    ) -> Result<ImageRecord, ImageHostError> {
        if patch.alt.is_none() && patch.description.is_none() {
            return Err(ImageHostError::InvalidRequest(
                "patch must include alt or description".to_string(),
            ));
        }
        let alt_for_check = patch.alt.clone().unwrap_or_default();
        let description_for_check = patch.description.clone().unwrap_or_default();
        validate_text(&alt_for_check, &description_for_check)?;

        let mut guard = self.index.write().await;
        let mut next = guard.clone();
        let record = next
            .images
            .get_mut(image_id)
            .ok_or(ImageHostError::NotFound)?;
        if let Some(alt) = patch.alt {
            record.alt = alt;
        }
        if let Some(description) = patch.description {
            record.description = description;
        }
        record.updated_at = now_unix_seconds()?;
        let updated = record.clone();
        self.save_index(&next)?;
        *guard = next;
        Ok(updated)
    }

    pub async fn replace_image(
        &self,
        image_id: &str,
        bytes: Vec<u8>,
        patch: ImageMetaPatch,
        max_upload_bytes: usize,
    ) -> Result<ImageRecord, ImageHostError> {
        let alt_for_check = patch.alt.clone().unwrap_or_default();
        let description_for_check = patch.description.clone().unwrap_or_default();
        validate_text(&alt_for_check, &description_for_check)?;
        let info = validate_image_bytes(&bytes, max_upload_bytes)?;
        let now = now_unix_seconds()?;

        let mut guard = self.index.write().await;
        let mut next = guard.clone();
        let old_record = next
            .images
            .get(image_id)
            .cloned()
            .ok_or(ImageHostError::NotFound)?;
        let old_path = self.record_path(&old_record);
        if !old_path.exists() {
            return Err(ImageHostError::Conflict(format!(
                "image file missing for {image_id}"
            )));
        }

        let filename = format!("{image_id}_{now}.{}", info.extension);
        let relative_path = build_relative_path(now, &filename);
        let target_path = self.record_path_from_relative(&relative_path);
        write_image_file(&target_path, &bytes)?;

        let mut updated = old_record.clone();
        updated.filename = filename;
        updated.relative_path = relative_path;
        updated.content_type = info.content_type;
        updated.size_bytes = info.size_bytes;
        updated.width = info.width;
        updated.height = info.height;
        updated.sha256 = info.sha256;
        updated.updated_at = now;
        if let Some(alt) = patch.alt {
            updated.alt = alt;
        }
        if let Some(description) = patch.description {
            updated.description = description;
        }
        next.images.insert(image_id.to_string(), updated.clone());
        if let Err(err) = self.save_index(&next) {
            let _ = fs::remove_file(&target_path);
            return Err(err.into());
        }
        *guard = next;
        if let Err(err) = fs::remove_file(&old_path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                eprintln!("[solin-blog] remove old image failed: {err}");
            }
        }
        Ok(updated)
    }

    pub async fn delete_image(&self, image_id: &str) -> Result<(), ImageHostError> {
        let mut guard = self.index.write().await;
        let mut next = guard.clone();
        let record = next.images.remove(image_id).ok_or(ImageHostError::NotFound)?;
        self.save_index(&next)?;
        *guard = next;
        let path = self.record_path(&record);
        if let Err(err) = fs::remove_file(&path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                eprintln!("[solin-blog] remove image file failed: {err}");
            }
        }
        Ok(())
    }

    fn save_index(&self, index: &ImageIndex) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(index).context("serialize image index")?;
        atomic_write(&self.index_path(), &bytes).context("write image index")?;
        Ok(())
    }

    fn index_path(&self) -> PathBuf {
        self.base_dir.join("index.json")
    }

    fn record_path(&self, record: &ImageRecord) -> PathBuf {
        self.record_path_from_relative(&record.relative_path)
    }

    fn record_path_from_relative(&self, relative_path: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        for segment in relative_path.split('/') {
            path.push(segment);
        }
        path
    }
}

impl ImageMeta {
    pub fn from_record(record: ImageRecord, base_url: &str) -> Self {
        let path = format!("/images/{}/{}", record.image_id, record.filename);
        let url = if base_url.trim().is_empty() {
            path
        } else {
            format!("{}{}", base_url.trim_end_matches('/'), path)
        };
        Self {
            image_id: record.image_id,
            filename: record.filename,
            url,
            relative_path: record.relative_path,
            content_type: record.content_type,
            size_bytes: record.size_bytes,
            width: record.width,
            height: record.height,
            sha256: record.sha256,
            alt: record.alt,
            description: record.description,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

struct ImageFileInfo {
    extension: &'static str,
    content_type: String,
    size_bytes: u64,
    width: u32,
    height: u32,
    sha256: String,
}

fn validate_image_bytes(bytes: &[u8], max_upload_bytes: usize) -> Result<ImageFileInfo, ImageHostError> {
    if bytes.is_empty() {
        return Err(ImageHostError::InvalidRequest("file is required".to_string()));
    }
    if bytes.len() > max_upload_bytes {
        return Err(ImageHostError::PayloadTooLarge);
    }
    let format = ::image::guess_format(bytes).map_err(|_| {
        ImageHostError::UnsupportedMediaType("unsupported or invalid image".to_string())
    })?;
    let (extension, content_type) = match format {
        ::image::ImageFormat::Png => ("png", "image/png"),
        ::image::ImageFormat::Jpeg => ("jpg", "image/jpeg"),
        ::image::ImageFormat::WebP => ("webp", "image/webp"),
        ::image::ImageFormat::Gif => ("gif", "image/gif"),
        ::image::ImageFormat::Bmp => ("bmp", "image/bmp"),
        _ => {
            return Err(ImageHostError::UnsupportedMediaType(
                "only PNG, JPEG, WEBP, GIF and BMP are supported".to_string(),
            ));
        }
    };
    let decoded = ::image::load_from_memory_with_format(bytes, format).map_err(|_| {
        ImageHostError::UnsupportedMediaType("image decode failed".to_string())
    })?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let sha256 = format!("{:x}", hasher.finalize());
    Ok(ImageFileInfo {
        extension,
        content_type: content_type.to_string(),
        size_bytes: bytes.len() as u64,
        width: decoded.width(),
        height: decoded.height(),
        sha256,
    })
}

fn validate_text(alt: &str, description: &str) -> Result<(), ImageHostError> {
    if alt.chars().count() > 200 {
        return Err(ImageHostError::InvalidRequest(
            "alt must be at most 200 characters".to_string(),
        ));
    }
    if description.chars().count() > 1000 {
        return Err(ImageHostError::InvalidRequest(
            "description must be at most 1000 characters".to_string(),
        ));
    }
    Ok(())
}

fn generate_unique_image_id(index: &ImageIndex) -> Result<String> {
    for _ in 0..16 {
        let image_id = generate_image_id()?;
        if !index.images.contains_key(&image_id) {
            return Ok(image_id);
        }
    }
    bail!("failed to generate unique image id")
}

fn generate_image_id() -> Result<String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut bytes = [0u8; 16];
    getrandom(&mut bytes).map_err(|err| anyhow::anyhow!("getrandom image id failed: {err}"))?;
    let mut out = String::from("img_");
    for byte in bytes {
        out.push(ALPHABET[(byte as usize) % ALPHABET.len()] as char);
    }
    Ok(out)
}

fn build_relative_path(timestamp: i64, filename: &str) -> String {
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::UNIX_EPOCH);
    format!("files/{}/{:02}/{}", datetime.format("%Y"), datetime.month(), filename)
}

fn write_image_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create image file dir {:?}", parent))?;
    }
    let tmp_path = path.with_extension("uploading");
    fs::write(&tmp_path, bytes).with_context(|| format!("write temp image {:?}", tmp_path))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("move image {:?} -> {:?}", tmp_path, path))?;
    Ok(())
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create parent dir {:?}", parent))?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, data).with_context(|| format!("write temp file {:?}", tmp_path))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("rename temp file {:?} -> {:?}", tmp_path, path))?;
    Ok(())
}

fn now_unix_seconds() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?;
    Ok(duration.as_secs().min(i64::MAX as u64) as i64)
}

trait DateTimeMonth {
    fn month(&self) -> u32;
}

impl DateTimeMonth for chrono::DateTime<chrono::Utc> {
    fn month(&self) -> u32 {
        use chrono::Datelike;
        Datelike::month(self)
    }
}
