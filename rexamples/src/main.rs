use rings::prelude::{tokio, tokio_cron_scheduler};
use rings::service::ServiceTrait;

#[allow(dead_code, unused)]
mod mringm;

#[ringm::service]
#[ringm::default_any]
struct ArgsService {}

#[ringm::service]
#[ringm::default_any]
struct LanuchService {}

#[tokio::main]
async fn main() {
    mringm::its_service().await;
}

impl ServiceTrait for ArgsService {
    fn name(&self) -> &'static str {
        ArgsService::service_name()
    }

    fn initialize(&mut self) {}

    fn release(&mut self) {}

    fn ready(&self) -> bool {
        true
    }

    fn schedules(&self) -> Vec<tokio_cron_scheduler::Job> {
        vec![]
    }
}

impl ServiceTrait for LanuchService {
    fn name(&self) -> &'static str {
        LanuchService::service_name()
    }

    fn initialize(&mut self) {}

    fn release(&mut self) {}

    fn ready(&self) -> bool {
        true
    }

    fn schedules(&self) -> Vec<tokio_cron_scheduler::Job> {
        vec![]
    }
}
