use rings::axum::http::request::Parts;
use rings::axum::Router;
use std::sync::Arc;

pub fn use_signator(router: Router) -> Router {
    router
}

fn signator_conf() -> std::collections::HashMap<String, String> {
    let rebit = rings::conf::rebit().try_read().expect("Failed to read RINGS configuration");
    rebit.web_middleware("api", "signator").expect("missing signator in Web[API]")
}

async fn sig_key_loader(u: String) -> Result<String, rings::erx::Erx> {
    println!("API:loading key from {}", u);
    Ok(u)
}
