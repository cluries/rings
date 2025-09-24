//https://kaisery.github.io/trpl-zh-cn/ch19-06-macros.html

pub mod api;
pub mod context;
pub mod define;
pub mod except;
pub mod input;
pub mod javascript;
pub mod luaction;
pub mod messages;
pub mod middleware;
pub mod request;
pub mod route;
pub mod session;
pub mod tools;
pub mod types;
pub mod url;
pub mod validation;

use crate::rings::{RingState, RingsMod, SafeRS};
use crate::web::luaction::LuaAction;

use async_trait::async_trait;
use axum::Router;
use std::sync::{Arc, RwLock};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::{error, info};

/// merge web routes
#[macro_export]
macro_rules! web_route_merge {
    ( $( $x:expr ),* ) => {
        {
            let mut routes:Vec<Router> = vec![];

            $(
                routes.extend($x);
            )*

            routes
        }
    };
}

#[derive(Clone)]
pub struct Web {
    name: String,
    bind: String,
    router: Router,
    stage: SafeRS,
    luactions: Arc<RwLock<Vec<LuaAction>>>,
    router_maker: fn() -> Vec<Router>,
    router_reconfiger: Option<fn(router: Router) -> Router>,
    middleware_manager: Arc<crate::web::middleware::Manager>,
}

///
pub fn make_web(
    name: &str, bind: &str, router_maker: fn() -> Vec<Router>, middlewares: Vec<Box<dyn crate::web::middleware::Middleware>>,
) -> Web {
    Web {
        name: name.to_string(),
        bind: bind.to_string(),
        router: Router::default(),
        stage: RingState::srs_init(),
        luactions: Default::default(),
        router_maker,
        middleware_manager: Arc::new(crate::web::middleware::Manager::new(middlewares)),
        router_reconfiger: None,
    }
}

impl crate::conf::Web {
    pub fn bind_addr(&self) -> String {
        let bind = self.bind.clone().unwrap_or("0.0.0.0".to_string());
        format!("{}:{}", bind, self.port)
    }
}

impl Web {
    fn web_spec(&mut self) {
        let mut router = Router::default();
        let maker = self.router_maker;
        for route in maker() {
            router = router.merge(route);
        }

        let luactions = self.luactions.read().expect("luactions lock poisoned");
        if luactions.len() > 0 {
            info!("lua action found. adding lua [{}] actions", luactions.len());
            for luaction in luactions.iter() {
                router = router.merge(luaction.route());
            }
        }

        // if self.luactions.read().unwrap().is_empty() {
        //     //TODO
        // }

        router = router.layer(ValidateRequestHeaderLayer::accept("application/json"));
        if let Some(extra) = self.router_reconfiger {
            router = extra(router);
        }

        self.router = router
    }

    pub fn set_router_maker(&mut self, maker: fn() -> Vec<Router>) -> &mut Self {
        self.router_maker = maker;
        self
    }

    pub fn set_router_reconfiger(&mut self, configer: fn(router: Router) -> Router) -> &mut Self {
        self.router_reconfiger = Some(configer);
        self
    }

    pub fn middleware_manager(&mut self) -> Arc<crate::web::middleware::Manager> {
        Arc::clone(&self.middleware_manager)
    }
}

#[async_trait]
impl RingsMod for Web {
    fn name(&self) -> String {
        format!("WebMod[ {} ]", self.name)
    }

    fn duplicate_able(&self) -> bool {
        false
    }

    async fn initialize(&mut self) -> Result<(), crate::erx::Erx> {
        self.web_spec();

        RingState::srs_set(&self.stage, RingState::Ready)?;

        Ok(())
    }

    async fn unregister(&mut self) -> Result<(), crate::erx::Erx> {
        self.shutdown().await
    }

    async fn shutdown(&mut self) -> Result<(), crate::erx::Erx> {
        let current = RingState::srs_get_must(&self.stage)?;

        if !current.is_ready_to_terminating() {
            return Err(crate::erx::Erx::new(
                format!("Ring:{} current state:{} can not terminate", self.name, <RingState as Into<&str>>::into(current)).as_str(),
            ));
        }

        RingState::srs_set(&self.stage, RingState::Terminating)?;

        Ok(())
    }

    // async fn fire(&mut self) -> Result<(), crate::erx::Erx> {
    //     info!("WebMod[ {} ] fire", &self.name);
    //     let listen = tokio::net::TcpListener::bind(self.bind.as_str()).await.map_err(crate::erx::smp)?;
    //     let graceful = |stage: Arc<RwLock<RingState>>, name: String| async move {
    //         loop {
    //             let stage = *stage.read().unwrap();
    //             if stage == RingState::Terminating || stage == RingState::Terminated {
    //                 info!("WebMod[ {} ] terminating, current state:{:?}", name, stage);
    //                 break;
    //             }
    //             tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    //         }
    //     };
    //
    //     let serve = axum::serve(listen, self.router.clone())
    //         .with_graceful_shutdown(
    //             graceful(self.stage.clone(), self.name.clone())
    //         );
    //
    //     info!("WebMod[ {} ] try served : {}", &self.name,  &self.bind);
    //     serve.await.expect(format!("WebMod[ {} ] failed to served : {}", &self.name, &self.bind).as_str());
    //
    //     *self.stage.write().unwrap() = RingState::Terminated;
    //
    //     Ok(())
    // }

    async fn fire(&mut self) -> Result<(), crate::erx::Erx> {
        let web_listen = |name: String, bind: String, router: Router, stage: Arc<RwLock<RingState>>| async move {
            let listen = tokio::net::TcpListener::bind(bind.as_str()).await;
            if listen.is_err() {
                error!("[{} - webserver] can't bind to : {}  ERROR: {}", &name, bind, listen.unwrap_err());
                return;
            }

            let graceful = |stage: Arc<RwLock<RingState>>, name: String| async move {
                loop {
                    let stage = *stage.read().unwrap();
                    if stage == RingState::Terminating || stage == RingState::Terminated {
                        info!("WebMod[ {} ] terminating, current state:{:?}", name, stage);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            };

            let serve = axum::serve(listen.unwrap(), router).with_graceful_shutdown(graceful(Arc::clone(&stage), name.clone()));

            info!("WebMod[ {} ] try served : {}", &name, bind);
            serve.await.expect(format!("WebMod[ {} ] failed to served : {}", &name, bind).as_str());

            // *stage.write().unwrap() = RingState::Terminated;
            let _ = RingState::srs_set_must(&stage, RingState::Terminated);
        };

        let integrated_router = crate::web::middleware::Manager::integrated(self.middleware_manager.clone(), self.router.clone());

        tokio::spawn(web_listen(self.name.clone(), self.bind.clone(), integrated_router, Arc::clone(&self.stage)));

        Ok(())
    }

    fn stage(&self) -> RingState {
        RingState::srs_get_must(&self.stage).unwrap()
    }

    fn level(&self) -> i64 {
        0
    }
}

// impl crate::any::AnyTrait for Web {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }
// }

crate::impl_any_trait!(Web);
