use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{Router, middleware, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use solin_blog::{
    config::generate_mcp_token,
    mcp::BlogMcpServer,
    server::{
        index_handler, log_request, page_handler, public_asset_handler, sitemap_handler,
        token_generator_handler,
    },
    store::PageStore,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let store = Arc::new(PageStore::new("data"));
    let mut mcp_token = std::env::var("MCP_TOKEN")
        .unwrap_or_default()
        .trim()
        .to_string();
    if mcp_token.is_empty() {
        mcp_token = generate_mcp_token();
        println!("[solin-blog] MCP token generated: {mcp_token}");
    }

    let mcp_path = format!("/{}/mcp", mcp_token);
    let mcp_server = BlogMcpServer::new(Arc::clone(&store));
    let mcp_service = StreamableHttpService::new(
        move || Ok(mcp_server.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/tools/token-generator", get(token_generator_handler))
        .route("/pages/{slug}", get(page_handler))
        .route("/sitemap.xml", get(sitemap_handler))
        .route("/public/{*path}", get(public_asset_handler))
        .nest_service(mcp_path.as_str(), mcp_service)
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
    println!("[solin-blog] MCP endpoint: http://{addr}{mcp_path}");
    axum::serve(listener, app).await.expect("serve http");
}
