//! MCP（Model Context Protocol）模块
//!
//! 提供 AI 客户端连接的工具接口

pub mod auth;
pub mod dto;
pub mod tools;
pub mod utils;

pub use auth::{TokenStore, extract_bearer_token, mcp_auth_middleware};
pub use dto::*;
pub use tools::BlogMcpServer;
pub use utils::{build_page_full_url, resolve_site_url_from_env};
