use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoMeta {
    pub title: String,
    pub description: String,
    pub keywords: Option<Vec<String>>,
    pub canonical_url: Option<String>,
    #[serde(default)]
    pub extra: Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub seo: SeoMeta,
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

        let meta_bytes = serde_json::to_vec_pretty(meta).context("serialize meta.json")?;
        atomic_write(&meta_path, &meta_bytes).context("write meta.json")?;
        atomic_write(&html_path, html.as_bytes()).context("write index.html")?;

        let mut index = self.load_index()?;
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
        let meta_bytes = serde_json::to_vec_pretty(meta).context("serialize meta.json")?;
        atomic_write(&meta_path, &meta_bytes).context("write meta.json")?;

        let mut index = self.load_index()?;
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
        atomic_write(&html_path, html.as_bytes()).context("write index.html")?;

        let index = self.load_index()?;
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
