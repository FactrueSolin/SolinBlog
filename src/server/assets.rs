use std::path::{Component, Path as FsPath, PathBuf};

use axum::{
    extract::Path,
    http::{StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse},
};
use mime_guess::MimeGuess;

use crate::web::render_404_html;

pub async fn public_asset_handler(Path(path): Path<String>) -> impl IntoResponse {
    if path.is_empty() {
        return match render_404_html() {
            Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render 404 failed: {err}"),
            )
                .into_response(),
        };
    }
    let Ok(safe_path) = sanitize_public_path(&path) else {
        return match render_404_html() {
            Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render 404 failed: {err}"),
            )
                .into_response(),
        };
    };
    let full_path = PathBuf::from("public").join(&safe_path);
    let data = match std::fs::read(&full_path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return match render_404_html() {
                Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("render 404 failed: {err}"),
                )
                    .into_response(),
            };
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("read public asset failed: {err}"),
            )
                .into_response();
        }
    };
    let mime = guess_mime_type(&full_path);
    ([(CONTENT_TYPE, mime.as_ref())], data).into_response()
}

pub fn sanitize_public_path(raw: &str) -> Result<PathBuf, ()> {
    let mut cleaned = PathBuf::new();
    for segment in raw.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(());
        }
        let segment_path = FsPath::new(segment);
        let mut segment_components = segment_path.components();
        match segment_components.next() {
            Some(Component::Normal(_)) if segment_components.next().is_none() => {}
            _ => return Err(()),
        }
        cleaned.push(segment);
    }
    if cleaned.as_os_str().is_empty() {
        return Err(());
    }
    Ok(cleaned)
}

fn guess_mime_type(path: &FsPath) -> mime_guess::Mime {
    MimeGuess::from_path(path).first_or_octet_stream()
}
