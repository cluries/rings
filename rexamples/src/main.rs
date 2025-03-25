#[allow(dead_code, unused)]
mod rm;


#[ringm::service("service some one")]
struct SomeOne {}

#[ringm::service("service some two")]
struct SomeTwo {}

fn main() {
    ringm::serviced!("service developer");
}
