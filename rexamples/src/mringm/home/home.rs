use std::any::Any;
use rings::any::AnyTrait;
use rings::rex::tokio_cron_scheduler;
use rings::service::ServiceTrait;
#[ringm::service(mringm, home, home)]
pub struct Home {
    pub username: String,
    pub email: String,
}

impl AnyTrait for Home {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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
