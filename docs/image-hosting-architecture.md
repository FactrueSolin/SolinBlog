# 图床功能架构设计

## 目标与边界

图床作为 SolinBlog 的独立能力加入项目：提供一个不进入首页导航的 `/image` 前端管理页，支持上传图片并返回公开访问 URL。图片文件存储在 `data` 目录下，图片读取公开，上传、删除、更新、管理列表等写操作复用 `MCP_TOKEN` 做 Bearer 鉴权。

本设计保持项目简介和现有边界：页面发布继续由 `PageStore` 负责，现有 `src/image.rs` 的 SearXNG 图片搜索语义不改动；新增图床能力建议使用独立模块，避免把“搜索图片”和“托管图片”混在一起。

## 路由设计

| 路由 | 方法 | 鉴权 | 作用 |
| --- | --- | --- | --- |
| `/image` | `GET` | 否 | 图床管理前端页面，不加入首页导航 |
| `/images/{image_id}/{filename}` | `GET` | 否 | 公开读取图片文件 |
| `/api/images` | `POST` | 是 | 上传图片，返回图片访问 URL |
| `/api/images` | `GET` | 是 | 管理端列出图片元数据 |
| `/api/images/{image_id}` | `GET` | 是 | 管理端获取单张图片元数据 |
| `/api/images/{image_id}` | `PATCH` | 是 | 更新图片元数据，例如 alt、description |
| `/api/images/{image_id}` | `PUT` | 是 | 替换图片文件，URL 中的 `image_id` 保持不变 |
| `/api/images/{image_id}` | `DELETE` | 是 | 删除图片文件和元数据 |

公开图片 URL 使用 `/images/{image_id}/{filename}`，而不是直接暴露 `data` 路径。这样可以保留 `data` 的内部目录结构自由，也能统一 MIME、缓存头和路径穿越防护。

## 后端 API 规范

图床 API 第一版只提供 REST JSON + multipart，不引入 session、cookie、GraphQL 或预签名上传。除公开图片读取外，所有 `/api/images` 接口都必须带：

```http
Authorization: Bearer <MCP_TOKEN>
```

所有 JSON API 统一返回 `ImageApiResponse<T>`：

```json
{
  "success": true,
  "data": {},
  "error": null
}
```

错误响应统一返回：

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "image_not_found",
    "message": "image not found"
  }
}
```

错误码建议固定为以下集合，handler 不返回临时字符串作为业务判断依据：

| HTTP 状态码 | code | 场景 |
| --- | --- | --- |
| `400` | `invalid_request` | 请求字段缺失、分页参数非法、JSON 无法解析 |
| `401` | `unauthorized` | 缺失 Bearer token 或 token 不匹配 |
| `404` | `image_not_found` | 图片 ID 不存在、公开文件不存在 |
| `409` | `image_conflict` | 索引状态冲突，例如替换时旧文件已丢失但索引存在 |
| `413` | `payload_too_large` | 上传体超过 `IMAGE_MAX_UPLOAD_MB` |
| `415` | `unsupported_media_type` | 非图片、图片解码失败或类型不在白名单 |
| `500` | `internal_error` | 文件系统、索引写入、未知服务端错误 |

### 数据模型

`ImageMeta` 是管理端返回的核心结构：

```json
{
  "image_id": "img_a1b2c3d4e5f6",
  "filename": "img_a1b2c3d4e5f6.png",
  "url": "https://example.com/images/img_a1b2c3d4e5f6/img_a1b2c3d4e5f6.png",
  "relative_path": "files/2026/05/img_a1b2c3d4e5f6.png",
  "content_type": "image/png",
  "size_bytes": 123456,
  "width": 1200,
  "height": 800,
  "sha256": "...",
  "alt": "",
  "description": "",
  "created_at": 1779123456,
  "updated_at": 1779123456
}
```

`relative_path` 只给管理端调试和备份使用，前端展示与复制必须使用 `url`。`image_id` 由服务端生成，建议格式为 `img_` + 12 到 16 位 URL-safe 随机串；不要使用原文件名、时间戳或 hash 作为唯一 ID，避免暴露上传行为和产生可枚举路径。

### `POST /api/images` 上传图片

请求类型：`multipart/form-data`。

| 字段 | 类型 | 必填 | 约束 |
| --- | --- | --- | --- |
| `file` | file | 是 | 单文件；超过大小限制直接拒绝 |
| `alt` | string | 否 | 最大 200 字符，超出返回 `400` |
| `description` | string | 否 | 最大 1000 字符，超出返回 `400` |

成功状态码：`201 Created`。

```json
{
  "success": true,
  "data": {
    "image_id": "img_a1b2c3d4e5f6",
    "url": "https://example.com/images/img_a1b2c3d4e5f6/img_a1b2c3d4e5f6.png",
    "meta": {}
  },
  "error": null
}
```

上传处理顺序必须固定：读取 body 限制、解码验证图片、计算 `sha256`、生成 ID 和扩展名、写入临时文件、原子移动到目标路径、原子写入索引。若索引写入失败，必须尝试删除刚写入的文件，避免孤儿文件无限增长。

### `GET /api/images` 管理端列表

查询参数：

| 参数 | 默认值 | 约束 | 说明 |
| --- | --- | --- | --- |
| `limit` | `50` | `1..=100` | 每页数量 |
| `offset` | `0` | `>=0` | 偏移量 |
| `q` | 空 | 最大 100 字符 | 在 `image_id`、`filename`、`alt`、`description` 中做大小写不敏感包含搜索 |

成功状态码：`200 OK`。

```json
{
  "success": true,
  "data": {
    "items": [],
    "total": 0,
    "limit": 50,
    "offset": 0
  },
  "error": null
}
```

排序固定为 `created_at desc`。第一版不做游标分页，因为 `index.json` 是单文件索引，offset 分页足够简单；如果后续迁移数据库，再增加 cursor，不要提前复杂化。

### `GET /api/images/{image_id}` 获取元数据

成功状态码：`200 OK`。返回 `ImageMeta`。不存在返回 `404 image_not_found`。

### `PATCH /api/images/{image_id}` 更新元数据

请求类型：`application/json`。

```json
{
  "alt": "新的替代文本",
  "description": "新的管理备注"
}
```

字段都可选，但不能提交空对象；空对象返回 `400 invalid_request`。`PATCH` 只允许修改描述类元数据，不允许改 `filename`、`relative_path`、`content_type`、`sha256`、尺寸和时间创建字段。

成功状态码：`200 OK`，返回更新后的 `ImageMeta`。

### `PUT /api/images/{image_id}` 替换图片

请求类型：`multipart/form-data`。

| 字段 | 类型 | 必填 | 约束 |
| --- | --- | --- | --- |
| `file` | file | 是 | 新图片文件 |
| `alt` | string | 否 | 提供则覆盖旧值，不提供则保留旧值 |
| `description` | string | 否 | 提供则覆盖旧值，不提供则保留旧值 |

成功状态码：`200 OK`。`image_id` 保持不变，但 `filename`、`relative_path`、`content_type`、`size_bytes`、`width`、`height`、`sha256`、`updated_at` 会更新，公开 URL 也可能因扩展名变化而变化。

替换顺序必须固定：先确认旧元数据存在，再验证新图片，再写新文件，再原子更新索引，最后删除旧文件。禁止先删除旧文件再写新文件，避免替换失败导致原 URL 立即失效。

### `DELETE /api/images/{image_id}` 删除图片

成功状态码：`200 OK`。

```json
{
  "success": true,
  "data": {
    "deleted": true,
    "image_id": "img_a1b2c3d4e5f6"
  },
  "error": null
}
```

删除接口采用非幂等语义：首次删除成功返回 `200`，再次删除返回 `404 image_not_found`。这样管理端能明确发现重复操作或状态不同步。

### `GET /images/{image_id}/{filename}` 公开读取

公开读取不返回 JSON。服务端通过 `image_id` 查索引，然后校验 URL 中的 `filename` 必须等于索引中的 `filename`；不一致返回 `404`，不要重定向到真实文件名，避免旧扩展名长期可用。

成功响应头：

```http
Content-Type: image/png
Content-Length: 123456
Cache-Control: public, max-age=31536000, immutable
X-Content-Type-Options: nosniff
ETag: "sha256-..."
```

`ETag` 使用 `sha256` 派生。若请求带 `If-None-Match` 且命中，返回 `304 Not Modified`。公开读取必须使用 `tokio_util::io::ReaderStream` 或等价流式响应，不应把大文件完整读入内存。

## 存储设计

建议新增 `ImageStore`，根目录挂在 `data/images`。

```text
data/
  images/
    index.json
    files/
      2026/
        05/
          img_a1b2c3d4.png
          img_e5f6g7h8.webp
```

`index.json` 只保存元数据，图片二进制放入 `files`。文件名由服务端生成，避免用户上传文件名影响路径安全。

```json
{
  "images": {
    "img_a1b2c3d4": {
      "image_id": "img_a1b2c3d4",
      "filename": "img_a1b2c3d4.png",
      "relative_path": "files/2026/05/img_a1b2c3d4.png",
      "content_type": "image/png",
      "size_bytes": 123456,
      "width": 1200,
      "height": 800,
      "sha256": "...",
      "alt": "",
      "description": "",
      "created_at": 1779123456,
      "updated_at": 1779123456
    }
  }
}
```

`ImageStore` 建议提供以下方法：

| 方法 | 作用 |
| --- | --- |
| `create_image(bytes, original_filename, content_type, meta)` | 校验并保存图片，写入索引 |
| `list_images()` | 读取索引并按时间倒序返回 |
| `get_image(image_id)` | 返回元数据 |
| `open_image(image_id)` | 返回文件路径、MIME 和字节流所需信息 |
| `update_image_meta(image_id, patch)` | 只更新描述类元数据 |
| `replace_image(image_id, bytes, original_filename, content_type)` | 原 ID 替换文件和尺寸信息 |
| `delete_image(image_id)` | 删除文件并从索引移除 |

索引写入沿用 `store.rs` 的原子写入思想：先写临时文件，再 rename 覆盖。删除时先更新索引再删除旧文件，失败时记录日志；替换时先写新文件，再更新索引，最后删除旧文件。

`ImageStore` 在运行时建议以 `Arc<ImageStore>` 注入 handler。索引读写第一版使用 `tokio::sync::RwLock` 包住内存快照：列表和读取走读锁，上传、替换、删除、元数据更新走写锁。写锁内完成索引状态变更和原子落盘，文件二进制写入可以先在锁外完成临时文件写入，再进入写锁提交索引，避免长时间阻塞列表读取。

启动时必须执行一次 `ImageStore::load_or_init(data/images)`：目录不存在则创建，`index.json` 不存在则创建空索引，索引 JSON 解析失败时服务启动失败并输出明确错误，不自动覆盖。图床属于资产存储，损坏索引自动重建会隐藏数据风险。

## 后端技术选型

第一版后端技术选型以“少组件、强校验、容易备份”为原则：

| 领域 | 选型 | 方案指导 |
| --- | --- | --- |
| Web 框架 | 继续使用 `axum 0.8` | 复用现有 server、Router、middleware，不引入新 Web 框架 |
| 上传协议 | `axum` `multipart` feature | `POST` 和 `PUT` 使用 multipart；`Cargo.toml` 需要把 `axum` 改为 `features = ["macros", "multipart"]` |
| Body 限制 | `DefaultBodyLimit` 或 route layer | 默认 10MB，可用 `IMAGE_MAX_UPLOAD_MB` 覆盖；限制必须挂在图片 API 路由上 |
| 图片验证 | `image = "0.25"` 运行时依赖 | 当前只在 build-dependencies 中；图床要移动或复制到 `[dependencies]`，启用 `png/jpeg/webp/gif/bmp` |
| MIME 判断 | 解码结果优先，`mime_guess` 辅助 | 不信任客户端 `Content-Type`；响应 MIME 由最终文件类型决定 |
| 文件流 | `tokio-util::io::ReaderStream` | 公开读取走流式响应，避免一次性读入内存 |
| 索引格式 | `data/images/index.json` | 第一版不引入 SQLite；方便 Docker volume 和手动备份 |
| 并发控制 | `tokio::sync::RwLock` | 单进程部署足够；多实例部署前必须迁移数据库或增加文件锁 |
| ID 生成 | `getrandom` | 复用现有依赖生成 URL-safe 随机 ID |
| 时间戳 | Unix seconds | 与现有 `PageStore` 风格保持一致 |
| 日志 | `tracing` 后续补充 | 当前项目如果尚未接入 tracing，第一版可先用明确错误返回，不强行扩大改造面 |

明确不选的方案：第一版不使用 S3/OSS、数据库、图片压缩队列、缩略图生成、CDN 刷新、用户多租户和权限分级。这些能力只有在图片量、访问量或多用户管理真实出现后再设计，避免把图床做成独立网盘系统。

需要新增或调整的环境变量：

| 变量 | 默认值 | 作用 |
| --- | --- | --- |
| `IMAGE_MAX_UPLOAD_MB` | `10` | 单张图片上传上限 |
| `SITE_URL` | 无 | 非请求上下文生成图片 URL 时使用 |
| `MCP_TOKEN` | 必填 | 管理 API Bearer 鉴权 |

第一版不需要单独的 `IMAGE_PUBLIC_BASE_URL`。URL 生成优先使用请求头中的 host/proto，脱离请求上下文时使用 `SITE_URL`，保持和页面 URL 生成策略一致。

## 鉴权设计

鉴权复用现有 `MCP_TOKEN` 和 `TokenStore`。建议把 `mcp_auth_middleware` 重命名或补充导出为更通用的 `bearer_auth_middleware`，也可以先直接复用现有函数，语义上作为“受保护 API 鉴权中间件”。

管理 API 使用独立受保护 Router：

```rust
let protected_image_api = Router::new()
    .route("/api/images", get(list_images).post(upload_image))
    .route(
        "/api/images/{image_id}",
        get(get_image).patch(update_image).put(replace_image).delete(delete_image),
    )
    .layer(middleware::from_fn_with_state(token_store.clone(), mcp_auth_middleware));
```

公开读取路由不挂鉴权：

```rust
.route("/image", get(image_page_handler))
.route("/images/{image_id}/{filename}", get(image_asset_handler))
.merge(protected_image_api)
```

前端 `/image` 页面本身可以公开访问，但真正上传、删除、更新时必须让用户输入 token，前端以 `Authorization: Bearer <token>` 调用 API。这样不需要服务端 session，也不会把 token 写进页面。

## 上传协议

上传协议已在“后端 API 规范”中固定为 `multipart/form-data`，字段如下：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `file` | 是 | 图片文件 |
| `alt` | 否 | 图片替代文本 |
| `description` | 否 | 管理备注 |

兼容说明：如果实现阶段希望保持更轻量，也可以先返回以下扁平响应，但对外稳定规范应以 `ImageApiResponse<T>` 为准：

```json
{
  "success": true,
  "image_id": "img_a1b2c3d4",
  "url": "https://example.com/images/img_a1b2c3d4/img_a1b2c3d4.png",
  "meta": {
    "image_id": "img_a1b2c3d4",
    "filename": "img_a1b2c3d4.png",
    "content_type": "image/png",
    "size_bytes": 123456,
    "width": 1200,
    "height": 800,
    "created_at": 1779123456,
    "updated_at": 1779123456
  },
  "error": null
}
```

URL 生成逻辑复用 `resolve_base_url()` 的请求头优先策略；MCP 或非请求上下文可继续用 `SITE_URL`。

## 前端页面设计

`/image` 页面建议放在 `front/image.html`，由 `image_page_handler` 读取并注入 Umami 脚本。这个页面不参与 `front/index.html` 的页面列表，也不写入 `PageStore`。

页面最小功能：

| 区域 | 功能 |
| --- | --- |
| Token 输入 | 用户粘贴 `MCP_TOKEN`，仅保存在浏览器内存或 `sessionStorage` |
| 上传区 | 拖拽或选择图片，填写 alt、description |
| 结果区 | 展示公开 URL、Markdown 图片语法、HTML img 标签 |
| 管理区 | 拉取图片列表，支持复制 URL、替换、更新描述、删除 |

页面视觉可以沿用现有前端模板的暗亮主题变量，但不要把 `/image` 链接加入首页导航或站点 header。

## 文件校验与安全

上传必须做服务端校验，不信任浏览器传来的 MIME。

建议规则：

| 规则 | 设计 |
| --- | --- |
| 类型白名单 | `png`、`jpeg`、`webp`、`gif`、`bmp`，后续再考虑 `svg` |
| MIME 识别 | 优先用图片解码验证，响应 MIME 用 `mime_guess` 或解码结果 |
| 大小限制 | 默认 `10MB`，可用 `IMAGE_MAX_UPLOAD_MB` 覆盖 |
| 路径安全 | 所有文件名服务端生成，所有读取通过 `image_id` 查索引 |
| 缓存 | 公开图片返回 `Cache-Control: public, max-age=31536000, immutable` |
| 删除 | 删除后返回 404，不保留公开访问能力 |
| SVG | 默认不支持，除非单独做脚本清洗和响应头隔离 |

如果要支持大文件上传，Axum 需要配置 body limit，例如使用 `DefaultBodyLimit`，否则默认限制可能不符合预期。

后端还必须保证以下安全约束：公开读取只接受索引中存在的 `image_id` 和完全匹配的 `filename`；任何用户输入的文件名都不能参与路径拼接；`alt` 和 `description` 返回给前端时依靠 JSON 转义，渲染到 HTML 时由前端使用 `textContent` 或等价安全写入；所有错误日志不得打印 Bearer token。

## 模块落点

建议代码落点如下：

| 文件 | 职责 |
| --- | --- |
| `src/image_host.rs` | `ImageStore`、元数据结构、校验、保存、索引管理 |
| `src/web/image_handlers.rs` | `/image`、公开图片读取、管理 API handler |
| `src/web/mod.rs` | 导出 image handlers |
| `src/bin/server.rs` | 初始化 `Arc<ImageStore>`，挂载公开路由和受保护 API |
| `front/image.html` | 图床管理页面 |
| `Cargo.toml` | 启用 `axum` 的 `multipart` feature；把 `image` crate 加入运行时 dependencies |

当前 `image` crate 只在 build-dependencies 中使用。图床运行时如果要解码图片、读取宽高，需要把 `image = "0.25"` 加入 `[dependencies]`，或先只保存 MIME 和大小，把宽高作为后续增强。建议第一版加入运行时 `image` 依赖，因为服务端校验图片真实性比只看扩展名更可靠。

## MCP 工具扩展

目标只要求管理 API；如果后续希望 AI 直接上传图片，可以补充 MCP 工具，但不作为第一版必须项。

建议预留工具名：

| 工具 | 作用 |
| --- | --- |
| `upload_image` | base64 上传图片，返回 URL |
| `list_images` | 列出图床图片 |
| `delete_image` | 删除图床图片 |
| `update_image` | 更新图片元数据 |

注意 MCP base64 上传会放大请求体，不适合大图。Web API 的 multipart 上传应作为主路径。

## 实施顺序

第一阶段先实现后端基础能力：新增 `ImageStore`、公开读取路由、受保护上传 API，确保上传后能拿到 URL 并访问图片。

第二阶段实现 `/image` 页面：上传、复制 URL、管理列表、删除。页面不接入主导航。

第三阶段补齐更新和替换：`PATCH` 元数据、`PUT` 替换图片、尺寸读取、错误提示优化。

第四阶段补文档和验证：更新 README 的功能列表，但保持项目介绍简洁；运行 `cargo check`。

## 验收标准

| 场景 | 期望结果 |
| --- | --- |
| 访问 `/image` | 返回图床管理页面，首页没有导航入口 |
| 无 token 上传 | 返回 `401 Unauthorized` |
| 使用 `Authorization: Bearer $MCP_TOKEN` 上传图片 | 返回 `success=true` 和公开 URL |
| 浏览器打开返回的 URL | 能直接看到图片，不需要 token |
| 删除图片后访问旧 URL | 返回 404 |
| 替换图片后使用旧 filename URL 访问 | 返回 404，使用新 URL 可访问 |
| 请求带 `If-None-Match` 访问未变化图片 | 返回 304 |
| 重启服务 | 已上传图片和索引仍在 `data/images` 中 |
| 运行 `cargo check` | 编译通过 |

## 架构结论

图床能力应该作为 SolinBlog 的“公开资产托管”子系统，而不是博客页面系统的一部分。公开读取与受保护管理路由分离，存储统一落在 `data/images`，鉴权复用 `MCP_TOKEN`，前端通过独立 `/image` 页面提供管理体验。这样既满足 AI 发布文章时快速获得图片 URL 的需求，也不会破坏现有首页、页面存储和 MCP 工具结构。
