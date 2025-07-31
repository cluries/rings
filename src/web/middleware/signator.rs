use crate::erx::{self, Erx};
use crate::tools::hash;
use crate::web::middleware::{ApplyKind, Context, Middleware, MiddlewareFuture, Pattern};
use crate::web::{define::HttpMethod, request::clone_request, url::parse_query};
use axum::{
    extract::Request,
    http::{request::Parts, HeaderMap, HeaderValue},
    response::Response,
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

pub type KeyLoader = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> + Send + Sync>;

#[derive(Debug)]
pub enum Error {
    // === 请求体相关错误 ===
    /// 请求体过大
    BodyTooLarge(usize),
    /// 请求体JSON格式错误
    BodyJsonInvalid(String),
    /// 请求体读取失败
    BodyReadFailed(String),

    // === 签名头部相关错误 ===
    /// 缺少必需的签名头部
    MissingHeaders(Vec<String>),
    /// 用户ID格式错误
    InvalidUserId(String),
    /// 时间戳格式错误
    InvalidTimestamp(String),
    /// 时间戳超出允许范围
    TimestampOutOfRange { timestamp: i64, max_diff: i64 },
    /// 随机数长度不符合要求
    InvalidNonceLength { length: usize, min: usize, max: usize },
    /// 签名长度不符合要求
    InvalidSignatureLength { length: usize, expected: usize },

    // === 随机数验证相关错误 ===
    /// Redis连接失败
    RedisConnectionFailed(String),
    /// 随机数重复使用
    NonceReused(String),
    /// Redis操作失败
    RedisOperationFailed(String),

    // === 密钥和签名验证相关错误 ===
    /// 密钥加载失败
    KeyLoadingFailed(Erx),
    /// 签名验证失败
    SignatureVerificationFailed(SignatureDebugInfo),

    // === 系统级错误 ===
    /// 内部服务器错误
    InternalError(String),
}

// impl From<Error> for crate::web::api::Out<_> {
//     fn from(err: Error) -> crate::web::api::Out {
        
//     }
// }
    

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // 请求体相关错误
            Error::BodyTooLarge(size) => write!(f, "Request body too large: {} bytes (max: {} bytes)", size, MAX_BODY_SIZE),
            Error::BodyJsonInvalid(msg) => write!(f, "Invalid JSON in request body: {}", msg),
            Error::BodyReadFailed(msg) => write!(f, "Failed to read request body: {}", msg),

            // 签名头部相关错误
            Error::MissingHeaders(headers) => write!(f, "Missing required headers: {}", headers.join(", ")),
            Error::InvalidUserId(user_id) => write!(f, "Invalid user ID format: {}", user_id),
            Error::InvalidTimestamp(timestamp) => write!(f, "Invalid timestamp format: {}", timestamp),
            Error::TimestampOutOfRange { timestamp, max_diff } => {
                write!(f, "Timestamp {} is outside acceptable range (±{} seconds)", timestamp, max_diff)
            },
            Error::InvalidNonceLength { length, min, max } => {
                write!(f, "Nonce length {} is invalid (must be between {} and {} characters)", length, min, max)
            },
            Error::InvalidSignatureLength { length, expected } => {
                write!(f, "Signature length {} is invalid (expected {} characters)", length, expected)
            },

            // 随机数验证相关错误
            Error::RedisConnectionFailed(msg) => write!(f, "Redis connection failed: {}", msg),
            Error::NonceReused(nonce) => write!(f, "Nonce has been used recently: {}", nonce),
            Error::RedisOperationFailed(msg) => write!(f, "Redis operation failed: {}", msg),

            // 密钥和签名验证相关错误
            Error::KeyLoadingFailed(err) => write!(f, "Failed to load signing key: {}", err.description()),
            Error::SignatureVerificationFailed(debug) => write!(f, "Signature verification failed: {}", debug),

            // 系统级错误
            Error::InternalError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        match err.kind() {
            redis::ErrorKind::IoError => Error::RedisConnectionFailed(err.to_string()),
            redis::ErrorKind::AuthenticationFailed => Error::RedisConnectionFailed(format!("Redis authentication failed: {}", err)),
            redis::ErrorKind::TypeError => Error::RedisOperationFailed(format!("Redis type error: {}", err)),
            _ => Error::RedisOperationFailed(err.to_string()),
        }
    }
}

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

    pub async fn exec(&self, request: axum::extract::Request) -> Result<axum::extract::Request, Error> {
        let (payload_request, mut request) = clone_request(request).await;

        let payload = Payload::from_request(payload_request).await?;
        payload.validate_headers()?;

        let loader = Arc::clone(&self.key_loader);
        let key = loader(payload.get_user_id()).await.map_err(Error::KeyLoadingFailed)?;

        if let Err(invalid) = payload.validate_signature(key) {
            if self.backdoor.is_empty() || !self.backdoor.eq(&payload.get_dev_skip()) {
                return Err(invalid);
            }
        }

        self.validate_nonce(&payload).await?;

        let context = crate::web::context::Context::new(payload.get_user_id());
        request.extensions_mut().insert(context);

        Ok(request)
    }

    async fn validate_nonce(&self, payload: &Payload) -> Result<(), Error> {
        let mut conn: redis::aio::MultiplexedConnection = self.redis_client.get_multiplexed_tokio_connection().await?;

        let name = format!("XR:{}", payload.get_user_id());
        let nonce = payload.get_nonce();

        let score: Option<i64> = conn.zscore(name.as_str(), &nonce).await?;

        let score = score.unwrap_or(0);
        let current: i64 = chrono::Local::now().timestamp();

        if (current - score).abs() < self.nonce_lifetime {
            return Err(Error::NonceReused(payload.get_nonce()));
        }

        let mut pipe = redis::pipe();
        pipe.zadd(name.as_str(), &nonce, current);
        pipe.zrembyscore(name.as_str(), "-inf", current - self.nonce_lifetime);
        pipe.expire(name.as_str(), self.nonce_lifetime);
        let _r = pipe.query_async::<Vec<i64>>(&mut conn).await?;

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
        Signator::middleware_name()
    }

    fn on_request(&self, context: Context, request: Request) -> Option<MiddlewareFuture<Request>> {
        let signator = self.clone();

        let r = Box::pin(async move {
            match signator.exec(request).await {
                Ok(req) => Ok((context, req)),
                Err(error) => {
                    let mut context = context;

                    let (status_code, error_message) = match &error {
                        // 400 Bad Request - 客户端请求格式错误
                        Error::BodyTooLarge(_)
                        | Error::BodyJsonInvalid(_)
                        | Error::BodyReadFailed(_)
                        | Error::MissingHeaders(_)
                        | Error::InvalidUserId(_)
                        | Error::InvalidTimestamp(_)
                        | Error::TimestampOutOfRange { .. }
                        | Error::InvalidNonceLength { .. }
                        | Error::InvalidSignatureLength { .. } => (400, "Bad Request"),

                        // 401 Unauthorized - 认证失败
                        Error::SignatureVerificationFailed(_) => (401, "Unauthorized"),

                        // 409 Conflict - 随机数重复使用
                        Error::NonceReused(_) => (409, "Conflict"),

                        // 500 Internal Server Error - 服务器内部错误
                        Error::RedisConnectionFailed(_)
                        | Error::RedisOperationFailed(_)
                        | Error::KeyLoadingFailed(_)
                        | Error::InternalError(_) => (500, "Internal Server Error"),
                    };

                    let error_body = serde_json::json!({
                        "error": error_message,
                        "message": error.to_string(),
                        "code": status_code
                    });

                    let res = Response::builder()
                        .status(status_code)
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(error_body.to_string()))
                        .unwrap_or_default();

                    context.make_abort_with_response(Signator::middleware_name(), &error.to_string(), res);
                    Err((context, erx::Erx::new("signator")))
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

    user_id: Option<String>,
    timestamp: Option<String>,
    nonce: Option<String>,
    signature: Option<String>,
    dev_skip: Option<String>,

    queries: HashMap<String, String>,
    body: Option<serde_json::Value>,
}

#[derive(Default, Debug, Serialize)]
pub struct SignatureDebugInfo {
    payload: String,
    key: String,
    server_signature: String,
    client_signature: String,
}

impl std::fmt::Display for SignatureDebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "payload='{}', expected='{}', received='{}'", self.payload, self.server_signature, self.client_signature)
    }
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
        Payload { method, path, user_id: None, timestamp: None, nonce: None, signature: None, dev_skip: None, queries, body: None }
    }

    fn should_read_body(&self) -> bool {
        HttpMethod::POST.is(&self.method)
            || HttpMethod::PUT.is(&self.method)
            || HttpMethod::DELETE.is(&self.method)
            || HttpMethod::OPTIONS.is(&self.method)
            || HttpMethod::PATCH.is(&self.method)
            || HttpMethod::PATCH.is(&self.method)
    }

    async fn from_request(req: axum::extract::Request) -> Result<Self, Error> {
        let (parts, body) = req.into_parts();

        let path = parts.uri.path().to_string();
        let method = parts.method.as_str().to_uppercase();
        let queries = parse_query(parts.uri.query().unwrap_or_default());

        let mut payload = Payload::new(method, path, queries);

        if payload.should_read_body() {
            let body: Result<serde_json::Value, Error> = match axum::body::to_bytes(body, MAX_BODY_SIZE).await {
                Ok(bytes) => {
                    if bytes.len() < 1 {
                        Ok(serde_json::Value::default())
                    } else {
                        match serde_json::from_slice::<serde_json::Value>(&bytes) {
                            Ok(json) => Ok(json),
                            Err(err) => Err(Error::BodyJsonInvalid(err.to_string())),
                        }
                    }
                },
                Err(err) => {
                    // 检查是否是因为body过大导致的错误
                    let err_str = err.to_string();
                    if err_str.contains("body too large") || err_str.contains("payload too large") {
                        Err(Error::BodyTooLarge(MAX_BODY_SIZE))
                    } else {
                        Err(Error::BodyReadFailed(err_str))
                    }
                },
            };

            if let Err(err) = body {
                return Err(err);
            }

            payload.body = body.ok();
        }

        payload.extract_headers(parts.headers);

        Ok(payload)
    }

    fn extract_headers(&mut self, headers: HeaderMap<HeaderValue>) {
        let header = |n| -> Option<String> { headers.get(n).and_then(|value| value.to_str().ok()).map(String::from) };

        self.user_id = header(header_names::U);
        self.timestamp = header(header_names::T);
        self.nonce = header(header_names::R);
        self.signature = header(header_names::S);
        self.dev_skip = header(header_names::D);
    }

    pub fn get_user_id(&self) -> String {
        self.user_id.clone().unwrap_or_default()
    }

    pub fn get_nonce(&self) -> String {
        self.nonce.clone().unwrap_or_default()
    }

    pub fn get_signature(&self) -> String {
        self.signature.clone().unwrap_or_default()
    }

    pub fn get_dev_skip(&self) -> String {
        self.dev_skip.clone().unwrap_or_default()
    }

    fn validate_signature(&self, key: String) -> Result<(), Error> {
        let payload = self.payload();
        let server_signature = hash::hmac_sha1(&payload, &key);
        let client_signature = self.get_signature();

        if server_signature != client_signature {
            return Err(Error::SignatureVerificationFailed(SignatureDebugInfo {
                payload,
                key: key.clone(),
                server_signature,
                client_signature,
            }));
        }

        Ok(())
    }

    fn validate_headers(&self) -> Result<(), Error> {
        // 检查必需的头部字段
        let mut missing_headers = Vec::new();
        if self.user_id.is_none() {
            missing_headers.push("X-U".to_string());
        }
        if self.timestamp.is_none() {
            missing_headers.push("X-T".to_string());
        }
        if self.nonce.is_none() {
            missing_headers.push("X-R".to_string());
        }
        if self.signature.is_none() {
            missing_headers.push("X-S".to_string());
        }

        if !missing_headers.is_empty() {
            return Err(Error::MissingHeaders(missing_headers));
        }

        // 验证时间戳格式和范围
        let timestamp_str = self.timestamp.as_ref().unwrap();
        let timestamp = timestamp_str.parse::<i64>().map_err(|_| Error::InvalidTimestamp(timestamp_str.clone()))?;

        if timestamp < MAX_TIME_DIFF || (chrono::Utc::now().timestamp() - timestamp).abs() > MAX_TIME_DIFF {
            return Err(Error::TimestampOutOfRange { timestamp, max_diff: MAX_TIME_DIFF });
        }

        // 验证随机数长度
        let nonce_length = self.nonce.as_ref().unwrap().len();
        if nonce_length <= MIN_NONCE_LENGTH || nonce_length >= MAX_NONCE_LENGTH {
            return Err(Error::InvalidNonceLength { length: nonce_length, min: MIN_NONCE_LENGTH, max: MAX_NONCE_LENGTH });
        }

        // 验证签名长度
        let signature_length = self.signature.as_ref().unwrap().len();
        if signature_length != SIGNATURE_LENGTH {
            return Err(Error::InvalidSignatureLength { length: signature_length, expected: SIGNATURE_LENGTH });
        }

        Ok(())
    }

    fn build_header_payload(&self) -> String {
        let mut payload = String::new();
        payload.push_str(self.method.to_uppercase().as_str());
        payload.push_str(",");
        payload.push_str(self.path.as_str());
        payload.push_str(",{");
        if let Some(user_id) = &self.user_id {
            payload.push_str(user_id);
            payload.push_str(",");
        }
        if let Some(timestamp) = &self.timestamp {
            payload.push_str(timestamp);
            payload.push_str(",");
        }
        if let Some(nonce) = &self.nonce {
            payload.push_str(nonce);
        }

        payload.push_str("}");
        payload
    }

    fn append_query_payload(&self, mut payload: String) -> String {
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
        let mut payload = self.append_query_payload(self.build_header_payload());

        if let Some(body) = &self.body {
            payload.push_str(",");
            let body_payload = Self::serialize_json_value(body);
            payload.push_str(body_payload.as_str());
        }

        payload
    }

    fn serialize_array(array: &Vec<serde_json::Value>) -> String {
        let mut payload = String::new();
        let mut array_len = array.len();

        payload.push_str("[");

        for item in array {
            payload.push_str(Self::serialize_json_value(item).as_str());
            array_len -= 1;
            if array_len > 0 {
                payload.push_str(",");
            }
        }
        payload.push_str("]");
        payload
    }

    fn serialize_object(object: &serde_json::Map<String, serde_json::Value>) -> String {
        let mut payload = String::new();

        let mut object_keys: Vec<String> = object.keys().cloned().collect();
        object_keys.sort();
        payload.push_str("{");

        let mut object_size = object_keys.len();
        for key in object_keys {
            let val = object.get(&key).unwrap();
            payload.push_str(key.as_str());
            payload.push_str("=");
            payload.push_str(Self::serialize_json_value(val).as_str());

            object_size -= 1;
            if object_size > 0 {
                payload.push_str(",");
            }
        }

        payload.push_str("}");
        payload
    }

    fn serialize_json_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(i) => i.to_string(),
            serde_json::Value::String(s) => s.to_string(),
            serde_json::Value::Array(array) => Self::serialize_array(array),
            serde_json::Value::Object(object) => Self::serialize_object(object),
        }
    }
}
