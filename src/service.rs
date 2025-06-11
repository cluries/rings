use crate::erx::Erx;
use std::sync::{Arc, RwLock};
use tokio::sync::OnceCell;
use tokio_cron_scheduler::Job;

static SHARED_MANAGER: OnceCell<ServiceManager> = OnceCell::const_new();

static SHARED_SERVICE_NAME: &str = "SharedServiceManager";

/// shared service manager init
pub(crate) async fn shared_service_manager() -> &'static ServiceManager {
    SHARED_MANAGER
        .get_or_init(|| async {
            tracing::info!("Initializing shared service manager");
            ServiceManager::new(SHARED_SERVICE_NAME)
        })
        .await
}

/// registe to shared service manager
/// # Arguments
/// * `name` - service name
/// * `service` - service
pub async fn registe_to_shared<T: ServiceTrait + Default>() {
    let shared_service_manager = shared_service_manager().await;
    shared_service_manager.register::<T>().expect("registration failed");
}

/// Service Trait
/// # Methods
/// * `name` - get service name
/// * `initialize` - initialize service
/// * `release` - release service
/// * `ready` - check service is ready
/// * `schedules` - get service schedules
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

/// Service Manager
/// # Fields
/// * `name` - service manager name
/// * `managed` - managed services
pub struct ServiceManager {
    name: String,
    managed: RwLock<Vec<Arc<RwLock<Box<dyn ServiceTrait>>>>>,
}

/// Managed Service
/// Arc<RwLock<Box<dyn ServiceTrait>>>
pub type Managed = Arc<RwLock<Box<dyn ServiceTrait>>>;

/// Service Manager
/// # Fields
/// * `name` - service manager name
/// * `managed` - managed services
/// # Methods
/// * `new` - make new service manager
/// * `name` - get service manager name
/// * `managed_by_name` - get managed service by name
/// * `managed_services` - get managed services
/// * `register` - register service
/// * `unregister` - unregister service
/// * `get` - get managed service
/// * `shared` - get shared service manager
impl ServiceManager {
    /// make new service manager
    /// # Arguments
    /// * `name` - service manager name
    /// # Returns
    /// * `ServiceManager` - service manager
    pub fn new(name: &str) -> Self {
        ServiceManager { name: name.to_string(), managed: RwLock::new(Vec::new()) }
    }

    /// get service manager name
    /// # Returns
    /// * `&str` - service manager name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// get managed service by name
    /// # Arguments
    /// * `name` - service name
    /// # Returns
    /// * `Option<Arc<RwLock<Box<dyn ServiceTrait>>>>` - managed service
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

    /// get managed services
    /// # Returns
    /// * `Vec<Arc<RwLock<Box<dyn ServiceTrait>>>>` - managed services
    /// # Panics
    /// * `std::sync::PoisonError` - if managed services is poisoned
    pub fn managed_services(&self) -> Vec<Arc<RwLock<Box<dyn ServiceTrait>>>> {
        self.managed.read().unwrap().clone()
    }

    /// register service
    /// # Arguments
    /// * `name` - service name
    /// * `service` - service
    /// # Returns
    /// * `Result<Arc<RwLock<Box<dyn ServiceTrait>>>, Erx>` - managed service
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

    /// unregister service
    /// # Arguments
    /// * `name` - service name
    /// # Returns
    /// * `Result<(), Erx>` - unregister result
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

    /// get managed service
    /// # Arguments
    /// * `name` - service name
    /// # Returns
    /// * `Option<Arc<RwLock<Box<dyn ServiceTrait>>>>` - managed service
    pub fn get<T: ServiceTrait + Default>(&self) -> Option<Arc<RwLock<Box<dyn ServiceTrait>>>> {
        self.managed_by_name(T::default().name())
    }

    ///  let r = m.using::<TestService, _, _>(|srv| {
    ///      let r = srv.rnd();
    ///      async move { r }
    ///  }).await;
    pub async fn using<T, F, Fut>(&self, invoke: F) -> Result<Fut::Output, Erx>
    where
        T: ServiceTrait + Default,
        F: Fn(&T) -> Fut,
        Fut: std::future::Future,
    {
        let name = T::default().name().to_owned();
        let managed = self.managed_by_name(&name).ok_or(Erx::new(&format!("Service '{}' was not registered!", &name)))?;
        let read_guard = managed.try_read().map_err(crate::erx::smp)?;

        let service = (&*read_guard).as_any().downcast_ref::<T>().ok_or(Erx::new(format!("Service '{}' cast error", &name).as_str()))?;
        let output = invoke(service).await;
        Ok(output)
    }

    ///  let r = m.using_mut::<TestService, _, _>(|srv| {
    ///      let r = srv.rnd();
    ///      async move { r }
    ///  }).await;
    pub async fn using_mut<T, F, Fut>(&self, invoke: F) -> Result<Fut::Output, Erx>
    where
        T: ServiceTrait + Default,
        F: Fn(&mut T) -> Fut,
        Fut: std::future::Future,
    {
        let name = T::default().name().to_owned();
        let managed = self.managed_by_name(&name).ok_or(Erx::new(&format!("Service '{}' was not registered!", &name)))?;
        let mut write_guard = managed.try_write().map_err(crate::erx::smp)?;

        let service = (&mut *write_guard)
            .as_any_mut()
            .downcast_mut::<T>()
            .ok_or(Erx::new(format!("Service '{}' cast error", &name).as_str()))?;
        let output = invoke(service).await;
        Ok(output)
    }

    /// get shared service manager
    /// # Returns
    /// * `&'static ServiceManager` - shared service manager
    pub async fn shared() -> &'static ServiceManager {
        shared_service_manager().await
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use crate::any::AnyTrait;
    use crate::tools::rand::rand_i64;
    use std::any::Any;

    #[tokio::test]
    async fn test_service_manager() {
        let m = shared_service_manager().await;
        m.register::<TestService>();

        // let arc = m.get::<TestService>().unwrap();
        // let mut guard = arc.write().unwrap();
        // assert_eq!((*guard).name(), "testservice");
        //
        // let t = (&mut *guard).as_any_mut().downcast_mut::<TestService>().unwrap();
        //
        // println!("{}", t.iam());
        //
        // println!("{}", t.iam_mut());

        let r = m
            .using::<TestService, _, _>(|srv| {
                let r = srv.rnd();
                async move { r }
            })
            .await;
        println!("==={:#?}", r);
    }

    struct TestService {}

    impl Default for TestService {
        fn default() -> Self {
            TestService {}
        }
    }

    impl AnyTrait for TestService {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    impl ServiceTrait for TestService {
        fn name(&self) -> &str {
            "testservice"
        }

        fn initialize(&mut self) {
            println!("Service '{}' initialized!", self.name());
        }

        fn release(&mut self) {}

        fn ready(&self) -> bool {
            true
        }

        fn schedules(&self) -> Vec<Job> {
            Vec::new()
        }
    }

    impl TestService {
        fn iam(&self) -> String {
            String::from("iam")
        }

        fn iam_mut(&mut self) -> String {
            String::from("iam mutable")
        }

        fn rnd(&self) -> i64 {
            rand_i64(1, 100)
        }
    }
}
