#[ringm::service("defaults")]
pub struct Auth {
    pub username: String,
    pub password: String,
}
