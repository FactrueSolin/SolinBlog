use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use getrandom::getrandom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoMeta {
    #[serde(alias = "title")]
    pub seo_title: String,
    pub description: String,
    pub keywords: Option<Vec<String>>,
    #[serde(default)]
    pub extra: Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub seo: SeoMeta,
    #[serde(default)]
    pub page_uid: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub extra: Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoreIndex {
    #[serde(default)]
    pub pages: BTreeMap<String, PageIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIndexEntry {
    pub page_id: String,
    pub seo: SeoMeta,
    #[serde(default)]
    pub page_uid: String,
    pub original_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PageStore {
    pub base_dir: PathBuf,
}

impl Default for PageStore {
    fn default() -> Self {
        Self::new("data")
    }
}

impl PageStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn create_page_auto_uid(&self, meta: &PageMeta, html: &str) -> Result<PageMeta> {
        let index = self.load_index()?;
        let uid = generate_unique_page_uid(&index)?;
        let mut meta_with_uid = meta.clone();
        meta_with_uid.page_uid = uid.clone();
        self.create_page(&uid, &meta_with_uid, html)?;
        let (saved_meta, _) = self.load_page(&uid)?;
        Ok(saved_meta)
    }

    pub fn resolve_page_id_by_uid(&self, page_uid: &str) -> Result<Option<String>> {
        let index = self.load_index()?;
        if index.pages.contains_key(page_uid) {
            return Ok(Some(page_uid.to_string()));
        }
        let matched = index
            .pages
            .iter()
            .find_map(|(page_id, entry)| {
                if entry.page_uid == page_uid {
                    Some(page_id.clone())
                } else {
                    None
                }
            });
        Ok(matched)
    }

    pub fn create_page(&self, page_id: &str, meta: &PageMeta, html: &str) -> Result<()> {
        if self.page_exists(page_id)? {
            bail!("page already exists: {}", page_id);
        }
        self.save_page(page_id, meta, html)
    }

    pub fn save_page(&self, page_id: &str, meta: &PageMeta, html: &str) -> Result<()> {
        fs::create_dir_all(&self.base_dir)
            .with_context(|| format!("create base dir {:?}", self.base_dir))?;

        let safe_id = sanitize_page_id(page_id);
        let page_dir = self.base_dir.join(&safe_id);
        fs::create_dir_all(&page_dir)
            .with_context(|| format!("create page dir {:?}", page_dir))?;

        let meta_path = page_dir.join("meta.json");
        let html_path = page_dir.join("index.html");

        let mut index = self.load_index()?;
        let existing_meta = if meta_path.exists() {
            let existing_raw = fs::read_to_string(&meta_path)
                .with_context(|| format!("read meta.json {:?}", meta_path))?;
            let existing_meta: PageMeta =
                serde_json::from_str(&existing_raw).context("parse meta.json")?;
            Some(existing_meta)
        } else {
            None
        };
        let existing_uid = existing_meta
            .as_ref()
            .map(|value| value.page_uid.clone())
            .filter(|uid| !uid.is_empty());
        let index_uid = index
            .pages
            .get(&safe_id)
            .map(|entry| entry.page_uid.clone())
            .filter(|uid| !uid.is_empty());
        let fallback_uid = if meta.page_uid.is_empty() {
            None
        } else {
            Some(meta.page_uid.clone())
        };
        let page_uid = match existing_uid.or(index_uid).or(fallback_uid) {
            Some(uid) => uid,
            None => generate_unique_page_uid(&index)?,
        };
        let now_ts = now_unix_seconds()?;
        let existing_created_at = existing_meta
            .as_ref()
            .map(|value| value.created_at)
            .filter(|value| *value > 0);
        let fallback_created_at = if meta.created_at > 0 {
            Some(meta.created_at)
        } else {
            None
        };
        let created_at = existing_created_at.or(fallback_created_at).unwrap_or(now_ts);
        let updated_at = now_ts;
        let mut meta_to_write = meta.clone();
        meta_to_write.page_uid = page_uid.clone();
        meta_to_write.created_at = created_at;
        meta_to_write.updated_at = updated_at;

        let meta_bytes =
            serde_json::to_vec_pretty(&meta_to_write).context("serialize meta.json")?;
        atomic_write(&meta_path, &meta_bytes).context("write meta.json")?;
        validate_html(html).context("validate html")?;
        atomic_write(&html_path, html.as_bytes()).context("write index.html")?;
        let original_id = index
            .pages
            .get(&safe_id)
            .and_then(|entry| entry.original_id.clone())
            .or_else(|| {
                if safe_id == page_id {
                    None
                } else {
                    Some(page_id.to_string())
                }
            });
        index.pages.insert(
            safe_id.clone(),
            PageIndexEntry {
                page_id: safe_id,
                seo: meta_to_write.seo.clone(),
                page_uid: page_uid.clone(),
                original_id,
            },
        );
        self.save_index(&index)?;

        Ok(())
    }

    pub fn update_page(&self, page_id: &str, meta: &PageMeta, html: &str) -> Result<()> {
        if !self.page_exists(page_id)? {
            bail!("page not found: {}", page_id);
        }
        self.save_page(page_id, meta, html)
    }

    pub fn load_page(&self, page_id: &str) -> Result<(PageMeta, String)> {
        let safe_id = sanitize_page_id(page_id);
        let page_dir = self.base_dir.join(&safe_id);
        let meta_path = page_dir.join("meta.json");
        let html_path = page_dir.join("index.html");

        let meta_raw = fs::read_to_string(&meta_path)
            .with_context(|| format!("read meta.json {:?}", meta_path))?;
        let meta: PageMeta = serde_json::from_str(&meta_raw).context("parse meta.json")?;

        let html = fs::read_to_string(&html_path)
            .with_context(|| format!("read index.html {:?}", html_path))?;

        Ok((meta, html))
    }

    pub fn get_page_meta(&self, page_id: &str) -> Result<PageMeta> {
        let (meta, _) = self.load_page(page_id)?;
        Ok(meta)
    }

    pub fn get_page_html(&self, page_id: &str) -> Result<String> {
        let (_, html) = self.load_page(page_id)?;
        Ok(html)
    }

    pub fn update_page_meta(&self, page_id: &str, meta: &PageMeta) -> Result<()> {
        if !self.page_exists(page_id)? {
            bail!("page not found: {}", page_id);
        }

        let safe_id = sanitize_page_id(page_id);
        let meta_path = self.base_dir.join(&safe_id).join("meta.json");
        let mut index = self.load_index()?;
        let existing_meta = if meta_path.exists() {
            let existing_raw = fs::read_to_string(&meta_path)
                .with_context(|| format!("read meta.json {:?}", meta_path))?;
            let existing_meta: PageMeta =
                serde_json::from_str(&existing_raw).context("parse meta.json")?;
            Some(existing_meta)
        } else {
            None
        };
        let existing_uid = existing_meta
            .as_ref()
            .map(|value| value.page_uid.clone())
            .filter(|uid| !uid.is_empty());
        let index_uid = index
            .pages
            .get(&safe_id)
            .map(|entry| entry.page_uid.clone())
            .filter(|uid| !uid.is_empty());
        let fallback_uid = if meta.page_uid.is_empty() {
            None
        } else {
            Some(meta.page_uid.clone())
        };
        let page_uid = match existing_uid.or(index_uid).or(fallback_uid) {
            Some(uid) => uid,
            None => generate_unique_page_uid(&index)?,
        };
        let now_ts = now_unix_seconds()?;
        let existing_created_at = existing_meta
            .as_ref()
            .map(|value| value.created_at)
            .filter(|value| *value > 0);
        let fallback_created_at = if meta.created_at > 0 {
            Some(meta.created_at)
        } else {
            None
        };
        let created_at = existing_created_at.or(fallback_created_at).unwrap_or(now_ts);
        let updated_at = now_ts;
        let mut meta_to_write = meta.clone();
        meta_to_write.page_uid = page_uid.clone();
        meta_to_write.created_at = created_at;
        meta_to_write.updated_at = updated_at;
        let meta_bytes =
            serde_json::to_vec_pretty(&meta_to_write).context("serialize meta.json")?;
        atomic_write(&meta_path, &meta_bytes).context("write meta.json")?;
        let original_id = index
            .pages
            .get(&safe_id)
            .and_then(|entry| entry.original_id.clone())
            .or_else(|| {
                if safe_id == page_id {
                    None
                } else {
                    Some(page_id.to_string())
                }
            });
        index.pages.insert(
            safe_id.clone(),
            PageIndexEntry {
                page_id: safe_id,
                seo: meta_to_write.seo.clone(),
                page_uid,
                original_id,
            },
        );
        self.save_index(&index)?;

        Ok(())
    }

    pub fn update_page_html(&self, page_id: &str, html: &str) -> Result<()> {
        if !self.page_exists(page_id)? {
            bail!("page not found: {}", page_id);
        }

        let safe_id = sanitize_page_id(page_id);
        let html_path = self.base_dir.join(&safe_id).join("index.html");
        validate_html(html).context("validate html")?;
        atomic_write(&html_path, html.as_bytes()).context("write index.html")?;

        let meta_path = self.base_dir.join(&safe_id).join("meta.json");
        let meta_raw = fs::read_to_string(&meta_path)
            .with_context(|| format!("read meta.json {:?}", meta_path))?;
        let mut meta: PageMeta = serde_json::from_str(&meta_raw).context("parse meta.json")?;
        let mut index = self.load_index()?;
        let now_ts = now_unix_seconds()?;
        let index_uid = index
            .pages
            .get(&safe_id)
            .map(|entry| entry.page_uid.clone())
            .filter(|uid| !uid.is_empty());
        let meta_uid = if meta.page_uid.is_empty() {
            None
        } else {
            Some(meta.page_uid.clone())
        };
        let page_uid = match meta_uid.or(index_uid) {
            Some(uid) => uid,
            None => generate_unique_page_uid(&index)?,
        };
        if meta.created_at <= 0 {
            meta.created_at = now_ts;
        }
        meta.updated_at = now_ts;
        meta.page_uid = page_uid.clone();
        let meta_bytes = serde_json::to_vec_pretty(&meta).context("serialize meta.json")?;
        atomic_write(&meta_path, &meta_bytes).context("write meta.json")?;

        let original_id = index
            .pages
            .get(&safe_id)
            .and_then(|entry| entry.original_id.clone())
            .or_else(|| {
                if safe_id == page_id {
                    None
                } else {
                    Some(page_id.to_string())
                }
            });
        index.pages.insert(
            safe_id.clone(),
            PageIndexEntry {
                page_id: safe_id,
                seo: meta.seo.clone(),
                page_uid,
                original_id,
            },
        );
        self.save_index(&index)?;

        Ok(())
    }

    pub fn delete_page(&self, page_id: &str) -> Result<()> {
        let safe_id = sanitize_page_id(page_id);
        if !self.page_exists(page_id)? {
            bail!("page not found: {}", page_id);
        }

        let page_dir = self.base_dir.join(&safe_id);
        fs::remove_dir_all(&page_dir)
            .with_context(|| format!("remove page dir {:?}", page_dir))?;

        let mut index = self.load_index()?;
        index.pages.remove(&safe_id);
        self.save_index(&index)?;

        Ok(())
    }

    pub fn page_exists(&self, page_id: &str) -> Result<bool> {
        let safe_id = sanitize_page_id(page_id);
        let index = self.load_index()?;
        if index.pages.contains_key(&safe_id) {
            return Ok(true);
        }
        let page_dir = self.base_dir.join(&safe_id);
        Ok(page_dir.is_dir())
    }

    pub fn list_pages(&self) -> Result<Vec<String>> {
        let index = self.load_index()?;
        Ok(index.pages.keys().cloned().collect())
    }

    pub fn list_page_entries(&self) -> Result<Vec<PageIndexEntry>> {
        let index = self.load_index()?;
        Ok(index.pages.values().cloned().collect())
    }

    pub fn rebuild_index(&self) -> Result<StoreIndex> {
        fs::create_dir_all(&self.base_dir)
            .with_context(|| format!("create base dir {:?}", self.base_dir))?;

        let mut index = StoreIndex::default();
        for entry in fs::read_dir(&self.base_dir)
            .with_context(|| format!("read base dir {:?}", self.base_dir))?
        {
            let entry = entry.context("read dir entry")?;
            let file_type = entry.file_type().context("read dir entry type")?;
            if !file_type.is_dir() {
                continue;
            }
            let page_id = entry.file_name().to_string_lossy().to_string();
            let meta_path = entry.path().join("meta.json");
            let meta_raw = match fs::read_to_string(&meta_path) {
                Ok(raw) => raw,
                Err(_) => continue,
            };
            let meta: PageMeta = match serde_json::from_str(&meta_raw) {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            index.pages.insert(
                page_id.clone(),
                PageIndexEntry {
                    page_id,
                    seo: meta.seo,
                    page_uid: meta.page_uid,
                    original_id: None,
                },
            );
        }

        self.save_index(&index)?;
        Ok(index)
    }

    fn load_index(&self) -> Result<StoreIndex> {
        let index_path = self.index_path();
        match fs::read_to_string(&index_path) {
            Ok(raw) => match serde_json::from_str::<StoreIndex>(&raw) {
                Ok(index) => Ok(index),
                Err(_) => self.rebuild_index(),
            },
            Err(_) => self.rebuild_index(),
        }
    }

    fn save_index(&self, index: &StoreIndex) -> Result<()> {
        fs::create_dir_all(&self.base_dir)
            .with_context(|| format!("create base dir {:?}", self.base_dir))?;
        let index_path = self.index_path();
        let bytes = serde_json::to_vec_pretty(index).context("serialize index.json")?;
        atomic_write(&index_path, &bytes).context("write index.json")?;
        Ok(())
    }

    fn index_path(&self) -> PathBuf {
        self.base_dir.join("index.json")
    }
}

pub fn validate_html(html: &str) -> Result<()> {
    if html.trim().is_empty() {
        bail!("html is empty or whitespace");
    }
    if let Some(pos) = html.as_bytes().iter().position(|byte| *byte == 0) {
        bail!("html contains NUL byte at index {}", pos);
    }

    let bytes = html.as_bytes();
    let mut index = 0usize;
    let mut stack: Vec<(String, usize)> = Vec::new();

    while index < bytes.len() {
        if bytes[index] != b'<' {
            index += 1;
            continue;
        }

        if index + 3 < bytes.len()
            && bytes[index + 1] == b'!'
            && bytes[index + 2] == b'-'
            && bytes[index + 3] == b'-'
        {
            if let Some(end) = find_bytes(bytes, index + 4, b"-->") {
                index = end + 3;
                continue;
            }
            bail!("unterminated comment at index {}", index);
        }

        if index + 1 < bytes.len() && bytes[index + 1] == b'!' {
            if let Some(end) = find_tag_end(bytes, index + 2) {
                index = end + 1;
                continue;
            }
            bail!("unterminated declaration at index {}", index);
        }

        if index + 1 < bytes.len() && bytes[index + 1] == b'/' {
            let (name, after_name) = parse_tag_name(bytes, index + 2, index)?;
            let end = find_tag_end(bytes, after_name)
                .ok_or_else(|| anyhow::anyhow!("unterminated closing tag at index {}", index))?;
            let name = name.to_ascii_lowercase();
            let Some((open_tag, open_index)) = stack.pop() else {
                bail!("unexpected closing tag </{}> at index {}", name, index);
            };
            if open_tag != name {
                bail!(
                    "mismatched closing tag </{}> at index {}, expected </{}> for tag opened at index {}",
                    name,
                    index,
                    open_tag,
                    open_index
                );
            }
            index = end + 1;
            continue;
        }

        let (name, after_name) = parse_tag_name(bytes, index + 1, index)?;
        let end = find_tag_end(bytes, after_name)
            .ok_or_else(|| anyhow::anyhow!("unterminated opening tag at index {}", index))?;
        let is_self_closing = is_self_closing(bytes, index + 1, end);
        let name = name.to_ascii_lowercase();

        if name == "script" || name == "style" {
            if is_self_closing {
                index = end + 1;
                continue;
            }
            stack.push((name.clone(), index));
            let closing = format!("</{}>", name);
            if let Some(close_start) = find_bytes_case_insensitive(bytes, end + 1, closing.as_bytes())
            {
                let close_end = close_start + closing.len();
                let _ = stack.pop();
                index = close_end;
                continue;
            }
            bail!("unterminated <{}> starting at index {}", name, index);
        }

        if !is_self_closing && !is_void_element(&name) {
            stack.push((name, index));
        }
        index = end + 1;
    }

    if let Some((tag, open_index)) = stack.pop() {
        bail!("unclosed tag <{}> starting at index {}", tag, open_index);
    }

    Ok(())
}

pub fn sanitize_page_id(page_id: &str) -> String {
    let sanitized: String = page_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "page".to_string()
    } else {
        sanitized
    }
}

const PAGE_UID_LEN: usize = 16;
const PAGE_UID_ALPHABET: &[u8; 62] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn generate_page_uid() -> Result<String> {
    let mut bytes = [0u8; PAGE_UID_LEN];
    getrandom(&mut bytes).map_err(|err| anyhow::anyhow!("getrandom page uid failed: {}", err))?;
    let mut out = String::with_capacity(PAGE_UID_LEN);
    for byte in bytes {
        let idx = (byte % 62) as usize;
        out.push(PAGE_UID_ALPHABET[idx] as char);
    }
    Ok(out)
}

fn generate_unique_page_uid(index: &StoreIndex) -> Result<String> {
    for _ in 0..8 {
        let uid = generate_page_uid()?;
        if !index.pages.values().any(|entry| entry.page_uid == uid) {
            return Ok(uid);
        }
    }
    bail!("failed to generate unique page uid")
}

fn now_unix_seconds() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?;
    Ok(duration.as_secs().min(i64::MAX as u64) as i64)
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir {:?}", parent))?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, data).with_context(|| format!("write temp file {:?}", tmp_path))?;
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp_path, path)
        .with_context(|| format!("rename temp file {:?} -> {:?}", tmp_path, path))?;
    Ok(())
}

fn parse_tag_name(bytes: &[u8], mut index: usize, tag_start: usize) -> Result<(String, usize)> {
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    let start = index;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b':' {
            index += 1;
        } else {
            break;
        }
    }
    if start == index {
        bail!("missing tag name at index {}", tag_start);
    }
    let name = std::str::from_utf8(&bytes[start..index]).context("read tag name")?;
    Ok((name.to_string(), index))
}

fn find_tag_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            None => {
                if byte == b'\'' || byte == b'"' {
                    quote = Some(byte);
                } else if byte == b'>' {
                    return Some(index);
                }
            }
            Some(active) => {
                if byte == active {
                    quote = None;
                }
            }
        }
        index += 1;
    }
    None
}

fn is_self_closing(bytes: &[u8], start: usize, end: usize) -> bool {
    let mut index = end;
    while index > start {
        let byte = bytes[index - 1];
        if byte.is_ascii_whitespace() {
            index -= 1;
            continue;
        }
        return byte == b'/';
    }
    false
}

fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn find_bytes(haystack: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(start.min(haystack.len()));
    }
    if start >= haystack.len() || needle.len() > haystack.len() {
        return None;
    }
    let end = haystack.len().saturating_sub(needle.len());
    for index in start..=end {
        if &haystack[index..index + needle.len()] == needle {
            return Some(index);
        }
    }
    None
}

fn find_bytes_case_insensitive(haystack: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(start.min(haystack.len()));
    }
    if start >= haystack.len() || needle.len() > haystack.len() {
        return None;
    }
    let needle_lower: Vec<u8> = needle.iter().map(|byte| byte.to_ascii_lowercase()).collect();
    let end = haystack.len().saturating_sub(needle_lower.len());
    for index in start..=end {
        let mut matched = true;
        for (offset, expected) in needle_lower.iter().enumerate() {
            if haystack[index + offset].to_ascii_lowercase() != *expected {
                matched = false;
                break;
            }
        }
        if matched {
            return Some(index);
        }
    }
    None
}
