use crate::erx::Layouted;
use crate::tos;
use crate::web::api::Out;
use crate::web::define;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Except is use in controller/action.
// wrapper some pre-defined error.
// except object can fast convert to response
#[derive(Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub enum Except {
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    Unknown(String),
    InvalidParam(String),
    InvalidParams(Vec<String>),
    Fuzzy(String, String),
    FuzzyService(String, String),
    FuzzyModel(String, String),
    FuzzyAction(String, String),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExceptGrow {
    except: Except,
    grows: HashMap<String, String>,
}

impl Except {
    /// convert to response object
    pub fn out<T>(&self) -> Out<T>
    where
        T: Serialize,
    {
        // use crate::erx::{COMM, FUZZ};
        use crate::erx::PreL4;

        let defined_wrapper = |c: define::HttpCode| Out::<T> {
            code: Layouted::common(PreL4::COMM.four(), &format!("{:04}", c.code())).into(),
            message: Some(c.message().into()),
            data: None,
            debug: None,
            profile: None,
        };

        match self {
            Except::Unauthorized => defined_wrapper(define::HttpCode::Unauthorized),
            Except::Forbidden => defined_wrapper(define::HttpCode::Forbidden),
            Except::NotFound => defined_wrapper(define::HttpCode::NotFound),
            Except::InternalServerError => defined_wrapper(define::HttpCode::InternalServerError),
            Except::Unknown(m) => {
                let m = if m.is_empty() {
                    "Hi there! Something unexpected happened, but our engineers have already been notified."
                } else {
                    m
                };
                Out::<T> {
                    code: Layouted::common(PreL4::COMM.four(), "9999").into(),
                    message: tos!(m),
                    data: None,
                    debug: None,
                    profile: None,
                }
            },
            Except::InvalidParam(m) => {
                let m = if m.is_empty() { "invalid params" } else { m };
                Out::<T> {
                    code: Layouted::common(PreL4::COMM.four(), "1000").into(),
                    message: tos!(m),
                    data: None,
                    debug: None,
                    profile: None,
                }
            },
            Except::InvalidParams(m) => {
                let m = if m.is_empty() { "invalid params".to_string() } else { m.join(", ") };
                Out::<T> {
                    code: Layouted::common(PreL4::COMM.four(), "1000").into(),
                    message: Some(m),
                    data: None,
                    debug: None,
                    profile: None,
                }
            },
            Except::Fuzzy(detail, m) => Out::<T> {
                code: Layouted::common(PreL4::FUZZ.four(), detail).into(),
                message: tos!(m),
                data: None,
                debug: None,
                profile: None,
            },
            Except::FuzzyService(detail, m) => Out::<T> {
                code: Layouted::service(PreL4::FUZZ.four(), detail).into(),
                message: tos!(m),
                data: None,
                debug: None,
                profile: None,
            },
            Except::FuzzyModel(detail, m) => Out::<T> {
                code: Layouted::model(PreL4::FUZZ.four(), detail).into(),
                message: tos!(m),
                data: None,
                debug: None,
                profile: None,
            },
            Except::FuzzyAction(detail, m) => Out::<T> {
                code: Layouted::action(PreL4::FUZZ.four(), detail).into(),
                message: tos!(m),
                data: None,
                debug: None,
                profile: None,
            },
        }
    }

    pub fn grow(self) -> ExceptGrow {
        ExceptGrow { except: self, grows: HashMap::new() }
    }
}

impl ExceptGrow {
    pub fn add(&mut self, key: &str, value: &str) -> &mut Self {
        self.grows.insert(key.to_string(), value.to_string());
        self
    }

    pub fn add_all(&mut self, all: HashMap<String, String>) -> &mut Self {
        self.grows.extend(all);
        self
    }

    pub fn get(&self, key: String) -> Option<&String> {
        self.grows.get(&key)
    }

    pub fn get_mut(&mut self, key: String) -> Option<&mut String> {
        self.grows.get_mut(&key)
    }

    pub fn get_default(&self, key: String, val: String) -> String {
        self.grows.get(&key).unwrap_or(&val).to_string()
    }

    pub fn grows(&self) -> HashMap<String, String> {
        self.grows.clone()
    }

    pub fn mut_grows(&mut self) -> &mut HashMap<String, String> {
        &mut self.grows
    }

    pub fn grows_size(&self) -> usize {
        self.grows.len()
    }

    pub fn diminish(self) -> Except {
        self.except
    }

    pub fn out<T>(&self) -> Out<T>
    where
        T: Serialize,
    {
        let mut out = self.except.out();
        if !self.grows.is_empty() {
            let mut debug = out.debug.unwrap_or_default();
            for (key, value) in &self.grows {
                debug.add_item(key, value);
            }
            out.debug = Some(debug);
        }
        out
    }
}
