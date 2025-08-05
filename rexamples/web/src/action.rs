use crate::action::api::api_actions;
use crate::action::doc::doc_actions;
use crate::action::front::front_actions;
use rings::axum::Router;
use rings::web_route_merge;

pub mod api;
pub mod doc;
pub mod front;
pub mod tool;

pub fn all_actions() -> Vec<Router> {
    web_route_merge!(front_actions(), api_actions(), doc_actions())
}
