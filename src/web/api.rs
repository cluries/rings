use crate::erx::LayoutedC;
use crate::web::except::Except;
use axum::http::StatusCode;
use axum::response::Response;
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


impl<T: Serialize> axum::response::IntoResponse for Out<T> {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self);
        const API_HEADERS: [(&str, &str); 2] = [
            ("Content-Type", "application/json"),
            ("Powered-By", "rebit"),
        ];

        match body {
            Ok(body) => {
                let status = StatusCode::OK;
                (status, API_HEADERS, body).into_response()
            }
            Err(err) => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;

                let body = Except::Unknown(err.to_string()).out::<()>();
                let body = serde_json::to_string(&body).unwrap_or(String::from("json serialization error"));
                (status, API_HEADERS, body).into_response()
            }
        }
    }
}