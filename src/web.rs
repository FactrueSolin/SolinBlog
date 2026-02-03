use crate::store::{PageMeta, PageStore};
use anyhow::{bail, Context, Result};
use pulldown_cmark::{Options, Parser, html};
use chrono::{TimeZone, Utc};

pub fn build_page_url(page_id: &str, seo_title: &str) -> String {
    if seo_title.is_empty() {
        format!("/pages/{}", page_id)
    } else {
        format!("/pages/{}+{}", seo_title, page_id)
    }
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
    let header_html = std::fs::read_to_string("front/header.html")
        .context("read front/header.html template")?;
    let template = std::fs::read_to_string("front/index.html")
        .context("read front/index.html template")?;
    let entries = store.list_page_entries().context("list page entries")?;
    let mut pages = Vec::new();
    for entry in entries {
        let meta = store
            .get_page_meta(&entry.page_id)
            .with_context(|| format!("load page meta {}", entry.page_id))?;
        pages.push((entry, meta));
    }
    pages.sort_by(|(left_entry, left_meta), (right_entry, right_meta)| {
        right_meta
            .updated_at
            .cmp(&left_meta.updated_at)
            .then_with(|| right_meta.created_at.cmp(&left_meta.created_at))
            .then_with(|| right_entry.page_id.cmp(&left_entry.page_id))
    });
    let mut rows = String::new();
    for (entry, meta) in pages {
        let display_title = if entry.seo.title.is_empty() {
            &entry.seo.seo_title
        } else {
            &entry.seo.title
        };
        let title = escape_html(display_title);
        let description = escape_html(&entry.seo.description);
        let data_title = escape_html_attr(display_title);
        let data_description = escape_html_attr(&entry.seo.description);
        let keywords = entry
            .seo
            .keywords
            .as_ref()
            .map(|items| items.join(", "))
            .filter(|value| !value.trim().is_empty())
            .map(|value| escape_html(&value))
            .unwrap_or_else(|| "无".to_string());
        let data_keywords = entry
            .seo
            .keywords
            .as_ref()
            .map(|items| items.join(", "))
            .filter(|value| !value.trim().is_empty())
            .map(|value| escape_html_attr(&value))
            .unwrap_or_else(|| "无".to_string());
        let page_id_attr = escape_html_attr(&entry.page_id);
        let url = build_page_url(&entry.page_id, &entry.seo.seo_title);
        let url_attr = escape_html_attr(&url);
        let updated_at = escape_html(&format_display_timestamp(meta.updated_at));
        rows.push_str(&format!(
            "<article class=\"card\" data-page-id=\"{page_id_attr}\" data-title=\"{data_title}\" data-description=\"{data_description}\" data-keywords=\"{data_keywords}\"><div class=\"card-header\"><h2><a href=\"{url_attr}\">{title}</a></h2><span class=\"updated-at\">更新：{updated_at}</span></div><p class=\"description\">{description}</p><div class=\"keywords\"><span>关键词：</span><span class=\"keyword-value\">{keywords}</span></div><div class=\"actions\"><a class=\"read-more\" href=\"{url_attr}\">阅读页面</a></div></article>",
        ));
    }

    if rows.is_empty() {
        rows.push_str(
            "<div class=\"empty\">暂无页面内容，请先通过 MCP 接口发布页面。</div>",
        );
    }

    let beian_number = std::env::var("BEIAN_NUMBER")
        .unwrap_or_default()
        .trim()
        .to_string();
    let beian_html = if beian_number.is_empty() {
        String::new()
    } else {
        format!(
            "<footer class=\"beian\">{}</footer>",
            escape_html(&beian_number)
        )
    };

    let site_subtitle = std::env::var("SITE_SUBTITLE")
        .unwrap_or_default()
        .trim()
        .to_string();
    let site_subtitle = if site_subtitle.is_empty() {
        "AI 原生博客 · 最新页面列表".to_string()
    } else {
        site_subtitle
    };

    let rendered = replace_template(
        &template,
        &[
            ("site_header", &header_html),
            ("page_list", &rows),
            ("site_title", "SolinBlog"),
            ("site_subtitle", &site_subtitle),
            ("beian_number", &beian_html),
        ],
    )?;

    Ok(rendered)
}

fn replace_template(template: &str, values: &[(&str, &str)]) -> Result<String> {
    let mut out = template.to_string();
    for (key, value) in values {
        let placeholder = format!("{{{{{}}}}}", key);
        if !out.contains(&placeholder) {
            bail!("template missing placeholder {placeholder}");
        }
        out = out.replace(&placeholder, value);
    }
    Ok(out)
}

pub fn render_page_html(meta: &PageMeta, html: &str) -> String {
    let title = if meta.seo.title.is_empty() {
        &meta.seo.seo_title
    } else {
        &meta.seo.title
    };
    inject_seo_meta(html, title, &meta.seo)
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

pub fn render_markdown_page(markdown: &str) -> Result<String> {
    let markdown_html = markdown_to_html(markdown);
    let header_html = std::fs::read_to_string("front/header.html")
        .context("read front/header.html template")?;
    let template = std::fs::read_to_string("front/markdown.html")
        .context("read front/markdown.html template")?;
    let rendered = replace_template(
        &template,
        &[("site_header", &header_html), ("markdown_html", &markdown_html)],
    )?;
    Ok(rendered)
}

pub fn render_sitemap_xml(store: &PageStore, base_url: &str) -> Result<String> {
    let entries = store.list_page_entries().context("list page entries")?;
    let mut body = String::new();
    let base = normalize_base_url(base_url);
    for entry in entries {
        let meta = store
            .get_page_meta(&entry.page_id)
            .with_context(|| format!("load page meta {}", entry.page_id))?;
        let page_path = build_page_url(&entry.page_id, &entry.seo.seo_title);
        let page_url = format!("{}{}", base, page_path);
        let lastmod = format_unix_timestamp(meta.updated_at);
        body.push_str("  <url>\n");
        body.push_str(&format!(
            "    <loc>{}</loc>\n",
            escape_xml(&page_url)
        ));
        body.push_str(&format!(
            "    <lastmod>{}</lastmod>\n",
            escape_xml(&lastmod)
        ));
        body.push_str("    <changefreq>weekly</changefreq>\n");
        body.push_str("    <priority>0.8</priority>\n");
        body.push_str("  </url>\n");
    }

    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{}</urlset>",
        body
    ))
}

pub fn inject_seo_meta(html: &str, title: &str, seo: &crate::store::SeoMeta) -> String {
    let escaped_title = escape_html(title);
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

fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_string()
}

fn format_unix_timestamp(timestamp: i64) -> String {
    let safe_ts = timestamp.max(0);
    let datetime = Utc
        .timestamp_opt(safe_ts, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp(0, 0));
    datetime.to_rfc3339()
}

fn format_display_timestamp(timestamp: i64) -> String {
    let safe_ts = timestamp.max(0);
    let datetime = Utc
        .timestamp_opt(safe_ts, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp(0, 0));
    datetime.format("%Y-%m-%d %H:%M").to_string()
}
