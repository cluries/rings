// pub struct Middleware {
//     pub focus: fn(parts: &axum::http::request::Parts) -> bool,
//     pub work: fn(request: axum::extract::Request, next: axum::middleware::Next) -> Option<axum::response::Response>,
// }

use axum::response::IntoResponse;
use axum::Router;
use futures_util::future::BoxFuture;
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service, ServiceBuilder};

mod jwt;
mod signature;

pub trait Middleware {
    fn focus(&self, parts: &axum::http::request::Parts) -> bool;

    fn exec(&self) -> dyn FnMut(axum::extract::Request) -> Result<axum::extract::Request, axum::response::Response>;
}

#[derive(Clone)]
pub struct Middle<S>
where
    S: Clone + Send,
{
    inner: S,
    ware: Arc<dyn Middleware>,
}

#[derive(Clone)]
pub struct MiddleLayer {
    ware: Arc<dyn Middleware>,
}

impl<S> Layer<S> for MiddleLayer
where
    S: Clone + Send,
{
    type Service = Middle<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Middle { inner, ware: Arc::clone(&self.ware) }
    }
}

impl MiddleLayer {
    pub fn integrated(&self, router: Router) -> Router {
        router.layer(ServiceBuilder::new().layer(self))
    }
}

impl<S> Service<axum::extract::Request> for Middle<S>
where
    S: Service<axum::extract::Request, Response = axum::response::Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: axum::extract::Request) -> Self::Future {
        let ware = Arc::clone(&self.ware);
        let (parts, body) = request.into_parts();

        if !ware.focus(&parts) {
            let request = axum::extract::Request::from_parts(parts, body);
            let future = self.inner.call(request);
            return Box::pin(async move { Ok(future.await?) });
        }

        let mut inner = self.inner.clone();

        Box::pin(async move {
            let request = axum::extract::Request::from_parts(parts, body);
            let f = ware.exec();
            match f(request) {
                Ok(request) => {
                    let response: axum::response::Response = inner.call(request).await?;
                    Ok(response)
                },
                Err(response) => Ok(response),
            }
        })
    }
}
