pub struct Middleware {
    pub focus: fn(parts: &axum::http::request::Parts) -> bool,
    pub work: fn(request: axum::extract::Request, next: axum::middleware::Next) -> Option<axum::response::Response>,
}
