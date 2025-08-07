use rings::{
    app::{AppBuilder, AppBuilderWebReconfigor},
    axum, rings::R, tokio
};
use web::action::api::api_actions;


#[tokio::main]
async fn main() {

    let mut builder = AppBuilder::new("rexamples").await;
    builder.use_model().await;

    rings::hey_service!(service);

    builder.use_scheduler().await;

    let mut rc: Vec<AppBuilderWebReconfigor> = vec![
        AppBuilderWebReconfigor{
            name: String::from("API"), 
            router_maker: api_actions, 
            reconfigor: |x| {
                fn extra(router: axum::Router) -> axum::Router {
                    router
                }
                x.set_router_reconfiger(extra)
            },
            middlewares: vec![
                
            ]
        }
    ];

    builder.use_web(&mut rc).await;

    let rings_app = builder.build();

    R::perform(&rings_app).await;
}


