use crate::web::except::Except;

/// see https://github.com/Keats/validator
///
pub struct Inputs;

impl Inputs {
    pub fn guard<T: validator::Validate>(inputs: &T) -> Option<Except> {
        match inputs.validate() {
            Ok(_) => None,
            Err(errs) => {
                let except: Vec<String> = errs.into_errors().iter().map(|err| err.0.to_string()).collect();
                Some(Except::InvalidParams(except))
            },
        }
    }
}
