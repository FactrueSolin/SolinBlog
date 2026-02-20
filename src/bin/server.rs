//! SolinBlog 服务器入口
//!
//! 启动 Web 服务器和 MCP 接口

use axum::{
    middleware,
    routing::get,
    Router,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use solin_blog::mcp::{BlogMcpServer, TokenStore, mcp_auth_middleware};
use solin_blog::store::PageStore;
use solin_blog::web::{
    index_handler, page_handler, public_asset_handler, sitemap_handler,
    token_generator_handler,
};
use solin_blog::web::generate_mcp_token;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // 初始化数据存储
    let store = Arc::new(PageStore::new("data"));

    // 读取 MCP_TOKEN 环境变量
    let mut mcp_token = std::env::var("MCP_TOKEN")
        .unwrap_or_default()
        .trim()
        .to_string();
    if mcp_token.is_empty() {
        mcp_token = generate_mcp_token();
        println!("[solin-blog] MCP token generated: {mcp_token}");
    }

    // 创建 Token 存储
    let token_store = Arc::new(TokenStore::new(vec![mcp_token.clone()]));

    // 创建 MCP 服务器
    let mcp_server = BlogMcpServer::new(Arc::clone(&store), Arc::clone(&token_store));
    let mcp_service = StreamableHttpService::new(
        move || Ok(mcp_server.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    // 创建受保护的 MCP 路由（需要认证）
    let protected_mcp_router = Router::new()
        .nest_service("/mcp", mcp_service)
        .layer(middleware::from_fn_with_state(
            token_store.clone(),
            mcp_auth_middleware,
        ));

    // 创建主应用路由
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/tools/token-generator", get(token_generator_handler))
        .route("/pages/{slug}", get(page_handler))
        .route("/sitemap.xml", get(sitemap_handler))
        .route("/public/{*path}", get(public_asset_handler))
        .merge(protected_mcp_router)
        .layer(middleware::from_fn(solin_blog::web::log_request))
        .with_state(store);

    // 绑定监听地址
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
    println!("[solin-blog] MCP endpoint: http://{addr}/mcp");
    println!("[solin-blog] Authorization: Bearer {mcp_token}");

    axum::serve(listener, app).await.expect("serve http");
}
