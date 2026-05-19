//! Web 请求处理器

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode, header::CONTENT_TYPE},
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use mime_guess::MimeGuess;
use std::path::{Component, Path as FsPath, PathBuf};
use std::sync::Arc;

use crate::store::PageStore;
use crate::web::{
    inject_umami_script, parse_page_id_from_slug, render_404_html, render_index_html,
    render_page_html, render_sitemap_xml,
};

use super::config::resolve_base_url;

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
pub async fn index_handler(State(store): State<Arc<PageStore>>) -> impl IntoResponse {
    match render_index_html(&store) {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("render index failed：{err}"),
        )
            .into_response(),
    }
}

/// 页面处理器
pub async fn page_handler(
    State(store): State<Arc<PageStore>>,
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
    match store.load_page(&page_id) {
        Ok((meta, html)) => {
            let rendered = render_page_html(&page_id, &meta, &html);
            if let Err(err) = store.increment_view_count(&page_id) {
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
    State(store): State<Arc<PageStore>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let base_url = resolve_base_url(&headers);
    match render_sitemap_xml(&store, &base_url) {
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
    ([(CONTENT_TYPE, mime.as_ref())], data).into_response()
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
