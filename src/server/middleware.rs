use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};

pub async fn log_request(req: Request<Body>, next: Next) -> Response {
    let upgrade = req
        .headers()
        .get("upgrade")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-");
    let connection = req
        .headers()
        .get("connection")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-");
    println!(
        "[solin-blog] {} {} upgrade={} connection={}",
        req.method(),
        req.uri(),
        upgrade,
        connection
    );
    let response = next.run(req).await;
    println!("[solin-blog] -> {}", response.status());
    response
}
