use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode, header::CONTENT_TYPE},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use getrandom::getrandom;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters, wrapper::Json},
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use solin_blog::store::{PageMeta, PageStore, validate_html};
use solin_blog::web::{
    parse_page_id_from_slug, render_index_html, render_page_html, render_sitemap_xml,
};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PushPageRequest {
    seo_title: String,
    description: String,
    keywords: Option<Vec<String>>,
    html: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SeoMetaResponse {
    seo_title: String,
    description: String,
    keywords: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PageMetaResponse {
    seo: SeoMetaResponse,
    page_uid: String,
    created_at: i64,
    updated_at: i64,
    view_count: u64,
}

impl From<PageMeta> for PageMetaResponse {
    fn from(meta: PageMeta) -> Self {
        Self {
            seo: SeoMetaResponse {
                seo_title: meta.seo.seo_title,
                description: meta.seo.description,
                keywords: meta.seo.keywords,
            },
            page_uid: meta.page_uid,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            view_count: meta.view_count,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PushPageResponse {
    success: bool,
    page_id: Option<String>,
    url: Option<String>,
    meta: Option<PageMetaResponse>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct GetAllPageResponse {
    success: bool,
    pages: Vec<PageWithMeta>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PageWithMeta {
    page_id: String,
    url: String,
    meta: PageMetaResponse,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PageIdRequest {
    page_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct GetPageByIdResponse {
    success: bool,
    page: Option<PageWithHtml>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PageWithHtml {
    page_id: String,
    url: String,
    meta: PageMetaResponse,
    html: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct DeletePageResponse {
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct UpdatePageRequest {
    page_id: String,
    seo_title: Option<String>,
    description: Option<String>,
    keywords: Option<Vec<String>>,
    html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct UpdatePageResponse {
    success: bool,
    url: Option<String>,
    meta: Option<PageMetaResponse>,
    error: Option<String>,
}

#[derive(Clone)]
struct BlogMcpServer {
    store: Arc<PageStore>,
    tool_router: ToolRouter<BlogMcpServer>,
}

#[tool_router(router = tool_router)]
impl BlogMcpServer {
    fn new(store: Arc<PageStore>) -> Self {
        Self {
            store,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Create a new blog page and return its page_id (page_uid)")]
    async fn push_page(
        &self,
        Parameters(params): Parameters<PushPageRequest>,
    ) -> Result<Json<PushPageResponse>, String> {
        let meta = PageMeta {
            seo: solin_blog::store::SeoMeta {
                seo_title: params.seo_title,
                description: params.description,
                keywords: params.keywords,
                extra: Default::default(),
            },
            page_uid: String::new(),
            created_at: 0,
            updated_at: 0,
            view_count: 0,
            extra: Default::default(),
        };

        if let Err(err) = validate_html(&params.html) {
            return Ok(Json(PushPageResponse {
                success: false,
                page_id: None,
                url: None,
                meta: None,
                error: Some(err.to_string()),
            }));
        }

        match self.store.create_page_auto_uid(&meta, &params.html) {
            Ok(saved_meta) => Ok(Json(PushPageResponse {
                url: Some(build_page_full_url(
                    &resolve_site_url_from_env(),
                    &saved_meta.page_uid,
                    &saved_meta.seo.seo_title,
                )),
                success: true,
                page_id: Some(saved_meta.page_uid.clone()),
                meta: Some(saved_meta.into()),
                error: None,
            })),
            Err(err) => Ok(Json(PushPageResponse {
                success: false,
                page_id: None,
                url: None,
                meta: None,
                error: Some(err.to_string()),
            })),
        }
    }

    #[tool(description = "List all blog page metadata")]
    async fn get_all_page(&self) -> Result<Json<GetAllPageResponse>, String> {
        let entries = match self.store.list_page_entries() {
            Ok(entries) => entries,
            Err(err) => {
                return Ok(Json(GetAllPageResponse {
                    success: false,
                    pages: Vec::new(),
                    error: Some(err.to_string()),
                }));
            }
        };

        let base_url = resolve_site_url_from_env();
        let mut pages = Vec::new();
        for entry in entries {
            let meta = self.store.get_page_meta(&entry.page_id).ok();
            if let Some(meta) = meta {
                let url = build_page_full_url(&base_url, &meta.page_uid, &meta.seo.seo_title);
                pages.push(PageWithMeta {
                    page_id: meta.page_uid.clone(),
                    url,
                    meta: meta.into(),
                });
            }
        }

        Ok(Json(GetAllPageResponse {
            success: true,
            pages,
            error: None,
        }))
    }

    #[tool(description = "Get blog page by page_id (page_uid)")]
    async fn get_page_by_id(
        &self,
        Parameters(params): Parameters<PageIdRequest>,
    ) -> Result<Json<GetPageByIdResponse>, String> {
        let resolved_id = match self.store.resolve_page_id_by_uid(&params.page_id) {
            Ok(Some(id)) => id,
            Ok(None) => {
                return Ok(Json(GetPageByIdResponse {
                    success: false,
                    page: None,
                    error: Some("page not found".to_string()),
                }));
            }
            Err(err) => {
                return Ok(Json(GetPageByIdResponse {
                    success: false,
                    page: None,
                    error: Some(err.to_string()),
                }));
            }
        };

        let base_url = resolve_site_url_from_env();
        match self.store.load_page(&resolved_id) {
            Ok((meta, html)) => Ok(Json(GetPageByIdResponse {
                success: true,
                page: Some(PageWithHtml {
                    page_id: meta.page_uid.clone(),
                    url: build_page_full_url(&base_url, &meta.page_uid, &meta.seo.seo_title),
                    meta: meta.into(),
                    html,
                }),
                error: None,
            })),
            Err(err) => Ok(Json(GetPageByIdResponse {
                success: false,
                page: None,
                error: Some(err.to_string()),
            })),
        }
    }

    #[tool(description = "Delete blog page by page_id (page_uid)")]
    async fn delete_page(
        &self,
        Parameters(params): Parameters<PageIdRequest>,
    ) -> Result<Json<DeletePageResponse>, String> {
        let resolved_id = match self.store.resolve_page_id_by_uid(&params.page_id) {
            Ok(Some(id)) => id,
            Ok(None) => {
                return Ok(Json(DeletePageResponse {
                    success: false,
                    error: Some("page not found".to_string()),
                }));
            }
            Err(err) => {
                return Ok(Json(DeletePageResponse {
                    success: false,
                    error: Some(err.to_string()),
                }));
            }
        };

        match self.store.delete_page(&resolved_id) {
            Ok(_) => Ok(Json(DeletePageResponse {
                success: true,
                error: None,
            })),
            Err(err) => Ok(Json(DeletePageResponse {
                success: false,
                error: Some(err.to_string()),
            })),
        }
    }

    #[tool(description = "Update blog page by page_id (page_uid)")]
    async fn update_page(
        &self,
        Parameters(params): Parameters<UpdatePageRequest>,
    ) -> Result<Json<UpdatePageResponse>, String> {
        let resolved_id = match self.store.resolve_page_id_by_uid(&params.page_id) {
            Ok(Some(id)) => id,
            Ok(None) => {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    url: None,
                    meta: None,
                    error: Some("page not found".to_string()),
                }));
            }
            Err(err) => {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    url: None,
                    meta: None,
                    error: Some(err.to_string()),
                }));
            }
        };

        let (mut meta, mut html) = match self.store.load_page(&resolved_id) {
            Ok(data) => data,
            Err(err) => {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    url: None,
                    meta: None,
                    error: Some(err.to_string()),
                }));
            }
        };

        if let Some(seo_title) = params.seo_title {
            meta.seo.seo_title = seo_title;
        }
        if let Some(description) = params.description {
            meta.seo.description = description;
        }
        if let Some(keywords) = params.keywords {
            meta.seo.keywords = Some(keywords);
        }
        if let Some(new_html) = params.html {
            if let Err(err) = validate_html(&new_html) {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    url: None,
                    meta: None,
                    error: Some(err.to_string()),
                }));
            }
            html = new_html.to_string();
        }

        match self.store.update_page(&resolved_id, &meta, &html) {
            Ok(_) => {
                let (saved_meta, _) = match self.store.load_page(&resolved_id) {
                    Ok(data) => data,
                    Err(err) => {
                        return Ok(Json(UpdatePageResponse {
                            success: false,
                            url: None,
                            meta: None,
                            error: Some(err.to_string()),
                        }));
                    }
                };
                Ok(Json(UpdatePageResponse {
                    success: true,
                    url: Some(build_page_full_url(
                        &resolve_site_url_from_env(),
                        &saved_meta.page_uid,
                        &saved_meta.seo.seo_title,
                    )),
                    meta: Some(saved_meta.into()),
                    error: None,
                }))
            }
            Err(err) => Ok(Json(UpdatePageResponse {
                success: false,
                url: None,
                meta: None,
                error: Some(err.to_string()),
            })),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BlogMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides tools: push_page, get_all_page, get_page_by_id, delete_page, update_page."
                    .to_string(),
            ),
        }
    }
}

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

fn generate_mcp_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = [0u8; 16];
    getrandom(&mut bytes).expect("generate mcp token");
    bytes
        .iter()
        .map(|value| {
            let index = (*value as usize) % CHARSET.len();
            CHARSET[index] as char
        })
        .collect()
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

async fn index_handler(
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

async fn sitemap_handler(
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

async fn page_handler(
    State(store): State<Arc<PageStore>>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let Some(page_id) = parse_page_id_from_slug(&slug) else {
        return (StatusCode::NOT_FOUND, format!("invalid page slug: {slug}")).into_response();
    };
    match store.load_page(&page_id) {
        Ok((meta, html)) => {
            let rendered = render_page_html(&meta, &html);
            if let Err(err) = store.increment_view_count(&page_id) {
                eprintln!("[solin-blog] increment view count failed: {err}");
            }
            Html(rendered).into_response()
        }
        Err(err) => (StatusCode::NOT_FOUND, format!("page not found: {err}")).into_response(),
    }
}

async fn token_generator_handler() -> impl IntoResponse {
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
        panic!("SITE_URL is required to resolve base url when request headers are missing");
    }
    trimmed.to_string()
}

fn resolve_site_url_from_env() -> String {
    let value = std::env::var("SITE_URL").unwrap_or_default();
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        panic!("SITE_URL is required for MCP URL generation");
    }
    trimmed.to_string()
}

fn build_page_full_url(base_url: &str, page_id: &str, seo_title: &str) -> String {
    let path = solin_blog::web::build_page_url(page_id, seo_title);
    format!("{}{}", base_url.trim_end_matches('/'), path)
}
