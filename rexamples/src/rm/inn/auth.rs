

#[ringm::service(rm::service::inn)]
pub struct Auth {
    pub username: String,
    pub password: String,
}
