use crate::erx::{Erx, LayoutedC};
use crate::web::except::Except;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const ORIGIN_ERX_CODE: &'static str = "ORIGIN_ERX_CODE";
const JSON_SERIAL_ERROR: &'static str = "JSON serialization error";
const OPTION_NONE_MESSAGE: &'static str = "Sorry, some error occurred, but no message was provided";

pub type OutAny = Out<serde_json::Value>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Out<T: Serialize> {
    pub code: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<Debug>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Debug {
    others: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {}

impl Debug {
    pub fn new() -> Debug {
        Debug { others: HashMap::new() }
    }

    pub fn add_other(&mut self, key: &str, value: &str) -> &mut Debug {
        self.others.insert(key.to_string(), value.to_string());
        self
    }
}

impl<T: Serialize> Out<T> {
    pub fn new(code: LayoutedC, message: Option<String>, data: Option<T>) -> Self {
        Out { code: code.into(), message, data, debug: None, profile: None }
    }

    pub fn only_code(code: LayoutedC) -> Self {
        Out { code: code.into(), message: None, data: None, debug: None, profile: None }
    }

    pub fn code_message(code: LayoutedC, message: &str) -> Self {
        Out {
            code: code.into(),
            message: if message.is_empty() { None } else { Some(message.to_string()) },
            data: None,
            debug: None,
            profile: None,
        }
    }

    pub fn ok(data: T) -> Self {
        Out { code: LayoutedC::okay().into(), message: None, data: Some(data), debug: None, profile: None }
    }

    pub fn set_debug(&mut self, debug: Debug) {
        self.debug = Some(debug);
    }

    pub fn set_profile(&mut self, profile: Profile) {
        self.profile = Some(profile);
    }
}

impl<T: Serialize> From<Except> for Out<T> {
    fn from(except: Except) -> Self {
        except.out()
    }
}

impl<T: Serialize> From<Erx> for Out<T> {
    fn from(value: Erx) -> Self {
        Except::Fuzzy(value.code().get_detail().to_string(), value.message().to_string())
            .grow()
            .add(ORIGIN_ERX_CODE, &value.code().layout_string())
            .out()
    }
}

impl<T: Serialize> From<Option<T>> for Out<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(data) => Out::ok(data),
            None => Except::Unknown(OPTION_NONE_MESSAGE.to_string()).into(),
        }
    }
}

impl<T: Serialize, E: ToString> From<Result<T, E>> for Out<T> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => Out::ok(v),
            Err(e) => {
                let message = e.to_string();
                Except::Unknown(message).into()
            },
        }
    }
}

impl<T: Serialize> axum::response::IntoResponse for Out<T> {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self);

        let (status, body) = match body {
            Ok(body) => (StatusCode::OK, body),
            Err(err) => {
                let body = serde_json::to_string(&Except::Unknown(err.to_string()).out::<()>()).unwrap_or(JSON_SERIAL_ERROR.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, body)
            },
        };

        let mut response = Response::new(body);
        *response.status_mut() = status;

        let headers = response.headers_mut();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Powered-By", HeaderValue::from_static("Rings"));

        response.into_response()
    }
}
