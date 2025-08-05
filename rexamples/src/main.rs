use rings::{
    app::{AppBuilder, AppBuilderWebReconfigor},
    axum, rings::R, tokio,
};
use web::action::api::api_actions;


#[tokio::main]
async fn main() {

    let mut builder = AppBuilder::new("rexamples").await;
    builder.use_model().await;

    rings::hey_service!(service);

    builder.use_scheduler().await;

    let rc: Vec<AppBuilderWebReconfigor> = vec![
        ("api".to_string(), api_actions, |x| {
            fn extra(router: axum::Router) -> axum::Router {
                // use web::middleware::api::signator::use_signator;
                // use_signator(router)

                router
            }
            x.set_router_reconfiger(extra)
        }),

        #[cfg(feature = "frontend")]
        {
            use rings::app::web_reconfig_simple;
            use web::action::front::front_actions;
            web_reconfig_simple("front", front_actions)
        },
        // ("front".to_string(), front_actions, |web: &mut Web| -> &mut Web { web })
    ];

    builder.use_web(rc).await;

    let rings_app = builder.build();

    R::perform(&rings_app).await;
}
 