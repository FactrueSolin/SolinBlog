use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, Query, State, multipart::MultipartError},
    http::{
        HeaderMap, HeaderValue, Request, StatusCode,
        header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, ETAG, IF_NONE_MATCH},
    },
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_util::io::ReaderStream;

use crate::image_host::{ImageHostError, ImageMeta, ImageMetaPatch, ImageStore};
use crate::mcp::{TokenStore, extract_bearer_token};
use crate::web::{inject_umami_script, resolve_base_url};

#[derive(Clone)]
pub struct ImageWebState {
    pub store: Arc<ImageStore>,
    pub max_upload_bytes: usize,
}

#[derive(Serialize)]
struct ImageApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ImageApiErrorBody>,
}

#[derive(Serialize)]
struct ImageApiErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
pub struct UploadImageResponse {
    pub image_id: String,
    pub url: String,
    pub meta: ImageMeta,
}

#[derive(Serialize)]
pub struct ListImagesResponse {
    pub items: Vec<ImageMeta>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Serialize)]
pub struct DeleteImageResponse {
    pub deleted: bool,
    pub image_id: String,
}

#[derive(Deserialize)]
pub struct ListImagesQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateImageRequest {
    pub alt: Option<String>,
    pub description: Option<String>,
}

struct MultipartImageInput {
    file: Vec<u8>,
    alt: Option<String>,
    description: Option<String>,
}

pub async fn image_auth_middleware(
    State(token_store): State<Arc<TokenStore>>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    match extract_bearer_token(&headers) {
        Some(token) if token_store.is_valid(&token) => Ok(next.run(request).await),
        _ => Err(api_error(ImageHostError::Unauthorized)),
    }
}

pub async fn image_page_handler() -> impl IntoResponse {
    match std::fs::read_to_string("front/image.html") {
        Ok(html) => Html(inject_umami_script(&html)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read image html failed: {err}"),
        )
            .into_response(),
    }
}

pub async fn upload_image_handler(
    State(state): State<ImageWebState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let base_url = resolve_base_url(&headers);
    let input = match read_multipart_image(multipart, state.max_upload_bytes).await {
        Ok(input) => input,
        Err(err) => return api_error(err),
    };
    let record = match state
        .store
        .create_image(
            input.file,
            input.alt.unwrap_or_default(),
            input.description.unwrap_or_default(),
            state.max_upload_bytes,
        )
        .await
    {
        Ok(record) => record,
        Err(err) => return api_error(err),
    };
    let meta = ImageMeta::from_record(record, &base_url);
    api_success_with_status(
        StatusCode::CREATED,
        UploadImageResponse {
            image_id: meta.image_id.clone(),
            url: meta.url.clone(),
            meta,
        },
    )
}

pub async fn list_images_handler(
    State(state): State<ImageWebState>,
    headers: HeaderMap,
    Query(query): Query<ListImagesQuery>,
) -> Response {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let base_url = resolve_base_url(&headers);
    match state
        .store
        .list_images(limit, offset, query.q.as_deref())
        .await
    {
        Ok((items, total)) => {
            let items = items
                .into_iter()
                .map(|item| ImageMeta::from_record(item, &base_url))
                .collect();
            api_success(ListImagesResponse {
                items,
                total,
                limit,
                offset,
            })
        }
        Err(err) => api_error(err),
    }
}

pub async fn get_image_handler(
    State(state): State<ImageWebState>,
    headers: HeaderMap,
    Path(image_id): Path<String>,
) -> Response {
    let base_url = resolve_base_url(&headers);
    match state.store.get_image(&image_id).await {
        Ok(record) => api_success(ImageMeta::from_record(record, &base_url)),
        Err(err) => api_error(err),
    }
}

pub async fn update_image_handler(
    State(state): State<ImageWebState>,
    headers: HeaderMap,
    Path(image_id): Path<String>,
    Json(request): Json<UpdateImageRequest>,
) -> Response {
    let base_url = resolve_base_url(&headers);
    let patch = ImageMetaPatch {
        alt: request.alt,
        description: request.description,
    };
    match state.store.update_image_meta(&image_id, patch).await {
        Ok(record) => api_success(ImageMeta::from_record(record, &base_url)),
        Err(err) => api_error(err),
    }
}

pub async fn replace_image_handler(
    State(state): State<ImageWebState>,
    headers: HeaderMap,
    Path(image_id): Path<String>,
    multipart: Multipart,
) -> Response {
    let base_url = resolve_base_url(&headers);
    let input = match read_multipart_image(multipart, state.max_upload_bytes).await {
        Ok(input) => input,
        Err(err) => return api_error(err),
    };
    let patch = ImageMetaPatch {
        alt: input.alt,
        description: input.description,
    };
    match state
        .store
        .replace_image(&image_id, input.file, patch, state.max_upload_bytes)
        .await
    {
        Ok(record) => api_success(ImageMeta::from_record(record, &base_url)),
        Err(err) => api_error(err),
    }
}

pub async fn delete_image_handler(
    State(state): State<ImageWebState>,
    Path(image_id): Path<String>,
) -> Response {
    match state.store.delete_image(&image_id).await {
        Ok(()) => api_success(DeleteImageResponse {
            deleted: true,
            image_id,
        }),
        Err(err) => api_error(err),
    }
}

pub async fn image_asset_handler(
    State(state): State<ImageWebState>,
    headers: HeaderMap,
    Path((image_id, filename)): Path<(String, String)>,
) -> Response {
    let public = match state.store.get_public_image(&image_id, &filename).await {
        Ok(public) => public,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let etag = format!("\"sha256-{}\"", public.record.sha256);
    if headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == etag)
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        if let Ok(value) = HeaderValue::from_str(&etag) {
            response.headers_mut().insert(ETAG, value);
        }
        return response;
    }
    let file = match tokio::fs::File::open(&public.path).await {
        Ok(file) => file,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let stream = ReaderStream::new(file);
    let mut response = Response::new(Body::from_stream(stream));
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&public.record.content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    if let Ok(value) = HeaderValue::from_str(&public.record.size_bytes.to_string()) {
        response.headers_mut().insert(CONTENT_LENGTH, value);
    }
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    response.headers_mut().insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    if let Ok(value) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(ETAG, value);
    }
    response
}

async fn read_multipart_image(
    mut multipart: Multipart,
    max_upload_bytes: usize,
) -> Result<MultipartImageInput, ImageHostError> {
    let mut file = None;
    let mut alt = None;
    let mut description = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(multipart_error)?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                if file.is_some() {
                    return Err(ImageHostError::InvalidRequest(
                        "only one file is allowed".to_string(),
                    ));
                }
                let bytes = field.bytes().await.map_err(multipart_error)?;
                if bytes.len() > max_upload_bytes {
                    return Err(ImageHostError::PayloadTooLarge);
                }
                file = Some(bytes.to_vec());
            }
            "alt" => {
                alt = Some(field.text().await.map_err(multipart_error)?);
            }
            "description" => {
                description = Some(field.text().await.map_err(multipart_error)?);
            }
            _ => {}
        }
    }

    let file = file.ok_or_else(|| ImageHostError::InvalidRequest("file is required".to_string()))?;
    Ok(MultipartImageInput {
        file,
        alt,
        description,
    })
}

fn multipart_error(error: MultipartError) -> ImageHostError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        ImageHostError::PayloadTooLarge
    } else {
        ImageHostError::InvalidRequest(error.body_text())
    }
}

fn api_success<T: Serialize>(data: T) -> Response {
    api_success_with_status(StatusCode::OK, data)
}

fn api_success_with_status<T: Serialize>(status: StatusCode, data: T) -> Response {
    (
        status,
        Json(ImageApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }),
    )
        .into_response()
}

fn api_error(error: ImageHostError) -> Response {
    let status = match error {
        ImageHostError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        ImageHostError::Unauthorized => StatusCode::UNAUTHORIZED,
        ImageHostError::NotFound => StatusCode::NOT_FOUND,
        ImageHostError::Conflict(_) => StatusCode::CONFLICT,
        ImageHostError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ImageHostError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ImageHostError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(ImageApiResponse::<()> {
            success: false,
            data: None,
            error: Some(ImageApiErrorBody {
                code: error.code(),
                message: error.message(),
            }),
        }),
    )
        .into_response()
}
