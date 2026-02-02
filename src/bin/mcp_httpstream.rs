use rmcp::{
    ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        tool::Parameters,
        wrapper::Json,
    },
    model::{
        Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use SolinBlog::store::{PageMeta, PageStore, validate_html};

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
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PushPageResponse {
    success: bool,
    page_id: Option<String>,
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
    meta: Option<PageMetaResponse>,
    error: Option<String>,
}

#[derive(Clone)]
struct BlogMcpServer {
    store: PageStore,
    tool_router: ToolRouter<BlogMcpServer>,
}

#[tool_router(router = tool_router)]
impl BlogMcpServer {
    fn new(store: PageStore) -> Self {
        Self {
            store,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Create a new page and return its page_id (page_uid)")]
    async fn push_page(
        &self,
        Parameters(params): Parameters<PushPageRequest>,
    ) -> Result<Json<PushPageResponse>, String> {
        let meta = PageMeta {
            seo: SolinBlog::store::SeoMeta {
                seo_title: params.seo_title,
                description: params.description,
                keywords: params.keywords,
                extra: Default::default(),
            },
            page_uid: String::new(),
            created_at: 0,
            updated_at: 0,
            extra: Default::default(),
        };

        if let Err(err) = validate_html(&params.html) {
            return Ok(Json(PushPageResponse {
                success: false,
                page_id: None,
                meta: None,
                error: Some(err.to_string()),
            }));
        }

        match self.store.create_page_auto_uid(&meta, &params.html) {
            Ok(saved_meta) => Ok(Json(PushPageResponse {
                success: true,
                page_id: Some(saved_meta.page_uid.clone()),
                meta: Some(saved_meta.into()),
                error: None,
            })),
            Err(err) => Ok(Json(PushPageResponse {
                success: false,
                page_id: None,
                meta: None,
                error: Some(err.to_string()),
            })),
        }
    }

    #[tool(description = "List all page metadata")]
    async fn get_all_page(&self) -> Result<Json<GetAllPageResponse>, String> {
        let entries = match self.store.list_page_entries() {
            Ok(entries) => entries,
            Err(err) => {
                return Ok(Json(GetAllPageResponse {
                    success: false,
                    pages: Vec::new(),
                    error: Some(err.to_string()),
                }))
            }
        };

        let mut pages = Vec::new();
        for entry in entries {
            let meta = self.store.get_page_meta(&entry.page_id).ok();
            if let Some(meta) = meta {
                pages.push(PageWithMeta {
                    page_id: meta.page_uid.clone(),
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

    #[tool(description = "Get page by page_id (page_uid)")]
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
                }))
            }
            Err(err) => {
                return Ok(Json(GetPageByIdResponse {
                    success: false,
                    page: None,
                    error: Some(err.to_string()),
                }))
            }
        };

        match self.store.load_page(&resolved_id) {
            Ok((meta, html)) => Ok(Json(GetPageByIdResponse {
                success: true,
                page: Some(PageWithHtml {
                    page_id: meta.page_uid.clone(),
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

    #[tool(description = "Delete page by page_id (page_uid)")]
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
                }))
            }
            Err(err) => {
                return Ok(Json(DeletePageResponse {
                    success: false,
                    error: Some(err.to_string()),
                }))
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

    #[tool(description = "Update page by page_id (page_uid)")]
    async fn update_page(
        &self,
        Parameters(params): Parameters<UpdatePageRequest>,
    ) -> Result<Json<UpdatePageResponse>, String> {
        let resolved_id = match self.store.resolve_page_id_by_uid(&params.page_id) {
            Ok(Some(id)) => id,
            Ok(None) => {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    meta: None,
                    error: Some("page not found".to_string()),
                }))
            }
            Err(err) => {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    meta: None,
                    error: Some(err.to_string()),
                }))
            }
        };

        let (mut meta, mut html) = match self.store.load_page(&resolved_id) {
            Ok(data) => data,
            Err(err) => {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    meta: None,
                    error: Some(err.to_string()),
                }))
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
                            meta: None,
                            error: Some(err.to_string()),
                        }))
                    }
                };
                Ok(Json(UpdatePageResponse {
                    success: true,
                    meta: Some(saved_meta.into()),
                    error: None,
                }))
            }
            Err(err) => Ok(Json(UpdatePageResponse {
                success: false,
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
async fn main() -> anyhow::Result<()> {
    let store = PageStore::default();
    let server = BlogMcpServer::new(store);
    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.expect("listen for ctrl-c");
        })
        .await;
    Ok(())
}
