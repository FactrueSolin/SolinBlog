use getrandom::getrandom;

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

pub fn generate_mcp_token() -> String {
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
