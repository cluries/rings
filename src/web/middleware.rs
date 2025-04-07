pub mod signature;

use std::task::{Context, Poll};
use axum::http::request::Parts;
use axum::Router;
use tower::Service;

pub type CallR = Result<axum::extract::Request, axum::response::Response>;

pub trait Middleware {
    type Arguments: Send + Sync + Clone;

    fn make(args: Self::Arguments) -> Self;

    fn focus(&self, parts: &Parts) -> bool;

    fn priority(&self) -> i32;

    fn call(&self) -> Box<dyn FnMut(axum::extract::Request) -> CallR>;
}

pub struct LaunchPad<M: Middleware> {
    middleware: M,
}

impl<M: Middleware> LaunchPad<M> {
    pub fn new(middleware: M) -> Self {
        Self { middleware }
    }

    pub fn using(&self, router: Router) -> Router {
        router
    }
}

impl<M, S, Request> Service<Request> for LaunchPad<S>
where
    S: Service<Request>,
    M: Middleware,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    type MArgs = M::Arguments;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {}

    fn call(&mut self, request: Request) -> Self::Future {}
}


#[cfg(test)]
mod tests {
    struct TMiddle {}

    #[test]
    fn test_middleware() {}
}
