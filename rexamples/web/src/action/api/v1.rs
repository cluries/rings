use rings::axum::Router;

mod system;
mod public;

pub fn v1_actions() -> Vec<Router> {
    vec![
        public::public_action(),
        system::system_action()
    ]
}
