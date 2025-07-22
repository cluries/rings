// 实现对axum middleware的抽象
pub mod signature;

use axum::http::request::Parts;
use axum::{extract::Request, response::Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use crate::erx;
use axum::http::Method;
use std::task::{Context as TaskContext, Poll};
use tower::{Layer, Service};

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

#[derive(Debug, Clone)]
pub struct Metrics {
    pub start_time: Instant,
    pub request_count: u64,
    pub error_count: u64,
    pub processing_time: Option<std::time::Duration>,
}

pub struct Chain {
    pub name: String,
    pub metrics: Metrics,
}

pub struct Context {
    pub request: Request,
    pub response: Option<Response>,
    pub chains: Vec<Chain>,
    pub aborted: bool,
}

pub type MiddlewareFuture = Pin<Box<dyn Future<Output = Result<Arc<Context>, erx::Erx>> + Send>>;

pub trait Middleware: Send + Sync {
    fn name(&self) -> &'static str;

    fn on_request(&self, _context: Arc<Context>) -> Option<MiddlewareFuture> {
        None
    }

    fn on_response(&self, _context: Arc<Context>) -> Option<MiddlewareFuture> {
        None
    }

    /// 可选：中间件优先级，数值越大优先级越高
    fn priority(&self) -> Option<i32> {
        None
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
    manager: Arc<Manager>,
}

pub struct ManagerService<S> {
    inner: S,
    manager: Arc<Manager>,
}

impl Context {
    pub fn new(request: Request) -> Self {
        Self { request, response: None, chains: vec![], aborted: false }
    }
}

impl Pattern {
    pub fn check(&self, path: &str) -> bool {
        match self {
            Pattern::Prefix(prefix) => path.starts_with(prefix),
            Pattern::Suffix(suffix) => path.ends_with(suffix),
            Pattern::Contains(contains) => path.contains(contains),
            Pattern::Regex(regex) => match regex::Regex::new(regex) {
                Ok(re) => re.is_match(path),
                Err(e) => {
                    tracing::error!("Invalid regex pattern: {}", e);
                    false
                },
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
        self.middlewares.sort_by(|a, b| {
            let a_priority = a.priority().unwrap_or(0);
            let b_priority = b.priority().unwrap_or(0);
            b_priority.cmp(&a_priority)
        });
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

        if let Some(methods) = middleware.methods() {
            let request_method = parts.method.clone();
            for method in methods {
                match method {
                    ApplyKind::Include(m) => {
                        if request_method == &m {
                            return true;
                        }
                    },
                    ApplyKind::Exclude(m) => {
                        if request_method != &m {
                            return true;
                        }
                    },
                }
            }
        }

        if let Some(patterns) = middleware.patterns() {
            let path = parts.uri.path();
            for pattern in patterns {
                match pattern {
                    ApplyKind::Include(p) => {
                        if p.check(path) {
                            return true;
                        }
                    },
                    ApplyKind::Exclude(p) => {
                        if !p.check(path) {
                            return true;
                        }
                    },
                }
            }
        }

        false
    }

    pub fn integrated(_manager: Arc<Manager>, router: axum::Router) -> axum::Router {
        // 创建一个新的中间件层

        router
    }
}

impl<S> Layer<S> for ManagerLayer
where
    S: Service<Request, Response = Response> + Send + 'static,
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
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<erx::Erx>,
{
    type Response = Response;
    type Error = erx::Erx;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let manager = Arc::clone(&self.manager);
        let (parts, body) = req.into_parts();
        let applicable_middlewares = manager.applys(&parts);
        let mut context = Context::new(Request::from_parts(parts, body));

        Box::pin(async move {
            for m in applicable_middlewares {}

            for m in applicable_middlewares {}
        })
    }
}
