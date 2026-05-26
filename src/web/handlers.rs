//! Web 请求处理器

use axum::{
    body::Body,
    extract::{Path, State},
    http::{
        HeaderMap, Request, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use mime_guess::MimeGuess;
use std::path::{Component, Path as FsPath, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::store::PageStore;
use crate::web::{
    inject_umami_script, parse_page_id_from_slug, render_404_html, render_index_html,
    render_page_html, render_sitemap_xml,
};

use super::config::resolve_base_url;

#[derive(Debug, Clone)]
pub struct PageWebState {
    pub store: Arc<PageStore>,
    pub home_cache: Arc<Mutex<HomePageCache>>,
}

impl PageWebState {
    pub fn new(store: Arc<PageStore>) -> Self {
        Self {
            store,
            home_cache: Arc::new(Mutex::new(HomePageCache::default())),
        }
    }
}

#[derive(Debug, Default)]
pub struct HomePageCache {
    key: Option<HomePageCacheKey>,
    html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HomePageCacheKey {
    index_json: Option<FileCacheKey>,
    header_html: Option<FileCacheKey>,
    index_html: Option<FileCacheKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileCacheKey {
    modified_nanos: Option<u128>,
    len: u64,
}

/// 日志中间件
pub async fn log_request(req: Request<Body>, next: Next) -> Response {
    let upgrade = req
        .headers()
        .get("upgrade")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-");
    let connection = req
        .headers()
        .get("connection")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-");
    println!(
        "[solin-blog] {} {} upgrade={} connection={}",
        req.method(),
        req.uri(),
        upgrade,
        connection
    );
    let response = next.run(req).await;
    println!("[solin-blog] -> {}", response.status());
    response
}

/// 首页处理器
pub async fn index_handler(State(state): State<PageWebState>) -> impl IntoResponse {
    match cached_index_html(&state).await {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("render index failed：{err}"),
        )
            .into_response(),
    }
}

async fn cached_index_html(state: &PageWebState) -> anyhow::Result<String> {
    let key = home_page_cache_key(&state.store)?;
    let mut cache = state.home_cache.lock().await;
    if cache.key.as_ref() == Some(&key) {
        return Ok(cache.html.clone());
    }

    let html = render_index_html(&state.store)?;
    cache.key = Some(key);
    cache.html = html.clone();
    Ok(html)
}

fn home_page_cache_key(store: &PageStore) -> anyhow::Result<HomePageCacheKey> {
    Ok(HomePageCacheKey {
        index_json: file_cache_key(&store.base_dir.join("index.json"))?,
        header_html: file_cache_key(FsPath::new("front/header.html"))?,
        index_html: file_cache_key(FsPath::new("front/index.html"))?,
    })
}

fn file_cache_key(path: &FsPath) -> anyhow::Result<Option<FileCacheKey>> {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let modified_nanos = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos());
            Ok(Some(FileCacheKey {
                modified_nanos,
                len: metadata.len(),
            }))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// 页面处理器
pub async fn page_handler(
    State(state): State<PageWebState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let Some(page_id) = parse_page_id_from_slug(&slug) else {
        return match render_404_html() {
            Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render 404 failed: {err}"),
            )
                .into_response(),
        };
    };
    match state.store.load_page(&page_id) {
        Ok((meta, html)) => {
            let rendered = render_page_html(&meta, &html);
            if let Err(err) = state.store.increment_view_count(&page_id) {
                eprintln!("[solin-blog] increment view count failed: {err}");
            }
            Html(rendered).into_response()
        }
        Err(_err) => match render_404_html() {
            Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render 404 failed: {err}"),
            )
                .into_response(),
        },
    }
}

/// Token 生成器页面处理器
pub async fn token_generator_handler() -> impl IntoResponse {
    match std::fs::read_to_string("front/token-generator.html") {
        Ok(html) => Html(inject_umami_script(&html)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read token generator html failed: {err}"),
        )
            .into_response(),
    }
}

/// Sitemap 处理器
pub async fn sitemap_handler(
    State(state): State<PageWebState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let base_url = resolve_base_url(&headers);
    match render_sitemap_xml(&state.store, &base_url) {
        Ok(xml) => ([(CONTENT_TYPE, "application/xml")], xml).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("render sitemap failed: {err}"),
        )
            .into_response(),
    }
}

/// 公共资源处理器
pub async fn public_asset_handler(Path(path): Path<String>) -> impl IntoResponse {
    if path.is_empty() {
        return match render_404_html() {
            Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render 404 failed: {err}"),
            )
                .into_response(),
        };
    }
    let Ok(safe_path) = sanitize_public_path(&path) else {
        return match render_404_html() {
            Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render 404 failed: {err}"),
            )
                .into_response(),
        };
    };
    let full_path = PathBuf::from("public").join(&safe_path);
    let data = match std::fs::read(&full_path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return match render_404_html() {
                Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("render 404 failed: {err}"),
                )
                    .into_response(),
            };
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("read public asset failed: {err}"),
            )
                .into_response();
        }
    };
    let mime = guess_mime_type(&full_path);
    (
        [
            (CONTENT_TYPE, mime.as_ref()),
            (CACHE_CONTROL, "public, max-age=86400"),
        ],
        data,
    )
        .into_response()
}

/// 清理和验证公共路径
fn sanitize_public_path(raw: &str) -> Result<PathBuf, ()> {
    let mut cleaned = PathBuf::new();
    for segment in raw.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(());
        }
        let segment_path = FsPath::new(segment);
        let mut segment_components = segment_path.components();
        match segment_components.next() {
            Some(Component::Normal(_)) if segment_components.next().is_none() => {}
            _ => return Err(()),
        }
        cleaned.push(segment);
    }
    if cleaned.as_os_str().is_empty() {
        return Err(());
    }
    Ok(cleaned)
}

/// 猜测 MIME 类型
fn guess_mime_type(path: &FsPath) -> mime_guess::Mime {
    MimeGuess::from_path(path).first_or_octet_stream()
}
