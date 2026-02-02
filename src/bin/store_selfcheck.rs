use anyhow::{ensure, Context, Result};
use serde_json::Map;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use SolinBlog::store::{sanitize_page_id, PageMeta, PageStore, SeoMeta};

struct PageDirGuard {
    page_dir: PathBuf,
}

impl Drop for PageDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.page_dir);
    }
}

struct IndexSnapshotGuard {
    data_dir: PathBuf,
    index_path: PathBuf,
    index_bytes: Option<Vec<u8>>,
    data_dir_existed: bool,
}

impl Drop for IndexSnapshotGuard {
    fn drop(&mut self) {
        match &self.index_bytes {
            Some(bytes) => {
                if let Err(err) = fs::create_dir_all(&self.data_dir)
                    .and_then(|_| fs::write(&self.index_path, bytes))
                {
                    println!("restore index.json failed: {}", err);
                } else {
                    println!("restore index.json ok");
                }
            }
            None => {
                if self.index_path.exists() {
                    if let Err(err) = fs::remove_file(&self.index_path) {
                        println!("remove index.json failed: {}", err);
                    } else {
                        println!("remove index.json ok");
                    }
                }
            }
        }

        if !self.data_dir_existed && self.data_dir.is_dir() {
            match fs::read_dir(&self.data_dir) {
                Ok(mut entries) => {
                    if entries.next().is_none() {
                        if let Err(err) = fs::remove_dir(&self.data_dir) {
                            println!("remove empty data dir failed: {}", err);
                        } else {
                            println!("remove empty data dir ok");
                        }
                    }
                }
                Err(err) => {
                    println!("read data dir for cleanup failed: {}", err);
                }
            }
        }
    }
}

fn main() -> Result<()> {
    println!("store selfcheck start");

    let data_dir = Path::new("data");
    let data_dir_existed = data_dir.is_dir();
    let index_path = data_dir.join("index.json");
    let index_bytes = if index_path.exists() {
        Some(fs::read(&index_path).context("read index.json snapshot")?)
    } else {
        None
    };
    let _index_guard = IndexSnapshotGuard {
        data_dir: data_dir.to_path_buf(),
        index_path: index_path.clone(),
        index_bytes,
        data_dir_existed,
    };

    let store = PageStore::new("data");

    let unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let pid = std::process::id();
    let page_id = format!("store-selfcheck-{}-{}", unix_secs, pid);
    let safe_id = sanitize_page_id(&page_id);
    let _page_guard = PageDirGuard {
        page_dir: data_dir.join(&safe_id),
    };

    let meta = PageMeta {
        seo: SeoMeta {
            seo_title: "Store Selfcheck".to_string(),
            description: "CRUD selfcheck for store".to_string(),
            keywords: Some(vec!["selfcheck".to_string(), "store".to_string()]),
            extra: Map::new(),
        },
        page_uid: String::new(),
        created_at: 0,
        updated_at: 0,
        extra: Map::new(),
    };
    let html = concat!(
        "<!doctype html>",
        "<html>",
        "<head><meta charset=\"utf-8\"><title>Store Selfcheck</title></head>",
        "<body><main><h1>Store Selfcheck</h1><p>ok</p></main></body>",
        "</html>"
    );

    println!("create page");
    store
        .create_page(&page_id, &meta, html)
        .context("create page")?;
    println!("create ok");

    println!("load page");
    let (loaded_meta, loaded_html) = store.load_page(&page_id).context("load page")?;
    ensure!(loaded_meta.page_uid.len() == 16, "page uid len mismatch");
    ensure!(
        loaded_meta.page_uid.chars().all(|ch| ch.is_ascii_alphanumeric()),
        "page uid charset mismatch"
    );
    ensure!(loaded_meta.created_at > 0, "created_at missing");
    ensure!(loaded_meta.updated_at > 0, "updated_at missing");
    let initial_uid = loaded_meta.page_uid.clone();
    let initial_created_at = loaded_meta.created_at;
    ensure!(
        loaded_meta.seo.seo_title == meta.seo.seo_title,
        "title mismatch"
    );
    ensure!(
        loaded_meta.seo.description == meta.seo.description,
        "description mismatch"
    );
    ensure!(loaded_html == html, "html mismatch");
    println!("load ok");

    let meta2 = PageMeta {
        seo: SeoMeta {
            seo_title: "Store Selfcheck Updated".to_string(),
            description: "Updated description".to_string(),
            keywords: Some(vec!["selfcheck".to_string(), "update".to_string()]),
            extra: Map::new(),
        },
        page_uid: String::new(),
        created_at: 0,
        updated_at: 0,
        extra: Map::new(),
    };

    println!("update meta");
    store
        .update_page_meta(&page_id, &meta2)
        .context("update meta")?;
    let (updated_meta, _) = store.load_page(&page_id).context("load after meta")?;
    ensure!(updated_meta.page_uid == initial_uid, "page uid changed");
    ensure!(
        updated_meta.created_at == initial_created_at,
        "created_at changed"
    );
    ensure!(updated_meta.updated_at >= initial_created_at, "updated_at invalid");
    ensure!(
        updated_meta.seo.seo_title == meta2.seo.seo_title,
        "updated title mismatch"
    );
    ensure!(
        updated_meta.seo.description == meta2.seo.description,
        "updated description mismatch"
    );
    println!("update meta ok");

    let html2 = concat!(
        "<!doctype html>",
        "<html>",
        "<head><meta charset=\"utf-8\"><title>Store Selfcheck Updated</title></head>",
        "<body><main><h1>Updated</h1><p>updated body</p></main></body>",
        "</html>"
    );
    println!("update html");
    store
        .update_page_html(&page_id, html2)
        .context("update html")?;
    let (_, updated_html) = store.load_page(&page_id).context("load after html")?;
    let (updated_meta_after_html, _) = store.load_page(&page_id).context("load after html meta")?;
    ensure!(
        updated_meta_after_html.page_uid == initial_uid,
        "page uid changed after html"
    );
    ensure!(
        updated_meta_after_html.created_at == initial_created_at,
        "created_at changed after html"
    );
    ensure!(updated_html == html2, "updated html mismatch");
    println!("update html ok");

    println!("list pages");
    let pages = store.list_pages().context("list pages")?;
    ensure!(pages.iter().any(|id| id == &safe_id), "page not in index");
    println!("list pages ok");

    println!("delete page");
    store.delete_page(&page_id).context("delete page")?;
    ensure!(!store.page_exists(&page_id)?, "page still exists after delete");
    println!("delete ok");

    println!("store selfcheck done");
    Ok(())
}
