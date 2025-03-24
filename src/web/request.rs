use axum::body::Bytes;
use tracing::error;

// TODO 2019.10.24
//
// i think this is a stupid method to clone a request
// but i don't know how to do it, so i just do it, i will make it better later
//
pub async fn clone_request(req: axum::extract::Request) -> (axum::extract::Request, axum::extract::Request) {
    const LIMIT: usize = 1024 * 1024 * 32;

    let (parts, body) = req.into_parts();

    let bytes = axum::body::to_bytes(body, LIMIT).await.unwrap_or_else(|e| {
        error!("axum::body::to_bytes error: {}", e);
        Bytes::new()
    });

    (
        axum::extract::Request::from_parts(parts.clone(), axum::body::Body::from(bytes.clone())),
        axum::extract::Request::from_parts(parts.clone(), axum::body::Body::from(bytes.clone())),
    )
}
