use crate::{
    rings::{RingsApplication, R},
    s,
    web::make_web,
};
use std::sync::Arc;
use tracing::warn;

/// rings app builder
pub struct AppBuilder {
    rings_app: RingsApplication,
}

// pub type AppBuilderWebReconfigor = (String,
//     fn() -> Vec<axum::Router>,
//     fn(web: &mut crate::web::Web) -> &mut crate::web::Web,
//     Vec<Box<dyn crate::web::middleware::Middleware>>
// );

pub struct AppBuilderWebReconfigor {
    pub name: String,
    pub router_maker: fn() -> Vec<axum::Router>,
    pub reconfigor: fn(web: &mut crate::web::Web) -> &mut crate::web::Web,
    pub middlewares: Vec<Box<dyn crate::web::middleware::Middleware>>,
}

/// web_reconfig_simple
pub fn web_reconfig_simple(name: &str, router_maker: fn() -> Vec<axum::Router>) -> AppBuilderWebReconfigor {
    AppBuilderWebReconfigor {
        name: name.to_string(),
        router_maker,
        reconfigor: app_builder_web_reconfigor_extra_default,
        middlewares: Vec::new(),
    }
}

fn app_builder_web_reconfigor_extra_default(web: &mut crate::web::Web) -> &mut crate::web::Web {
    web
}

impl AppBuilder {
    /// new rings app builder
    ///
    /// # Arguments
    ///
    /// * `defaults_name` - The name of the configuration file.
    ///
    /// # Returns
    ///
    /// * `AppBuilder` - The rings app builder.
    pub async fn new(defaults_name: &str) -> Self {
        let name = crate::conf::GetDefault::string("name", s!(defaults_name)).await;
        let rings_app: RingsApplication = R::make(&name).await;
        AppBuilder { rings_app }
    }

    /// use model
    ///
    pub async fn use_model(&mut self) -> &mut Self {
        let rebit = crate::conf::rebit().read().await;
        let backends = &rebit.model.backends;
        match backends {
            None => {
                warn!("No model backend found, pass init model connection.");
            },
            Some(backends) => {
                let backends = backends.clone();
                crate::model::initialize_model_connection(backends).await;
            },
        }
        self
    }

    /// enable web
    ///
    /// # Arguments
    ///
    /// * `reconfigor` - The web reconfigor.
    ///
    /// # Returns
    ///
    /// * `AppBuilder` - The rings app builder.
    pub async fn use_web(&mut self, reconfigor: &mut Vec<AppBuilderWebReconfigor>) -> &mut Self {
        let rebit = crate::conf::rebit().read().await;

        if !rebit.has_web() {
            warn!("no web configuration found, pass init web.");
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

        let webs = rebit.web.clone();
        for (wb_name, wb) in webs {
            let find = reconfigor.iter().position(|x| x.name.eq(&wb_name)).map(|i| reconfigor.remove(i));
            let figor = match find {
                None => {
                    warn!("Reconfigor not found web iterm: {}", &wb_name);
                    continue;
                },
                Some(v) => v,
            };

            let mut web = make_web(&figor.name, wb.bind_addr().as_str(), figor.router_maker, figor.middlewares);
            (figor.reconfigor)(&mut web);

            rings_app.register_mod(web).await;
        }

        self
    }

    /// enable scheduler
    ///
    /// # Returns
    ///
    /// * `AppBuilder` - The rings app builder.
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
