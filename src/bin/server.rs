//! SolinBlog 服务器入口
//!
//! 启动 Web 服务器和 MCP 接口

use axum::{Json, Router, extract::DefaultBodyLimit, middleware, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use solin_blog::image_host::ImageStore;
use solin_blog::mcp::{BlogMcpServer, TokenStore, mcp_auth_middleware};
use solin_blog::openapi::build_openapi_json;
use solin_blog::store::PageStore;
use solin_blog::web::generate_token;
use solin_blog::web::{
    ImageWebState, delete_image_handler, get_image_handler, image_asset_handler,
    image_auth_middleware, image_page_handler, index_handler, list_images_handler, page_handler,
    public_asset_handler, replace_image_handler, sitemap_handler, token_generator_handler,
    update_image_handler, upload_image_handler,
};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // 初始化数据存储
    let store = Arc::new(PageStore::new("data"));
    let image_store =
        Arc::new(ImageStore::load_or_init("data/images").expect("initialize image hosting store"));
    let image_max_upload_mb = std::env::var("IMAGE_MAX_UPLOAD_MB")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(10);
    let image_state = ImageWebState {
        store: Arc::clone(&image_store),
        max_upload_bytes: image_max_upload_mb.saturating_mul(1024 * 1024),
    };

    // 读取鉴权 token。TOKEN 是正式配置名；MCP_TOKEN 作为历史兼容，避免旧部署无法访问。
    let mut valid_tokens = Vec::new();
    for key in ["TOKEN", "MCP_TOKEN"] {
        let value = std::env::var(key).unwrap_or_default().trim().to_string();
        if !value.is_empty() && !valid_tokens.contains(&value) {
            valid_tokens.push(value);
        }
    }
    let token = match valid_tokens.first() {
        Some(token) => token.clone(),
        None => {
            let token = generate_token();
            valid_tokens.push(token.clone());
            token
        }
    };
    if std::env::var("TOKEN").unwrap_or_default().trim().is_empty()
        && std::env::var("MCP_TOKEN")
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        println!("[solin-blog] token generated: {token}");
    }

    // 创建 Token 存储
    let token_store = Arc::new(TokenStore::new(valid_tokens));

    // 创建 MCP 服务器
    let mcp_server = BlogMcpServer::new(
        Arc::clone(&store),
        Arc::clone(&image_store),
        Arc::clone(&token_store),
    );
    let mcp_service = StreamableHttpService::new(
        move || Ok(mcp_server.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    // 创建受保护的 MCP 路由（需要认证）
    let protected_mcp_router =
        Router::new()
            .nest_service("/mcp", mcp_service)
            .layer(middleware::from_fn_with_state(
                token_store.clone(),
                mcp_auth_middleware,
            ));

    let page_router = Router::new()
        .route("/", get(index_handler))
        .route("/tools/token-generator", get(token_generator_handler))
        .route("/pages/{slug}", get(page_handler))
        .route("/sitemap.xml", get(sitemap_handler))
        .route("/public/{*path}", get(public_asset_handler))
        .with_state(store);

    let public_image_router = Router::new()
        .route("/image", get(image_page_handler))
        .route("/images/{image_id}/{filename}", get(image_asset_handler))
        .with_state(image_state.clone());

    let protected_image_router = Router::new()
        .route(
            "/api/images",
            get(list_images_handler).post(upload_image_handler),
        )
        .route(
            "/api/images/{image_id}",
            get(get_image_handler)
                .patch(update_image_handler)
                .put(replace_image_handler)
                .delete(delete_image_handler),
        )
        .layer(DefaultBodyLimit::max(image_state.max_upload_bytes))
        .layer(middleware::from_fn_with_state(
            token_store.clone(),
            image_auth_middleware,
        ))
        .with_state(image_state);

    // 创建主应用路由
    let openapi_json = build_openapi_json();
    let app = Router::new()
        .route("/openapi.json", get(|| async { Json(openapi_json) }))
        .merge(page_router)
        .merge(public_image_router)
        .merge(protected_image_router)
        .merge(protected_mcp_router)
        .layer(middleware::from_fn(solin_blog::web::log_request));

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
    println!("[solin-blog] image page: http://{addr}/image");
    println!("[solin-blog] Authorization: Bearer {token}");

    axum::serve(listener, app).await.expect("serve http");
}
