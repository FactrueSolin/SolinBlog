use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool_handler,
};

use crate::store::PageStore;

#[derive(Clone)]
pub struct BlogMcpServer {
    pub(crate) store: Arc<PageStore>,
    pub(crate) tool_router: ToolRouter<BlogMcpServer>,
}

impl BlogMcpServer {
    pub fn new(store: Arc<PageStore>) -> Self {
        Self {
            store,
            tool_router: Self::build_tool_router(),
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
                "This server provides tools: push_page, push_markdown, get_all_page, get_page_by_id, delete_page, update_page, update_markdown_page, get_blog_style, get_html_style."
                    .to_string(),
            ),
        }
    }
}
