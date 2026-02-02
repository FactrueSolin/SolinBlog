use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use solin_blog::store::PageStore;
use solin_blog::web::{parse_page_id_from_slug, render_index_html, render_page_html};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let store = Arc::new(PageStore::new("data"));
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/pages/{slug}", get(page_handler))
        .with_state(store)
        .layer(middleware::from_fn(log_request));

    let host = std::env::var("WEB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("WEB_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = match host.parse::<IpAddr>() {
        Ok(ip) => SocketAddr::from((ip, port)),
        Err(_) => SocketAddr::from(([127, 0, 0, 1], port)),
    };
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind http listener");
    println!("[solin-blog] http server listening on http://{addr}");
    axum::serve(listener, app).await.expect("serve http");
}

async fn log_request(req: Request<Body>, next: Next) -> Response {
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

async fn index_handler(State(store): State<Arc<PageStore>>, _headers: HeaderMap) -> impl IntoResponse {
    match render_index_html(&store) {
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
