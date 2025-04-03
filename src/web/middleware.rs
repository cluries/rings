pub mod signature;

use axum::http::request::Parts;
use axum::Router;

#[derive(Clone)]
pub enum CallM<R, S>
where
    R: Clone + Send + Sync,
    S: Clone + Send + Sync,
{
    Request(R),
    Response(S),
}

pub trait Middleware {
    type Arguments: Send + Sync + Clone;

    fn make(args: Self::Arguments) -> Self;

    fn focus(&self, parts: &Parts) -> bool;

    fn priority(&self) -> i32;

    fn call(&self) -> Box<dyn FnMut(axum::extract::Request) -> CallM<axum::extract::Request, axum::response::Response>>;
}

pub struct R<M: Middleware> {
    middleware: M,
}

impl<R, S> CallM<R, S> {
    pub fn request(value: R) -> Self {
        CallM::Request(value)
    }

    pub fn response(value: S) -> Self {
        CallM::Response(value)
    }

    pub fn is_request(&self) -> bool {
        matches!(self, CallM::Request(..))
    }

    pub fn is_response(&self) -> bool {
        matches!(self, CallM::Response(..))
    }
}

impl<M: Middleware> R<M> {
    pub fn new(middleware: M) -> Self {
        Self { middleware }
    }

    pub fn using(&self, router: Router) -> Router {
        router
    }
}

#[cfg(test)]
mod tests {
    struct TMiddle {}

    #[test]
    fn test_middleware() {}
}
