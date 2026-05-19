//! SolinBlog - AI 原生博客系统
//!
//! 这是一个基于 Rust 的博客系统，提供 MCP（Model Context Protocol）接口供 AI 客户端连接。
//!
//! ## 模块结构
//!
//! - [`store`] - 页面存储和管理
//! - [`web_core`] - 网页渲染核心功能
//! - [`web`] - Web 请求处理和配置
//! - [`mcp`] - MCP 工具接口
//! - [`image`] - 图片搜索功能
//! - [`image_host`] - 图床托管功能

pub mod store;
pub mod web_core;
pub mod web;
pub mod mcp;
pub mod image;
pub mod image_host;
pub mod openapi;
