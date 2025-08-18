use rings::tools::rand::rand_i64;
use rings::web::middleware::signator::{debug_level, Signator, SignatorConfig};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn use_signator() -> Signator {
    let conf = signator_conf();
    let redis_url = conf.get("redis_url").expect("unable get signator redis url").clone();

    // async fn sig_key_loader(u: String) -> Result<String, rings::erx::Erx> {
    //     println!("API:loading key from {}", u);
    //     Ok(format!("{}-{}", u, rand_i64(1, 10000000)))
    // }
    //
    // fn loader(user_id: String) -> Pin<Box<dyn Future<Output = Result<String, rings::erx::Erx>> + Send>> {
    //
    //     Box::pin(sig_key_loader(user_id))
    // }

    let loader = |user_id: String| {
        Box::pin(async move {
            println!("API: loading key from {}", user_id);
            Ok(format!("{}-{}", user_id, rand_i64(1, 10_000_000)))
        }) as Pin<Box<dyn Future<Output = Result<String, rings::erx::Erx>> + Send>>
    };

    let mut config = SignatorConfig::new(Arc::new(loader), redis_url);
    config.set_debug_level(debug_level::LOG_AND_RESPONSE);
    Signator::new(config).expect("Failed to create Signator")
}

fn signator_conf() -> std::collections::HashMap<String, String> {
    let rebit = rings::conf::rebit().try_read().expect("Failed to read RINGS configuration");
    rebit.web_middleware("api", "signator").expect("missing signator in Web[API]")
}
