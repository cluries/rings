use crate::action::api::v1::v1_actions;
use rings::axum::Router;

pub mod code;
pub mod v1;

pub fn api_actions() -> Vec<Router> {
    let mut routers = Vec::new();
    routers.extend(v1_actions());

    routers
}
