use ringm;
use rings::{tokio_cron_scheduler, tracing};

#[ringm::service(system, service)]
#[ringm::default_any]
pub struct Service {}


impl rings::service::ServiceTrait for Service {
    fn name(&self) -> &'static str {
        Self::service_name()
    }

    fn initialize(&mut self) {
        tracing::info!("System service initialized");
    }

    fn release(&mut self) {}

    fn ready(&self) -> bool {
        true
    }

    fn schedules(&self) -> Vec<tokio_cron_scheduler::Job> {
        vec![]
    }
}
