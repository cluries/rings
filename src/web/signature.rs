use crate::erx::{Layouted, LayoutedC, Erx};
use crate::tools::hash;
use crate::web::api::Out;
use crate::web::request::clone_request;
use crate::web::url::parse_query;

use axum::response::IntoResponse;
use futures_util::future::BoxFuture;
use redis::Commands;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::info;
use crate::erx;

static DEFAULT_RAND_LIFE: i64 = 300;

fn make_code(detail: &str) -> LayoutedC {
    Layouted::middleware("SIGN", detail)
}


macro_rules! make_err_res {
    ($x:expr) => {
        Out::<()>{
            code: make_code($x).into(),
            message:None,
            data: None,
        }.into_response()
    };

    ($x:expr, $y:expr) => {
       Out::<()> {
            code:make_code($x).into(),
            message: Some($y),
            data: None,
       }.into_response()
    };

    ($x:expr, $y:expr, $z:expr) => {
        Out{
            code:make_code($x).into(),
            message: Some($y),
            data: Some($z),
       }.into_response()
    };

    ($($x:expr),*) => {
        panic!("processing more than 3 arguments: {:?}", [$($x),*]);
    };
}

pub type KeyLoader = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output=Result<String, Erx>> + Send>> + Send + Sync>;

pub struct Signator {
    rear: String, // 后门，开发时候方便用
    excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>,
    nonce_lifetime: i64,
    key_loader: KeyLoader,
    redis_client: redis::Client,
}

impl Clone for Signator {
    fn clone(&self) -> Self {
        Signator {
            rear: self.rear.clone(),
            excludes: self.excludes.clone(),
            nonce_lifetime: self.nonce_lifetime,
            key_loader: Arc::clone(&self.key_loader),
            redis_client: self.redis_client.clone(),
        }
    }
}

impl Signator {
    pub fn new(redis_url: &str, key_loader: KeyLoader) -> Self {
        Self::with_rear(redis_url, Arc::clone(&key_loader), String::default())
    }

    pub fn with_rear(redis_url: &str, key_loader: KeyLoader, rear: String) -> Self {
        Signator {
            rear,
            excludes: vec![],
            nonce_lifetime: DEFAULT_RAND_LIFE,
            key_loader,
            redis_client: redis::Client::open(redis_url).unwrap_or_else(|err| {
                panic!("failed to connect to redis: {}", err);
            }),
        }
    }

    pub fn add_exclude(&mut self, exclude: fn(parts: &axum::http::request::Parts) -> bool) -> &mut Self {
        self.excludes.push(exclude);
        self
    }

    pub fn exclude(&self, parts: &axum::http::request::Parts) -> bool {
        self.excludes.iter().any(|exclude| exclude(parts))
    }

    pub async fn exec(&self, request: axum::extract::Request) -> Result<axum::extract::Request, axum::response::Response> {
        let (payload_request, mut request) = clone_request(request).await;
        let payload = Payload::from_request(payload_request).await;
        if let Err(error) = payload {
            return Err(make_err_res!("PASE", error));
        }

        let payload = payload.unwrap();
        if let Err(reason) = payload.guard() {
            return Err(make_err_res!("FMAT", reason.to_string()));
        }

        let loader = Arc::clone(&self.key_loader);
        let key = match loader(payload.val_xu()).await {
            Ok(key) => key,
            Err(erx) => {
                return Err(make_err_res!("LDKY", erx.message().to_string()));
            }
        };

        if let Err((error, debug)) = payload.valid(key) {
            let skip = !self.rear.is_empty() && self.rear.eq(&payload.val_ds());
            if !skip {
                return Err(make_err_res!("INVD", error, debug));
            }
        }

        if let Err(reason) = self.rand_guard(payload.val_xu(), payload.val_xr()).await {
            return Err(make_err_res!("RAND", reason.message_string()));
        }

        use crate::web::context::Context;
        let context = Context::new(payload.val_xu());
        request.extensions_mut().insert(context);

        Ok(request)
    }

    async fn rand_guard(&self, xu: String, xr: String) -> erx::ResultEX {
        let mut conn = self.redis_client.get_connection().map_err(erx::smp)?;

        let name = format!("XR:{}", xu);
        let score: i64 = conn.zscore(name.as_str(), xr.as_str()).unwrap_or(0);
        let current: i64 = chrono::Local::now().timestamp();

        if (current - score).abs() < self.nonce_lifetime {
            return Err("duplicate rand value".into());
        }

        let mut pipe = redis::pipe();
        pipe.zadd(name.as_str(), xr.as_str(), current);
        pipe.zrembyscore(name.as_str(), "-inf", current - self.nonce_lifetime);
        pipe.expire(name.as_str(), self.nonce_lifetime);
        let _ = pipe.query::<Vec<i64>>(&mut conn);

        Ok(())
    }
}

#[derive(Clone)]
pub struct SigLayer {
    signator: Arc<Signator>,
}

#[derive(Clone)]
pub struct SigMiddle<S>
where
    S: Clone,
{
    inner: S,
    signator: Arc<Signator>,
}

impl SigLayer {
    pub fn new(redis_url: &str, key_loader: KeyLoader, excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>) -> Self {
        Self::with_rear(redis_url, key_loader, excludes, "".to_string())
    }

    pub fn with_rear(redis_url: &str, key_loader: KeyLoader, excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>, rear: String) -> Self {
        info!("new signator: {}", redis_url);
        let mut s = Signator::with_rear(redis_url, key_loader, rear);
        for exclude in excludes {
            s.add_exclude(exclude);
        }

        SigLayer {
            signator: Arc::new(s),
        }
    }
}

impl<S> Layer<S> for SigLayer
where
    S: Clone,
{
    type Service = SigMiddle<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SigMiddle {
            inner,
            signator: Arc::clone(&self.signator),
        }
    }
}

impl<S> Service<axum::extract::Request> for SigMiddle<S>
where
    S: Service<axum::extract::Request, Response=axum::response::Response>
    + Send
    + Clone
    + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: axum::extract::Request) -> Self::Future {
        let sig = Arc::clone(&self.signator);
        let (parts, body) = request.into_parts();

        if sig.exclude(&parts) {
            let request = axum::extract::Request::from_parts(parts, body);
            let future = self.inner.call(request);
            return Box::pin(async move { Ok(future.await?) });
        }

        let mut inner = self.inner.clone();
        Box::pin(async move {
            let request = axum::extract::Request::from_parts(parts, body);
            match sig.exec(request).await {
                Ok(request) => {
                    let response: axum::response::Response = inner.call(request).await?;
                    Ok(response)
                }
                Err(response) => Ok(response),
            }
        })
    }
}

struct Payload {
    method: String,
    path: String,

    xu: Option<String>,
    xt: Option<String>,
    xr: Option<String>,
    xs: Option<String>,
    ds: Option<String>, //dev_skip

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


static XU: &str = "XU";
static XT: &str = "XT";
static XR: &str = "XR";
static XS: &str = "XS";
static DS: &str = "X-DEVELOPMENT-SKIP";

static POST: &str = "POST";
static PUT: &str = "PUT";
static DELETE: &str = "DELETE";
static GET: &str = "GET";
static HEAD: &str = "HEAD";
static PATCH: &str = "PATCH";
static OPTIONS: &str = "OPTIONS";


impl Payload {
    async fn from_request(req: axum::extract::Request) -> Result<Self, String> {
        let (parts, body) = req.into_parts();

        let headers = parts.headers;
        let header = |n| -> Option<String> {
            headers.get(n).and_then(|value| value.to_str().ok()).map(String::from)
        };

        let xu = header(XU);
        let xt = header(XT);
        let xr = header(XR);
        let xs = header(XS);
        let ds = header(DS);

        let path = parts.uri.path().to_string();
        let method = parts.method.as_str().to_uppercase();

        let queries = parse_query(parts.uri.query().unwrap_or_default());

        let mut bd: Option<serde_json::Value> = None;
        if method == POST || method == PUT || method == DELETE || method == OPTIONS || method == PATCH {
            const LIMIT: usize = 1024 * 1024 * 32;
            let body: Result<serde_json::Value, String> =
                match axum::body::to_bytes(body, LIMIT).await {
                    Ok(bytes) => {
                        if bytes.len() < 1 {
                            Ok(serde_json::Value::default())
                        } else {
                            match serde_json::from_slice::<serde_json::Value>(&bytes) {
                                Ok(json) => Ok(json),
                                Err(err) => Err(err.to_string()),
                            }
                        }
                    }
                    Err(err) => Err(err.to_string()),
                };

            if let Err(err) = body {
                return Err(err);
            }
            bd = body.ok();
        }

        let payload = Payload { method, path, xu, xt, xr, xs, ds, queries, body: bd };
        Ok(payload)
    }

    pub fn val_xu(&self) -> String {
        self.xu.clone().unwrap_or_default()
    }

    pub fn val_xr(&self) -> String {
        self.xr.clone().unwrap_or_default()
    }

    pub fn val_ds(&self) -> String {
        self.ds.clone().unwrap_or_default()
    }

    fn valid(&self, key: String) -> Result<(), (String, Debug)> {
        let load = self.payload();
        let hash = hash::hmac_sha1(&load, &key);

        if !hash.eq(self.xs.as_ref().unwrap_or(&String::default())) {
            let debug = Debug {
                payload: load,
                key: key.clone(),
                client: self.val_xu(),
                server: hash,
            };
            return Err((String::from("invalid signature"), debug));
        }

        Ok(())
    }

    fn guard(&self) -> Result<(), &str> {
        if self.xu.is_none() || self.xt.is_none() || self.xr.is_none() || self.xs.is_none() {
            return Err("missing signature data in header");
        }

        const MAX: i64 = 60 * 5;
        let xti = self.xt.as_ref().unwrap().parse::<i64>().unwrap_or(0);
        if xti < MAX || (chrono::Utc::now().timestamp() - xti).abs() > MAX {
            return Err("the time difference is too large");
        }

        let length = self.xr.as_ref().unwrap().len();
        if length <= 8 || length >= 40 {
            return Err("random string length invalid");
        }

        if self.xs.as_ref().unwrap().len() != 40 {
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
