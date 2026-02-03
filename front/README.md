# front/index.html 编辑说明（首页模板）

本文档说明如何安全地编辑首页模板 [`front/index.html`](front/index.html:1)。该文件会在运行时被服务端读取并进行占位符替换，替换逻辑见 [`render_index_html()`](src/web.rs:35) 与 [`replace_template()`](src/web.rs:91)。

## 1. 文件用途说明

- [`front/index.html`](front/index.html:1)：SolinBlog **首页 HTML 模板**。
- 服务端在渲染首页时会：
  1. 读取模板文件内容（字符串）
  2. 生成“页面卡片列表”等动态内容
  3. 使用字符串替换将模板里的 `{{...}}` 占位符替换成真实内容（不是完整的模板引擎）

注意：替换逻辑会校验模板中是否存在必须的占位符；如果缺少，会直接报错并导致首页渲染失败（HTTP 500），详见 [`replace_template()`](src/web.rs:91)。

## 2. 必须保留的占位符（不可删除/不可改名）

以下占位符名称必须 **完全一致**（大小写/下划线都必须相同），否则渲染会失败。

| 占位符 | 出现位置（默认模板） | 作用 | 备注 |
|---|---|---|---|
| `{{page_list}}` | [`front/index.html`](front/index.html:119) | 首页“最新页面列表”的主体内容（由服务端生成多条 `<article class="card">...` 拼接而成） | 通常建议放在一个块级容器中，便于布局/滚动/网格样式。 |
| `{{site_title}}` | [`front/index.html`](front/index.html:6) 与 [`front/index.html`](front/index.html:115) | 站点标题（目前服务端固定为 `SolinBlog`） | 可以出现多次；替换会对模板中所有匹配内容生效。 |
| `{{site_subtitle}}` | [`front/index.html`](front/index.html:116) | 站点副标题（目前服务端固定为 `AI 原生博客 · 最新页面列表`） | 用于首页头部说明文字。 |
| `{{beian_number}}` | [`front/index.html`](front/index.html:121) | 备案信息区域（若未配置则为空字符串；若配置则输出 `<footer class="beian">...</footer>`） | 这是一个“可为空”的占位符，但占位符本身仍必须存在。 |

### 2.1 占位符的硬性规则

1. **不能删除**上述占位符。
2. **不能修改名称**（例如把 `{{page_list}}` 改成 `{{pages}}` 会直接导致渲染失败）。
3. **不能改写花括号形式**：必须是双大括号 `{{...}}`，且中间不要插入额外空格或 HTML 标签。
4. `{{page_list}}` 替换结果包含完整 HTML 片段（多个 `<article>`），不要对其做 HTML 转义。

## 3. 可以自由修改的内容

在保留上述占位符的前提下，你可以自由调整：

- CSS 样式（默认内联在 `<style>` 中）：颜色、字体、间距、卡片样式、响应式等。
- 页面布局结构：例如把 header 改成更复杂的导航、把列表放到双栏布局、增加侧边栏等。
- 其他静态内容：例如页脚版权、友情链接、说明文字、统计脚本（请自行评估隐私与性能）。

只要最终的模板文件中仍然包含必须占位符，服务端就可以继续正常渲染。

## 4. 修改示例与建议

### 4.1 修改样式（推荐：只动 CSS，不动占位符）

例如想把背景改成深色，并让卡片更紧凑：

```html
<!-- 仅示意：编辑 front/index.html 的 <style> -->
<style>
  body { background: #0b1220; color: #e5e7eb; }
  header { background: linear-gradient(120deg, #111827, #064e3b); }
  .card { background: #0f172a; border-color: rgba(148, 163, 184, 0.2); }
  .card-header a { color: #e5e7eb; }
</style>
```

建议：

- 优先通过修改类名对应的 CSS 达到效果，减少对 HTML 结构的破坏。
- 如果你删除了 `.card-list` / `.card` 等默认样式类，请同时补齐新的样式，否则列表可能变成“无样式的长文本”。

### 4.2 调整布局（可移动占位符，但不要拆分/包裹错误）

示例：增加一个侧边栏，并把 `{{page_list}}` 放进主栏。

```html
<main class="container layout">
  <aside class="sidebar">
    <h2>关于本站</h2>
    <p>这里放一些静态介绍内容。</p>
  </aside>
  <section class="content">
    <div class="card-list">{{page_list}}</div>
  </section>
</main>
```

注意事项：

- 不要把 `{{page_list}}` 拆成两半或插入到属性中（例如 `class="{{page_list}}"`），它替换的是一段 HTML 结构。
- 如果你把 `{{beian_number}}` 移动到某个容器内部，请考虑其可能为空字符串；空时不应影响布局（例如避免留下很大的空白区域）。

### 4.3 关于 `<title>` 与 SEO

首页模板里 `<title>` 默认也是 `{{site_title}}`（见 [`front/index.html`](front/index.html:6)），因此首页标题会和头部 H1 同步。

- 可以在 `<head>` 中添加额外的静态 meta（如主题色、图标、OG 标签等）。
- 但请不要移除 `{{site_title}}`，否则首页渲染会失败。

## 5. 快速自检清单（改完必看）

- [ ] 模板中仍然存在 `{{page_list}}`、`{{site_title}}`、`{{site_subtitle}}`、`{{beian_number}}`
- [ ] 占位符名称未改动、未增加空格、未变更为其他括号形式
- [ ] `{{page_list}}` 仍位于一个适合插入“多条 `<article>`”的块级位置

