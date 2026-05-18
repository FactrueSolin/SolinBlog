//! Web 模块
//!
//! 提供网页渲染和请求处理功能

pub mod config;
pub mod handlers;
pub mod image_handlers;

pub use config::{generate_mcp_token, resolve_base_url};
pub use handlers::{
    index_handler, log_request, page_handler, public_asset_handler, sitemap_handler,
    token_generator_handler,
};
pub use image_handlers::{
    ImageWebState, delete_image_handler, get_image_handler, image_asset_handler,
    image_auth_middleware, image_page_handler, list_images_handler, replace_image_handler,
    update_image_handler, upload_image_handler,
};

// 重新导出原有的 web 功能
pub use crate::web_core::{
    build_page_url, inject_seo_meta, inject_umami_script, markdown_to_html,
    parse_page_id_from_slug, render_404_html, render_index_html, render_markdown_page,
    render_page_html, render_sitemap_xml,
};

// validate_html 在 store 模块中
pub use crate::store::validate_html;
