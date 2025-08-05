use crate::action::tool::api_v1;
use rings::axum::Router;
use rings::axum::routing::post;

mod version;

pub fn system_action() -> Router {
    Router::new().route(api_v1("system/version").as_str(), post(version::version_action))
}
