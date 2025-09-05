use crate::erx;
use crate::rings::RingState;
use crate::service::ServiceManager;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as ToKioRwLock;
use tokio_cron_scheduler::JobScheduler;
use tracing::{error, info, warn};
use uuid::Uuid;

/// scheduler manager
///
pub struct SchedulerManager {
    stage: Arc<RwLock<RingState>>,
    count: u64,
    scheduler: Arc<ToKioRwLock<JobScheduler>>,
}

impl SchedulerManager {
    /// create a new scheduler manager
    pub(crate) async fn new() -> Self {
        let mut scheduler = JobScheduler::new().await.unwrap();
        scheduler.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("scheduler.set_shutdown_handler called.");
            })
        }));

        Self { stage: Arc::new(RwLock::new(RingState::Init)), count: 0, scheduler: Arc::new(ToKioRwLock::new(scheduler)) }
    }

    pub async fn add_job(&mut self, job: tokio_cron_scheduler::Job) -> erx::ResultE<String> {
        let scheduler = Arc::clone(&self.scheduler);
        let guard = scheduler.try_write().map_err(erx::smp)?;
        guard.add(job).await.map(Into::into).map_err(erx::smp)
    }

    pub async fn remove_job(&mut self, job_id: String) -> erx::ResultE<()> {
        let scheduler = Arc::clone(&self.scheduler);
        let guard = scheduler.try_write().map_err(erx::smp)?;
        let uuid = Uuid::from_str(&job_id).map_err(erx::smp)?;
        guard.remove(&uuid).await.map_err(erx::smp)
    }
}

pub const SCHEDULER_MANAGER_NAME: &str = "SchedulerManager";

// macro_rules! epanic {
//     ($ex:expr) => {
//         error!($ex);
//         panic!($ex);
//     };
// }

impl SchedulerManager {
    /// debug info
    pub fn debug(&self) {
        info!("scheduler manager {} ", SCHEDULER_MANAGER_NAME);
    }

    /// debug mut info
    pub fn debug_mut(&mut self) {
        self.count += 1;
        info!("scheduler manager {} mut counter:{}", SCHEDULER_MANAGER_NAME, self.count);
    }
}

#[async_trait]
impl crate::rings::RingsMod for SchedulerManager {
    fn name(&self) -> String {
        SCHEDULER_MANAGER_NAME.to_string()
    }

    fn duplicate_able(&self) -> bool {
        false
    }

    async fn initialize(&mut self) -> Result<(), crate::erx::Erx> {
        let mut futures = vec![]; // Vec<Box<dyn Future<Output = ()> + Send>>;
        let srv_manager = ServiceManager::shared().await;

        let managed: Vec<crate::service::Managed> = srv_manager.managed_services();
        let scheduler = self.scheduler.clone(); // Arc<ToKioRwLock<JobScheduler>>;

        for service in managed {
            match service.try_read() {
                Ok(service) => {
                    for job in service.schedules() {
                        let service_name = service.name().to_string();
                        let job_id = job.guid().to_string();

                        let scheduler = Arc::clone(&scheduler);
                        futures.push(async move {
                            let scheduler = Arc::clone(&scheduler);
                            let scher = scheduler.write().await;
                            match scher.add(job).await {
                                Ok(_) => {
                                    info!("Add schedule job[{}] from service[{}] SUCCESS", job_id, service_name);
                                },
                                Err(e) => {
                                    error!("Add schedule job[{}] from service[{}] FAILED, Error:{}.", job_id, service_name, e.to_string());
                                },
                            }
                        });
                    }
                },
                Err(ex) => {
                    error!("scheduler service lock poisoned: {}", ex);
                },
            }
        }

        let scheduled_count = futures.len();
        futures_util::future::join_all(futures).await;

        info!("scheduler manager {} load service scheduled count:{}", SCHEDULER_MANAGER_NAME, scheduled_count);

        Ok(())
    }

    async fn unregister(&mut self) -> Result<(), crate::erx::Erx> {
        self.shutdown().await
    }

    async fn shutdown(&mut self) -> Result<(), crate::erx::Erx> {
        info!("scheduler manager [{}] shutdown", SCHEDULER_MANAGER_NAME);
        let current = self.stage.try_read().map_err(crate::erx::smp)?.clone();
        if !current.is_ready_to_terminating() {
            return Err(crate::erx::Erx::new(
                format!("Ring:{} current state:{} can not terminate", self.name(), <RingState as Into<&str>>::into(current)).as_str(),
            ));
        }

        let scheduler = Arc::clone(&self.scheduler);
        if let Err(ex) = scheduler.write().await.shutdown().await {
            error!("scheduler service lock poisoned: {}", ex);
        }

        *self.stage.try_write().map_err(crate::erx::smp)? = RingState::Terminating;

        Ok(())
    }

    async fn fire(&mut self) -> Result<(), crate::erx::Erx> {
        *self.stage.write().unwrap() = RingState::Working;

        let stage = self.stage.clone();
        let dog = async move {
            info!("scheduler manager [{}] fire, dog watch it....", SCHEDULER_MANAGER_NAME);
            let duration = tokio::time::Duration::from_millis(100); // 100ms
            let mut stage_read_lock_failures: i64 = 0;
            loop {
                match stage.try_read() {
                    Ok(stage) => {
                        let stage = *stage;
                        if stage == RingState::Terminating || stage == RingState::Terminated {
                            break;
                        }
                    },
                    Err(ex) => {
                        warn!("scheduler stage lock poisoned: {}", ex);
                        stage_read_lock_failures += 1;
                    },
                }
                tokio::time::sleep(duration).await;
            }

            *stage.write().unwrap() = RingState::Terminated;

            stage_read_lock_failures
        };

        let scheduler = self.scheduler.clone();
        let run = async move {
            let scheduler = Arc::clone(&scheduler);
            let scher = scheduler.write().await;
            match scher.start().await {
                Ok(_) => {
                    info!("scheduler start success");
                },
                Err(err) => {
                    error!("scheduler failed to start: {}", err);
                },
            }
        };

        tokio::spawn(async {
            run.await;
            dog.await;
        });

        Ok(())
    }

    fn stage(&self) -> RingState {
        self.stage.read().unwrap().clone()
    }

    fn level(&self) -> i64 {
        i64::MAX
    }
}

// impl crate::any::AnyTrait for SchedulerManager {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }
// }

crate::impl_any_trait!(SchedulerManager);
