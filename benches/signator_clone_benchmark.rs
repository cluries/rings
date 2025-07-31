use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rings::web::middleware::signator::SignatorConfig;
use rings::web::middleware::{ApplyKind, Pattern};
use rings::web::define::HttpMethod;

fn create_complex_config() -> SignatorConfig {
    SignatorConfig::new()
        .priority(100)
        .nonce_lifetime(600)
        .include_method(HttpMethod::POST)
        .include_method(HttpMethod::PUT)
        .include_method(HttpMethod::DELETE)
        .include_method(HttpMethod::PATCH)
        .include_prefix("/api/v1/", true)
        .include_prefix("/api/v2/", true)
        .exclude_prefix("/public/", false)
        .include_suffix(".json", true)
        .exclude_suffix(".html", false)
        .include_contains("admin", true)
        .exclude_contains("temp", false)
        .include_regex(r"^/api/v\d+/users/\d+$")
        .exclude_regex(r".*\.(css|js|png|jpg|gif)$")
        .apply(|parts| {
            let path = parts.uri.path();
            path.starts_with("/api/") && !path.contains("/public/")
        })
}

fn benchmark_config_clone(c: &mut Criterion) {
    let config = create_complex_config();
    
    c.bench_function("signator_config_clone", |b| {
        b.iter(|| {
            let cloned = black_box(config.clone());
            black_box(cloned)
        })
    });
}

fn benchmark_multiple_clones(c: &mut Criterion) {
    let config = create_complex_config();
    
    c.bench_function("signator_config_1000_clones", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let cloned = black_box(config.clone());
                black_box(cloned);
            }
        })
    });
}

criterion_group!(benches, benchmark_config_clone, benchmark_multiple_clones);
criterion_main!(benches);