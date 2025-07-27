// 实现对axum middleware的抽象
pub mod signature;

use crate::erx;
use axum::{
    extract::Request,
    http::{request::Parts, Method},
    response::Response,
};
use dashmap::DashMap;
use futures_util::future::BoxFuture;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::task::{Context as TaskContext, Poll};
use std::time::{Duration, Instant};
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service, ServiceBuilder,
};

static REGEX_CACHE: Lazy<Mutex<DashMap<String, regex::Regex>>> = Lazy::new(|| Default::default());

pub type MiddlewareFuture = Pin<Box<dyn Future<Output = Result<(), erx::Erx>> + Send>>;

pub enum Pattern {
    Prefix(String),
    Suffix(String),
    Contains(String),
    Regex(String),
}

impl Pattern {
    pub fn check(&self, path: &str) -> bool {
        match self {
            Pattern::Prefix(prefix) => path.starts_with(prefix),
            Pattern::Suffix(suffix) => path.ends_with(suffix),
            Pattern::Contains(contains) => path.contains(contains),
            Pattern::Regex(regs) => Self::regex(regs, path),
        }
    }

    fn regex(regs: &str, path: &str) -> bool {
        let invalid_regex = |pattern, error| -> bool {
            tracing::error!("Invalid regex pattern '{}': {}", pattern, error);
            false
        };

        match REGEX_CACHE.try_lock() {
            Ok(cache) => match cache.get(regs) {
                Some(regex) => regex.is_match(path),
                _ => match regex::Regex::new(regs) {
                    Ok(regex) => {
                        let result = regex.is_match(path);
                        cache.insert(regs.to_string(), regex);
                        result
                    },
                    Err(e) => invalid_regex(regs, e),
                },
            },
            Err(_) => match regex::Regex::new(regs) {
                Ok(regex) => regex.is_match(path),
                Err(e) => invalid_regex(regs, e),
            },
        }
    }
}

pub enum ApplyKind<T> {
    Include(T),
    Exclude(T),
}

impl<T> ApplyKind<T> {
    /// pub fn apply(&self, tester: impl Fn(&T) -> bool) -> bool
    pub fn apply<F>(&self, checker: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        match self {
            ApplyKind::Include(p) => checker(p),
            ApplyKind::Exclude(p) => !checker(p),
        }
    }
}

/// Middleware metrics tracking structure
///
/// This structure maintains various metrics about middleware performance including:
/// - Request and response counts
/// - Error counts
/// - Latency measurements (total, min, max, average)
/// - Latency Units (microseconds) μs
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Count of requests received
    pub request_count: u64,

    /// Count of responses sent
    pub response_count: u64,

    /// Count of request processing errors
    pub request_error_count: u64,

    /// Count of response processing errors
    pub response_error_count: u64,

    /// Cumulative request processing latency
    pub request_latency: u64,

    /// Cumulative response processing latency
    pub response_latency: u64,

    /// Maximum request processing latency observed
    pub max_request_latency: u64,

    /// Maximum response processing latency observed
    pub max_response_latency: u64,

    /// Minimum request processing latency observed
    pub min_request_latency: u64,

    /// Minimum response processing latency observed
    pub min_response_latency: u64,

    /// Average request latency calculator
    pub avg_request_latency: AvgCalculator,

    /// Average response latency calculator
    pub avg_response_latency: AvgCalculator,

    /// Trailing average request latency calculator
    pub avg_request_latency_tailer: AvgCalculator,

    /// Trailing average response latency calculator
    pub avg_response_latency_tailer: AvgCalculator,
    //请求延迟直方图 （Request Latency Histogram）
    // pub request_latency_hist: Vec<u64>,

    //响应延迟直方图 （Request Latency Histogram）
    // pub response_latency_hist: Vec<u64>,
}

impl Metrics {
    /// Update the average latency metrics by calculating the average from the tailer calculators
    /// and adding them to the main average calculators. Returns self for method chaining.
    pub fn update_avg(&mut self) -> &mut Self {
        if let avg @ 1.. = self.avg_request_latency_tailer.avg_then_reset() {
            self.avg_request_latency.add(avg);
        }

        if let avg @ 1.. = self.avg_response_latency_tailer.avg_then_reset() {
            self.avg_response_latency.add(avg);
        }

        self
    }

    pub fn add_request(&mut self, errored: bool, duration: Duration) -> &mut Self {
        self.request_count += 1;
        if errored {
            self.request_error_count += 1;
        } else {
            let latency = duration.as_micros() as u64;
            self.avg_request_latency_tailer.add(latency);
            self.min_request_latency = self.min_request_latency.min(latency);
            self.max_request_latency = self.max_request_latency.max(latency);
        }

        self
    }

    pub fn add_response(&mut self, errored: bool, duration: Duration) -> &mut Self {
        self.response_count += 1;
        if errored {
            self.response_error_count += 1;
        } else {
            let latency = duration.as_micros() as u64;
            self.avg_response_latency_tailer.add(latency);
            self.min_response_latency = self.min_response_latency.min(latency);
            self.max_response_latency = self.max_response_latency.max(latency);
        }

        self
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            request_count: 0,
            response_count: 0,
            request_error_count: 0,
            response_error_count: 0,
            request_latency: 0,
            response_latency: 0,
            max_request_latency: 0,
            max_response_latency: 0,
            min_request_latency: u64::MAX,
            min_response_latency: u64::MAX,
            avg_request_latency: AvgCalculator::default(),
            avg_response_latency: AvgCalculator::default(),
            avg_request_latency_tailer: AvgCalculator::default(),
            avg_response_latency_tailer: AvgCalculator::default(),
        }
    }
}

/// Average calculator for tracking metrics
///
/// This structure maintains a running sum and count to calculate averages.
/// It provides methods to add values, calculate the current average,
/// calculate and reset in one operation, and reset the calculator.
#[derive(Debug, Clone, Default)]
pub struct AvgCalculator {
    pub sum: u64,
    pub count: u64,
}

impl AvgCalculator {
    pub fn add(&mut self, value: u64) -> &mut Self {
        self.sum += value;
        self.count += 1;

        self
    }

    pub fn avg(&self) -> u64 {
        if self.count == 0 {
            return 0;
        }
        self.sum / self.count
    }

    pub fn avg_then_reset(&mut self) -> u64 {
        let avg = self.avg();
        self.reset();
        avg
    }

    pub fn reset(&mut self) -> &mut Self {
        self.sum = 0;
        self.count = 0;

        self
    }
}

#[derive(Debug, Clone)]
pub struct Point {
    pub created: Option<Instant>,
    pub esapsed: Duration,
    pub errored: bool,
}

impl Point {
    pub fn clac_elapsed(&mut self) -> &mut Self {
        if let Some(instant) = self.created {
            self.esapsed = instant.elapsed();
        }
        self
    }
}

impl Default for Point {
    fn default() -> Self {
        Self { created: None, esapsed: Duration::default(), errored: false }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub name: String,
    pub request: Point,
    pub response: Point,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), request: Default::default(), response: Default::default() }
    }

    pub fn re_begin(&mut self) -> &mut Self {
        self.request.created = Some(Instant::now());
        self
    }

    pub fn re_errored(&mut self) -> &mut Self {
        self.request.errored = true;
        self
    }

    pub fn re_end(&mut self) -> &mut Self {
        self.request.clac_elapsed();
        self
    }

    pub fn rs_begin(&mut self) -> &mut Self {
        self.response.created = Some(Instant::now());
        self
    }

    pub fn rs_errored(&mut self) -> &mut Self {
        self.response.errored = true;
        self
    }

    pub fn rs_end(&mut self) -> &mut Self {
        self.response.clac_elapsed();
        self
    }
}

#[derive(Debug)]
pub struct Abort {
    pub middleware: String,
    pub message: String,
    pub abort_at: std::time::Instant,
    pub abort_response: Option<Response>,
}

impl Default for Abort {
    fn default() -> Self {
        Abort { middleware: String::new(), message: String::new(), abort_at: std::time::Instant::now(), abort_response: None }
    }
}

#[derive(Debug)]
pub struct Context {
    pub metadata: IndexMap<String, String>,
    pub aborted: Option<Abort>,
    pub start_at: std::time::Instant,
    pub chains: Vec<Node>,
}

impl Context {
    pub fn new() -> Self {
        Context::default()
    }

    pub fn aborted(&self) -> bool {
        self.aborted.is_some()
    }

    ///  Abort middleware processing with message
    ///
    pub fn make_abort(&mut self, middleware: impl Into<String>, message: impl Into<String>) {
        self.aborted = Some(Abort {
            middleware: middleware.into(),
            message: message.into(),
            abort_at: std::time::Instant::now(),
            abort_response: None,
        });
    }

    /// Abort middleware processing with a custom response
    ///
    /// This method sets the aborted state with the provided middleware name,
    /// error message, and a custom response that will be returned to the client.
    ///
    pub fn make_abort_with_response(&mut self, middleware: impl Into<String>, message: impl Into<String>, response: impl Into<Response>) {
        self.aborted = Some(Abort {
            middleware: middleware.into(),
            message: message.into(),
            abort_at: std::time::Instant::now(),
            abort_response: Some(response.into()),
        });
    }

    /// elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_at.elapsed()
    }

    /// Insert a single metadata key-value pair
    pub fn insert_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata key-value pairs to the context
    ///
    /// This method accepts various types of key-value collections:
    /// ```
    /// // IndexMap
    /// context.extend_metadata(&index_map);
    /// // HashMap
    /// context.extend_metadata(&hash_map);
    /// // Vec
    /// context.extend_metadata(&vec);
    /// // Arrays
    /// context.extend_metadata([("k1", "v1"), ("k2", "v2")]);
    /// // Iterators
    /// context.extend_metadata(hash_map.iter());
    /// // Mixed types
    /// context.extend_metadata(vec![("key".to_string(), "value"), ("key2", "value2".to_string())]);
    /// ```
    pub fn extend_metadata<I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in iter {
            self.metadata.insert(key.into(), value.into());
        }
        self
    }

    ///
    pub fn extend_metadata_owned<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        for (key, value) in iter {
            self.metadata.insert(key, value);
        }
        self
    }
}

impl Default for Context {
    fn default() -> Self {
        Self { metadata: IndexMap::new(), aborted: None, start_at: std::time::Instant::now(), chains: vec![] }
    }
}

///

pub trait Middleware: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;

    fn on_request(&self, _context: &mut Context, _request: &mut Request) -> Option<MiddlewareFuture> {
        None
    }

    fn on_response(&self, _context: &mut Context, _response: &mut Response) -> Option<MiddlewareFuture> {
        None
    }

    /// 可选：中间件优先级，数值越大优先级越高
    fn priority(&self) -> i32 {
        0
    }

    /// 可选：判断中间件是否应该处理这个请求
    /// 优先级 focus > methods > patterns
    /// - 如果foucs返回不为None,直接使用foucs的返回值判定
    fn apply(&self, _parts: &Parts) -> Option<bool> {
        None
    }

    /// 可选：HTTP 方法过滤
    fn methods(&self) -> Option<Vec<ApplyKind<Method>>> {
        None
    }

    /// 可选：路径匹配模式
    fn patterns(&self) -> Option<Vec<ApplyKind<Pattern>>> {
        None
    }
}

#[derive(Debug)]
pub struct Manager {
    middlewares: Vec<Box<dyn Middleware>>,
    metrics: DashMap<String, Arc<RwLock<Metrics>>>,
}

// #[derive(Debug)]
// pub struct Manager<M>
// where
//     M: Middleware,
// {
//     middlewares: Vec<M>,
// }

impl Manager {
    pub fn new() -> Self {
        Self { middlewares: Vec::new(), metrics: Default::default() }
    }

    pub fn add(&mut self, middleware: Box<dyn Middleware>) -> &mut Self {
        let name = middleware.name().to_string();
        for m in &self.middlewares {
            if m.name() == name {
                panic!("Middleware with name '{}' already exists", name);
            }
        }

        if !self.metrics.contains_key(&name) {
            self.metrics.insert(name, Default::default());
        }

        self.middlewares.push(middleware);
        self.middlewares.sort_by(|a, b| a.priority().cmp(&b.priority()));
        self
    }

    pub fn applys(&self, parts: &Parts) -> Vec<&Box<dyn Middleware>> {
        let mut middlewares = Vec::new();
        for middleware in &self.middlewares {
            if self.should_apply_middleware(middleware, parts) {
                middlewares.push(middleware);
            }
        }
        middlewares
    }

    fn should_apply_middleware(&self, middleware: &Box<dyn Middleware>, parts: &Parts) -> bool {
        if let Some(apply) = middleware.apply(parts) {
            return apply;
        }

        middleware.methods().map_or(false, |methods| methods.iter().any(|method| method.apply(|m| m.eq(&parts.method))))
            && middleware.patterns().map_or(false, |patterns| {
                let path = parts.uri.path();
                patterns.iter().any(|pattern| pattern.apply(|p| p.check(path)))
            })
    }

    pub fn metrics_update<F>(&self, name: &str, f: F) -> Result<(), erx::Erx>
    where
        F: FnOnce(&mut Metrics) -> Result<(), erx::Erx>,
    {
        self.metrics.get_mut(name).map_or(Err(erx::Erx::new("metrics not found")), |metrics_ref| {
            let metrics_ref = Arc::clone(&metrics_ref);
            let metrics_guard = metrics_ref.try_write();
            match metrics_guard {
                Ok(mut metrics_guard) => f(&mut metrics_guard),
                Err(ex) => Err(erx::Erx::new(ex.to_string().as_str())),
            }
        })
    }

    pub fn integrated(manager: Arc<Manager>, router: axum::Router) -> axum::Router {
        router.layer(ManagerLayer { manager: Arc::clone(&manager) })
    }
}

impl Into<ServiceBuilder<Stack<ManagerLayer, Identity>>> for ManagerLayer {
    fn into(self) -> ServiceBuilder<Stack<ManagerLayer, Identity>> {
        ServiceBuilder::new().layer(self)
    }
}

#[derive(Debug, Clone)]
pub struct ManagerLayer {
    pub manager: Arc<Manager>,
}

#[derive(Debug, Clone)]
pub struct ManagerService<S>
where
    S: Clone,
{
    inner: S,
    manager: Arc<Manager>,
}

impl<S> Layer<S> for ManagerLayer
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    S::Error: Into<erx::Erx>,
{
    type Service = ManagerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ManagerService { inner, manager: Arc::clone(&self.manager) }
    }
}

impl<S> Service<Request> for ManagerService<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let manager = Arc::clone(&self.manager);

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let mut middles = manager.applys(&parts);

            let mut context = Context::new();
            let mut req = Request::from_parts(parts, body);

            let mut counter: usize = 0;

            for m in middles.iter_mut() {
                if context.aborted() {
                    break;
                }
                counter += 1;

                let name = m.name();

                let mut node = Node::new(name);
                node.re_begin();

                if let Some(f) = m.on_request(&mut context, &mut req) {
                    if let Err(e) = f.await {
                        node.re_errored();
                        tracing::error!("middleware '{}' on_request handle error: {}", name, e);
                    }
                }

                node.re_end();

                let _ = manager.metrics_update(name, |m| {
                    m.add_request(node.request.errored, node.request.esapsed);
                    Ok(())
                });

                context.chains.push(node);
            }

            let mut res = if let Some(abt) = &mut context.aborted {
                abt.abort_response.take().unwrap_or_else(make_response)
            } else {
                inner.call(req).await.unwrap_or_else(|e| {
                    tracing::error!("Failed to handle request: {:?}", e);
                    make_response()
                })
            };

            while counter > 0 {
                counter -= 1;

                let m = middles[counter];
                let name = m.name();

                let mut node = Node::new(name);
                node.rs_begin();

                if let Some(f) = m.on_response(&mut context, &mut res) {
                    if let Err(e) = f.await {
                        node.rs_errored();
                        tracing::error!("middleware '{}' on_request handle error: {}", name, e);
                    }
                }

                node.rs_end();
                let _ = manager.metrics_update(name, |m| {
                    m.add_response(node.response.errored, node.response.esapsed);
                    Ok(())
                });
            }

            Ok(res)
        })
    }
}

fn make_response() -> Response {
    let body = r#"{"message": "Hello, World!"}"#;
    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {}
