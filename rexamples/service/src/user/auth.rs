use rings::tokio_cron_scheduler;

#[ringm::service(user, auth)]
#[ringm::default_any]
pub struct Auth {}

impl rings::service::ServiceTrait for Auth {
    fn name(&self) -> &'static str {
        Self::service_name()
    }

    fn initialize(&mut self) {}

    fn release(&mut self) {}

    fn ready(&self) -> bool {
        true
    }

    fn schedules(&self) -> Vec<tokio_cron_scheduler::Job> {
        vec![
            schedule::scone()
        ]
    }
}
