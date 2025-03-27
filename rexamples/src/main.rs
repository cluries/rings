#[allow(dead_code, unused)]
mod rm;


#[ringm::service]
struct SomeOne {}

#[ringm::service]
struct SomeTwo {}

fn main() {


    ringm::serviced!("crate");
}
