use crate::erx::{Erx, LayoutedC};
use crate::web::except::Except;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::http::header::{CONTENT_TYPE};

use serde::{Deserialize, Serialize};

pub type OutAny = Out<serde_json::Value>;


#[derive(Serialize, Deserialize, Debug)]
pub struct Out<T: Serialize> {
    pub code: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}


impl<T: Serialize> Out<T> {
    pub fn new(code: LayoutedC, message: Option<String>, data: Option<T>) -> Self {
        let code: String = code.into();
        Out {
            code,
            message,
            data,
        }
    }

    pub fn only_code(code: LayoutedC) -> Self {
        let code: String = code.into();
        Out {
            code,
            message: None,
            data: None,
        }
    }

    pub fn code_message(code: LayoutedC, message: &str) -> Self {
        let mut m = None;
        if !message.is_empty() {
            m = Some(message.to_string());
        }

        let code = code.into();
        Out {
            code,
            message: m,
            data: None,
        }
    }

    pub fn ok(data: T) -> Self {
        Out {
            code: LayoutedC::okay().into(),
            message: None,
            data: Some(data),
        }
    }
}

impl<T: Serialize> From<Except> for Out<T> {
    fn from(except: Except) -> Self {
        except.out()
    }
}


impl<T: Serialize> From<Erx> for Out<T> {
    fn from(value: Erx) -> Self {
        Except::Fuzzy(
            value.code().layout_string(),
            value.message().to_string(),
        ).into()
    }
}

impl<T: Serialize> From<Option<T>> for Out<T> {
    fn from(value: Option<T>) -> Self {
        static OPTION_NONE_MESSAGE: &str = "sorry, some error occurred, but no message was provided";
        match value {
            Some(data) => Out::ok(data),
            None => Except::Unknown(OPTION_NONE_MESSAGE.to_string()).into()
        }
    }
}

impl<T: Serialize, E: ToString> From<Result<T, E>> for Out<T> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => {
                Out::ok(v)
            }
            Err(e) => {
                let message = e.to_string();
                Except::Unknown(message).into()
            }
        }
    }
}

impl<T: Serialize> axum::response::IntoResponse for Out<T> {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self);
        // const API_HEADERS: [(&str, &str); 2] = [
        //     ("Content-Type", "application/json"),
        //     ("Powered-By", "rebit"),
        // ];


        let (status, body) = match body {
            Ok(body) => {
                (StatusCode::OK, body)
            }
            Err(err) => {
                static JSE: &str = "json serialization error";
                let body = serde_json::to_string(
                    &Except::Unknown(
                        err.to_string()
                    ).out::<()>()
                ).unwrap_or(JSE.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, body)
            }
        };

        let mut response = Response::new(body);
        *response.status_mut() = status;
        
        let headers = response.headers_mut();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Powered-By", HeaderValue::from_static("REBIT"));

        response.into_response()
    }
}


