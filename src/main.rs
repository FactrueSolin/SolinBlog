use axum::{
    extract::{Host, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

use SolinBlog::store::PageStore;
use SolinBlog::web::{parse_page_id_from_slug, render_index_html, render_page_html};

#[tokio::main]
async fn main() {
    let store = Arc::new(PageStore::new("data"));
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/pages/:slug", get(page_handler))
        .with_state(store);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind http listener");
    axum::serve(listener, app).await.expect("serve http");
}

async fn index_handler(
    State(store): State<Arc<PageStore>>,
    Host(host): Host,
) -> impl IntoResponse {
    match render_index_html(&store, &host) {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("render index failed: {err}"),
        )
            .into_response(),
    }
}

async fn page_handler(
    State(store): State<Arc<PageStore>>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let Some(page_id) = parse_page_id_from_slug(&slug) else {
        return (
            StatusCode::NOT_FOUND,
            format!("invalid page slug: {slug}"),
        )
            .into_response();
    };
    match store.load_page(&page_id) {
        Ok((meta, html)) => Html(render_page_html(&meta, &html)).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            format!("page not found: {err}"),
        )
            .into_response(),
    }
}
