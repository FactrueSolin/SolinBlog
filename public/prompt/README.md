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
