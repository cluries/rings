use rings::{tokio_cron_scheduler, tracing};

#[ringm::service(public, cnregion)]
#[ringm::default_any]
pub struct CNRRegion {}

impl rings::service::ServiceTrait for CNRRegion {
    fn name(&self) -> &'static str {
        Self::service_name()
    }

    fn initialize(&mut self) {
        tracing::info!("CNRRegion Service initialized");
    }

    fn release(&mut self) {
        tracing::info!("CNRRegion Service released");
    }

    fn ready(&self) -> bool {
        true
    }

    fn schedules(&self) -> Vec<tokio_cron_scheduler::Job> {
        vec![]
    }
}

impl CNRRegion {
    pub fn rnd(&mut self, max: i32) -> i32 {
        max - 1
    }
}
