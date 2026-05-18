# 图谱报告 - .（2026-05-19）

## 语料检查

- 大型语料：39 个文件，约 580,609 词。语义抽取会消耗较多模型 token；如需进一步精细分析，可考虑对子目录单独运行，或只运行 AST 结构抽取。

## 摘要

- 270 个节点，427 条边，24 个社区（报告展示 22 个，省略 2 个过薄社区）。
- 抽取来源：86% EXTRACTED，13% INFERRED，1% AMBIGUOUS；INFERRED 共 56 条边，平均置信度 0.81。
- Token 成本：0 input，0 output。

## 社区导航

- 前端模板
- HTML 渲染工具
- 页面存储持久化
- MCP 服务端 API
- 部署配置
- API 数据类型
- 测试与路由
- 已发布 MCP 工具
- 图片搜索
- 存储自检
- MCP 认证
- 构建期图片转换
- 服务启动
- Rust 项目实践

## God Nodes（最核心抽象）

1. `PageStore` - 27 **条边**
2. `validate_html()` - 14 条边
3. `BlogMcpServer` - 12 条边
4. `sanitize_page_id()` - 11 条边
5. `Home Page Template` - 11 条边
6. `Docker Compose SolinBlog Service` - 10 条边
7. `Published MCP Tools` - 10 条边
8. `inject_seo_meta()` - 9 条边
9. `Reusable Site Header` - 9 条边
10. `generate_unique_page_uid()` - 8 条边

## 意外连接（可能是你没意识到的跨域关系）

- `macOS LaunchAgent Deployment` --semantically_similar_to--> `Docker Deployment Guide` [INFERRED] [语义相似]
  来源：just/just.md -> DOCKER.md
- `Required Placeholder Contract` --conceptually_related_to--> `Front Template Runtime Mount` [INFERRED]
  来源：front/README.md -> DOCKER.md
- `Light Theme Background Image Asset` --references--> `light.png Light Background Asset` [AMBIGUOUS]
  来源：public/light0.png -> public/README.md
- `Dark Theme Background Image Asset` --references--> `night.png Dark Background Asset` [AMBIGUOUS]
  来源：public/night0.png -> public/README.md
- `get_html_style HTML Template Tool` --implements--> `get_html_style Tool` [INFERRED]
  来源：public/prompt/README.md -> README.md

## 超边（组关系）

- **Docker 运行时配置**：dockercompose_solinblog_service、dockercompose_env_file、dockercompose_web_host、dockercompose_port_3002、dockercompose_data_volume、dockercompose_front_volume、dockercompose_public_volume [EXTRACTED 1.00]
- **页面 CRUD MCP 工具**：README_create_page_tool、README_update_page_tool、README_update_markdown_page_tool、README_delete_page_tool、README_list_pages_tool、README_get_page_tool、gongnengshu_pagestore [INFERRED 0.86]
- **前端模板占位符系统**：index_home_template、frontreadme_placeholder_contract、index_site_title_placeholder、index_site_subtitle_placeholder、index_page_list_placeholder、index_beian_number_placeholder、frontreadme_string_replacement_templates [EXTRACTED 1.00]

## 社区（共 24 个，省略 2 个过薄社区）

### 社区 0 - “前端模板”

凝聚度：0.06

节点（43）：get_html_style Tool、404 Return Home Link、404 Light and Dark Theme CSS、404 Not Found Page、404 Site Header Placeholder、Required Placeholder Contract、String Replacement Template Mechanism、Home Template Editing Guide（另有 35 个）

### 社区 1 - “HTML 渲染工具”

凝聚度：0.13

节点（31）：build_page_url()、escape_html()、escape_html_attr()、find_bytes_ci()、find_html_tag_end()、find_tag_end()、format_display_timestamp()、format_unix_timestamp()（另有 23 个）

### 社区 2 - “页面存储持久化”

凝聚度：0.19

节点（6）：atomic_write()、generate_unique_page_uid()、now_unix_seconds()、PageStore、sanitize_page_id()、to_url_slug()

### 社区 3 - “MCP 服务端 API”

凝聚度：0.13

节点（15）：BlogMcpServer、build_page_full_url()、resolve_site_url_from_env()、find_bytes()、find_bytes_case_insensitive()、find_tag_end()、generate_page_uid()、is_self_closing()（另有 7 个）

### 社区 4 - “部署配置”

凝聚度：0.07

节点（28）：AI Native Blog Project、Cargo Check Verification Policy、Data Persistence、Docker Deployment Guide、Environment Variable Configuration、Front Template Runtime Mount、MCP Endpoint /mcp、MCP_TOKEN Bearer Authentication Token（另有 20 个）

### 社区 5 - “API 数据类型”

凝聚度：0.08

节点（23）：BlogStyle、DeletePageResponse、GetAllPageRequest、GetAllPageResponse、GetBlogStyleRequest、GetBlogStyleResponse、GetHtmlStyleRequest、GetHtmlStyleResponse（另有 15 个）

### 社区 6 - “测试与路由”

凝聚度：0.12

节点（24）：search_images Tool、Data Storage Layer Tests、HTML Validation Tests、Image Search Tests、Manual Shell Test Scripts、MCP Interface Tests、Store CRUD Selfcheck、SolinBlog Test Guide（另有 16 个）

### 社区 7 - “已发布 MCP 工具”

凝聚度：0.18

节点（11）：create_page Tool、delete_page Tool、get_blog_style Tool、get_page Tool、list_pages Tool、Published MCP Tools、update_markdown_page Tool、update_page Tool（另有 3 个）

### 社区 8 - “图片搜索”

凝聚度：0.28

节点（8）：ImageSearchItem、ImageSearchResponse、ImageSearchResult、resolve_searxng_url()、search_images()、search_single()、SearxImageResult、SearxResponse

### 社区 9 - “存储自检”

凝聚度：0.33

节点（3）：IndexSnapshotGuard、main()、PageDirGuard

### 社区 10 - “MCP 认证”

凝聚度：0.47

节点（3）：extract_bearer_token()、mcp_auth_middleware()、TokenStore

### 社区 11 - “构建期图片转换”

凝聚度：0.60

节点（5）：convert_special_images()、convert_to_png()、find_first_source()、main()、read_first_matching_file()

## 需要人工复核的模糊边

- `icon.png Favicon Asset` -> `Favicon Image Asset` [AMBIGUOUS]
  来源：public/icon.png；关系：references
- `light.png Light Background Asset` -> `Light Theme Background Image Asset` [AMBIGUOUS]
  来源：public/light0.png；关系：references
- `night.png Dark Background Asset` -> `Dark Theme Background Image Asset` [AMBIGUOUS]
  来源：public/night0.png；关系：references

## 知识缺口

- **68 个孤立节点**：`ImageSearchItem`、`ImageSearchResult`、`ImageSearchResponse`、`SearxResponse`、`SearxImageResult`（另有 63 个）。这些节点连接数小于等于 1，可能意味着缺少边，或者相关组件文档不足。
- **2 个过薄社区（少于 3 个节点）未展示**：可通过 `graphify query` 继续探索这些孤立节点。

## 建议追问

- **`icon.png Favicon Asset` 和 `Favicon Image Asset` 的准确关系是什么？**
  该边被标记为 AMBIGUOUS，关系为 references，置信度较低。
- **`light.png Light Background Asset` 和 `Light Theme Background Image Asset` 的准确关系是什么？**
  该边被标记为 AMBIGUOUS，关系为 references，置信度较低。
- **`night.png Dark Background Asset` 和 `Dark Theme Background Image Asset` 的准确关系是什么？**
  该边被标记为 AMBIGUOUS，关系为 references，置信度较低。
- **为什么 `validate_html()` 会把 `MCP Server API` 和 `Page Store Persistence` 连接起来？**
  该节点具有较高的中介中心性（0.078），是跨社区桥接节点。
- **为什么 `Published MCP Tools` 会连接 `Published MCP Tools`、`Frontend Templates`、`Deployment Configuration`、`Testing And Routes`？**
  该节点具有较高的中介中心性（0.077），是跨社区桥接节点。
- **为什么 `render_markdown_page()` 会连接 `HTML Rendering Utilities` 和 `MCP Server API`？**
  该节点具有较高的中介中心性（0.071），是跨社区桥接节点。
- **围绕 `validate_html()` 的 4 条推断关系是否真的正确？**
  `validate_html()` 有 4 条 INFERRED 边，属于模型推理出的连接，需要人工验证。
