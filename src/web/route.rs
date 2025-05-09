use axum::Router;

pub fn merge(a: Router, b: Router) -> Router {
    Router::new().merge(a).merge(b)
}

pub fn merge_vec(routers: Vec<Router>) -> Router {
    let mut router = Router::new();
    for r in routers {
        router = merge(router, r);
    }
    router
}

pub fn merge_dict(routers: std::collections::HashMap<String, Router>) -> Router {
    let mut router = Router::new();
    for (_, r) in routers {
        router = merge(router, r);
    }
    router
}
