

// 实现对axum middleware的抽象
pub mod signature; 


use axum::http::request::Parts;
use axum::{extract::Request, response::Response};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use axum::http::Method;
use crate::erx;

pub enum Pattern  {
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
    pub metrics: Metrics,
    pub chains: Vec<Chain>,
    pub aborted: bool,
}


pub type MiddlewareFuture = Pin<Box<dyn Future<Output = Result<Context, erx::Erx>> + Send>>;

pub trait Middleware: Send + Sync {

    fn name(&self) -> &'static str;

    fn on_request(&self, _context: Context) -> Option<MiddlewareFuture> {
        None
    }

    fn on_response(&self, _context: Context) -> Option<MiddlewareFuture> {
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

impl Manager {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add(&mut self, middleware: Box<dyn Middleware>) -> &mut Self {
        for m in &self.middlewares {
            if m.name() == middleware.name() {
                panic!("Middleware with name '{}' already exists", middleware.name());
            }
        }
        self.middlewares.push(middleware);
        self
    }

    pub fn sort(&mut self) {
        self.middlewares.sort_by(|a, b| {
            let a_priority = a.priority().unwrap_or(0);
            let b_priority = b.priority().unwrap_or(0);
            b_priority.cmp(&a_priority)
        });
    }

    pub fn integrated(&self, router:axum::Router) -> axum::Router {
        router
    }

    pub fn applys(&self, parts: &Parts) -> Vec<&Box<dyn Middleware>> {
        let mut middlewares = Vec::new();
        for middleware in &self.middlewares {
            if let Some(apply) = middleware.apply(parts) {
                if apply {
                    middlewares.push(middleware);
                }
            } else {
                if let Some(methods) = middleware.methods() {
                    for method in methods {
                        match method {
                            ApplyKind::Include(method) => {
                                if parts.method == method {
                                    middlewares.push(middleware);
                                }
                            }
                            ApplyKind::Exclude(method) => {
                                if parts.method != method {
                                    middlewares.push(middleware);
                                }
                            }
                        }
                    }
                }

                if let Some(patterns) = middleware.patterns() {
                    for pattern in patterns {
                        match pattern {
                            ApplyKind::Include(pattern) => {
                                match pattern {
                                    Pattern::Prefix(prefix) => {
                                        if parts.uri.path().starts_with(&prefix) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                    Pattern::Suffix(suffix) => {
                                        if parts.uri.path().ends_with(&suffix) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                    Pattern::Contains(contains) => {
                                        if parts.uri.path().contains(&contains) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                    Pattern::Regex(regex) => {
                                        if regex::Regex::new(&regex).unwrap().is_match(parts.uri.path()) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                }
                            }
                            ApplyKind::Exclude(pattern) => {
                                match pattern {
                                    Pattern::Prefix(prefix) => {
                                        if !parts.uri.path().starts_with(&prefix) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                    Pattern::Suffix(suffix) => {
                                        if !parts.uri.path().ends_with(&suffix) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                    Pattern::Contains(contains) => {
                                        if !parts.uri.path().contains(&contains) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                    Pattern::Regex(regex) => {
                                        if !regex::Regex::new(&regex).unwrap().is_match(parts.uri.path()) {
                                            middlewares.push(middleware);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }  
            }
        }
        
        middlewares
    }

    
}

 