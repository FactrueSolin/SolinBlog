//! MCP 工具相关的请求/响应数据结构

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::image_host::ImageMeta;
use crate::store::PageMeta;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PushPageRequest {
    pub seo_title: String,
    pub description: String,
    pub keywords: Option<Vec<String>>,
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PushMarkdownRequest {
    pub seo_title: String,
    pub description: String,
    pub keywords: Option<Vec<String>>,
    pub markdown: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SeoMetaResponse {
    pub seo_title: String,
    pub description: String,
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PageMetaResponse {
    pub seo: SeoMetaResponse,
    pub page_uid: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub view_count: u64,
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
pub struct PushPageResponse {
    pub success: bool,
    pub page_id: Option<String>,
    pub url: Option<String>,
    pub meta: Option<PageMetaResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetAllPageResponse {
    pub success: bool,
    pub pages: Vec<PageWithMeta>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetAllPageRequest {
    /// 预留参数，保持 schema 的 properties 非空
    pub reserved: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PageWithMeta {
    pub page_id: String,
    pub url: String,
    pub meta: PageMetaResponse,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PageIdRequest {
    pub page_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetPageByIdRequest {
    pub page_id: Option<String>,
    pub ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetPageByIdResponse {
    pub success: bool,
    pub pages: Vec<PageWithHtml>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PageWithHtml {
    pub page_id: String,
    pub url: String,
    pub meta: PageMetaResponse,
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeletePageResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdatePageRequest {
    pub page_id: String,
    pub seo_title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateMarkdownPageRequest {
    pub page_id: String,
    pub seo_title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub markdown: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdatePageResponse {
    pub success: bool,
    pub url: Option<String>,
    pub meta: Option<PageMetaResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ImageSearchRequest {
    pub keywords: Vec<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlogStyle {
    PplxStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HtmlStyleType {
    Default,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBlogStyleRequest {
    /// 博文风格类型
    pub style: BlogStyle,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetHtmlStyleRequest {
    /// HTML 风格类型
    pub style: HtmlStyleType,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetBlogStyleResponse {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetHtmlStyleResponse {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct McpImageMeta {
    pub image_id: String,
    pub filename: String,
    pub url: String,
    pub relative_path: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub alt: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<ImageMeta> for McpImageMeta {
    fn from(meta: ImageMeta) -> Self {
        Self {
            image_id: meta.image_id,
            filename: meta.filename,
            url: meta.url,
            relative_path: meta.relative_path,
            content_type: meta.content_type,
            size_bytes: meta.size_bytes,
            width: meta.width,
            height: meta.height,
            sha256: meta.sha256,
            alt: meta.alt,
            description: meta.description,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct McpImageError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListImagesRequest {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub q: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListImagesResponse {
    pub success: bool,
    pub images: Vec<McpImageMeta>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub error: Option<McpImageError>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetImageRequest {
    pub image_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetImageResponse {
    pub success: bool,
    pub image: Option<McpImageMeta>,
    pub error: Option<McpImageError>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateImageRequest {
    pub image_id: String,
    pub alt: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateImageResponse {
    pub success: bool,
    pub image: Option<McpImageMeta>,
    pub error: Option<McpImageError>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteImageRequest {
    pub image_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteImageResponse {
    pub success: bool,
    pub image_id: Option<String>,
    pub error: Option<McpImageError>,
}
