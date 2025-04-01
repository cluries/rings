use crate::{
    rings::{RingsApplication, R},
    s,
    web::make_web,
};
use std::sync::Arc;

pub struct AppBuilder {
    rings_app: RingsApplication,
}

pub type AppBuilderWebReconfigor = (String, fn() -> Vec<axum::Router>, fn(web: &mut crate::web::Web) -> &mut crate::web::Web);

pub fn web_reconfig_simple(name: &str, router_maker: fn() -> Vec<axum::Router>) -> AppBuilderWebReconfigor {
    (name.to_string(), router_maker, app_builder_web_reconfigor_extra_default)
}

fn app_builder_web_reconfigor_extra_default(web: &mut crate::web::Web) -> &mut crate::web::Web {
    web
}

impl AppBuilder {
    pub async fn new(defaults_name: &str) -> Self {
        let name = crate::conf::GetDefault::string("name", s!(defaults_name));
        let rings_app: RingsApplication = R::make(&name).await;
        AppBuilder { rings_app }
    }

    pub async fn use_model(&mut self) -> &mut Self {
        let rebit = crate::conf::rebit().read().expect("Failed to read config rebit");
        let backends = &rebit.model.backends;
        crate::model::initialize_model_connection(backends).await;
        self
    }

    ///
    pub async fn use_web(&mut self, reconfigor: Vec<AppBuilderWebReconfigor>) -> &mut Self {
        let rebit = crate::conf::rebit().read().expect("Failed to read config rebit");

        if rebit.webs.is_empty() {
            return self;
        }

        let rings_app = Arc::clone(&self.rings_app);
        let mut rings_app = match rings_app.try_write() {
            Ok(w) => w,
            Err(err) => {
                tracing::error!("init_web rings write guard:{}", err);
                panic!("{:?}", err);
            },
        };

        for wb in rebit.webs.iter() {
            let find = reconfigor.iter().find(|r| r.0.eq(&wb.name));
            let (name, router_maker, reconf) = match find {
                None => {
                    tracing::warn!("Reconfigor not found web iterm: {}", wb.name);
                    continue;
                },
                Some(v) => v,
            };

            let mut web = make_web(name, wb.bind_addr().as_str(), *router_maker);
            reconf(&mut web);

            rings_app.register_mod(web).await;
        }

        self
    }

    pub async fn use_scheduler(&mut self) -> &mut Self {
        let scheduler_manager = crate::scheduler::SchedulerManager::new().await;
        let app = Arc::clone(&self.rings_app);
        app.write().unwrap().register_mod(scheduler_manager).await;
        self
    }

    pub fn build(self) -> RingsApplication {
        self.rings_app
    }
}
