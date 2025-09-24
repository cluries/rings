// Don't change this value, it will be replaced by the commit build time
pub static COMMIT_BUILD: &'static str = "20250308100426";

// Don't change this value, it will be replaced by the version
pub static VERSION: &'static str = "0.1.0 - Dev";

pub mod any;
pub mod app;
pub mod conf;
pub mod erx;
pub mod fns;
pub mod log;
pub mod macros;
pub mod migrate;
pub mod model;
pub mod prelude;
pub mod rings;
pub mod scheduler;
pub mod service;
pub mod tools;

pub mod web;
pub use prelude::*;

#[macro_export]
macro_rules! impl_any_trait {
    ($t:ty) => {
        impl $crate::any::AnyTrait for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}
