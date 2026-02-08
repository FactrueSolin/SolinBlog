use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse},
};

use crate::{
    store::PageStore,
    web::{
        parse_page_id_from_slug, render_404_html, render_index_html, render_page_html,
        render_sitemap_xml,
    },
};

pub async fn index_handler(
    State(store): State<Arc<PageStore>>,
    _headers: HeaderMap,
) -> impl IntoResponse {
    match render_index_html(&store) {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("render index failed: {err}"),
        )
            .into_response(),
    }
}

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
            let rendered = render_page_html(&meta, &html);
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

pub async fn token_generator_handler() -> impl IntoResponse {
    match std::fs::read_to_string("front/token-generator.html") {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read token generator html failed: {err}"),
        )
            .into_response(),
    }
}

fn resolve_base_url(headers: &HeaderMap) -> String {
    if let Some(host) = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let scheme = headers
            .get("x-forwarded-proto")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("http");
        return format!("{}://{}", scheme, host)
            .trim_end_matches('/')
            .to_string();
    }

    let value = std::env::var("SITE_URL").unwrap_or_default();
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        eprintln!(
            "[solin-blog] WARNING: SITE_URL is not set and request headers missing host, sitemap URLs will be relative"
        );
        return String::new();
    }
    trimmed.to_string()
}
