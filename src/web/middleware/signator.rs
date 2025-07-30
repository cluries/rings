use crate::erx;
use crate::erx::{Erx, Layouted, LayoutedC};
use crate::tools::hash;
use crate::web::middleware::{ApplyKind, Context, Middleware, MiddlewareFuture, Pattern};
use crate::web::{api::Out, define::HttpMethod, request::clone_request, url::parse_query};
use axum::{
    extract::Request,
    http::{request::Parts, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// 默认配置常量
const DEFAULT_NONCE_LIFETIME: i64 = 300; // 5分钟
const MAX_BODY_SIZE: usize = 1024 * 1024 * 32; // 32MB
const MAX_TIME_DIFF: i64 = 60 * 5; // 5分钟时间差
const MIN_NONCE_LENGTH: usize = 8;
const MAX_NONCE_LENGTH: usize = 40;
const SIGNATURE_LENGTH: usize = 40;

// 错误码常量
mod error_c {
    pub const SIGNATURE: &str = "SIGN";
    pub const PAYLOAD: &str = "PAYL";
    pub const FORMAT: &str = "FRMT";
    pub const LOAD: &str = "LOAD";
    pub const INVALID: &str = "INVD";
}

#[inline]
fn layouted_middleware(detail: &str) -> LayoutedC {
    Layouted::middleware(error_c::SIGNATURE, detail)
}

macro_rules! rout {
    ($x:expr) => {
        Out::<()>{code:layouted_middleware($x).into(), message:None, data:None, debug:None, profile:None}.into_response()
    };

    ($x:expr, $y:expr) => {
       Out::<()> {code:layouted_middleware($x).into(), message:Some($y), data:None, debug:None, profile:None}.into_response()
    };

    ($x:expr, $y:expr, $z:expr) => {
        Out{code:layouted_middleware($x).into(), message:Some($y), data:Some($z), debug:None, profile:None}.into_response()
    };

    ($($x:expr),*) => {
        panic!("processing more than 3 arguments: {:?}", [$($x),*]);
    };
}

pub type KeyLoader = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> + Send + Sync>;


pub struct Signator {
    backdoor: String, // 后门，开发时候方便用
    nonce_lifetime: i64,
    key_loader: KeyLoader,
    redis_client: redis::Client,
}

impl Signator {
    pub fn new(redis_url: &str, key_loader: KeyLoader) -> Self {
        Self::with_backdoor(redis_url, Arc::clone(&key_loader), String::default())
    }

    pub fn with_backdoor(redis_url: &str, key_loader: KeyLoader, backdoor: String) -> Self {
        Signator {
            backdoor,
            nonce_lifetime: DEFAULT_NONCE_LIFETIME,
            key_loader,
            redis_client: redis::Client::open(redis_url).unwrap_or_else(|err| {
                tracing::error!("{} {}", redis_url, err);
                panic!("failed to connect to redis: {}", err);
            }),
        }
    }

    pub async fn exec(&self, request: axum::extract::Request) -> Result<axum::extract::Request, axum::response::Response> {
        let (payload_request, mut request) = clone_request(request).await;

        let payload = Payload::from_request(payload_request).await.map_err(|e| rout!(error_c::PAYLOAD, e))?;
        payload.guard().map_err(|e| rout!(error_c::FORMAT, e.into()))?;

        let loader = Arc::clone(&self.key_loader);
        let key = loader(payload.xget_u()).await.map_err(|e| rout!(error_c::LOAD, e.message_string()))?;

        if let Err((error, debug)) = payload.valid(key) {
            if self.backdoor.is_empty() || !self.backdoor.eq(&payload.xget_d()) {
                return Err(rout!(error_c::INVALID, error, debug));
            }
        }

        self.rand_guard(&payload).await.map_err(|e| rout!(error_c::INVALID, e.message_string()))?;

        let context = crate::web::context::Context::new(payload.xget_u());
        request.extensions_mut().insert(context);

        Ok(request)
    }

    async fn rand_guard(&self, payload: &Payload) -> erx::ResultEX {
        let mut conn: redis::aio::MultiplexedConnection = self.redis_client.get_multiplexed_tokio_connection().await.map_err(erx::smp)?;

        let name = format!("XR:{}", payload.xget_u());
        let xr = payload.xget_r();

        let score: Option<i64> = conn.zscore(name.as_str(), &xr).await.map_err(erx::smp)?;

        let score = score.unwrap_or(0);
        let current: i64 = chrono::Local::now().timestamp();

        if (current - score).abs() < self.nonce_lifetime {
            return Err("duplicate rand value".into());
        }

        let mut pipe = redis::pipe();
        pipe.zadd(name.as_str(), &xr, current);
        pipe.zrembyscore(name.as_str(), "-inf", current - self.nonce_lifetime);
        pipe.expire(name.as_str(), self.nonce_lifetime);
        let _r = pipe.query_async::<Vec<i64>>(&mut conn).await.map_err(erx::smp)?;

        Ok(())
    }
}

impl Clone for Signator {
    fn clone(&self) -> Self {
        Signator {
            backdoor: self.backdoor.clone(),
            nonce_lifetime: self.nonce_lifetime,
            key_loader: Arc::clone(&self.key_loader),
            redis_client: self.redis_client.clone(),
        }
    }
}

impl std::fmt::Debug for Signator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signator")
            .field("backdoor", &self.backdoor)
            .field("nonce_lifetime", &self.nonce_lifetime)
            .field("redis_client", &self.redis_client)
            .finish()
    }
}

impl Middleware for Signator {
    fn name(&self) -> &'static str {
        "signator"
    }

    fn on_request(&self, context: Context, request: Request) -> Option<MiddlewareFuture<Request>> {
        let signator = self.clone();

        let r = Box::pin(async move {
            match signator.exec(request).await {
                Ok(req) => Ok((context, req)),
                Err(res) => {

                    // context.make_abort_with_response("signator", "message", res);
                    
                    Err(erx::Erx::new("message"))
                },
            }
        });

        Some(r)
    }

    fn on_response(&self, _context: Context, _response: Response) -> Option<MiddlewareFuture<Response>> {
        None
    }

    /// 可选：中间件优先级，数值越大优先级越高
    fn priority(&self) -> i32 {
        0
    }

    /// 可选：判断中间件是否应该处理这个请求
    /// 优先级 focus > methods > patterns
    /// - 如果foucs返回不为None,直接使用foucs的返回值判定
    fn apply(&self, _parts: &Parts) -> Option<bool> {
        None
    }

    /// 可选：HTTP 方法过滤
    fn methods(&self) -> Option<Vec<ApplyKind<HttpMethod>>> {
        None
    }

    /// 可选：路径匹配模式
    fn patterns(&self) -> Option<Vec<ApplyKind<Pattern>>> {
        None
    }
}

struct Payload {
    method: String,
    path: String,

    xu: Option<String>, // userid
    xt: Option<String>, // timestamp
    xr: Option<String>, // nonce
    xs: Option<String>, // signature
    ds: Option<String>, // X-DEVELOPMENT-SKIP

    queries: HashMap<String, String>,
    body: Option<serde_json::Value>,
}

#[derive(Default, Debug, Serialize)]
struct Debug {
    payload: String,
    key: String,
    server: String,
    client: String,
}

mod header_names {

    /// user id
    pub(crate) const U: &'static str = "X-U";

    /// timestamp
    pub(crate) const T: &'static str = "X-T";

    /// nonce
    pub(crate) const R: &'static str = "X-R";

    ///signature
    pub(crate) const S: &'static str = "X-S";

    /// development skip
    pub(crate) const D: &'static str = "X-DEVELOPMENT-SKIP";
}

impl Payload {
    fn new(method: String, path: String, queries: HashMap<String, String>) -> Self {
        Payload { method, path, xu: None, xt: None, xr: None, xs: None, ds: None, queries, body: None }
    }

    fn body_guard(&self) -> bool {
        HttpMethod::POST.is(&self.method)
            || HttpMethod::PUT.is(&self.method)
            || HttpMethod::DELETE.is(&self.method)
            || HttpMethod::OPTIONS.is(&self.method)
            || HttpMethod::PATCH.is(&self.method)
            || HttpMethod::PATCH.is(&self.method)
    }

    async fn from_request(req: axum::extract::Request) -> Result<Self, String> {
        let (parts, body) = req.into_parts();

        let path = parts.uri.path().to_string();
        let method = parts.method.as_str().to_uppercase();
        let queries = parse_query(parts.uri.query().unwrap_or_default());

        let mut payload = Payload::new(method, path, queries);

        if payload.body_guard() {
            let body: Result<serde_json::Value, String> = match axum::body::to_bytes(body, MAX_BODY_SIZE).await {
                Ok(bytes) => {
                    if bytes.len() < 1 {
                        Ok(serde_json::Value::default())
                    } else {
                        match serde_json::from_slice::<serde_json::Value>(&bytes) {
                            Ok(json) => Ok(json),
                            Err(err) => Err(err.to_string()),
                        }
                    }
                },
                Err(err) => Err(err.to_string()),
            };

            if let Err(err) = body {
                return Err(err);
            }

            payload.body = body.ok();
        }

        payload.load_header(parts.headers);

        Ok(payload)
    }

    fn load_header(&mut self, headers: HeaderMap<HeaderValue>) {
        let header = |n| -> Option<String> { headers.get(n).and_then(|value| value.to_str().ok()).map(String::from) };

        self.xu = header(header_names::U);
        self.xt = header(header_names::T);
        self.xr = header(header_names::R);
        self.xs = header(header_names::S);
        self.ds = header(header_names::D);
    }

    pub fn xget_u(&self) -> String {
        self.xu.clone().unwrap_or_default()
    }

    pub fn xget_r(&self) -> String {
        self.xr.clone().unwrap_or_default()
    }

    pub fn xget_s(&self) -> String {
        self.xs.clone().unwrap_or_default()
    }

    pub fn xget_d(&self) -> String {
        self.ds.clone().unwrap_or_default()
    }

    fn valid(&self, key: String) -> Result<(), (String, Debug)> {
        let load = self.payload();
        let hash = hash::hmac_sha1(&load, &key);

        if !hash.eq(self.xs.as_ref().unwrap_or(&String::default())) {
            let debug = Debug { payload: load, key: key.clone(), client: self.xget_s(), server: hash };
            return Err((String::from("invalid signature"), debug));
        }

        Ok(())
    }

    fn guard(&self) -> Result<(), &str> {
        if self.xu.is_none() || self.xt.is_none() || self.xr.is_none() || self.xs.is_none() {
            return Err("missing signature data in header");
        }

        let xti = self.xt.as_ref().unwrap().parse::<i64>().unwrap_or(0);
        if xti < MAX_TIME_DIFF || (chrono::Utc::now().timestamp() - xti).abs() > MAX_TIME_DIFF {
            return Err("the time difference is too large");
        }

        let length = self.xr.as_ref().unwrap().len();
        if length <= MIN_NONCE_LENGTH || length >= MAX_NONCE_LENGTH {
            return Err("random string length invalid");
        }

        if self.xs.as_ref().unwrap().len() != SIGNATURE_LENGTH {
            return Err("invalid signature data in header");
        }

        Ok(())
    }

    fn header_payload(&self) -> String {
        let mut payload = String::new();
        payload.push_str(self.method.to_uppercase().as_str());
        payload.push_str(",");
        payload.push_str(self.path.as_str());
        payload.push_str(",{");
        if let Some(xu) = &self.xu {
            payload.push_str(xu);
            payload.push_str(",");
        }
        if let Some(xt) = &self.xt {
            payload.push_str(xt);
            payload.push_str(",");
        }
        if let Some(xr) = &self.xr {
            payload.push_str(xr);
        }

        payload.push_str("}");
        payload
    }

    fn queries_payload(&self, mut payload: String) -> String {
        let mut size = self.queries.len();
        if size < 1 {
            return payload;
        }

        let mut query_keys: Vec<String> = self.queries.keys().cloned().collect();
        query_keys.sort();

        payload.push_str(",{");
        for k in query_keys {
            payload.push_str(&k);
            payload.push_str("=");
            payload.push_str(self.queries.get(&k).unwrap());

            size -= 1;
            if size > 0 {
                payload.push_str(",");
            }
        }

        payload.push_str("}");

        payload
    }

    fn payload(&self) -> String {
        let mut payload = self.queries_payload(self.header_payload());

        if let Some(body) = &self.body {
            payload.push_str(",");
            let body_payload = Self::json_payload(body);
            payload.push_str(body_payload.as_str());
        }

        payload
    }

    fn array_formatter(array: &Vec<serde_json::Value>) -> String {
        let mut payload = String::new();
        let mut array_len = array.len();

        payload.push_str("[");

        for item in array {
            payload.push_str(Self::json_payload(item).as_str());
            array_len -= 1;
            if array_len > 0 {
                payload.push_str(",");
            }
        }
        payload.push_str("]");
        payload
    }

    fn object_formatter(object: &serde_json::Map<String, serde_json::Value>) -> String {
        let mut payload = String::new();

        let mut object_keys: Vec<String> = object.keys().cloned().collect();
        object_keys.sort();
        payload.push_str("{");

        let mut object_size = object_keys.len();
        for key in object_keys {
            let val = object.get(&key).unwrap();
            payload.push_str(key.as_str());
            payload.push_str("=");
            payload.push_str(Self::json_payload(val).as_str());

            object_size -= 1;
            if object_size > 0 {
                payload.push_str(",");
            }
        }

        payload.push_str("}");
        payload
    }

    fn json_payload(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(i) => i.to_string(),
            serde_json::Value::String(s) => s.to_string(),
            serde_json::Value::Array(array) => Self::array_formatter(array),
            serde_json::Value::Object(object) => Self::object_formatter(object),
        }
    }
}
