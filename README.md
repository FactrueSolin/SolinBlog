# SolinBlog

## 项目介绍

AI 原生博客，开放 MCP 接口供 AI 连接，直接发布原始 HTML 页面。

## 功能介绍

- `create_page` - 创建页面
- `update_page` - 更新页面（HTML）
- `update_markdown_page` - 更新页面（Markdown）
- `delete_page` - 删除页面
- `list_pages` - 列出所有页面
- `get_page` - 获取页面内容
- `search_images` - 图片搜索（SearXNG）
- `get_blog_style` - 获取博文风格模板
- `get_html_style` - 获取 HTML 网页风格模板

## 部署教程

仅支持 Docker 部署，参考 [`docker-compose.yml`](docker-compose.yml:1) 与 [`DOCKER.md`](DOCKER.md:1)。
