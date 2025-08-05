use rings::{axum::Router, tower_http, tracing::info};

pub fn doc_actions() -> Vec<Router> {
    if !rings::conf::rebit().read().unwrap().debug {
        return Vec::new();
    }

    info!("using doc actions");

    use tower_http::services::ServeDir;
    vec![Router::new().nest_service("/doc", ServeDir::new("./data/doc"))]
}
