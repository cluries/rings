use crate::rings::RingState;
use crate::service::{ServiceManager, ServiceTrait};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use tokio_cron_scheduler::JobScheduler;
use tracing::{error, info, warn};

/// SchedulerManager
pub struct SchedulerManager {
    stage: Arc<RwLock<RingState>>,
    count: u64,
    scheduler: Option<JobScheduler>,
}

impl SchedulerManager {
    pub(crate) fn new() -> Self {
        Self { stage: Arc::new(RwLock::new(RingState::Init)), count: 0, scheduler: None }
    }
}

pub const SCHEDULER_MANAGER_NAME: &str = "SchedulerManager";

impl SchedulerManager {
    pub fn debug(&self) {
        info!("scheduler manager {} ", SCHEDULER_MANAGER_NAME);
    }

    pub fn debug_mut(&mut self) {
        self.count += 1;
        info!("scheduler manager {} mut counter:{}", SCHEDULER_MANAGER_NAME, self.count);
    }
}

impl SchedulerManager {
    async fn load_service_scheduled(&self) {
        let srv_manager = ServiceManager::shared().await;
        let managed: Vec<Arc<RwLock<Box<dyn ServiceTrait>>>> = srv_manager.managed_services();

        let mut scheduled_count = 0;
        if let Some(scheduler) = self.scheduler.as_ref() {
            let mut futures = vec![];
            for service in managed {
                match service.try_read() {
                    Ok(service) => {
                        for job in service.schedules() {
                            futures.push(scheduler.add(job));
                        }
                    },
                    Err(ex) => {
                        error!("scheduler service lock poisoned: {}", ex);
                    },
                }
            }

            scheduled_count = futures.len();
            futures_util::future::join_all(futures).await;
        }

        info!("scheduler manager {} load service scheduled count:{}", SCHEDULER_MANAGER_NAME, scheduled_count);
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
        let mut scheduler = JobScheduler::new().await.unwrap();
        scheduler.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("scheduler shutdown");
            })
        }));

        self.scheduler = Some(scheduler);

        self.load_service_scheduled().await;

        // let stage = self.stage.clone();
        // let future = self.register_service_scheduled();
        // tokio::spawn(async move {
        //     future.await;
        //     *stage.write().unwrap() = RingState::Ready;
        // });

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

        if let Some(scheduler) = self.scheduler.as_mut() {
            if let Err(ex) = scheduler.shutdown().await {
                error!("scheduler service lock poisoned: {}", ex);
            }
        }

        *self.stage.try_write().map_err(crate::erx::smp)? = RingState::Terminating;

        Ok(())
    }

    async fn fire(&mut self) -> Result<(), crate::erx::Erx> {
        *self.stage.write().unwrap() = RingState::Working;

        let stage = self.stage.clone();
        let watch_dog = async move {
            let duration = tokio::time::Duration::from_millis(100);
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

        let run = async {};

        tokio::spawn(async {
            run.await;
            watch_dog.await;
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

/*

impl crate::rings::RingsModAsync for SchedulerManager {
    fn initialize_async(&mut self) -> impl Future<Output=()> + Send {
        async {
            let mut scheduler = JobScheduler::new().await.unwrap();
            scheduler.set_shutdown_handler(Box::new(|| {
                Box::pin(async move {
                    tracing::info!("scheduler shutdown");
                })
            }));


            self.scheduler = Some(scheduler);

            *self.stage.write().unwrap() = RingState::Ready;
        }
    }

    fn unregister_async(&mut self) -> impl Future<Output=()> + Send {
        async {}
    }

    fn shutdown_async(&mut self) -> impl Future<Output=()> + Send {
        async {}
    }

    fn fire_async(&mut self) -> impl Future<Output=()> + Send {
        async {}
    }
}
*/

impl crate::any::AnyTrait for SchedulerManager {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
