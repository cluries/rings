use crate::erx::{Erx, Layouted};
use crate::tools::hash;
use crate::web::api::Out;
use crate::web::middleware::{ApplyKind, Context, Middleware, MiddlewareEventErr, MiddlewareFuture, MiddlewareImpl, Pattern};
use crate::web::{define::HttpMethod, request::clone_request, url::parse_query};
use axum::{
    extract::Request,
    http::{request::Parts, HeaderMap, HeaderValue},
    response::IntoResponse,
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
const MAX_TIME_DEVIATION: i64 = 60 * 5; // 5分钟时间差
const MIN_NONCE_LENGTH: usize = 8;
const MAX_NONCE_LENGTH: usize = 40;
const SIGNATURE_LENGTH: usize = 40;

pub type KeyLoader = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> + Send + Sync>;

pub mod debug_level {
    pub const DISABLE: i8 = 0;
    pub const LOG_ONLY: i8 = 1 << 0;
    pub const RESPONSE_ONLY: i8 = 1 << 1;
    pub const LOG_AND_RESPONSE: i8 = LOG_ONLY | RESPONSE_ONLY;

    pub fn enable_response(v: i8) -> bool {
        v & RESPONSE_ONLY != 0
    }

    pub fn enable_log(v: i8) -> bool {
        v & LOG_ONLY != 0
    }
}

/// Signator 中间件配置
#[derive(Clone)]
pub struct SignatorConfig {
    pub debug_level: i8,

    /// 中间件优先级，数值越大优先级越高
    pub priority: i32,

    /// 密钥加载器（必填）
    pub key_loader: KeyLoader,

    /// Redis 连接 URL（必填）
    pub redis_url: String,

    /// 自定义应用逻辑
    pub apply: Option<Arc<dyn Fn(&Parts) -> bool + Send + Sync>>,
    /// HTTP 方法过滤 - 使用 Arc 减少 clone 成本
    pub methods: Option<Arc<Vec<ApplyKind<HttpMethod>>>>,
    /// 路径匹配模式 - 使用 Arc 减少 clone 成本
    pub patterns: Option<Arc<Vec<ApplyKind<Pattern>>>>,
    /// 随机数生命周期（秒）
    pub nonce_lifetime: i64,
    /// 后门，开发时候方便用
    pub backdoor: Option<String>,
}

impl std::fmt::Debug for SignatorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignatorConfig")
            .field("debug_level", &self.debug_level.to_string())
            .field("priority", &self.priority)
            .field("key_loader", &"KeyLoader")
            .field("redis_url", &self.redis_url)
            .field("apply", &self.apply.as_ref().map(|_| "Some(Fn)"))
            .field("methods", &self.methods)
            .field("patterns", &self.patterns)
            .field("nonce_lifetime", &self.nonce_lifetime)
            .field("backdoor", &self.backdoor)
            .finish()
    }
}

impl SignatorConfig {
    pub fn new(key_loader: KeyLoader, redis_url: String) -> Self {
        Self {
            debug_level: debug_level::DISABLE,
            priority: 0,
            key_loader,
            redis_url,
            apply: None,
            methods: None,
            patterns: None,
            nonce_lifetime: DEFAULT_NONCE_LIFETIME,
            backdoor: None,
        }
    }

    pub fn set_debug_level(mut self, level: i8) -> Self {
        self.debug_level = level;
        self
    }

    /// 设置优先级
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 设置自定义应用逻辑
    pub fn apply<F>(mut self, apply: F) -> Self
    where
        F: Fn(&Parts) -> bool + Send + Sync + 'static,
    {
        self.apply = Some(Arc::new(apply));
        self
    }

    /// 设置 HTTP 方法过滤
    pub fn methods(mut self, methods: Vec<ApplyKind<HttpMethod>>) -> Self {
        self.methods = Some(Arc::new(methods));
        self
    }

    /// 添加包含的 HTTP 方法
    pub fn include_method(mut self, method: HttpMethod) -> Self {
        let mut methods = self.methods.map(|arc| (*arc).clone()).unwrap_or_default();
        methods.push(ApplyKind::Include(method));
        self.methods = Some(Arc::new(methods));
        self
    }

    /// 添加排除的 HTTP 方法
    pub fn exclude_method(mut self, method: HttpMethod) -> Self {
        let mut methods = self.methods.map(|arc| (*arc).clone()).unwrap_or_default();
        methods.push(ApplyKind::Exclude(method));
        self.methods = Some(Arc::new(methods));
        self
    }

    /// 设置路径匹配模式
    pub fn patterns(mut self, patterns: Vec<ApplyKind<Pattern>>) -> Self {
        self.patterns = Some(Arc::new(patterns));
        self
    }

    /// 添加包含的路径模式
    pub fn include_pattern(mut self, pattern: Pattern) -> Self {
        let mut patterns = self.patterns.map(|arc| (*arc).clone()).unwrap_or_default();
        patterns.push(ApplyKind::Include(pattern));
        self.patterns = Some(Arc::new(patterns));
        self
    }

    /// 添加排除的路径模式
    pub fn exclude_pattern(mut self, pattern: Pattern) -> Self {
        let mut patterns = self.patterns.map(|arc| (*arc).clone()).unwrap_or_default();
        patterns.push(ApplyKind::Exclude(pattern));
        self.patterns = Some(Arc::new(patterns));
        self
    }

    /// 添加前缀匹配模式（包含）
    pub fn include_prefix(self, prefix: impl Into<String>, case_sensitive: bool) -> Self {
        self.include_pattern(Pattern::Prefix(prefix.into(), case_sensitive))
    }

    /// 添加前缀匹配模式（排除）
    pub fn exclude_prefix(self, prefix: impl Into<String>, case_sensitive: bool) -> Self {
        self.exclude_pattern(Pattern::Prefix(prefix.into(), case_sensitive))
    }

    /// 添加后缀匹配模式（包含）
    pub fn include_suffix(self, suffix: impl Into<String>, case_sensitive: bool) -> Self {
        self.include_pattern(Pattern::Suffix(suffix.into(), case_sensitive))
    }

    /// 添加后缀匹配模式（排除）
    pub fn exclude_suffix(self, suffix: impl Into<String>, case_sensitive: bool) -> Self {
        self.exclude_pattern(Pattern::Suffix(suffix.into(), case_sensitive))
    }

    /// 添加包含匹配模式（包含）
    pub fn include_contains(self, contains: impl Into<String>, case_sensitive: bool) -> Self {
        self.include_pattern(Pattern::Contains(contains.into(), case_sensitive))
    }

    /// 添加包含匹配模式（排除）
    pub fn exclude_contains(self, contains: impl Into<String>, case_sensitive: bool) -> Self {
        self.exclude_pattern(Pattern::Contains(contains.into(), case_sensitive))
    }

    /// 添加正则表达式匹配模式（包含）
    pub fn include_regex(self, regex: impl Into<String>) -> Self {
        self.include_pattern(Pattern::Regex(regex.into()))
    }

    /// 添加正则表达式匹配模式（排除）
    pub fn exclude_regex(self, regex: impl Into<String>) -> Self {
        self.exclude_pattern(Pattern::Regex(regex.into()))
    }

    /// 设置随机数生命周期（秒）
    pub fn nonce_lifetime(mut self, lifetime: i64) -> Self {
        self.nonce_lifetime = lifetime;
        self
    }

    /// 设置密钥加载器
    pub fn key_loader(mut self, key_loader: KeyLoader) -> Self {
        self.key_loader = key_loader;
        self
    }

    /// 设置后门
    pub fn backdoor(mut self, backdoor: String) -> Self {
        self.backdoor = Some(backdoor);
        self
    }

    /// 设置 Redis 连接 URL
    pub fn redis_url(mut self, redis_url: String) -> Self {
        self.redis_url = redis_url;
        self
    }

    /// 验证配置是否完整和有效
    pub fn validate(&self) -> Result<(), Error> {
        // 验证 nonce_lifetime 的合理性
        if self.nonce_lifetime <= 0 {
            return Err(Error::ConfigError("Nonce lifetime must be positive".to_string()));
        }

        if self.nonce_lifetime > 86400 {
            // 24小时
            return Err(Error::ConfigError("Nonce lifetime should not exceed 24 hours".to_string()));
        }

        // 验证 Redis URL 格式
        if !self.redis_url.starts_with("redis://") && !self.redis_url.starts_with("rediss://") {
            return Err(Error::ConfigError("Redis URL must start with 'redis://' or 'rediss://'".to_string()));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    /// 配置错误
    ConfigError(String),

    /// 请求体过大
    BodyTooLarge(usize),
    /// 请求体JSON格式错误
    BodyJsonInvalid(String),
    /// 请求体读取失败
    BodyReadFailed(String),

    /// 缺少必需的签名头部
    MissingHeaders(Vec<String>),
    /// 用户ID格式错误
    InvalidUserId(String),
    /// 时间戳格式错误
    InvalidTimestamp(String),
    /// 时间戳超出允许范围
    TimestampOutOfRange {
        timestamp: i64,
        max_diff: i64,
    },
    /// 随机数长度不符合要求
    InvalidNonceLength {
        length: usize,
        min: usize,
        max: usize,
    },
    /// 签名长度不符合要求
    InvalidSignatureLength {
        length: usize,
        expected: usize,
    },

    /// Redis连接失败
    RedisConnectionFailed(String),
    /// 随机数重复使用
    NonceReused(String),
    /// Redis操作失败
    RedisOperationFailed(String),

    /// 密钥加载失败
    KeyLoadingFailed(Erx),
    /// 签名验证失败
    SignatureVerificationFailed(SignatureDebugInfo),

    InternalError(String),
}

impl Error {
    fn make_out(&self, debug: bool) -> Out<()> {
        let message = self.to_string();
        let c = Layouted::middleware("SIGN", "EROR");

        let mut out = Out::new(c, Some(message), None);

        if debug {
            if let Error::SignatureVerificationFailed(debug) = self {
                out.add_debug_items(debug.make_map(false));
            }
        }

        out
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // 配置相关错误
            Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),

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
            Error::SignatureVerificationFailed(_debug) => write!(f, "Signature verification failed"),

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

/// Signator
pub struct Signator {
    config: Arc<SignatorConfig>,
    redis_client: redis::Client,
}

impl Signator {
    pub fn new(config: SignatorConfig) -> Result<Self, Error> {
        // 验证配置
        config.validate()?;

        let redis_client = redis::Client::open(config.redis_url.as_str()).map_err(|err| {
            tracing::error!("Failed to create Redis client for URL {}: {}", config.redis_url, err);
            Error::ConfigError(format!("Invalid Redis URL '{}': {}", config.redis_url, err))
        })?;

        let config = Arc::new(config);

        Ok(Signator { config, redis_client })
    }

    /// Authenticates an incoming HTTP request by validating its signature and preventing replay attacks.
    ///
    /// This method performs comprehensive request authentication through the following steps:
    ///
    /// 1. **Request Cloning**: Creates a clone of the request to preserve the original for downstream processing
    /// 2. **Payload Extraction**: Extracts authentication headers (user ID, timestamp, nonce, signature) and request body
    /// 3. **Header Validation**: Ensures all required headers are present and properly formatted
    /// 4. **Key Loading**: Retrieves the user's signing key using the configured key loader
    /// 5. **Signature Verification**: Validates the request signature against the expected signature computed from the payload
    /// 6. **Backdoor Check**: If signature verification fails, checks for development backdoor bypass
    /// 7. **Nonce Validation**: Prevents replay attacks by ensuring the nonce hasn't been used recently
    /// 8. **Context Injection**: Injects user context into the request for downstream middleware and handlers
    ///
    /// The authentication process protects against:
    /// - **Tampering**: Through HMAC-SHA1 signature verification
    /// - **Replay attacks**: Through nonce validation with Redis-based deduplication
    /// - **Time-based attacks**: Through timestamp validation within acceptable time windows
    ///
    /// Returns the authenticated request with user context injected, or an authentication error.
    //
    pub async fn authenticate(&self, request: axum::extract::Request) -> Result<axum::extract::Request, Error> {
        let (payload_request, mut request) = clone_request(request).await;

        let payload = Payload::from_request(payload_request).await?;
        payload.validate_headers()?;

        let key = (self.config.key_loader)(payload.get_user_id()).await.map_err(Error::KeyLoadingFailed)?;

        if let Err(invalid) = payload.validate_signature(key) {
            match self.config.backdoor.as_ref() {
                None => {
                    return Err(invalid);
                },
                Some(backdoor) => {
                    if !backdoor.eq(&payload.get_dev_skip()) {
                        return Err(invalid);
                    }
                },
            }
        }

        self.validate_nonce(&payload).await?;

        request
            .extensions_mut()
            .get_or_insert_default::<crate::web::context::Context>()
            .set_ident(payload.get_user_id(), self.name().to_string());

        Ok(request)
    }

    /// Validates that a nonce (number used once) hasn't been used recently to prevent replay attacks.
    ///
    /// This method performs the following operations:
    /// 1. Connects to Redis and checks if the nonce exists in a sorted set for the user
    /// 2. If the nonce exists and was used within the configured lifetime, returns an error
    /// 3. If the nonce is valid, adds it to Redis with current timestamp as score
    /// 4. Cleans up expired nonces from the sorted set to prevent memory bloat
    /// 5. Sets expiration on the Redis key to automatically clean up user data
    ///
    /// The nonce validation uses Redis sorted sets where:
    /// - Key: "XR:{user_id}"
    /// - Member: nonce value
    /// - Score: timestamp when nonce was used
    ///
    /// This ensures each nonce can only be used once within the configured time window,
    /// providing protection against replay attacks while automatically cleaning up old data.
    async fn validate_nonce(&self, payload: &Payload) -> Result<(), Error> {
        let mut redis_conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        let redis_key = format!("XR:{}", payload.get_user_id());
        let nonce_value = payload.get_nonce();

        let existing_score: Option<i64> = redis_conn.zscore(redis_key.as_str(), &nonce_value).await?;
        let last_used_timestamp = existing_score.unwrap_or(0);
        let current_timestamp: i64 = chrono::Local::now().timestamp();

        if (current_timestamp - last_used_timestamp).abs() < self.config.nonce_lifetime {
            return Err(Error::NonceReused(payload.get_nonce()));
        }

        // 清理过期的 nonce 并添加新的
        let mut pipeline = redis::pipe();
        pipeline.zadd(redis_key.as_str(), &nonce_value, current_timestamp);
        pipeline.zrembyscore(redis_key.as_str(), "-inf", current_timestamp - self.config.nonce_lifetime);
        pipeline.expire(redis_key.as_str(), self.config.nonce_lifetime);
        let _result = pipeline.query_async::<Vec<i64>>(&mut redis_conn).await?;

        Ok(())
    }
}

impl Clone for Signator {
    fn clone(&self) -> Self {
        Signator { config: self.config.clone(), redis_client: self.redis_client.clone() }
    }
}

impl std::fmt::Debug for Signator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signator").field("config", &self.config).field("redis_client", &self.redis_client).finish()
    }
}

impl Middleware for Signator {
    fn name(&self) -> &'static str {
        Signator::middleware_name()
    }

    fn on_request(&self, context: Context, request: Request) -> MiddlewareImpl<MiddlewareFuture<Request>, MiddlewareEventErr<Request>> {
        let signator = self.clone();

        // Creates an asynchronous future that handles the authentication process for incoming requests.
        //
        // This future performs the complete authentication workflow:
        // 1. **Request Authentication**: Calls the `authenticate` method to validate the request
        // 2. **Success Path**: If authentication succeeds, returns the authenticated request with injected user context
        // 3. **Error Path**: If authentication fails:
        //    - Converts the authentication error into a user-friendly error message
        //    - Creates an `Erx` error object for internal error tracking
        //    - Converts the error into an `Out<()>` response format
        //    - Modifies the middleware context to abort the request with the error response
        //    - Returns the error context and `Erx` object for upstream error handling
        //
        // The future ensures that authentication failures are properly handled and converted
        // into appropriate HTTP responses while maintaining the middleware chain's error handling contract.
        //
        // Returns:
        // - `Ok((context, request))`: On successful authentication with user context injected
        // - `Err((context, erx))`: On authentication failure with abort response set in context
        //

        let future = Box::pin(async move {
            match signator.authenticate(request).await {
                Ok(req) => Ok((context, req)),
                Err(error) => {
                    if debug_level::enable_log(signator.config.debug_level) {
                        if let Error::SignatureVerificationFailed(d) = &error {
                            tracing::error!("Signature verification failed: {:#?}", d.make_map(false));
                        }
                    }

                    let message = &error.to_string();
                    let erx = Erx::new(&message);
                    let out: Out<()> = error.make_out(debug_level::enable_response(signator.config.debug_level));

                    let mut context = context;
                    context.make_abort_with_response(Signator::middleware_name(), message, out.into_response());
                    Err((context, None, Some(erx)))
                },
            }
        });

        MiddlewareImpl::Implemented(future)
    }

    /// 可选：中间件优先级，数值越大优先级越高
    fn priority(&self) -> i32 {
        self.config.priority
    }

    /// 可选：判断中间件是否应该处理这个请求
    /// 优先级 apply > methods > patterns
    /// - 如果 apply 返回不为 None，直接使用 apply 的返回值判定
    fn apply(&self, parts: &Parts) -> Option<bool> {
        self.config.apply.as_ref().map(|f| f(parts))
    }

    /// 可选：HTTP 方法过滤
    fn methods(&self) -> Option<Vec<ApplyKind<HttpMethod>>> {
        self.config.methods.as_ref().map(|arc| (**arc).clone())
    }

    /// 可选：路径匹配模式
    fn patterns(&self) -> Option<Vec<ApplyKind<Pattern>>> {
        self.config.patterns.as_ref().map(|arc| (**arc).clone())
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

impl SignatureDebugInfo {
    pub fn make_map(&self, include_server_key: bool) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::from([
            ("payload".to_string(), self.payload.clone()),
            ("server".to_string(), self.server_signature.clone()),
            ("client".to_string(), self.client_signature.clone()),
        ]);

        if include_server_key {
            m.insert("key".to_string(), self.key.clone());
        }

        m
    }
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

    /// Extracts and parses authentication payload from an incoming HTTP request.
    ///
    /// This method performs the following operations:
    /// 1. **Request Decomposition**: Separates the request into parts (headers, URI, method) and body
    /// 2. **Basic Information Extraction**: Extracts HTTP method, path, and query parameters
    /// 3. **Conditional Body Reading**: Reads and parses JSON body for methods that typically carry payloads
    /// 4. **Size Validation**: Ensures request body doesn't exceed the maximum allowed size (32MB)
    /// 5. **JSON Parsing**: Attempts to parse the body as JSON, handling empty bodies gracefully
    /// 6. **Header Extraction**: Extracts authentication headers (user ID, timestamp, nonce, signature)
    ///
    /// The method handles different HTTP methods appropriately:
    /// - For GET/HEAD requests: Only processes headers and query parameters
    /// - For POST/PUT/DELETE/PATCH/OPTIONS requests: Additionally reads and parses the request body
    ///
    /// Error handling covers:
    /// - Body size exceeding limits
    /// - Invalid JSON format in request body
    /// - General body reading failures
    ///
    /// Returns a complete Payload struct containing all necessary information for signature verification.
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

    /// 从 HTTP 请求头中提取认证相关的头部信息
    ///
    /// 该方法从请求头中提取以下认证字段：
    /// - X-U: 用户ID
    /// - X-T: 时间戳
    /// - X-R: 随机数(nonce)
    /// - X-S: 签名
    /// - X-DEVELOPMENT-SKIP: 开发环境跳过验证的后门参数
    ///
    /// 所有提取的值都会被转换为 String 类型存储在 Payload 结构体中
    ///
    /// Extracts authentication headers from HTTP request headers.
    ///
    /// This method extracts the following authentication fields from request headers:
    /// - X-U: User ID
    /// - X-T: Timestamp
    /// - X-R: Nonce (number used once)
    /// - X-S: Signature
    /// - X-DEVELOPMENT-SKIP: Development backdoor parameter for skipping verification
    ///
    /// All extracted values are converted to String type and stored in the Payload struct.
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
        let server_signature = hash::hmac_sha1(&payload, &key).map_err(|s| Error::InternalError(s))?;

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

    /// Validates the presence and format of required authentication headers.
    ///
    /// This method performs comprehensive validation of authentication headers extracted from the HTTP request:
    ///
    /// 1. **Required Headers Check**: Ensures all mandatory authentication headers are present:
    ///    - X-U (User ID): Identifies the requesting user
    ///    - X-T (Timestamp): Request creation time for preventing stale requests
    ///    - X-R (Nonce): Random value for preventing replay attacks
    ///    - X-S (Signature): HMAC signature for request integrity verification
    ///
    /// 2. **Timestamp Validation**: Validates the timestamp format and ensures it falls within acceptable time bounds:
    ///    - Parses the timestamp string to i64 format
    ///    - Checks that the timestamp is not too old or too far in the future (±5 minutes)
    ///    - Prevents both stale request attacks and clock skew issues
    ///
    /// 3. **Nonce Format Validation**: Ensures the nonce meets security requirements:
    ///    - Validates length is between 8-40 characters for sufficient entropy
    ///    - Prevents both weak nonces (too short) and potential DoS attacks (too long)
    ///
    /// 4. **Signature Format Validation**: Verifies the signature format:
    ///    - Ensures signature is exactly 40 characters (SHA1 hex digest length)
    ///    - Prevents malformed signature attacks
    ///
    /// Returns an error if any validation fails, providing specific details about what went wrong.
    /// This validation layer provides the first line of defense against malformed or malicious requests.
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

        if timestamp < MAX_TIME_DEVIATION || (chrono::Utc::now().timestamp() - timestamp).abs() > MAX_TIME_DEVIATION {
            return Err(Error::TimestampOutOfRange { timestamp, max_diff: MAX_TIME_DEVIATION });
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

    /// Constructs the complete authentication payload string used for signature generation.
    ///
    /// This method builds the standardized payload format that combines all request components
    /// into a single string for HMAC signature generation. The payload structure ensures
    /// consistent signature verification across client and server implementations.
    ///
    /// The payload construction follows this format:
    /// `{METHOD},{PATH},{user_id,timestamp,nonce}[,{query_params}][,{body_json}]`
    ///
    /// Components included:
    /// 1. **Header Payload**: HTTP method, path, and authentication headers
    /// 2. **Query Parameters**: Sorted key-value pairs (if present)
    /// 3. **Request Body**: Serialized JSON content (if present)
    ///
    /// Example payloads:
    ///
    /// **GET request with query parameters:**
    /// ```
    /// GET,/api/users,{user123,1640995200,abc123def456},{limit=10,sort=name}
    /// ```
    ///
    /// **POST request with JSON body:**
    /// ```
    /// POST,/api/users,{user123,1640995200,abc123def456},{name=John,age=30}
    /// ```
    ///
    /// **Simple GET request:**
    /// ```
    /// GET,/api/status,{user123,1640995200,abc123def456}
    /// ```
    ///
    /// This standardized format ensures that identical requests will always generate
    /// the same payload string, enabling consistent signature verification and
    /// preventing signature bypass attacks through payload manipulation.
    ///
    ///
    fn payload(&self) -> String {
        let mut payload = self.append_query_payload(self.build_header_payload());

        if let Some(body) = &self.body {
            payload.push_str(",");
            let body_payload = Self::serialize_json_value(body);
            payload.push_str(body_payload.as_str());
        }

        payload
    }

    ///
    /// Builds the header portion of the authentication payload string.
    ///
    /// This method constructs the standardized header payload format used for signature generation:
    /// `{METHOD},{PATH},{user_id,timestamp,nonce}`
    ///
    /// The format includes:
    /// - HTTP method in uppercase (e.g., "GET", "POST")
    /// - Request path (e.g., "/api/users")
    /// - Authentication headers enclosed in braces, comma-separated:
    ///   - User ID (X-U header value)
    ///   - Timestamp (X-T header value)
    ///   - Nonce (X-R header value)
    ///
    /// Example output: `POST,/api/users,{user123,1640995200,abc123def456}`
    ///
    /// This standardized format ensures consistent signature generation across
    /// client and server implementations.
    ///
    ///
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

    ///
    /// Appends query parameters to the authentication payload string.
    ///
    /// This method extends the base payload with query parameters in a standardized format
    /// for consistent signature generation. The query parameters are:
    ///
    /// 1. **Sorted by Key**: All query parameter keys are sorted alphabetically to ensure
    ///    consistent ordering regardless of the original request parameter order
    /// 2. **Formatted as Key-Value Pairs**: Each parameter is formatted as `key=value`
    /// 3. **Comma-Separated**: Multiple parameters are separated by commas
    /// 4. **Enclosed in Braces**: The entire query section is wrapped in `{}`
    ///
    /// Format: `{key1=value1,key2=value2,key3=value3}`
    ///
    /// If no query parameters exist, the original payload is returned unchanged.
    ///
    /// This standardization ensures that requests with identical query parameters
    /// but different parameter ordering will generate the same signature.
    ///
    /// Example:
    /// - Input payload: `POST,/api/users,{user123,1640995200,abc123def456}`
    /// - Query params: `?sort=name&limit=10`
    /// - Output: `POST,/api/users,{user123,1640995200,abc123def456},{limit=10,sort=name}`
    ///
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

    /// Serializes a JSON array into a standardized string format for signature generation.
    ///
    /// This method converts a JSON array into a deterministic string representation
    /// that ensures consistent signature generation across different implementations.
    ///
    /// The serialization format:
    /// - Arrays are enclosed in square brackets: `[...]`
    /// - Elements are serialized recursively using `serialize_json_value`
    /// - Elements are separated by commas with no spaces
    /// - Empty arrays are represented as `[]`
    ///
    /// Examples:
    /// - `[1, 2, 3]` becomes `[1,2,3]`
    /// - `["a", "b"]` becomes `[a,b]`
    /// - `[{"key": "value"}]` becomes `[{key=value}]`
    /// - `[]` becomes `[]`
    ///
    /// This standardized format ensures that arrays with identical content
    /// will always produce the same string representation, which is crucial
    /// for consistent signature verification.
    ///
    ///
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

    /// Serializes a JSON object into a standardized string format for signature generation.
    ///
    /// This method converts a JSON object into a deterministic string representation
    /// that ensures consistent signature generation across different implementations.
    ///
    /// The serialization process:
    /// 1. **Key Sorting**: All object keys are sorted alphabetically to ensure
    ///    consistent ordering regardless of the original JSON key order
    /// 2. **Key-Value Formatting**: Each property is formatted as `key=value`
    /// 3. **Recursive Serialization**: Values are serialized recursively using `serialize_json_value`
    /// 4. **Comma Separation**: Multiple properties are separated by commas with no spaces
    /// 5. **Brace Enclosure**: The entire object is wrapped in curly braces `{}`
    ///
    /// Examples:
    /// - `{"name": "John", "age": 30}` becomes `{age=30,name=John}`
    /// - `{"nested": {"key": "value"}}` becomes `{nested={key=value}}`
    /// - `{}` becomes `{}`
    ///
    /// This standardized format ensures that objects with identical content
    /// but different key ordering will always produce the same string representation,
    /// which is crucial for consistent signature verification across client and server.
    ///
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

    /// Serializes a JSON value into a standardized string format for signature generation.
    ///
    /// This method provides a deterministic serialization of JSON values that ensures
    /// consistent signature generation across different implementations and platforms.
    ///
    /// The serialization handles all JSON value types:
    /// - **Null**: Serialized as the string "null"
    /// - **Boolean**: Serialized as "true" or "false"
    /// - **Number**: Serialized using the number's string representation
    /// - **String**: Serialized as the raw string value (without quotes)
    /// - **Array**: Recursively serialized using `serialize_array`
    /// - **Object**: Recursively serialized using `serialize_object`
    ///
    /// This standardized approach ensures that:
    /// 1. Identical JSON structures always produce identical string representations
    /// 2. The serialization is deterministic and reproducible
    /// 3. Complex nested structures are handled consistently
    /// 4. The output is suitable for cryptographic signature generation
    ///
    /// Examples:
    /// - `null` → `"null"`
    /// - `true` → `"true"`
    /// - `42` → `"42"`
    /// - `"hello"` → `"hello"`
    /// - `[1, 2]` → `"[1,2]"`
    /// - `{"key": "value"}` → `"{key=value}"`
    ///
    ///
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::define::HttpMethod;
    use crate::web::middleware::{ApplyKind, Pattern};

    use crate::web::middleware::ApplyTrait;

    #[test]
    fn test_signator_config_new() {
        let key_loader = Arc::new(|_: String| -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> {
            Box::pin(async { Ok("test_key".to_string()) })
        });

        let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string());
        assert_eq!(config.priority, 0);
        assert_eq!(config.nonce_lifetime, DEFAULT_NONCE_LIFETIME);
        assert!(config.apply.is_none());
        assert!(config.methods.is_none());
        assert!(config.patterns.is_none());
        assert!(config.backdoor.is_none());
        assert_eq!(config.redis_url, "redis://localhost:6379");
    }

    #[test]
    fn test_signator_config_builder() {
        let key_loader = Arc::new(|_: String| -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> {
            Box::pin(async { Ok("test_key".to_string()) })
        });

        let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
            .priority(100)
            .nonce_lifetime(600)
            .include_method(HttpMethod::POST)
            .include_prefix("/api/".to_string(), true)
            .exclude_suffix(".html".to_string(), false);

        assert_eq!(config.priority, 100);
        assert_eq!(config.nonce_lifetime, 600);
        assert!(config.methods.is_some());
        assert!(config.patterns.is_some());

        let methods = config.methods.unwrap();
        assert_eq!(methods.len(), 1);

        let patterns = config.patterns.unwrap();
        assert_eq!(patterns.len(), 2);
    }

    // #[test]
    // fn test_signator_config_apply() {
    //     let config = SignatorConfig::new()
    //         .apply(|parts| {
    //             parts.uri.path().starts_with("/admin/")
    //         });

    //     assert!(config.apply.is_some());

    //     // 创建一个模拟的 Parts 来测试 apply 函数
    //     let uri: Uri = "/admin/users".parse().unwrap();
    //     let method = Method::GET;
    //     let mut parts = Parts::default();
    //     parts.uri = uri;
    //     parts.method = method;

    //     let apply_fn = config.apply.unwrap();
    //     assert!(apply_fn(&parts));

    //     // 测试不匹配的路径
    //     let uri: Uri = "/public/info".parse().unwrap();
    //     let mut parts = Parts::default();
    //     parts.uri = uri;
    //     parts.method = Method::GET;

    //     assert!(!apply_fn(&parts));
    // }

    #[test]
    fn test_signator_config_debug() {
        let key_loader = Arc::new(|_: String| -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> {
            Box::pin(async { Ok("test_key".to_string()) })
        });

        let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string()).priority(50).apply(|_| true);

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SignatorConfig"));
        assert!(debug_str.contains("priority: 50"));
        assert!(debug_str.contains("Some(Fn)"));
    }

    #[test]
    fn test_pattern_matching() {
        // 测试前缀匹配
        let prefix_pattern = &Pattern::Prefix("/api/".to_string(), true);
        assert!(prefix_pattern.apply("/api/users"));
        assert!(!prefix_pattern.apply("/public/info"));

        // 测试后缀匹配
        let suffix_pattern = &Pattern::Suffix(".json".to_string(), true);
        assert!(suffix_pattern.apply("/api/users.json"));
        assert!(!suffix_pattern.apply("/api/users.html"));

        // 测试包含匹配
        let contains_pattern = &Pattern::Contains("admin".to_string(), true);
        assert!(contains_pattern.apply("/admin/users"));
        assert!(contains_pattern.apply("/api/admin/settings"));
        assert!(!contains_pattern.apply("/public/info"));

        // 测试正则表达式匹配
        let regex_pattern = &Pattern::Regex(r"^/api/v\d+/.*$".to_string());
        assert!(regex_pattern.apply("/api/v1/users"));
        assert!(regex_pattern.apply("/api/v2/posts"));
        assert!(!regex_pattern.apply("/api/users"));
    }

    #[test]
    fn test_apply_kind() {
        let method = HttpMethod::POST;

        let include_kind = ApplyKind::Include(method.clone());
        assert!(include_kind.apply("POST"));
        assert!(!include_kind.apply("GET"));

        let exclude_kind = ApplyKind::Exclude(method);
        assert!(!exclude_kind.apply("POST"));
        assert!(exclude_kind.apply("GET"));
    }

    #[test]
    fn test_config_validation() {
        let key_loader = Arc::new(|_: String| -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> {
            Box::pin(async { Ok("test_key".to_string()) })
        });

        // 测试无效的 nonce_lifetime
        let config = SignatorConfig::new(key_loader.clone(), "redis://localhost:6379".to_string()).nonce_lifetime(-1);
        assert!(matches!(config.validate(), Err(Error::ConfigError(_))));

        // 测试无效的 Redis URL
        let config = SignatorConfig::new(key_loader.clone(), "invalid://localhost:6379".to_string());
        assert!(matches!(config.validate(), Err(Error::ConfigError(_))));

        // 测试有效的配置
        let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string());
        assert!(config.validate().is_ok());
    }
}
