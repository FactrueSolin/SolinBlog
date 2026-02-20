//! Web 配置和环境变量管理

use axum::http::HeaderMap;

/// 从请求头或环境变量解析基础 URL
pub fn resolve_base_url(headers: &HeaderMap) -> String {
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
        eprintln!(
            "[solin-blog] WARNING: SITE_URL is not set and request headers missing host, sitemap URLs will be relative"
        );
        return String::new();
    }
    trimmed.to_string()
}

/// 生成 MCP Token
pub fn generate_mcp_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("generate mcp token");
    bytes
        .iter()
        .map(|value| {
            let index = (*value as usize) % CHARSET.len();
            CHARSET[index] as char
        })
        .collect()
}
