use crate::store::{PageMeta, PageStore};
use anyhow::{Context, Result};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'\'')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'#')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'\\')
    .add(b'+');

pub fn build_page_url(page_id: &str, seo_title: &str) -> String {
    let encoded_title = utf8_percent_encode(seo_title, PATH_SEGMENT_ENCODE_SET).to_string();
    format!("/pages/{encoded_title}+{page_id}")
}

pub fn parse_page_id_from_slug(slug: &str) -> Option<String> {
    let mut parts = slug.rsplitn(2, '+');
    let page_id = parts.next()?;
    if page_id.is_empty() {
        return None;
    }
    Some(page_id.to_string())
}

pub fn render_index_html(store: &PageStore) -> Result<String> {
    let entries = store.list_page_entries().context("list page entries")?;
    let mut rows = String::new();
    for entry in entries {
        let title = escape_html(&entry.seo.seo_title);
        let description = escape_html(&entry.seo.description);
        let keywords = entry
            .seo
            .keywords
            .as_ref()
            .map(|items| items.join(", "))
            .filter(|value| !value.trim().is_empty())
            .map(|value| escape_html(&value))
            .unwrap_or_else(|| "无".to_string());
        let page_id = escape_html(&entry.page_id);
        let url = build_page_url(&entry.page_id, &entry.seo.seo_title);
        let url_attr = escape_html_attr(&url);
        rows.push_str(&format!(
            "<article class=\"card\"><div class=\"card-header\"><h2><a href=\"{url_attr}\">{title}</a></h2><span class=\"page-id\">{page_id}</span></div><p class=\"description\">{description}</p><div class=\"keywords\"><span>关键词：</span><span class=\"keyword-value\">{keywords}</span></div><div class=\"actions\"><a class=\"read-more\" href=\"{url_attr}\">阅读页面</a></div></article>",
        ));
    }

    if rows.is_empty() {
        rows.push_str(
            "<div class=\"empty\">暂无页面内容，请先通过 MCP 接口发布页面。</div>",
        );
    }

    Ok(format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>SolinBlog</title><style>body{{margin:0;font-family:-apple-system,BlinkMacSystemFont,\"Segoe UI\",Roboto,\"PingFang SC\",\"Hiragino Sans GB\",\"Microsoft YaHei\",sans-serif;background:#f6f7fb;color:#1f2937}}header{{background:linear-gradient(120deg,#1e3a8a,#0f766e);color:#fff;padding:48px 24px}}header h1{{margin:0;font-size:32px}}header p{{margin:8px 0 0;opacity:.85}}main.container{{max-width:960px;margin:-32px auto 48px;padding:0 24px}}.card-list{{display:grid;gap:16px}}.card{{background:#fff;border-radius:16px;padding:20px 24px;box-shadow:0 12px 30px rgba(15,23,42,.08);border:1px solid #e5e7eb}}.card-header{{display:flex;justify-content:space-between;align-items:flex-start;gap:12px;flex-wrap:wrap}}.card-header h2{{margin:0;font-size:20px}}.card-header a{{color:#0f172a;text-decoration:none}}.card-header a:hover{{text-decoration:underline}}.page-id{{font-size:12px;color:#6b7280;background:#f3f4f6;border-radius:999px;padding:4px 10px}}.description{{margin:12px 0 0;color:#374151;line-height:1.6}}.keywords{{margin-top:12px;font-size:13px;color:#4b5563}}.keyword-value{{font-weight:600}}.actions{{margin-top:16px}}.read-more{{display:inline-block;padding:8px 16px;border-radius:999px;background:#0f766e;color:#fff;text-decoration:none;font-size:14px}}.read-more:hover{{background:#115e59}}.empty{{padding:32px;border-radius:16px;background:#fff;border:1px dashed #cbd5f5;color:#64748b;text-align:center}}</style></head><body><header><h1>SolinBlog</h1><p>AI 原生博客 · 最新页面列表</p></header><main class=\"container\"><section class=\"card-list\">{rows}</section></main></body></html>"
    ))
}

pub fn render_page_html(meta: &PageMeta, html: &str) -> String {
    inject_seo_meta(html, &meta.seo)
}

pub fn inject_seo_meta(html: &str, seo: &crate::store::SeoMeta) -> String {
    let escaped_title = escape_html(&seo.seo_title);
    let escaped_description = escape_html_attr(&seo.description);
    let keywords = seo
        .keywords
        .as_ref()
        .map(|items| items.join(", "))
        .filter(|value| !value.trim().is_empty())
        .map(|value| escape_html_attr(&value));

    let mut additions = String::new();
    additions.push_str(&format!("<title>{}</title>", escaped_title));
    additions.push_str(&format!(
        "<meta name=\"description\" content=\"{}\">",
        escaped_description
    ));
    if let Some(keyword_value) = keywords {
        additions.push_str(&format!(
            "<meta name=\"keywords\" content=\"{}\">",
            keyword_value
        ));
    }

    let mut out = String::new();
    let bytes = html.as_bytes();
    let mut index = 0usize;
    let mut head_range: Option<(usize, usize)> = None;
    while index < bytes.len() {
        if bytes[index] == b'<' {
            if let Some((name, after_name)) = parse_tag_name_ci(bytes, index + 1) {
                let name = name.to_ascii_lowercase();
                if name == "head" {
                    if let Some(end) = find_tag_end(bytes, after_name) {
                        let content_start = end + 1;
                        if let Some(close_start) = find_bytes_ci(bytes, content_start, b"</head") {
                            head_range = Some((content_start, close_start));
                            break;
                        }
                    }
                }
            }
        }
        index += 1;
    }

    if let Some((start, end)) = head_range {
        out.push_str(&html[..start]);
        let existing = &html[start..end];
        let cleaned = remove_head_seo_tags(existing);
        out.push_str(&additions);
        out.push_str(&cleaned);
        out.push_str(&html[end..]);
        return out;
    }

    if let Some(insert_at) = find_html_tag_end(bytes) {
        out.push_str(&html[..insert_at]);
        out.push_str("<head>");
        out.push_str(&additions);
        out.push_str("</head>");
        out.push_str(&html[insert_at..]);
        return out;
    }

    if let Some(body_pos) = find_bytes_ci(bytes, 0, b"<body") {
        out.push_str(&html[..body_pos]);
        out.push_str("<head>");
        out.push_str(&additions);
        out.push_str("</head>");
        out.push_str(&html[body_pos..]);
        return out;
    }

    format!("<head>{}</head>{}", additions, html)
}

fn remove_head_seo_tags(head_html: &str) -> String {
    let mut result = String::new();
    let bytes = head_html.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'<' {
            result.push(bytes[index] as char);
            index += 1;
            continue;
        }
        if let Some((name, after_name)) = parse_tag_name_ci(bytes, index + 1) {
            let lower = name.to_ascii_lowercase();
            if lower == "title" {
                if let Some(tag_end) = find_tag_end(bytes, after_name) {
                    if let Some(close_start) = find_bytes_ci(bytes, tag_end + 1, b"</title") {
                        if let Some(close_end) = find_tag_end(bytes, close_start + 2) {
                            index = close_end + 1;
                            continue;
                        }
                    }
                }
            }
            if lower == "meta" {
                if let Some(tag_end) = find_tag_end(bytes, after_name) {
                    let tag_html = &head_html[index..=tag_end];
                    if is_meta_named(tag_html, "description") || is_meta_named(tag_html, "keywords")
                    {
                        index = tag_end + 1;
                        continue;
                    }
                }
            }
        }
        result.push(bytes[index] as char);
        index += 1;
    }
    result
}

fn is_meta_named(tag_html: &str, name: &str) -> bool {
    let lower = tag_html.to_ascii_lowercase();
    let name_lower = name.to_ascii_lowercase();
    if let Some(pos) = lower.find("name") {
        let after = &lower[pos + 4..];
        if let Some(eq_pos) = after.find('=') {
            let mut value = after[eq_pos + 1..].trim_start();
            if value.starts_with('"') {
                value = &value[1..];
                if let Some(end) = value.find('"') {
                    return &value[..end] == name_lower;
                }
            } else if value.starts_with('\'') {
                value = &value[1..];
                if let Some(end) = value.find('\'') {
                    return &value[..end] == name_lower;
                }
            } else {
                let token = value
                    .split(|ch: char| ch.is_whitespace() || ch == '>')
                    .next()
                    .unwrap_or("");
                return token == name_lower;
            }
        }
    }
    false
}

fn find_html_tag_end(bytes: &[u8]) -> Option<usize> {
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'<' {
            if let Some((name, after_name)) = parse_tag_name_ci(bytes, index + 1) {
                if name.eq_ignore_ascii_case("html") {
                    return find_tag_end(bytes, after_name).map(|value| value + 1);
                }
            }
        }
        index += 1;
    }
    None
}

fn parse_tag_name_ci(bytes: &[u8], mut index: usize) -> Option<(String, usize)> {
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    if index < bytes.len() && bytes[index] == b'/' {
        index += 1;
    }
    let start = index;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b':' {
            index += 1;
        } else {
            break;
        }
    }
    if start == index {
        return None;
    }
    let name = std::str::from_utf8(&bytes[start..index]).ok()?;
    Some((name.to_string(), index))
}

fn find_tag_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            None => {
                if byte == b'\'' || byte == b'"' {
                    quote = Some(byte);
                } else if byte == b'>' {
                    return Some(index);
                }
            }
            Some(active) => {
                if byte == active {
                    quote = None;
                }
            }
        }
        index += 1;
    }
    None
}

fn find_bytes_ci(haystack: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(start.min(haystack.len()));
    }
    if start >= haystack.len() || needle.len() > haystack.len() {
        return None;
    }
    let needle_lower: Vec<u8> = needle.iter().map(|byte| byte.to_ascii_lowercase()).collect();
    let end = haystack.len().saturating_sub(needle_lower.len());
    for index in start..=end {
        let mut matched = true;
        for (offset, expected) in needle_lower.iter().enumerate() {
            if haystack[index + offset].to_ascii_lowercase() != *expected {
                matched = false;
                break;
            }
        }
        if matched {
            return Some(index);
        }
    }
    None
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_html_attr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
