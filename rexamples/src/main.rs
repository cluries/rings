use rings::any::AnyTrait;
use rings::service::ServiceTrait;
use std::any::Any;
use rings::rex::{tokio, tokio_cron_scheduler};

#[allow(dead_code, unused)]
mod mringm;


#[ringm::service]
struct ArgsService {}



#[ringm::service]
struct LanuchService {}


#[tokio::main]
async fn main() {

    ringm::serviced!();
}


impl AnyTrait for ArgsService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl AnyTrait for LanuchService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ServiceTrait for ArgsService {
    fn name(&self) -> &str {
        "Args"
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

impl ServiceTrait for LanuchService {
    fn name(&self) -> &str {
        "Lanuch"
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