# Blog Style Prompts

本目录存放博文风格指南文件，通过 `get_blog_style` MCP tool 提供访问。

## 文件与枚举值对应关系

| 文件名 | 枚举值 | 描述 |
|--------|--------|------|
| PPLX.xml | PPLX_STYLE | Perplexity 风格博文写作指南 |

## 添加新风格

1. 在本目录创建新的 XML 文件
2. 在 `src/main.rs` 中的 `BlogStyle` 枚举添加新值
3. 在 `get_blog_style` 方法中添加对应的文件读取逻辑

## HTML 样板：`get_html_style` MCP tool

`get_html_style` 用于返回已填充的 HTML 样板 XML，供生成 HTML 博文时直接使用。

### 参数

- 枚举参数：目前仅支持 `default`。

### 文件依赖关系

`get_html_style` 读取并组合以下三个文件：

- 模板文件：`public/prompt/HTML.xml`
- CSS 样式：`front/example.css`
- HTML 结构：`front/index.html`

### 占位符替换机制

工具会将模板中的占位符替换为对应内容：

- `{{EXAMPLE_CSS}}` → `front/example.css` 的内容
- `{{EXAMPLE_HTML}}` → `front/index.html` 的内容

替换完成后返回完整的 XML。

### 扩展新的 HTML 样式类型

1. 为新样式创建对应的模板和资源文件（如新的 HTML 模板、CSS、HTML 结构）。
2. 在 `src/main.rs` 中为 `get_html_style` 的枚举参数添加新值。
3. 在 `get_html_style` 的实现中增加对应的文件读取与占位符替换逻辑。
