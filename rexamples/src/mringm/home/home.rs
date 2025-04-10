use rings::any::AnyTrait;
use rings::rex::tokio_cron_scheduler;
use rings::service::ServiceTrait;
use std::any::Any;

#[ringm::service(mringm, home, home)]
#[ringm::default_any]
pub struct Home {
    pub username: String,
    pub email: String,
}


impl ServiceTrait for Home {
    fn name(&self) -> &str {
        "Home"
    }

    fn initialize(&mut self) {

    }

    fn release(&mut self) {

    }

    fn ready(&self) -> bool {
        true
    }

    fn schedules(&self) -> Vec<tokio_cron_scheduler::Job> {
        vec![]
    }
}
