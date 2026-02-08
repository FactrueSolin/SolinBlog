use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
