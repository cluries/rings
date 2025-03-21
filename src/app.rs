use crate::{
    rings::{RingsApplication, R},
    s,
    web::make_web,
};


pub struct AppBuilder {
    rings_app: RingsApplication,
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
    pub async fn use_web(&mut self, reconfigor: std::collections::HashMap<
        String, (fn() -> Vec<axum::Router>, fn(web: &mut crate::web::Web) -> &mut crate::web::Web,)>) -> &mut Self {
        let rebit = crate::conf::rebit()
            .read()
            .expect("Failed to read config rebit");

        for wb in rebit.webs.iter() {
            if !reconfigor.contains_key(&wb.name) {
                tracing::warn!("Reconfigor not found web iterm: {}", wb.name);
                continue;
            }

            let (router_maker, reconf) = reconfigor.get(&wb.name).unwrap();

            let mut web = make_web(&wb.name, wb.bind_addr().as_str(), *router_maker);
            reconf(&mut web);

            match self.rings_app.try_write() {
                Ok(mut ring) => {
                    ring.register_mod(web).await;
                }
                Err(err) => {
                    tracing::error!("init_web rings write guard:{}", err);
                    panic!("{:?}", err);
                }
            };
        }

        self
    }

    pub async fn use_scheduler(&mut self) -> &mut Self {
        // TODO
        panic!("not yet implemented");
        // self
    }

    pub async fn build(self) -> RingsApplication {
        self.rings_app
    }
}
