# SolinBlog 测试文档

本文档为测试人员提供完整的测试流程指南，覆盖所有功能模块。开发新功能后，应参照此文档确认已有功能不受影响，并为新功能补充对应测试项。

---

## 目录

- [环境准备](#环境准备)
- [一、编译检查](#一编译检查)
- [二、Store 自检工具](#二store-自检工具)
- [三、Web 路由测试](#三web-路由测试)
- [四、MCP 接口测试](#四mcp-接口测试)
- [五、数据存储层测试](#五数据存储层测试)
- [六、HTML 校验测试](#六html-校验测试)
- [七、图片搜索测试](#七图片搜索测试)
- [八、新功能测试规范](#八新功能测试规范)

---

## 环境准备

### 1. 环境变量

创建 `.env` 文件（或导出环境变量）：

```bash
# 必需
MCP_TOKEN=your_test_token_here   # MCP 认证 token，留空则自动生成
WEB_PORT=3000                    # HTTP 端口
WEB_HOST=127.0.0.1               # 监听地址

# 可选
SITE_URL=http://localhost:3000   # 站点 URL（MCP 响应中的完整 URL）
BEIAN_NUMBER=                    # 备案号（首页底部显示）
SITE_SUBTITLE=                   # 首页副标题
SEARXNG_URL=http://localhost:8080 # SearXNG 实例地址（图片搜索功能）
```

### 2. 启动服务

```bash
# 编译并运行
cargo run

# 启动后控制台会输出：
# [solin-blog] http server listening on http://127.0.0.1:3000
# [solin-blog] MCP endpoint: http://127.0.0.1:3000/{token}/mcp
```

> **注意**：记录控制台输出的 MCP token，后续 MCP 测试需要使用。

### 3. 测试脚本

项目提供了 `tests/` 目录下的 shell 脚本用于手动 API 测试：

```bash
chmod +x tests/*.sh
```

---

## 一、编译检查

所有代码修改后的第一步验证。

| 编号 | 测试项           | 命令           | 预期结果                       |
| ---- | ---------------- | -------------- | ------------------------------ |
| 1.1  | cargo check 通过 | `cargo check`  | 无 error                       |
| 1.2  | cargo build 通过 | `cargo build`  | 编译成功                       |
| 1.3  | clippy 检查      | `cargo clippy` | 无 error（warning 可酌情处理） |

---

## 二、Store 自检工具

内置的 CRUD 完整性自检工具，验证数据存储层基础功能。

| 编号 | 测试项          | 命令                              | 预期结果                          |
| ---- | --------------- | --------------------------------- | --------------------------------- |
| 2.1  | Store CRUD 自检 | `cargo run --bin store_selfcheck` | 输出 `selfcheck passed`，退出码 0 |

自检覆盖：
- 创建页面 → 加载验证 → 更新 meta → 更新 HTML → 浏览计数 → 索引重建 → 删除页面
- 验证 `page_uid` 长度为 16 位、纯字母数字
- 验证 `created_at` 在更新后不变
- 验证删除后确认 404

---

## 三、Web 路由测试

启动服务后，使用 `tests/test_web_routes.sh` 或手动 curl 测试。

### 3.1 首页 — `GET /`

| 编号  | 测试项                   | 预期结果                          |
| ----- | ------------------------ | --------------------------------- |
| 3.1.1 | 正常访问首页             | 返回 200，HTML 包含 `SolinBlog`   |
| 3.1.2 | 无页面时首页内容         | 包含 "暂无页面内容" 提示          |
| 3.1.3 | 有页面时首页内容         | 包含 `<article class="card"` 卡片 |
| 3.1.4 | 页面按 `updated_at` 倒序 | 最新更新的页面排在前面            |

### 3.2 文章页 — `GET /pages/{slug}`

| 编号  | 测试项                         | 预期结果                                                                         |
| ----- | ------------------------------ | -------------------------------------------------------------------------------- |
| 3.2.1 | 正常访问文章                   | 返回 200，HTML 含文章内容                                                        |
| 3.2.2 | SEO 标签注入                   | `<head>` 中包含 `<title>`、`<meta name="description">`、`<meta name="keywords">` |
| 3.2.3 | 浏览计数递增                   | 每次访问后 `view_count` +1                                                       |
| 3.2.4 | 不存在的 slug                  | 返回 404，显示 404 页面                                                          |
| 3.2.5 | 空 slug                        | 返回 404                                                                         |
| 3.2.6 | slug 格式 `seo-title+page_uid` | 正确解析 `page_uid` 部分                                                         |

### 3.3 Sitemap — `GET /sitemap.xml`

| 编号  | 测试项           | 预期结果                                    |
| ----- | ---------------- | ------------------------------------------- |
| 3.3.1 | 正常返回 sitemap | 返回 200，Content-Type 为 `application/xml` |
| 3.3.2 | 包含所有页面 URL | 每个页面对应一个 `<url>` 节点               |
| 3.3.3 | lastmod 格式正确 | RFC3339 格式时间戳                          |

### 3.4 Token 生成器 — `GET /tools/token-generator`

| 编号  | 测试项   | 预期结果            |
| ----- | -------- | ------------------- |
| 3.4.1 | 正常访问 | 返回 200，HTML 页面 |

### 3.5 静态资源 — `GET /public/{path}`

| 编号  | 测试项             | 预期结果                            |
| ----- | ------------------ | ----------------------------------- |
| 3.5.1 | 正常访问存在的文件 | 返回 200，Content-Type 匹配文件类型 |
| 3.5.2 | 不存在的文件       | 返回 404                            |
| 3.5.3 | 路径遍历攻击 `../` | 返回 404（安全拦截）                |
| 3.5.4 | 空路径             | 返回 404                            |

---

## 四、MCP 接口测试

MCP 接口通过 `/{token}/mcp` 路径访问。使用 `tests/test_mcp_tools.sh` 进行测试。

> **前置条件**：设置环境变量 `MCP_TOKEN` 后启动服务，或从启动日志中获取自动生成的 token。

### 4.1 push_page — 创建 HTML 页面

| 编号  | 测试项          | 预期结果                                       |
| ----- | --------------- | ---------------------------------------------- |
| 4.1.1 | 正常创建        | `success: true`，返回 `page_id`、`url`、`meta` |
| 4.1.2 | HTML 为空       | `success: false`，error 提示 HTML 为空         |
| 4.1.3 | HTML 标签不闭合 | `success: false`，error 提示标签不匹配         |
| 4.1.4 | 中文 seo_title  | `seo_title` 自动转为拼音 slug                  |
| 4.1.5 | 重复 seo_title  | 可以创建（page_uid 不同）                      |

### 4.2 push_markdown — 创建 Markdown 页面

| 编号  | 测试项                   | 预期结果                                     |
| ----- | ------------------------ | -------------------------------------------- |
| 4.2.1 | 正常创建                 | `success: true`，返回 `page_id`              |
| 4.2.2 | Markdown 正确渲染        | 生成的 HTML 包含对应标签（`<h1>`、`<p>` 等） |
| 4.2.3 | 保存 `content.md` 源文件 | `data/{page_id}/content.md` 存在             |

### 4.3 get_all_page — 列出所有页面

| 编号  | 测试项               | 预期结果                                |
| ----- | -------------------- | --------------------------------------- |
| 4.3.1 | 正常列出             | `success: true`，`pages` 数组非空       |
| 4.3.2 | 无页面时             | `success: true`，`pages` 为空数组       |
| 4.3.3 | 每个页面包含完整信息 | `page_id`、`url`、`meta`（含 seo 信息） |

### 4.4 get_page_by_id — 按 ID 获取页面

| 编号  | 测试项           | 预期结果                                  |
| ----- | ---------------- | ----------------------------------------- |
| 4.4.1 | 单个 ID 查询     | `success: true`，返回页面 HTML            |
| 4.4.2 | 多个 ID 批量查询 | `pages` 数组包含多个结果                  |
| 4.4.3 | 不存在的 ID      | `success: false`，error 提示 not found    |
| 4.4.4 | 空 ID            | `success: false`，error 提示 ids is empty |

### 4.5 delete_page — 删除页面

| 编号  | 测试项             | 预期结果                               |
| ----- | ------------------ | -------------------------------------- |
| 4.5.1 | 正常删除           | `success: true`                        |
| 4.5.2 | 确认删除后数据清除 | `data/{page_id}/` 目录不存在           |
| 4.5.3 | 确认索引更新       | `index.json` 不含已删除的 page_id      |
| 4.5.4 | 删除不存在的页面   | `success: false`，error 提示 not found |

### 4.6 update_page — 更新 HTML 页面

| 编号  | 测试项           | 预期结果                               |
| ----- | ---------------- | -------------------------------------- |
| 4.6.1 | 更新 seo_title   | 新标题生效，URL 变化                   |
| 4.6.2 | 更新 description | meta 中 description 更新               |
| 4.6.3 | 更新 keywords    | meta 中 keywords 更新                  |
| 4.6.4 | 更新 html 内容   | 新 HTML 保存成功                       |
| 4.6.5 | 更新无效 HTML    | `success: false`，校验拦截             |
| 4.6.6 | 更新不存在的页面 | `success: false`，error 提示 not found |
| 4.6.7 | 部分更新         | 仅更新指定字段，其余不变               |

### 4.7 update_markdown_page — 更新 Markdown 页面

| 编号  | 测试项              | 预期结果                             |
| ----- | ------------------- | ------------------------------------ |
| 4.7.1 | 更新 markdown 内容  | 新 Markdown 渲染为 HTML 保存         |
| 4.7.2 | content.md 同步更新 | `data/{page_id}/content.md` 内容更新 |
| 4.7.3 | 更新 seo 信息       | meta 中字段更新                      |

### 4.8 search_images — 图片搜索

| 编号  | 测试项             | 预期结果                                       |
| ----- | ------------------ | ---------------------------------------------- |
| 4.8.1 | 正常搜索           | `success: true`，返回图片列表                  |
| 4.8.2 | 空关键词           | `success: false`，error 提示 keywords is empty |
| 4.8.3 | 多关键词并发       | 每个关键词返回独立结果                         |
| 4.8.4 | limit 参数生效     | 返回图片数量不超过 limit                       |
| 4.8.5 | SEARXNG_URL 未配置 | `success: false`，error 提示未配置             |

### 4.9 get_blog_style — 获取博文风格

| 编号  | 测试项     | 预期结果                           |
| ----- | ---------- | ---------------------------------- |
| 4.9.1 | PPLX_STYLE | 返回 `public/prompt/PPLX.xml` 内容 |

### 4.10 get_html_style — 获取 HTML 风格

| 编号   | 测试项  | 预期结果                               |
| ------ | ------- | -------------------------------------- |
| 4.10.1 | DEFAULT | 返回模板内容，包含 example CSS 和 HTML |

---

## 五、数据存储层测试

存储功能通过 Store 自检 + MCP 接口间接覆盖。以下为重点验证项：

### 5.1 文件系统结构验证

| 编号  | 测试项                | 验证方式                                                         |
| ----- | --------------------- | ---------------------------------------------------------------- |
| 5.1.1 | 页面目录结构          | `data/{page_id}/` 下有 `meta.json` 和 `index.html`               |
| 5.1.2 | Markdown 页面额外文件 | `data/{page_id}/content.md` 存在                                 |
| 5.1.3 | 全局索引              | `data/index.json` 存在且 JSON 合法                               |
| 5.1.4 | meta.json 字段完整    | 包含 `seo`、`page_uid`、`created_at`、`updated_at`、`view_count` |

### 5.2 数据完整性

| 编号  | 测试项              | 验证方式                                |
| ----- | ------------------- | --------------------------------------- |
| 5.2.1 | page_uid 唯一性     | 创建多个页面后，所有 `page_uid` 不重复  |
| 5.2.2 | page_uid 格式       | 16 位字母数字字符                       |
| 5.2.3 | created_at 不可变   | 更新操作后 `created_at` 不变            |
| 5.2.4 | updated_at 自动更新 | 更新操作后 `updated_at` 变为当前时间    |
| 5.2.5 | 索引与文件同步      | `index.json` 中的页面与实际目录一一对应 |
| 5.2.6 | 原子写入            | 无残留 `.tmp` 文件                      |

### 5.3 索引重建

| 编号  | 测试项                 | 验证方式                                   |
| ----- | ---------------------- | ------------------------------------------ |
| 5.3.1 | 删除 index.json 后恢复 | 删除 `data/index.json`，重启服务后自动重建 |
| 5.3.2 | index.json 损坏后恢复  | 写入无效 JSON，重启后自动重建              |

---

## 六、HTML 校验测试

`validate_html` 函数在 push_page 和 update_page 时自动调用。

| 编号 | 测试项              | 输入                            | 预期结果                     |
| ---- | ------------------- | ------------------------------- | ---------------------------- |
| 6.1  | 空 HTML             | `""` 或纯空白                   | 错误：html is empty          |
| 6.2  | 含 NUL 字节         | `"<p>\x00</p>"`                 | 错误：contains NUL byte      |
| 6.3  | 标签不闭合          | `"<div><p>text</div>"`          | 错误：mismatched closing tag |
| 6.4  | 未闭合标签          | `"<div><p>text</p>"`            | 错误：unclosed tag           |
| 6.5  | Void 元素           | `"<br><hr><img src='x'>"`       | 通过（void 元素无需闭合）    |
| 6.6  | 自闭合标签          | `"<div />"`                     | 通过                         |
| 6.7  | Script / Style 标签 | `"<script>...</script>"`        | 正确匹配关闭标签             |
| 6.8  | HTML 注释           | `"<!-- comment -->"`            | 正确跳过                     |
| 6.9  | 完整 HTML5 文档     | 标准 `<!doctype html><html>...` | 通过                         |

---

## 七、图片搜索测试

> **前置条件**：需要配置 `SEARXNG_URL` 环境变量指向可用的 SearXNG 实例。

使用 `tests/test_image_search.sh` 测试，或通过 MCP 接口 `search_images` 工具调用。

| 编号 | 测试项           | 预期结果                             |
| ---- | ---------------- | ------------------------------------ |
| 7.1  | 单关键词搜索     | 返回图片列表                         |
| 7.2  | 多关键词并发搜索 | 每个关键词独立返回结果               |
| 7.3  | 结果数量限制     | 不超过 `limit`                       |
| 7.4  | SearXNG 不可达   | `success: false`，error 说明连接失败 |

---

## 八、新功能测试规范

开发新功能后，请按以下步骤进行测试：

### 步骤 1：编译检查

```bash
cargo check
```

### 步骤 2：运行 Store 自检

```bash
cargo run --bin store_selfcheck
```

### 步骤 3：启动服务并手动测试

```bash
# 设置测试 token
export MCP_TOKEN=test_token_123
cargo run
```

### 步骤 4：运行测试脚本

```bash
# Web 路由测试
./tests/test_web_routes.sh

# MCP 接口测试
./tests/test_mcp_tools.sh
```

### 步骤 5：补充测试项

在本文档对应章节中补充新增功能的测试清单，格式：

```markdown
| 编号 | 测试项           | 预期结果     |
| ---- | ---------------- | ------------ |
| X.X  | 描述新功能测试点 | 填写预期结果 |
```

### 新功能测试检查清单

- [ ] `cargo check` 通过
- [ ] `cargo run --bin store_selfcheck` 通过
- [ ] 新增的 Web 路由可正常访问
- [ ] 新增的 MCP 工具可正常调用
- [ ] 错误输入时返回合理的错误信息（不导致 panic）
- [ ] 已有功能的测试脚本仍然通过（回归测试）
- [ ] 本文档已补充对应测试项

---

## 附录：测试脚本说明

| 脚本                         | 用途                                |
| ---------------------------- | ----------------------------------- |
| `tests/test_web_routes.sh`   | Web 路由功能测试                    |
| `tests/test_mcp_tools.sh`    | MCP 全部工具接口测试（CRUD 全流程） |
| `tests/test_image_search.sh` | 图片搜索功能测试（需 SearXNG）      |
