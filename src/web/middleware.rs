// 实现对axum middleware的抽象
pub mod signator;

use crate::erx;
use crate::web::define::HttpMethod;
use axum::{extract::Request, http::request::Parts, response::Response};
use dashmap::DashMap;
use futures_util::future::BoxFuture;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context as TaskContext, Poll};
use std::time::{Duration, Instant};
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service, ServiceBuilder,
};

/// Considering the intended usage, the quantity here cannot be excessive, regardless of memory consumption.
static REGEX_CACHE: Lazy<DashMap<String, regex::Regex>> = Lazy::new(|| Default::default());

///
pub type MiddlewareFuture<T> = Pin<Box<dyn Future<Output = Result<(Context, T), (Context, erx::Erx)>> + Send>>;

pub trait ApplyTrait {
    fn apply(&self, value: &str) -> bool;
}

pub enum ApplyKind<T: ApplyTrait> {
    Include(T),
    Exclude(T),
}

impl<T> ApplyKind<T>
where
    T: ApplyTrait,
{
    /// pub fn apply(&self, tester: impl Fn(&T) -> bool) -> bool
    // pub fn apply<F>(&self, checker: F) -> bool
    // where
    //     F: Fn(&T) -> bool,
    // {
    //     match self {
    //         ApplyKind::Include(p) => checker(p),
    //         ApplyKind::Exclude(p) => !checker(p),
    //     }
    // }

    pub fn apply(&self, value: &str) -> bool {
        match self {
            ApplyKind::Include(t) => t.apply(value),
            ApplyKind::Exclude(t) => !t.apply(value),
        }
    }
}

pub enum Pattern {
    Prefix(String, bool),
    Suffix(String, bool),
    Contains(String, bool),
    Regex(String),
}

impl Pattern {
    fn regex(regs: &str, path: &str) -> bool {
        let invalid_regex = |pattern, error| -> bool {
            tracing::error!("Invalid regex pattern '{}': {}", pattern, error);
            false
        };

        match REGEX_CACHE.get(regs) {
            Some(regex) => regex.is_match(path),
            None => match regex::Regex::new(regs) {
                Ok(regex) => {
                    let result = regex.is_match(path);
                    REGEX_CACHE.insert(regs.into(), regex);
                    result
                },
                Err(e) => invalid_regex(regs, e),
            },
        }
    }
}

impl ApplyTrait for Pattern {
    fn apply(&self, path: &str) -> bool {
        match self {
            Pattern::Prefix(prefix, match_case) => {
                if *match_case {
                    path.starts_with(prefix)
                } else {
                    path.len() >= prefix.len() && path.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
                }
            },
            Pattern::Suffix(suffix, match_case) => {
                if *match_case {
                    path.ends_with(suffix)
                } else {
                    path.len() >= suffix.len() && path.as_bytes()[path.len() - suffix.len()..].eq_ignore_ascii_case(suffix.as_bytes())
                }
            },
            Pattern::Contains(contains, match_case) => {
                if *match_case {
                    path.contains(contains)
                } else {
                    path.to_ascii_lowercase().contains(&contains.to_ascii_lowercase())
                }
            },
            Pattern::Regex(regs) => Self::regex(regs, path),
        }

        // match self {
        //     Pattern::Prefix(prefix, match_case) => {
        //         if *match_case {
        //             path.starts_with(prefix)
        //         } else {
        //             path.to_lowercase().starts_with(&prefix.to_lowercase())
        //         }
        //     },
        //     Pattern::Suffix(suffix, match_case) => {
        //         if *match_case {
        //             path.ends_with(suffix)
        //         } else {
        //             path.to_lowercase().ends_with(&suffix.to_lowercase())
        //         }
        //     },
        //     Pattern::Contains(contains, match_case) => {
        //         if *match_case {
        //             path.contains(contains)
        //         } else {
        //             path.to_lowercase().contains(&contains.to_lowercase())
        //         }
        //     },
        //     Pattern::Regex(regs) => Self::regex(regs, path),
        // }
    }
}

impl ApplyTrait for HttpMethod {
    fn apply(&self, method: &str) -> bool {
        self.is(method)
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
    pub avg_request_latency: Averager,

    /// Average response latency calculator
    pub avg_response_latency: Averager,

    /// Trailing average request latency calculator
    pub avg_request_latency_trailer: Averager,

    /// Trailing average response latency calculator
    pub avg_response_latency_trailer: Averager,
    //请求延迟直方图 （Request Latency Histogram）
    // pub request_latency_hist: Vec<u64>,

    //响应延迟直方图 （Request Latency Histogram）
    // pub response_latency_hist: Vec<u64>,
}

impl Metrics {
    /// Update the average latency metrics by calculating the average from the tailer calculators
    /// and adding them to the main average calculators. Returns self for method chaining.
    pub fn update_avg(&mut self) -> &mut Self {
        if let avg @ 1.. = self.avg_request_latency_trailer.avg_then_reset() {
            self.avg_request_latency.add(avg);
        }

        if let avg @ 1.. = self.avg_response_latency_trailer.avg_then_reset() {
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
            self.avg_request_latency_trailer.add(latency);
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
            self.avg_response_latency_trailer.add(latency);
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
            avg_request_latency: Averager::default(),
            avg_response_latency: Averager::default(),
            avg_request_latency_trailer: Averager::default(),
            avg_response_latency_trailer: Averager::default(),
        }
    }
}

/// Average calculator for tracking metrics
///
/// This structure maintains a running sum and count to calculate averages.
/// It provides methods to add values, calculate the current average,
/// calculate and reset in one operation, and reset the calculator.
#[derive(Debug, Clone, Default)]
pub struct Averager {
    pub sum: u64,
    pub count: u64,
}

impl Averager {
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

/// re = request , so re_begin = request_begin
/// rs = response, so rs_begin = response_begin
impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), request: Default::default(), response: Default::default() }
    }

    /// request begin
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

/// if apply,methods,patterns all return None, this method is use for every request.
pub trait Middleware: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;

    fn on_request(&self, _context: Context, _request: Request) -> Option<MiddlewareFuture<Request>> {
        None
    }

    fn on_response(&self, _context: Context, _response: Response) -> Option<MiddlewareFuture<Response>> {
        None
    }

    /// Optional: middleware priority, higher values have higher priority
    fn priority(&self) -> i32 {
        0
    }

    /// Optional: determine if the middleware should handle this request
    /// Priority: apply > methods && patterns
    /// - If apply returns Some, use its return value directly for determination
    fn apply(&self, _parts: &Parts) -> Option<bool> {
        None
    }

    fn methods(&self) -> Option<Vec<ApplyKind<HttpMethod>>> {
        None
    }

    fn patterns(&self) -> Option<Vec<ApplyKind<Pattern>>> {
        None
    }
}

#[derive(Debug)]
pub struct Manager {
    middlewares: Vec<Box<dyn Middleware>>,
    metrics: DashMap<String, Arc<RwLock<Metrics>>>,
}

impl Manager {
    pub fn new() -> Self {
        Self { middlewares: Vec::new(), metrics: Default::default() }
    }

    pub fn add(&mut self, middleware: Box<dyn Middleware>) -> &mut Self {
        let name = middleware.name().to_string();
        for m in &self.middlewares {
            if m.name() == name {
                // panics is more suitable for all environments
                // reason:  middleware name is unique, if exists, it should panic.
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

    pub fn applies(&self, parts: &Parts) -> Vec<&Box<dyn Middleware>> {
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

        middleware.methods().map_or(true, |methods| {
            methods.iter().any(|method| {
                let m = parts.method.as_str();
                method.apply(m)
            })
        }) && middleware.patterns().map_or(true, |patterns| {
            let path = parts.uri.path();
            patterns.iter().any(|pattern| pattern.apply(path))
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

        let implement = async move {
            let (parts, body) = req.into_parts();
            let mut middles = manager.applies(&parts);

            let mut context = Some(Context::new());
            let mut request = Some(Request::from_parts(parts, body));
            let mut counter: usize = 0;

            for m in middles.iter_mut() {
                let mictx = context.take().unwrap();
                if mictx.aborted() {
                    break;
                }
                counter += 1;

                let name = m.name();
                let mut node = Node::new(name);
                node.re_begin();

                if let Some(f) = m.on_request(mictx, request.take().unwrap()) {
                    match f.await {
                        Ok(r) => {
                            context = Some(r.0);
                            request = Some(r.1);
                        },
                        Err(e) => {
                            node.re_errored();
                            context = Some(e.0);
                            tracing::error!("middleware '{}' on_request handle error: {}", name, e.1);
                        },
                    }
                }

                node.re_end();

                let _ = manager.metrics_update(name, |m| {
                    m.add_request(node.request.errored, node.request.esapsed);
                    Ok(())
                });

                context.as_mut().unwrap().chains.push(node);
                // context.unwrap().chains.push(node);
            }

            let response = if let Some(abt) = &mut context.as_mut().unwrap().aborted {
                abt.abort_response.take().unwrap_or_else(make_response)
            } else {
                inner.call(request.take().unwrap()).await.unwrap_or_else(|e| {
                    tracing::error!("Failed to handle request: {:?}", e);
                    make_response()
                })
            };

            // start process response

            let mut response = Some(response);
            while counter > 0 {
                counter -= 1;

                let m = middles[counter];
                let name = m.name();
                let mut node = Node::new(name);
                node.rs_begin();

                let mictx = context.take().unwrap();

                if let Some(f) = m.on_response(mictx, response.take().unwrap()) {
                    match f.await {
                        Ok(r) => {
                            context = Some(r.0);
                            response = Some(r.1);
                        },
                        Err(e) => {
                            node.rs_errored();
                            context = Some(e.0);
                            tracing::error!("middleware '{}' on_response handle error: {}", name, e.1);
                        },
                    }
                }

                node.rs_end();
                let _ = manager.metrics_update(name, |m| {
                    m.add_response(node.response.errored, node.response.esapsed);
                    Ok(())
                });
                context.as_mut().unwrap().chains.push(node);
            }

            Ok(response.unwrap())
        };

        Box::pin(implement)
    }
}

fn make_error_response(error: &str) -> Response {
    Response::builder()
        .status(500)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(format!(r#"{{"error": "{}"}}"#, error)))
        .unwrap_or_default()
}

fn make_response() -> Response {
    make_error_response("internal server error")
}

#[cfg(test)]
mod tests {}
