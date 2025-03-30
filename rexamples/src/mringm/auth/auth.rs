use std::any::Any;
use rings::any::AnyTrait;
use rings::rex::tokio_cron_scheduler;
use rings::service::ServiceTrait;

#[ringm::service(mringm, auth, auth)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

impl AnyTrait for Auth {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ServiceTrait for Auth {
    fn name(&self) -> &str {
        "auth"
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
