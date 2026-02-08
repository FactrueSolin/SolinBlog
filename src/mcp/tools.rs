use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::{Json, Parameters}},
    model::{CallToolResult, Content},
    tool, tool_router,
};

use crate::{
    config::resolve_site_url_from_env,
    mcp::{
        dto::{
            BlogStyle, DeletePageResponse, GetAllPageRequest, GetAllPageResponse,
            GetBlogStyleRequest, GetHtmlStyleRequest, GetPageByIdRequest, GetPageByIdResponse,
            HtmlStyleType, PageIdRequest, PageWithHtml, PageWithMeta, PushMarkdownRequest,
            PushPageRequest, PushPageResponse, UpdateMarkdownPageRequest, UpdatePageRequest,
            UpdatePageResponse,
        },
        server::BlogMcpServer,
    },
    store::{PageMeta, SeoMeta, validate_html},
    web::{build_page_url, render_markdown_page},
};

#[tool_router(router = tool_router)]
impl BlogMcpServer {
    pub(crate) fn build_tool_router() -> ToolRouter<BlogMcpServer> {
        Self::tool_router()
    }

    #[tool(description = "Create a new blog page and return its page_id (page_uid)")]
    async fn push_page(
        &self,
        Parameters(params): Parameters<PushPageRequest>,
    ) -> Result<Json<PushPageResponse>, String> {
        let meta = PageMeta {
            seo: SeoMeta {
                title: params.seo_title.clone(),
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

    #[tool(description = "推送一篇 Markdown 格式的博客文章")]
    async fn push_markdown(
        &self,
        Parameters(req): Parameters<PushMarkdownRequest>,
    ) -> Result<Json<PushPageResponse>, String> {
        let html = match render_markdown_page(&req.markdown) {
            Ok(rendered) => rendered,
            Err(err) => {
                return Ok(Json(PushPageResponse {
                    success: false,
                    page_id: None,
                    url: None,
                    meta: None,
                    error: Some(err.to_string()),
                }));
            }
        };

        if let Err(err) = validate_html(&html) {
            return Ok(Json(PushPageResponse {
                success: false,
                page_id: None,
                url: None,
                meta: None,
                error: Some(err.to_string()),
            }));
        }

        let meta = PageMeta {
            seo: SeoMeta {
                title: req.seo_title.clone(),
                seo_title: req.seo_title,
                description: req.description,
                keywords: req.keywords,
                extra: Default::default(),
            },
            page_uid: String::new(),
            created_at: 0,
            updated_at: 0,
            view_count: 0,
            extra: Default::default(),
        };

        match self
            .store
            .create_page_auto_uid_with_markdown(&meta, &html, Some(&req.markdown))
        {
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
    async fn get_all_page(
        &self,
        Parameters(_params): Parameters<GetAllPageRequest>,
    ) -> Result<Json<GetAllPageResponse>, String> {
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

    #[tool(
        description = "Get blog pages by page_id list (page_uid). Supports single page_id for backward compatibility"
    )]
    async fn get_page_by_id(
        &self,
        Parameters(params): Parameters<GetPageByIdRequest>,
    ) -> Result<Json<GetPageByIdResponse>, String> {
        let mut ids = Vec::new();
        if let Some(single_id) = params.page_id {
            if !single_id.trim().is_empty() {
                ids.push(single_id);
            }
        }
        if let Some(more_ids) = params.ids {
            ids.extend(more_ids.into_iter().filter(|id| !id.trim().is_empty()));
        }

        if ids.is_empty() {
            return Ok(Json(GetPageByIdResponse {
                success: false,
                pages: Vec::new(),
                error: Some("ids is empty".to_string()),
            }));
        }

        let base_url = resolve_site_url_from_env();
        let mut pages = Vec::new();
        let mut errors = Vec::new();

        for page_id in ids {
            let resolved_id = match self.store.resolve_page_id_by_uid(&page_id) {
                Ok(Some(id)) => id,
                Ok(None) => {
                    errors.push(format!("page not found: {page_id}"));
                    continue;
                }
                Err(err) => {
                    errors.push(format!("resolve page failed: {page_id}: {err}"));
                    continue;
                }
            };

            match self.store.load_page(&resolved_id) {
                Ok((meta, html)) => pages.push(PageWithHtml {
                    page_id: meta.page_uid.clone(),
                    url: build_page_full_url(&base_url, &meta.page_uid, &meta.seo.seo_title),
                    meta: meta.into(),
                    html,
                }),
                Err(err) => errors.push(format!("load page failed: {page_id}: {err}")),
            }
        }

        Ok(Json(GetPageByIdResponse {
            success: errors.is_empty(),
            pages,
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        }))
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

    #[tool(description = "Update markdown blog page by page_id (page_uid)")]
    async fn update_markdown_page(
        &self,
        Parameters(params): Parameters<UpdateMarkdownPageRequest>,
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
        let mut markdown_source: Option<String> = None;
        if let Some(markdown) = params.markdown {
            let rendered = match render_markdown_page(&markdown) {
                Ok(rendered) => rendered,
                Err(err) => {
                    return Ok(Json(UpdatePageResponse {
                        success: false,
                        url: None,
                        meta: None,
                        error: Some(err.to_string()),
                    }));
                }
            };
            if let Err(err) = validate_html(&rendered) {
                return Ok(Json(UpdatePageResponse {
                    success: false,
                    url: None,
                    meta: None,
                    error: Some(err.to_string()),
                }));
            }
            html = rendered;
            markdown_source = Some(markdown);
        }

        match self.store.update_page_with_markdown(
            &resolved_id,
            &meta,
            &html,
            markdown_source.as_deref(),
        ) {
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

    #[tool(name = "get_blog_style", description = "获取指定的博文写作风格指南")]
    async fn get_blog_style(
        &self,
        Parameters(params): Parameters<GetBlogStyleRequest>,
    ) -> Result<CallToolResult, McpError> {
        let style = &params.style;
        let content = match style {
            BlogStyle::PplxStyle => std::fs::read_to_string("public/prompt/PPLX.xml")
                .map_err(|err| McpError::internal_error(format!("读取文件失败: {err}"), None))?,
        };
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(
        name = "get_html_style",
        description = "获取 HTML 风格参考，1. 用户未指定样式，则默认为default。2. 在制作HTML博文时需先获得参考样式"
    )]
    async fn get_html_style(
        &self,
        Parameters(params): Parameters<GetHtmlStyleRequest>,
    ) -> Result<CallToolResult, McpError> {
        let style = &params.style;
        let template = match style {
            HtmlStyleType::Default => std::fs::read_to_string("public/prompt/HTML.xml")
                .map_err(|err| McpError::internal_error(format!("读取文件失败: {err}"), None))?,
        };
        let example_css = std::fs::read_to_string("front/example.css")
            .map_err(|err| McpError::internal_error(format!("读取文件失败: {err}"), None))?;
        let example_html = std::fs::read_to_string("front/index.html")
            .map_err(|err| McpError::internal_error(format!("读取文件失败: {err}"), None))?;
        let content = template
            .replace("{{EXAMPLE_CSS}}", &example_css)
            .replace("{{EXAMPLE_HTML}}", &example_html);
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }
}

pub(crate) fn build_page_full_url(base_url: &str, page_id: &str, seo_title: &str) -> String {
    let path = build_page_url(page_id, seo_title);
    format!("{}{}", base_url.trim_end_matches('/'), path)
}
