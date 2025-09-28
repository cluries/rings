static TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

/// shared tokio runtime
pub fn shared_tokio_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

pub fn tokio_block_on<F: core::future::Future>(future: F) -> F::Output {
    shared_tokio_runtime().block_on(future)
}
