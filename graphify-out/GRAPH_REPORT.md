# Graph Report - .  (2026-05-19)

## Corpus Check
- Large corpus: 39 files · ~580,609 words. Semantic extraction will be expensive (many Claude tokens). Consider running on a subfolder, or use --no-semantic to run AST-only.

## Summary
- 270 nodes · 427 edges · 24 communities (22 shown, 2 thin omitted)
- Extraction: 86% EXTRACTED · 13% INFERRED · 1% AMBIGUOUS · INFERRED: 56 edges (avg confidence: 0.81)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Frontend Templates|Frontend Templates]]
- [[_COMMUNITY_HTML Rendering Utilities|HTML Rendering Utilities]]
- [[_COMMUNITY_Page Store Persistence|Page Store Persistence]]
- [[_COMMUNITY_MCP Server API|MCP Server API]]
- [[_COMMUNITY_Deployment Configuration|Deployment Configuration]]
- [[_COMMUNITY_API Data Types|API Data Types]]
- [[_COMMUNITY_Testing And Routes|Testing And Routes]]
- [[_COMMUNITY_Published MCP Tools|Published MCP Tools]]
- [[_COMMUNITY_Image Search|Image Search]]
- [[_COMMUNITY_Store Selfcheck|Store Selfcheck]]
- [[_COMMUNITY_MCP Authentication|MCP Authentication]]
- [[_COMMUNITY_Build Image Conversion|Build Image Conversion]]
- [[_COMMUNITY_Server Startup|Server Startup]]
- [[_COMMUNITY_Rust Project Practice|Rust Project Practice]]

## God Nodes (most connected - your core abstractions)
1. `PageStore` - 27 edges
2. `validate_html()` - 14 edges
3. `BlogMcpServer` - 12 edges
4. `sanitize_page_id()` - 11 edges
5. `Home Page Template` - 11 edges
6. `Docker Compose SolinBlog Service` - 10 edges
7. `Published MCP Tools` - 10 edges
8. `inject_seo_meta()` - 9 edges
9. `Reusable Site Header` - 9 edges
10. `generate_unique_page_uid()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `macOS LaunchAgent Deployment` --semantically_similar_to--> `Docker Deployment Guide`  [INFERRED] [semantically similar]
  just/just.md → DOCKER.md
- `Required Placeholder Contract` --conceptually_related_to--> `Front Template Runtime Mount`  [INFERRED]
  front/README.md → DOCKER.md
- `Light Theme Background Image Asset` --references--> `light.png Light Background Asset`  [AMBIGUOUS]
  public/light0.png → public/README.md
- `Dark Theme Background Image Asset` --references--> `night.png Dark Background Asset`  [AMBIGUOUS]
  public/night0.png → public/README.md
- `get_html_style HTML Template Tool` --implements--> `get_html_style Tool`  [INFERRED]
  public/prompt/README.md → README.md

## Hyperedges (group relationships)
- **Docker Runtime Configuration** — dockercompose_solinblog_service, dockercompose_env_file, dockercompose_web_host, dockercompose_port_3002, dockercompose_data_volume, dockercompose_front_volume, dockercompose_public_volume [EXTRACTED 1.00]
- **Page CRUD MCP Tools** — README_create_page_tool, README_update_page_tool, README_update_markdown_page_tool, README_delete_page_tool, README_list_pages_tool, README_get_page_tool, gongnengshu_pagestore [INFERRED 0.86]
- **Front Template Placeholder System** — index_home_template, frontreadme_placeholder_contract, index_site_title_placeholder, index_site_subtitle_placeholder, index_page_list_placeholder, index_beian_number_placeholder, frontreadme_string_replacement_templates [EXTRACTED 1.00]

## Communities (24 total, 2 thin omitted)

### Community 0 - "Frontend Templates"
Cohesion: 0.06
Nodes (43): get_html_style Tool, 404 Return Home Link, 404 Light and Dark Theme CSS, 404 Not Found Page, 404 Site Header Placeholder, Required Placeholder Contract, String Replacement Template Mechanism, Home Template Editing Guide (+35 more)

### Community 1 - "HTML Rendering Utilities"
Cohesion: 0.13
Nodes (31): build_page_url(), escape_html(), escape_html_attr(), find_bytes_ci(), find_html_tag_end(), find_tag_end(), format_display_timestamp(), format_unix_timestamp() (+23 more)

### Community 2 - "Page Store Persistence"
Cohesion: 0.19
Nodes (6): atomic_write(), generate_unique_page_uid(), now_unix_seconds(), PageStore, sanitize_page_id(), to_url_slug()

### Community 3 - "MCP Server API"
Cohesion: 0.13
Nodes (15): BlogMcpServer, build_page_full_url(), resolve_site_url_from_env(), find_bytes(), find_bytes_case_insensitive(), find_tag_end(), generate_page_uid(), is_self_closing() (+7 more)

### Community 4 - "Deployment Configuration"
Cohesion: 0.07
Nodes (28): AI Native Blog Project, Cargo Check Verification Policy, Data Persistence, Docker Deployment Guide, Environment Variable Configuration, Front Template Runtime Mount, MCP Endpoint /mcp, MCP_TOKEN Bearer Authentication Token (+20 more)

### Community 5 - "API Data Types"
Cohesion: 0.08
Nodes (23): BlogStyle, DeletePageResponse, GetAllPageRequest, GetAllPageResponse, GetBlogStyleRequest, GetBlogStyleResponse, GetHtmlStyleRequest, GetHtmlStyleResponse (+15 more)

### Community 6 - "Testing And Routes"
Cohesion: 0.12
Nodes (24): search_images Tool, Data Storage Layer Tests, HTML Validation Tests, Image Search Tests, Manual Shell Test Scripts, MCP Interface Tests, Store CRUD Selfcheck, SolinBlog Test Guide (+16 more)

### Community 7 - "Published MCP Tools"
Cohesion: 0.18
Nodes (11): create_page Tool, delete_page Tool, get_blog_style Tool, get_page Tool, list_pages Tool, Published MCP Tools, update_markdown_page Tool, update_page Tool (+3 more)

### Community 8 - "Image Search"
Cohesion: 0.28
Nodes (8): ImageSearchItem, ImageSearchResponse, ImageSearchResult, resolve_searxng_url(), search_images(), search_single(), SearxImageResult, SearxResponse

### Community 9 - "Store Selfcheck"
Cohesion: 0.33
Nodes (3): IndexSnapshotGuard, main(), PageDirGuard

### Community 10 - "MCP Authentication"
Cohesion: 0.47
Nodes (3): extract_bearer_token(), mcp_auth_middleware(), TokenStore

### Community 11 - "Build Image Conversion"
Cohesion: 0.60
Nodes (5): convert_special_images(), convert_to_png(), find_first_source(), main(), read_first_matching_file()

## Ambiguous Edges - Review These
- `icon.png Favicon Asset` → `Favicon Image Asset`  [AMBIGUOUS]
  public/icon.png · relation: references
- `light.png Light Background Asset` → `Light Theme Background Image Asset`  [AMBIGUOUS]
  public/light0.png · relation: references
- `night.png Dark Background Asset` → `Dark Theme Background Image Asset`  [AMBIGUOUS]
  public/night0.png · relation: references

## Knowledge Gaps
- **68 isolated node(s):** `ImageSearchItem`, `ImageSearchResult`, `ImageSearchResponse`, `SearxResponse`, `SearxImageResult` (+63 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `icon.png Favicon Asset` and `Favicon Image Asset`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **What is the exact relationship between `light.png Light Background Asset` and `Light Theme Background Image Asset`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **What is the exact relationship between `night.png Dark Background Asset` and `Dark Theme Background Image Asset`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `validate_html()` connect `MCP Server API` to `Page Store Persistence`?**
  _High betweenness centrality (0.078) - this node is a cross-community bridge._
- **Why does `Published MCP Tools` connect `Published MCP Tools` to `Frontend Templates`, `Deployment Configuration`, `Testing And Routes`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **Why does `render_markdown_page()` connect `HTML Rendering Utilities` to `MCP Server API`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `validate_html()` (e.g. with `.push_page()` and `.push_markdown()`) actually correct?**
  _`validate_html()` has 4 INFERRED edges - model-reasoned connections that need verification._