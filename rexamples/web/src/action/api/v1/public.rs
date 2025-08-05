mod cnregion;

use crate::action::tool::api_v1;
use rings::axum::routing::get;
use rings::axum::Router;

#[inline]
fn cnregion(suffix: &str) -> String {
    let mut url = api_v1("public/cnregion/");
    url.push_str(suffix);
    url
}

pub fn public_action() -> Router {
    use cnregion::action::*;
    Router::new().route(
        cnregion("childrens").as_str(), 
        get(childrens)
    ).route(
        cnregion("point").as_str(),
        get(point)
    )
}
