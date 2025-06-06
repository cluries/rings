pub mod signature;
pub mod jwt;


// use axum::http::request::Parts;
// use axum::Router;

// pub trait Middleware {
//     type Arguments: Send + Sync + Clone;

//     fn make(args: Self::Arguments) -> Self;

//     fn focus(&self, parts: &Parts) -> bool;

//     fn priority(&self) -> i32;

//     fn call(&self) -> Box<dyn FnMut(axum::extract::Request) -> Result<axum::extract::Request, axum::response::Response>>;
// }

// pub struct LaunchPad<M: Middleware> {
//     middleware: M,
// }

// impl<M: Middleware> LaunchPad<M> {
//     pub fn new(middleware: M) -> Self {
//         Self { middleware }
//     }

//     pub fn using(&self, router: Router) -> Router {
//         router
//     }
// }

// #[cfg(test)]
// #[allow(unused)]
// mod tests {
//     struct TMiddle {}

//     #[test]
//     fn test_middleware() {}
// }
