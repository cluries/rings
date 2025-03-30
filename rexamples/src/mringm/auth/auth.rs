use rings::rex::tokio_cron_scheduler;
use rings::service::ServiceTrait;

#[ringm::service(mringm, auth, auth)]
#[ringm::default_any]
pub struct Auth {
    pub username: String,
    pub password: String,
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
