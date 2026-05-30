//! Import local SolinBlog pages into Strapi.
//!
//! Required environment variables:
//! - STRAPI_API_URL
//! - STRAPI_API_TOKEN
//!
//! Compatibility aliases are also accepted:
//! - STAPI_API_URL
//! - STAPI_API_TOKEN

use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solin_blog::store::{PageIndexEntry, PageMeta, PageStore};
use std::time::Duration;

const DEFAULT_DATA_DIR: &str = "data";
const SOLIN_PAGES_PATH: &str = "/api/solin-pages";

#[derive(Debug, Clone)]
struct Config {
    strapi_api_url: String,
    strapi_api_token: String,
    data_dir: String,
    dry_run: bool,
}

#[derive(Debug, Serialize)]
struct StrapiRequest<T> {
    data: T,
}

#[derive(Debug, Serialize)]
struct SolinPagePayload {
    page_id: String,
    page_uid: String,
    original_id: Option<String>,
    seo_title: String,
    seo: SeoMetaPayload,
    html: String,
    markdown: Option<String>,
    has_markdown: bool,
    created_at_unix: String,
    updated_at_unix: String,
    view_count: String,
    extra: Value,
    index_entry_snapshot: Value,
    raw_meta: Value,
}

#[derive(Debug, Serialize)]
struct SeoMetaPayload {
    title: String,
    seo_title: String,
    description: String,
    keywords: Option<Vec<String>>,
    extra: Value,
}

#[derive(Debug, Deserialize)]
struct StrapiListResponse {
    data: Vec<StrapiDocument>,
}

#[derive(Debug, Deserialize)]
struct StrapiDocument {
    id: Option<u64>,
    #[serde(rename = "documentId")]
    document_id: Option<String>,
}

#[derive(Debug, Default)]
struct ImportSummary {
    created: usize,
    updated: usize,
    skipped: usize,
    failed: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env_and_args()?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("build reqwest client")?;
    let store = PageStore::new(&config.data_dir);
    let mut entries = store
        .list_page_entries()
        .with_context(|| format!("list pages from {}", config.data_dir))?;
    entries.sort_by(|left, right| left.page_id.cmp(&right.page_id));

    println!(
        "[import-strapi] importing {} page(s) from {} into {}",
        entries.len(),
        config.data_dir,
        config.strapi_api_url
    );
    if config.dry_run {
        println!("[import-strapi] dry run enabled; no writes will be sent");
    }

    let mut summary = ImportSummary::default();
    for entry in entries {
        match import_one(&client, &config, &store, &entry).await {
            Ok(ImportAction::Created) => summary.created += 1,
            Ok(ImportAction::Updated) => summary.updated += 1,
            Ok(ImportAction::Skipped) => summary.skipped += 1,
            Err(err) => {
                summary.failed += 1;
                eprintln!("[import-strapi] failed {}: {err:#}", entry.page_id);
            }
        }
    }

    println!(
        "[import-strapi] done: {} created, {} updated, {} skipped, {} failed",
        summary.created, summary.updated, summary.skipped, summary.failed
    );
    if summary.failed > 0 {
        bail!("{} page(s) failed to import", summary.failed);
    }

    Ok(())
}

#[derive(Debug)]
enum ImportAction {
    Created,
    Updated,
    Skipped,
}

async fn import_one(
    client: &reqwest::Client,
    config: &Config,
    store: &PageStore,
    entry: &PageIndexEntry,
) -> Result<ImportAction> {
    let (meta, html) = store
        .load_page(&entry.page_id)
        .with_context(|| format!("load page {}", entry.page_id))?;
    let markdown = store
        .load_page_markdown(&entry.page_id)
        .with_context(|| format!("load markdown {}", entry.page_id))?;
    let payload = build_payload(entry, &meta, html, markdown)?;

    if config.dry_run {
        println!("[import-strapi] would import {}", entry.page_id);
        return Ok(ImportAction::Skipped);
    }

    let existing = find_existing_document(client, config, &entry.page_id).await?;
    match existing {
        Some(document_id) => {
            update_document(client, config, &document_id, &payload).await?;
            println!("[import-strapi] updated {}", entry.page_id);
            Ok(ImportAction::Updated)
        }
        None => {
            create_document(client, config, &payload).await?;
            println!("[import-strapi] created {}", entry.page_id);
            Ok(ImportAction::Created)
        }
    }
}

fn build_payload(
    entry: &PageIndexEntry,
    meta: &PageMeta,
    html: String,
    markdown: Option<String>,
) -> Result<SolinPagePayload> {
    let raw_meta = serde_json::to_value(meta).context("serialize raw meta")?;
    let index_entry_snapshot =
        serde_json::to_value(entry).context("serialize index entry snapshot")?;
    let page_uid = if meta.page_uid.trim().is_empty() {
        entry.page_uid.clone()
    } else {
        meta.page_uid.clone()
    };
    if page_uid.trim().is_empty() {
        bail!("page_uid is empty for {}", entry.page_id);
    }
    if meta.seo.seo_title.trim().is_empty() {
        bail!("seo.seo_title is empty for {}", entry.page_id);
    }

    let has_markdown = markdown.is_some();
    Ok(SolinPagePayload {
        page_id: entry.page_id.clone(),
        page_uid,
        original_id: entry.original_id.clone(),
        seo_title: meta.seo.seo_title.clone(),
        seo: SeoMetaPayload {
            title: meta.seo.title.clone(),
            seo_title: meta.seo.seo_title.clone(),
            description: meta.seo.description.clone(),
            keywords: meta.seo.keywords.clone(),
            extra: Value::Object(meta.seo.extra.clone()),
        },
        html,
        markdown,
        has_markdown,
        created_at_unix: meta.created_at.to_string(),
        updated_at_unix: meta.updated_at.to_string(),
        view_count: meta.view_count.to_string(),
        extra: Value::Object(meta.extra.clone()),
        index_entry_snapshot,
        raw_meta,
    })
}

async fn find_existing_document(
    client: &reqwest::Client,
    config: &Config,
    page_id: &str,
) -> Result<Option<String>> {
    let url = format!("{}{}", config.strapi_api_url, SOLIN_PAGES_PATH);
    let response = client
        .get(url)
        .bearer_auth(&config.strapi_api_token)
        .query(&[
            ("filters[page_id][$eq]", page_id),
            ("pagination[pageSize]", "1"),
        ])
        .send()
        .await
        .with_context(|| format!("query existing Strapi page {}", page_id))?;
    let status = response.status();
    let body = response.text().await.context("read Strapi list response")?;
    if !status.is_success() {
        bail!("Strapi list request failed with {status}: {body}");
    }

    let parsed: StrapiListResponse = serde_json::from_str(&body)
        .with_context(|| format!("parse Strapi list response: {body}"))?;
    Ok(parsed.data.first().and_then(StrapiDocument::identifier))
}

async fn create_document(
    client: &reqwest::Client,
    config: &Config,
    payload: &SolinPagePayload,
) -> Result<()> {
    let url = format!("{}{}", config.strapi_api_url, SOLIN_PAGES_PATH);
    send_write_request(
        client
            .post(url)
            .bearer_auth(&config.strapi_api_token)
            .json(&StrapiRequest { data: payload }),
        "create",
    )
    .await
}

async fn update_document(
    client: &reqwest::Client,
    config: &Config,
    document_id: &str,
    payload: &SolinPagePayload,
) -> Result<()> {
    let url = format!(
        "{}{}/{}",
        config.strapi_api_url, SOLIN_PAGES_PATH, document_id
    );
    send_write_request(
        client
            .put(url)
            .bearer_auth(&config.strapi_api_token)
            .json(&StrapiRequest { data: payload }),
        "update",
    )
    .await
}

async fn send_write_request(request: reqwest::RequestBuilder, action: &str) -> Result<()> {
    let response = request
        .send()
        .await
        .with_context(|| format!("send Strapi {action} request"))?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "<failed to read response body>".to_string());
    if status == StatusCode::FORBIDDEN {
        bail!("Strapi {action} request forbidden; check API token permissions: {body}");
    }
    bail!("Strapi {action} request failed with {status}: {body}");
}

impl StrapiDocument {
    fn identifier(&self) -> Option<String> {
        self.document_id
            .clone()
            .or_else(|| self.id.map(|value| value.to_string()))
    }
}

impl Config {
    fn from_env_and_args() -> Result<Self> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut data_dir = std::env::var("SOLINBLOG_DATA_DIR")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_DATA_DIR.to_string());
        let mut dry_run = false;

        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--dry-run" => {
                    dry_run = true;
                    index += 1;
                }
                "--data-dir" => {
                    let Some(value) = args.get(index + 1) else {
                        bail!("--data-dir requires a value");
                    };
                    data_dir = value.clone();
                    index += 2;
                }
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                unknown => bail!("unknown argument: {unknown}"),
            }
        }

        let strapi_api_url = read_env_any(&["STRAPI_API_URL", "STAPI_API_URL"])?;
        let strapi_api_token = read_env_any(&["STRAPI_API_TOKEN", "STAPI_API_TOKEN"])?;
        Ok(Self {
            strapi_api_url: normalize_base_url(&strapi_api_url),
            strapi_api_token,
            data_dir,
            dry_run,
        })
    }
}

fn read_env_any(keys: &[&str]) -> Result<String> {
    for key in keys {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    bail!("missing required env var; tried {}", keys.join(", "))
}

fn normalize_base_url(input: &str) -> String {
    input.trim().trim_end_matches('/').to_string()
}

fn print_help() {
    println!(
        "{}",
        json!({
            "usage": "cargo run --bin import_strapi_pages -- [--dry-run] [--data-dir data]",
            "env": ["STRAPI_API_URL", "STRAPI_API_TOKEN"],
            "compat_env": ["STAPI_API_URL", "STAPI_API_TOKEN"]
        })
    );
}
