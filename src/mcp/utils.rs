//! MCP 工具相关的辅助函数

/// 从环境变量解析站点 URL
pub fn resolve_site_url_from_env() -> String {
    let value = std::env::var("SITE_URL").unwrap_or_default();
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        eprintln!(
            "[solin-blog] WARNING: SITE_URL is not set, MCP response URLs will be relative paths"
        );
        return String::new();
    }
    trimmed.to_string()
}

/// 构建页面完整 URL
pub fn build_page_full_url(base_url: &str, page_id: &str, seo_title: &str) -> String {
    let path = crate::web::build_page_url(page_id, seo_title);
    format!("{}{}", base_url.trim_end_matches('/'), path)
}
