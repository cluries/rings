use crate::erx::Erx;
use std::sync::{Arc, RwLock};
use tokio::sync::OnceCell;
use tokio_cron_scheduler::Job;

static SHARED_MANAGER: OnceCell<ServiceManager> = OnceCell::const_new();

pub trait ServiceTrait: crate::any::AnyTrait + Send + Sync {
    fn name(&self) -> &str;
    fn initialize(&mut self);
    fn release(&mut self);
    fn ready(&self) -> bool;
    fn schedules(&self) -> Vec<Job>;
}

// fn is_service_scheduled<T: ServiceSchedulerTrait>() -> bool {
//     true
// }
//
// pub trait ServiceSchedulerTrait {
//     fn jobs(&self) -> Vec<Job>;
// }

// type d = fn <T, C>( target: C) -> Box<dyn FnMut(i32) -> Result<i32, Erx>>
// where
//     T: ServiceTrait + Default,
//     C: FnOnce() -> Result<Arc<RwLock<Box<dyn ServiceTrait>>>, Erx> ;

// type Managed = Arc<RwLock<Box<dyn ServiceTrait>>>;

// trait Managed {}
//
// type Invoker<S: Default + Sync + Clone, T: Managed, E: serde::Serialize> = fn(
//     Box<dyn FnMut(Box<S>, &T) -> Result<Arc<RwLock<Box<dyn ServiceTrait>>>, E>>,
// ) -> Arc<Vec<T>>;

pub struct ServiceManager {
    managed: RwLock<Vec<Arc<RwLock<Box<dyn ServiceTrait>>>>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        ServiceManager { managed: Default::default() }
    }

    fn managed_by_name(&self, name: &str) -> Option<Arc<RwLock<Box<dyn ServiceTrait>>>> {
        self.managed
            .read()
            .ok()?
            .iter()
            .find(|managed| match managed.try_read() {
                Err(_) => false,
                Ok(read) => read.name().eq(name),
            })
            .cloned()
    }

    pub fn managed_services(&self) -> Vec<Arc<RwLock<Box<dyn ServiceTrait>>>> {
        self.managed.read().unwrap().clone()
    }

    pub fn register<T>(&self) -> Result<Arc<RwLock<Box<dyn ServiceTrait>>>, Erx>
    where
        T: ServiceTrait + Default,
    {
        let mut ctx = T::default();
        let name = ctx.name().to_owned();

        if self.managed_by_name(&name).is_some() {
            return Err(Erx::new(format!("Service '{}' already registered!", name).as_str()));
        }

        match self.managed.try_write() {
            Ok(mut write_guard) => {
                ctx.initialize();
                let warp = Arc::new(RwLock::new(Box::new(ctx) as Box<dyn ServiceTrait>));
                write_guard.push(Arc::clone(&warp));
                Ok(warp)
            },
            Err(er) => Err(Erx::new(er.to_string().as_str())),
        }
    }

    pub fn unregister<T: ServiceTrait + Default>(&self) -> Result<(), Erx> {
        let name = T::default().name().to_owned();

        self.get::<T>()
            .ok_or(Erx::new(format!("Service '{}' was not registered!", name).as_str()))?
            .try_write()
            .map_err(crate::erx::smp)?
            .release();

        self.managed.try_write().map_err(crate::erx::smp)?.retain(|m| match m.try_read() {
            Err(ex) => {
                tracing::error!("{}", ex);
                true
            },
            Ok(srv) => !srv.name().eq(name.as_str()),
        });

        Ok(())
    }

    pub fn get<T: ServiceTrait + Default>(&self) -> Option<Arc<RwLock<Box<dyn ServiceTrait>>>> {
        self.managed_by_name(T::default().name())
    }

    pub async fn shared() -> &'static ServiceManager {
        SHARED_MANAGER.get_or_init(|| async { ServiceManager::new() }).await
    }
}
