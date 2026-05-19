//! MCP 认证中间件

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Token 存储结构
pub struct TokenStore {
    pub valid_tokens: Vec<String>,
}

impl TokenStore {
    pub fn new(valid_tokens: Vec<String>) -> Self {
        Self { valid_tokens }
    }

    pub fn is_valid(&self, token: &str) -> bool {
        self.valid_tokens.contains(&token.to_string())
    }
}

/// 从 Authorization 头提取 Bearer Token
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| auth_header.strip_prefix("Bearer ").map(|s| s.to_string()))
}

/// MCP 认证中间件
pub async fn mcp_auth_middleware(
    State(token_store): State<Arc<TokenStore>>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    match extract_bearer_token(&headers) {
        Some(token) if token_store.is_valid(&token) => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
