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

pub fn build_page_url(host: &str, page_id: &str, seo_title: &str) -> String {
    let encoded_title = utf8_percent_encode(seo_title, PATH_SEGMENT_ENCODE_SET).to_string();
    format!("http://{host}/pages/{encoded_title}+{page_id}")
}

pub fn parse_page_id_from_slug(slug: &str) -> Option<String> {
    let mut parts = slug.rsplitn(2, '+');
    let page_id = parts.next()?;
    if page_id.is_empty() {
        return None;
    }
    Some(page_id.to_string())
}

pub fn render_index_html(store: &PageStore, host: &str) -> Result<String> {
    let entries = store.list_page_entries().context("list page entries")?;
    let mut rows = String::new();
    for entry in entries {
        let title = escape_html(&entry.seo.seo_title);
        let description = escape_html(&entry.seo.description);
        let page_id = escape_html(&entry.page_id);
        let url = build_page_url(host, &entry.page_id, &entry.seo.seo_title);
        let url_attr = escape_html_attr(&url);
        rows.push_str(&format!(
            "<li><a href=\"{url_attr}\">{title}</a><p>{description}</p><small>{page_id}</small></li>",
        ));
    }

    Ok(format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>SolinBlog</title></head><body><main><h1>SolinBlog</h1><ul>{rows}</ul></main></body></html>"
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
