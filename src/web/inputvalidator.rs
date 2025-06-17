use crate::web::except::Except;

pub trait InputValidator {
    fn validate(&self) -> Option<Except>;
}
