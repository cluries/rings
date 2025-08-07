use rings::axum::http::request::Parts;
use rings::axum::Router;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use rings::web::middleware::signator::{
    Signator,
    SignatorConfig
};

 
pub fn use_signator() -> Signator {
    let key_loader = Arc::new(|user_id: String| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, rings::erx::Erx>> + Send>> {
        Box::pin(sig_key_loader(user_id))
    });
    
    let conf = signator_conf();
    let redis_url = conf.get("redis_url").unwrap_or(&"redis://localhost:6379".to_string()).clone();
    
    let config = SignatorConfig::new(key_loader, redis_url);
    
    Signator::new(config).expect("Failed to create Signator")
}
 

fn signator_conf() -> std::collections::HashMap<String, String> {
    let rebit = rings::conf::rebit().try_read().expect("Failed to read RINGS configuration");
    rebit.web_middleware("api", "signator").expect("missing signator in Web[API]")
}

async fn sig_key_loader(u: String) -> Result<String, rings::erx::Erx> {
    println!("API:loading key from {}", u);
    Ok(u)
}
