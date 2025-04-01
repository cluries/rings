use axum::Router;

pub fn merge(a: Router, b: Router) -> Router {
    Router::new().merge(a).merge(b)
}

 