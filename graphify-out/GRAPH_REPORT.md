# Graph Report - SolinBlog  (2026-05-19)

## Corpus Check
- 41 files · ~591,328 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 590 nodes · 884 edges · 47 communities (44 shown, 3 thin omitted)
- Extraction: 92% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 65 edges (avg confidence: 0.81)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d454b1cd`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]

## God Nodes (most connected - your core abstractions)
1. `PageStore` - 28 edges
2. `图床功能架构设计` - 16 edges
3. `validate_html()` - 15 edges
4. `ImageStore` - 13 edges
5. `BlogMcpServer` - 13 edges
6. `sanitize_page_id()` - 12 edges
7. `SolinBlog 测试文档` - 12 edges
8. `后端 API 规范` - 12 edges
9. `四、MCP 接口测试` - 11 edges
10. `前端页面设计` - 11 edges

## Surprising Connections (you probably didn't know these)
- `image_page_handler()` --calls--> `inject_umami_script()`  [INFERRED]
  src/web/image_handlers.rs → C:/Users/Daifuku/Sync/SolinBlog/src/web_core.rs
- `macOS LaunchAgent Deployment` --semantically_similar_to--> `Docker Deployment Guide`  [INFERRED] [semantically similar]
  just/just.md → DOCKER.md
- `Required Placeholder Contract` --conceptually_related_to--> `Front Template Runtime Mount`  [INFERRED]
  front/README.md → DOCKER.md
- `Light Theme Background Image Asset` --references--> `light.png Light Background Asset`  [AMBIGUOUS]
  public/light0.png → public/README.md
- `Dark Theme Background Image Asset` --references--> `night.png Dark Background Asset`  [AMBIGUOUS]
  public/night0.png → public/README.md

## Hyperedges (group relationships)
- **Docker Runtime Configuration** — dockercompose_solinblog_service, dockercompose_env_file, dockercompose_web_host, dockercompose_port_3002, dockercompose_data_volume, dockercompose_front_volume, dockercompose_public_volume [EXTRACTED 1.00]
- **Page CRUD MCP Tools** — README_create_page_tool, README_update_page_tool, README_update_markdown_page_tool, README_delete_page_tool, README_list_pages_tool, README_get_page_tool, gongnengshu_pagestore [INFERRED 0.86]
- **Front Template Placeholder System** — index_home_template, frontreadme_placeholder_contract, index_site_title_placeholder, index_site_subtitle_placeholder, index_page_list_placeholder, index_beian_number_placeholder, frontreadme_string_replacement_templates [EXTRACTED 1.00]

## Communities (47 total, 3 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.06
Nodes (43): get_html_style Tool, 404 Return Home Link, 404 Light and Dark Theme CSS, 404 Not Found Page, 404 Site Header Placeholder, Required Placeholder Contract, String Replacement Template Mechanism, Home Template Editing Guide (+35 more)

### Community 1 - "Community 1"
Cohesion: 0.17
Nodes (32): build_page_url(), escape_html(), escape_html_attr(), escape_xml(), find_bytes_ci(), find_html_tag_end(), find_tag_end(), format_display_timestamp() (+24 more)

### Community 2 - "Community 2"
Cohesion: 0.13
Nodes (18): atomic_write(), find_bytes(), find_bytes_case_insensitive(), find_tag_end(), generate_page_uid(), generate_unique_page_uid(), is_self_closing(), is_void_element() (+10 more)

### Community 3 - "Community 3"
Cohesion: 0.22
Nodes (3): BlogMcpServer, build_page_full_url(), resolve_site_url_from_env()

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (32): AI Native Blog Project, Data Persistence, Docker Deployment Guide, Environment Variable Configuration, Front Template Runtime Mount, MCP Endpoint /mcp, MCP_TOKEN Bearer Authentication Token, Non Root Container User UID 10001 (+24 more)

### Community 5 - "Community 5"
Cohesion: 0.14
Nodes (23): BlogStyle, DeletePageResponse, GetAllPageRequest, GetAllPageResponse, GetBlogStyleRequest, GetBlogStyleResponse, GetHtmlStyleRequest, GetHtmlStyleResponse (+15 more)

### Community 6 - "Community 6"
Cohesion: 0.08
Nodes (31): Cargo Check Verification Policy, search_images Tool, Compile Checks, Data Storage Layer Tests, HTML Validation Tests, Image Search Tests, Manual Shell Test Scripts, MCP Interface Tests (+23 more)

### Community 7 - "Community 7"
Cohesion: 0.04
Nodes (47): 1. 环境变量, 2. 启动服务, 3.1 首页 — `GET /`, 3.2 文章页 — `GET /pages/{slug}`, 3.3 Sitemap — `GET /sitemap.xml`, 3.4 Token 生成器 — `GET /tools/token-generator`, 3.5 静态资源 — `GET /public/{path}`, 3. 测试脚本 (+39 more)

### Community 8 - "Community 8"
Cohesion: 0.40
Nodes (8): ImageSearchItem, ImageSearchResponse, ImageSearchResult, resolve_searxng_url(), search_images(), search_single(), SearxImageResult, SearxResponse

### Community 9 - "Community 9"
Cohesion: 0.38
Nodes (3): IndexSnapshotGuard, main(), PageDirGuard

### Community 10 - "Community 10"
Cohesion: 0.09
Nodes (29): main(), extract_bearer_token(), mcp_auth_middleware(), TokenStore, build_openapi_json(), generate_mcp_token(), generate_token(), resolve_base_url() (+21 more)

### Community 11 - "Community 11"
Cohesion: 0.67
Nodes (5): convert_special_images(), convert_to_png(), find_first_source(), main(), read_first_matching_file()

### Community 12 - "Community 12"
Cohesion: 0.06
Nodes (34): code:text (data/), code:json ({), code:rust (let protected_image_api = Router::new()), code:rust (.route("/image", get(image_page_handler))), code:json ({), MCP 工具扩展, UX 目标, 上传区设计 (+26 more)

### Community 24 - "Community 24"
Cohesion: 0.06
Nodes (34): 1. 快速开始（Docker Compose 一键部署）, 2.1 全部支持的环境变量, 2.2 配置示例, 2.3 MCP 连接方式, 2. 环境变量配置, 3.1 Compose 挂载方式, 3.2 备份建议, 3.3 权限注意 (+26 more)

### Community 25 - "Community 25"
Cohesion: 0.11
Nodes (17): atomic_write(), build_relative_path(), DateTimeMonth, generate_image_id(), generate_unique_image_id(), ImageFileInfo, ImageHostError, ImageIndex (+9 more)

### Community 26 - "Community 26"
Cohesion: 0.08
Nodes (24): 1. 功能目录, 2. 核心功能实现流程图, 3.1 MCP 接口层 (`main.rs`), 3.2 数据存储层 (`store.rs`), 3.3 渲染层 (`web.rs`), 3.4 图片搜索 (`image.rs`), 3. 核心功能实现文字说明, 4.1 HTML 校验流程 (+16 more)

### Community 27 - "Community 27"
Cohesion: 0.25
Nodes (16): assert_json_eq(), assert_json_nonempty(), assert_status(), detect_cargo(), detect_python(), fail(), json_value(), log() (+8 more)

### Community 28 - "Community 28"
Cohesion: 0.11
Nodes (18): code:http (Authorization: Bearer <TOKEN>), code:json ({), code:json ({), code:json ({), code:json ({), code:json ({), code:json ({), code:json ({) (+10 more)

### Community 29 - "Community 29"
Cohesion: 0.12
Nodes (16): code:bash (just), code:bash (just status), code:bash (just logs), code:bash (just undeploy), code:bash (just check), code:bash (just test-image-api), code:bash (just build), code:bash (just run) (+8 more)

### Community 30 - "Community 30"
Cohesion: 0.15
Nodes (12): 1. 文件用途说明, 2.1 占位符的硬性规则, 2. 必须保留的占位符（不可删除/不可改名）, 3. 可以自由修改的内容, 4.1 修改样式（推荐：只动 CSS，不动占位符）, 4.2 调整布局（可移动占位符，但不要拆分/包裹错误）, 4.3 关于 `<title>` 与 SEO, 4. 修改示例与建议 (+4 more)

### Community 31 - "Community 31"
Cohesion: 0.22
Nodes (8): Blog Style Prompts, HTML 样板：`get_html_style` MCP tool, 占位符替换机制, 参数, 扩展新的 HTML 样式类型, 文件与枚举值对应关系, 文件依赖关系, 添加新风格

### Community 32 - "Community 32"
Cohesion: 0.22
Nodes (8): build.rs 自动图片转换说明, code:block1 (public/<path>  =>  /<path>), public 目录说明, URL 映射规则, 使用示例, 注意事项, 特殊文件说明（可选）, 目录用途

### Community 33 - "Community 33"
Cohesion: 0.40
Nodes (4): SolinBlog, 功能介绍, 部署教程, 项目介绍

## Ambiguous Edges - Review These
- `icon.png Favicon Asset` → `Favicon Image Asset`  [AMBIGUOUS]
  public/icon.png · relation: references
- `light.png Light Background Asset` → `Light Theme Background Image Asset`  [AMBIGUOUS]
  public/light0.png · relation: references
- `night.png Dark Background Asset` → `Dark Theme Background Image Asset`  [AMBIGUOUS]
  public/night0.png · relation: references

## Knowledge Gaps
- **191 isolated node(s):** `ImageRecord`, `ImageIndex`, `ImageMetaPatch`, `PublicImage`, `ImageFileInfo` (+186 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `icon.png Favicon Asset` and `Favicon Image Asset`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **What is the exact relationship between `light.png Light Background Asset` and `Light Theme Background Image Asset`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **What is the exact relationship between `night.png Dark Background Asset` and `Dark Theme Background Image Asset`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `render_markdown_page()` connect `Community 1` to `Community 3`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._
- **Why does `validate_html()` connect `Community 2` to `Community 3`?**
  _High betweenness centrality (0.028) - this node is a cross-community bridge._
- **Why does `inject_umami_script()` connect `Community 1` to `Community 10`?**
  _High betweenness centrality (0.020) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `validate_html()` (e.g. with `.push_page()` and `.push_markdown()`) actually correct?**
  _`validate_html()` has 4 INFERRED edges - model-reasoned connections that need verification._