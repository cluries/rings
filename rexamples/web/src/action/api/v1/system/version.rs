use rings::axum::extract::Extension;
use rings::serde;
use rings::web::api::Out;
use rings::web::context::Context;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
#[serde(crate = "rings::serde")]
pub struct Versions {
    development: Option<String>,
    stable: String,
}

pub async fn version_action(Extension(ctx): Extension<Context>) -> Out<Versions> {
    let mut vers = Versions { stable: "1.0.0".to_string(), development: None };
    if ctx.get_ident().as_str().starts_with("Development") {
        vers.development = Some("1.1.0".to_string());
    }

    Out::ok(vers)
}
