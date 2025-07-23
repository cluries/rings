// 实现对axum middleware的抽象
pub mod signature;

use crate::erx;
use axum::{
    extract::Request,
    http::{request::Parts, Method},
    response::Response,
};
use futures_util::future::BoxFuture;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll};
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service, ServiceBuilder,
};

static REGEX_CACHE: Lazy<Mutex<HashMap<String, regex::Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub enum Pattern {
    Prefix(String),
    Suffix(String),
    Contains(String),
    Regex(String),
}

pub enum ApplyKind<T> {
    Include(T),
    Exclude(T),
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub request_count: u64,
    pub error_count: u64,
    pub longest_on_request_time: u64,
    pub total_on_request_time: u64,
    pub longest_on_response_time: u64,
    pub total_on_response_time: u64,
}

#[derive(Debug, Clone)]
pub struct Chain {
    pub name: String,
    pub metrics: Metrics,
}

#[derive(Debug)]
pub struct Context {
    pub request: Option<Request>,
    pub response: Option<Response>,

    pub chains: Vec<Chain>,
    pub metadata: HashMap<String, String>,
    pub aborted: bool,
}

#[derive(Debug, Clone)]
pub enum Error {
    Abort(erx::Erx),
    Ingore(erx::Erx),
    Continue(erx::Erx),
}

///
pub type MiddlewareFuture = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

pub trait Middleware: Send + Sync {
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

pub struct Manager {
    middlewares: Vec<Box<dyn Middleware>>,
}

pub struct ManagerLayer {
    pub manager: Arc<Manager>,
}

pub struct ManagerService<S>
where
    S: Clone,
{
    inner: S,
    manager: Arc<Manager>,
}

impl Context {
    pub fn new() -> Self {
        Context::default()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self { request: None, response: None, chains: vec![], metadata: Default::default(), aborted: false }
    }
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
            Ok(mut cache) => match cache.get(regs) {
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

impl Manager {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add(&mut self, middleware: Box<dyn Middleware>) -> &mut Self {
        for m in &self.middlewares {
            if m.name() == middleware.name() {
                panic!("Middleware with name '{}' already exists", middleware.name());
            }
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

        // if let Some(methods) = middleware.methods() {
        //     if methods.iter().any(|method| method.apply(|m| m.eq(&parts.method))) {
        //         return true;
        //     }
        // }
        //
        //
        // if let Some(patterns) = middleware.patterns() {
        //     let path = parts.uri.path();
        //     if patterns.iter().any(|pattern| pattern.apply(|p| p.check(path))) {
        //         return true;
        //     }
        // }
        //
        // false

        middleware.methods().map_or(false, |methods| methods.iter().any(|method| method.apply(|m| m.eq(&parts.method))))
            || middleware.patterns().map_or(false, |patterns| {
                let path = parts.uri.path();
                patterns.iter().any(|pattern| pattern.apply(|p| p.check(path)))
            })
    }

    pub fn integrated(_manager: Arc<Manager>, router: axum::Router) -> axum::Router {
        // 创建一个新的中间件层
        // router.layer(ManagerLayer { manager: Arc::clone(&manager) })

        router
    }
}

impl Into<ServiceBuilder<Stack<ManagerLayer, Identity>>> for ManagerLayer {
    fn into(self) -> ServiceBuilder<Stack<ManagerLayer, Identity>> {
        ServiceBuilder::new().layer(self)
    }
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
            let middles = manager.applys(&parts);

            let mut context = Context::new();
            let mut req = Request::from_parts(parts, body);

            let mut counter: usize = 0;
            for m in middles.iter() {
                counter += 1;
                if let Some(mifuture) = m.on_request(&mut context, &mut req) {
                    if let Err(e) = mifuture.await {
                        tracing::error!("Failed to handle request: {:?}", e);
                    }
                }
            }

            let mut res = match inner.call(req).await {
                Ok(response) => response,
                Err(e) => {
                    tracing::error!("Failed to handle request: {:?}", e);
                    Response::builder().status(500).body(axum::body::Body::from("Internal Server Error")).unwrap()
                },
            };

            while counter > 0 {
                counter -= 1;
                if let Some(mifuture) = middles[counter].on_response(&mut context, &mut res) {
                    match mifuture.await {
                        Ok(_) => continue,
                        Err(e) => {
                            tracing::error!("Failed to handle response: {:?}", e);
                        },
                    }
                }
            }

            Ok(res)
        })
    }
}
