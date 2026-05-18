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

优先采用 `multipart/form-data`，字段如下：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `file` | 是 | 图片文件 |
| `alt` | 否 | 图片替代文本 |
| `description` | 否 | 管理备注 |

上传成功响应：

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

## 模块落点

建议代码落点如下：

| 文件 | 职责 |
| --- | --- |
| `src/image_host.rs` | `ImageStore`、元数据结构、校验、保存、索引管理 |
| `src/web/image_handlers.rs` | `/image`、公开图片读取、管理 API handler |
| `src/web/mod.rs` | 导出 image handlers |
| `src/bin/server.rs` | 初始化 `Arc<ImageStore>`，挂载公开路由和受保护 API |
| `front/image.html` | 图床管理页面 |
| `Cargo.toml` | 需要 multipart 时启用 `axum` 的 `multipart` feature；需要读尺寸时复用或移动 `image` crate 到 dependencies |

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
| 重启服务 | 已上传图片和索引仍在 `data/images` 中 |
| 运行 `cargo check` | 编译通过 |

## 架构结论

图床能力应该作为 SolinBlog 的“公开资产托管”子系统，而不是博客页面系统的一部分。公开读取与受保护管理路由分离，存储统一落在 `data/images`，鉴权复用 `MCP_TOKEN`，前端通过独立 `/image` 页面提供管理体验。这样既满足 AI 发布文章时快速获得图片 URL 的需求，也不会破坏现有首页、页面存储和 MCP 工具结构。
