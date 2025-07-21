use super::*;
use crate::erx::{Erx, Layouted, LayoutedC};
use crate::tools::hash;
use crate::web::api::Out;
use crate::web::request::clone_request;
use crate::web::url::parse_query;
use redis::AsyncCommands;

// 移除未使用的导入
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::fmt;

/// 签名验证相关的错误类型
#[derive(Debug, Clone)]
pub enum SignatorError {
    /// 载荷解析错误
    PayloadParse(String),
    /// 签名格式错误
    SignatureFormat(String),
    /// 密钥加载错误
    KeyLoad(String),
    /// 签名验证失败
    SignatureInvalid { error: String, debug: Debug },
    /// 随机数重复使用
    NonceReplay(String),
    /// Redis 连接错误
    RedisConnection(String),
    /// 时间戳验证失败
    TimestampInvalid(String),
}

impl fmt::Display for SignatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignatorError::PayloadParse(msg) => write!(f, "Payload parse error: {}", msg),
            SignatorError::SignatureFormat(msg) => write!(f, "Signature format error: {}", msg),
            SignatorError::KeyLoad(msg) => write!(f, "Key load error: {}", msg),
            SignatorError::SignatureInvalid { error, .. } => write!(f, "Signature invalid: {}", error),
            SignatorError::NonceReplay(msg) => write!(f, "Nonce replay: {}", msg),
            SignatorError::RedisConnection(msg) => write!(f, "Redis connection error: {}", msg),
            SignatorError::TimestampInvalid(msg) => write!(f, "Timestamp invalid: {}", msg),
        }
    }
}

impl std::error::Error for SignatorError {}

/// 向后兼容的调试信息结构
#[derive(Default, Debug, Clone, Serialize)]
pub struct Debug {
    payload: String,
    key: String,
    server: String,
    client: String,
}

impl SignatorError {
    /// 转换为 HTTP 响应
    pub fn into_response(self) -> Response {
        let (code_detail, message, data) = match &self {
            SignatorError::PayloadParse(_) => (PAYL_STR, self.to_string(), None),
            SignatorError::SignatureFormat(_) => (FRMT_STR, self.to_string(), None),
            SignatorError::KeyLoad(_) => (LOAD_STR, self.to_string(), None),
            SignatorError::SignatureInvalid { error, debug } => {
                (INVD_STR, error.clone(), Some(serde_json::to_value(debug).unwrap_or_default()))
            },
            SignatorError::NonceReplay(_) => (INVD_STR, self.to_string(), None),
            SignatorError::RedisConnection(_) => (INVD_STR, self.to_string(), None),
            SignatorError::TimestampInvalid(_) => (FRMT_STR, self.to_string(), None),
        };

        Out::<serde_json::Value> {
            code: make_code(code_detail).into(),
            message: Some(message),
            data,
            debug: None,
            profile: None,
        }.into_response()
    }
}

static DEFAULT_RAND_LIFE: i64 = 300;

static SIGN_STR: &str = "SIGN";
static PAYL_STR: &str = "PAYL";
static FRMT_STR: &str = "FRMT";
static LOAD_STR: &str = "LOAD";
static INVD_STR: &str = "INVD";

fn make_code(detail: &str) -> LayoutedC {
    Layouted::middleware(SIGN_STR, detail)
}

/// KeyLoader
pub type KeyLoader = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output=Result<String, Erx>> + Send>> + Send + Sync>;

/// 签名验证中间件
pub struct SignatorMiddleware {
    rear: String, // 后门，开发时候方便用
    excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>,
    nonce_lifetime: i64,
    key_loader: KeyLoader,
    redis_client: redis::Client,
}

impl Clone for SignatorMiddleware {
    fn clone(&self) -> Self {
        SignatorMiddleware {
            rear: self.rear.clone(),
            excludes: self.excludes.clone(),
            nonce_lifetime: self.nonce_lifetime,
            key_loader: Arc::clone(&self.key_loader),
            redis_client: self.redis_client.clone(),
        }
    }
}

impl SignatorMiddleware {
    pub fn new(redis_url: &str, key_loader: KeyLoader) -> Self {
        Self::with_rear(redis_url, key_loader, String::default())
    }

    pub fn with_rear(redis_url: &str, key_loader: KeyLoader, rear: String) -> Self {
        SignatorMiddleware {
            rear,
            excludes: vec![],
            nonce_lifetime: DEFAULT_RAND_LIFE,
            key_loader,
            redis_client: redis::Client::open(redis_url).unwrap_or_else(|err| {
                tracing::error!("{} {}", redis_url, err);
                panic!("failed to connect to redis: {}", err);
            }),
        }
    }

    pub fn add_exclude(&mut self, exclude: fn(parts: &axum::http::request::Parts) -> bool) -> &mut Self {
        self.excludes.push(exclude);
        self
    }

    pub fn with_excludes(mut self, excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>) -> Self {
        for exclude in excludes {
            self.excludes.push(exclude);
        }
        self
    }

    pub fn with_nonce_lifetime(mut self, lifetime: i64) -> Self {
        self.nonce_lifetime = lifetime;
        self
    }

    fn should_exclude(&self, parts: &Parts) -> bool {
        self.excludes.iter().any(|exclude| exclude(parts))
    }

    async fn validate_signature(&self, request: Request) -> Result<Request, Response> {
        let (payload_request, mut request) = clone_request(request).await;

        let payload = Payload::from_request(payload_request).await
            .map_err(|e| e.into_response())?;
        
        payload.guard()
            .map_err(|e| e.into_response())?;

        let loader = Arc::clone(&self.key_loader);
        let key = loader(payload.val_or_default_u()).await
            .map_err(|e| SignatorError::KeyLoad(e.message_string()).into_response())?;

        if let Err((error, debug)) = payload.valid(key) {
            if self.rear.is_empty() || Some(self.rear.as_str()) != payload.development_skip() {
                return Err(SignatorError::SignatureInvalid { 
                    error, 
                    debug: Debug {
                        payload: debug.payload,
                        key: debug.key,
                        server: debug.server_signature,
                        client: debug.client_signature,
                    }
                }.into_response());
            }
        }

        self.rand_guard(payload.user_id(), payload.nonce())
            .await
            .map_err(|e| e.into_response())?;

        use crate::web::context::Context;
        let context = Context::new(payload.user_id().to_string());
        request.extensions_mut().insert(context);

        Ok(request)
    }

    async fn rand_guard(&self, user_id: &str, nonce: &str) -> Result<(), SignatorError> {
        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await
            .map_err(|e| SignatorError::RedisConnection(e.to_string()))?;

        let name = format!("XR:{}", user_id);
        let score: Option<i64> = conn.zscore(name.as_str(), nonce).await
            .map_err(|e| SignatorError::RedisConnection(e.to_string()))?;
        
        let score = score.unwrap_or(0);
        let current: i64 = chrono::Local::now().timestamp();

        if (current - score).abs() < self.nonce_lifetime {
            return Err(SignatorError::NonceReplay("duplicate rand value".to_string()));
        }

        let mut pipe = redis::pipe();
        pipe.zadd(name.as_str(), nonce, current);
        pipe.zrembyscore(name.as_str(), "-inf", current - self.nonce_lifetime);
        pipe.expire(name.as_str(), self.nonce_lifetime);
        
        let _r = pipe.query_async::<Vec<i64>>(&mut conn).await
            .map_err(|e| SignatorError::RedisConnection(e.to_string()))?;

        Ok(())
    }
}

impl Middleware for SignatorMiddleware {
    fn focus(&self, parts: &Parts) -> bool {
        // 如果在排除列表中，则不处理
        !self.should_exclude(parts)
    }

    fn priority(&self) -> i32 {
        85 // 高优先级，在认证之前执行
    }

    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            self.validate_signature(request).await
        })
    }

    fn name(&self) -> &'static str {
        "SignatorMiddleware"
    }

    fn path_pattern(&self) -> Option<&str> {
        Some("/api/*") // 默认只对 API 路径进行签名验证
    }
}

// 签名相关的常量
mod constants {
    pub const XU: &str = "X-U";
    pub const XT: &str = "X-T";
    pub const XR: &str = "X-R";
    pub const XS: &str = "X-S";
    pub const DS: &str = "X-DEVELOPMENT-SKIP";
    
    pub const TIMESTAMP_MAX_DIFF: i64 = 60 * 5; // 5分钟
    pub const SIGNATURE_LENGTH: usize = 40;
    pub const MIN_NONCE_LENGTH: usize = 8;
    pub const MAX_NONCE_LENGTH: usize = 40;
    pub const BODY_SIZE_LIMIT: usize = 1024 * 1024 * 32; // 32MB
}

use constants::*;

/// 签名头部信息
#[derive(Debug, Clone)]
pub struct SignatureHeaders {
    pub user_id: String,
    pub timestamp: i64,
    pub nonce: String,
    pub signature: String,
    pub development_skip: Option<String>,
}

impl SignatureHeaders {
    /// 从请求头中提取签名信息
    fn from_headers(headers: &axum::http::HeaderMap) -> Option<Self> {
        let header = |name: &str| -> Option<String> {
            headers.get(name)
                .and_then(|value| value.to_str().ok())
                .map(String::from)
        };

        let user_id = header(XU)?;
        let timestamp_str = header(XT)?;
        let nonce = header(XR)?;
        let signature = header(XS)?;
        let development_skip = header(DS);

        let timestamp = timestamp_str.parse::<i64>().ok()?;

        Some(SignatureHeaders {
            user_id,
            timestamp,
            nonce,
            signature,
            development_skip,
        })
    }

    /// 验证签名头部格式
    fn validate(&self) -> Result<(), SignatorError> {
        // 验证时间戳
        let current_time = chrono::Utc::now().timestamp();
        if self.timestamp < TIMESTAMP_MAX_DIFF || (current_time - self.timestamp).abs() > TIMESTAMP_MAX_DIFF {
            return Err(SignatorError::TimestampInvalid("the time difference is too large".to_string()));
        }

        // 验证随机数长度
        if self.nonce.len() <= MIN_NONCE_LENGTH || self.nonce.len() >= MAX_NONCE_LENGTH {
            return Err(SignatorError::SignatureFormat("random string length invalid".to_string()));
        }

        // 验证签名长度
        if self.signature.len() != SIGNATURE_LENGTH {
            return Err(SignatorError::SignatureFormat("invalid signature data in header".to_string()));
        }

        Ok(())
    }

    /// 获取用于签名的头部字符串
    fn to_signature_string(&self) -> String {
        format!("{},{},{}", self.user_id, self.timestamp, self.nonce)
    }
}

/// 请求载荷信息
#[derive(Debug)]
pub struct RequestPayload {
    pub method: String,
    pub path: String,
    pub queries: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

impl RequestPayload {
    /// 从请求中提取载荷信息
    async fn from_request_parts(
        method: &str,
        path: &str,
        query: Option<&str>,
        body: axum::body::Body,
    ) -> Result<Self, SignatorError> {
        let queries = parse_query(query.unwrap_or_default());
        let body = Self::parse_body(method, body).await?;

        Ok(RequestPayload {
            method: method.to_uppercase(),
            path: path.to_string(),
            queries,
            body,
        })
    }

    /// 解析请求体
    async fn parse_body(method: &str, body: axum::body::Body) -> Result<Option<serde_json::Value>, SignatorError> {
        use crate::web::define::HttpMethod;

        let needs_body = HttpMethod::POST.is(method)
            || HttpMethod::PUT.is(method)
            || HttpMethod::DELETE.is(method)
            || HttpMethod::OPTIONS.is(method)
            || HttpMethod::PATCH.is(method);

        if !needs_body {
            return Ok(None);
        }

        let bytes = axum::body::to_bytes(body, BODY_SIZE_LIMIT).await
            .map_err(|e| SignatorError::PayloadParse(format!("Failed to read body: {}", e)))?;

        if bytes.is_empty() {
            return Ok(Some(serde_json::Value::default()));
        }

        let json = serde_json::from_slice::<serde_json::Value>(&bytes)
            .map_err(|e| SignatorError::PayloadParse(format!("Invalid JSON body: {}", e)))?;

        Ok(Some(json))
    }

    /// 生成用于签名的载荷字符串
    fn to_signature_string(&self, headers: &SignatureHeaders) -> String {
        let mut payload = format!("{},{},{{{}}}",
            self.method,
            self.path,
            headers.to_signature_string()
        );

        // 添加查询参数
        if !self.queries.is_empty() {
            let mut query_keys: Vec<String> = self.queries.keys().cloned().collect();
            query_keys.sort();

            payload.push_str(",{");
            for (i, key) in query_keys.iter().enumerate() {
                if i > 0 {
                    payload.push(',');
                }
                payload.push_str(&format!("{}={}", key, self.queries.get(key).unwrap()));
            }
            payload.push('}');
        }

        // 添加请求体
        if let Some(body) = &self.body {
            payload.push(',');
            payload.push_str(&JsonFormatter::format(body));
        }

        payload
    }
}

/// JSON 格式化器
struct JsonFormatter;

impl JsonFormatter {
    fn format(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(array) => Self::format_array(array),
            serde_json::Value::Object(object) => Self::format_object(object),
        }
    }

    fn format_array(array: &[serde_json::Value]) -> String {
        let items: Vec<String> = array.iter().map(Self::format).collect();
        format!("[{}]", items.join(","))
    }

    fn format_object(object: &serde_json::Map<String, serde_json::Value>) -> String {
        let mut keys: Vec<String> = object.keys().cloned().collect();
        keys.sort();
        
        let items: Vec<String> = keys.iter()
            .map(|key| format!("{}={}", key, Self::format(object.get(key).unwrap())))
            .collect();
        
        format!("{{{}}}", items.join(","))
    }
}

/// 签名验证器
pub struct SignatureValidator;

impl SignatureValidator {
    /// 验证签名
    pub fn validate(
        payload: &RequestPayload,
        headers: &SignatureHeaders,
        key: &str,
    ) -> Result<(), (String, SignatureDebugInfo)> {
        let payload_string = payload.to_signature_string(headers);
        let computed_signature = hash::hmac_sha1(&payload_string, key);

        if computed_signature != headers.signature {
            let debug = SignatureDebugInfo {
                payload: payload_string,
                key: key.to_string(),
                server_signature: computed_signature,
                client_signature: headers.signature.clone(),
            };
            return Err(("invalid signature".to_string(), debug));
        }

        Ok(())
    }
}

/// 签名调试信息
#[derive(Default, Debug, Serialize)]
pub struct SignatureDebugInfo {
    pub payload: String,
    pub key: String,
    pub server_signature: String,
    pub client_signature: String,
}

/// 重构后的 Payload 结构体，整合所有信息
pub struct Payload {
    headers: SignatureHeaders,
    request: RequestPayload,
}

impl Payload {
    /// 从请求中创建载荷
    async fn from_request(req: Request) -> Result<Self, SignatorError> {
        let (parts, body) = req.into_parts();

        // 提取签名头部
        let headers = SignatureHeaders::from_headers(&parts.headers)
            .ok_or_else(|| SignatorError::SignatureFormat("missing signature data in header".to_string()))?;

        // 提取请求载荷
        let request = RequestPayload::from_request_parts(
            parts.method.as_str(),
            parts.uri.path(),
            parts.uri.query(),
            body,
        ).await?;

        Ok(Payload { headers, request })
    }

    /// 验证载荷格式
    fn guard(&self) -> Result<(), SignatorError> {
        self.headers.validate()
    }

    /// 验证签名
    fn valid(&self, key: String) -> Result<(), (String, SignatureDebugInfo)> {
        SignatureValidator::validate(&self.request, &self.headers, &key)
    }

    /// 获取用户ID
    pub fn user_id(&self) -> &str {
        &self.headers.user_id
    }

    /// 获取随机数
    pub fn nonce(&self) -> &str {
        &self.headers.nonce
    }

    /// 获取开发跳过标识
    pub fn development_skip(&self) -> Option<&str> {
        self.headers.development_skip.as_deref()
    }

    // 保持向后兼容的方法
    pub fn val_or_default_u(&self) -> String {
        self.headers.user_id.clone()
    }

    pub fn val_or_default_r(&self) -> String {
        self.headers.nonce.clone()
    }

    pub fn val_or_default_d(&self) -> String {
        self.headers.development_skip.clone().unwrap_or_default()
    }
}#
[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use std::sync::Arc;

    // 模拟 key loader
    fn create_test_key_loader() -> KeyLoader {
        Arc::new(|_user_id: String| {
            Box::pin(async move {
                Ok("test_secret_key".to_string())
            })
        })
    }

    #[tokio::test]
    async fn test_signator_error_display() {
        let error = SignatorError::PayloadParse("test error".to_string());
        assert_eq!(error.to_string(), "Payload parse error: test error");

        let error = SignatorError::SignatureFormat("invalid format".to_string());
        assert_eq!(error.to_string(), "Signature format error: invalid format");

        let error = SignatorError::NonceReplay("duplicate nonce".to_string());
        assert_eq!(error.to_string(), "Nonce replay: duplicate nonce");
    }

    #[tokio::test]
    async fn test_signator_middleware_creation() {
        let key_loader = create_test_key_loader();
        let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader);
        
        assert_eq!(middleware.priority(), 85);
        assert_eq!(middleware.name(), "SignatorMiddleware");
        assert_eq!(middleware.path_pattern(), Some("/api/*"));
        assert_eq!(middleware.nonce_lifetime, DEFAULT_RAND_LIFE);
    }

    #[tokio::test]
    async fn test_signator_middleware_focus() {
        let key_loader = create_test_key_loader();
        let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader);
        
        // 创建测试请求
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/users")
            .body(Body::empty())
            .unwrap();

        let (parts, _) = request.into_parts();
        
        // 测试 focus 方法 - 应该处理 API 路径
        assert!(middleware.focus(&parts));
    }

    #[tokio::test]
    async fn test_signator_middleware_with_excludes() {
        let key_loader = create_test_key_loader();
        
        // 创建排除函数
        let exclude_health = |parts: &axum::http::request::Parts| -> bool {
            parts.uri.path() == "/api/health"
        };
        
        let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)
            .with_excludes(vec![exclude_health]);
        
        // 测试排除的路径
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();

        let (parts, _) = request.into_parts();
        
        // 应该被排除，不处理
        assert!(!middleware.focus(&parts));
        
        // 测试非排除的路径
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/users")
            .body(Body::empty())
            .unwrap();

        let (parts, _) = request.into_parts();
        
        // 应该处理
        assert!(middleware.focus(&parts));
    }

    #[test]
    fn test_signator_middleware_builder_pattern() {
        let key_loader = create_test_key_loader();
        
        let middleware = SignatorMiddleware::with_rear(
            "redis://localhost:6379", 
            key_loader, 
            "development_backdoor".to_string()
        ).with_nonce_lifetime(600);
        
        assert_eq!(middleware.rear, "development_backdoor");
        assert_eq!(middleware.nonce_lifetime, 600);
    }

    #[test]
    fn test_signature_headers_validation() {
        let headers = SignatureHeaders {
            user_id: "user123".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            nonce: "randomstring123".to_string(),
            signature: "a".repeat(40),
            development_skip: None,
        };

        assert!(headers.validate().is_ok());

        // 测试时间戳过期
        let old_headers = SignatureHeaders {
            timestamp: chrono::Utc::now().timestamp() - 400, // 超过5分钟
            ..headers.clone()
        };
        assert!(old_headers.validate().is_err());

        // 测试随机数长度无效
        let short_nonce_headers = SignatureHeaders {
            nonce: "short".to_string(),
            ..headers.clone()
        };
        assert!(short_nonce_headers.validate().is_err());

        // 测试签名长度无效
        let invalid_sig_headers = SignatureHeaders {
            signature: "invalid".to_string(),
            ..headers
        };
        assert!(invalid_sig_headers.validate().is_err());
    }

    #[test]
    fn test_json_formatter() {
        // 测试不同类型的 JSON 值格式化
        assert_eq!(JsonFormatter::format(&serde_json::Value::Null), "null");
        assert_eq!(JsonFormatter::format(&serde_json::Value::Bool(true)), "true");
        assert_eq!(JsonFormatter::format(&serde_json::Value::Number(serde_json::Number::from(42))), "42");
        assert_eq!(JsonFormatter::format(&serde_json::Value::String("test".to_string())), "test");

        // 测试数组格式化
        let array = serde_json::json!([1, "test", true]);
        assert_eq!(JsonFormatter::format(&array), "[1,test,true]");

        // 测试对象格式化
        let object = serde_json::json!({"b": 2, "a": 1});
        assert_eq!(JsonFormatter::format(&object), "{a=1,b=2}");
    }

    #[test]
    fn test_request_payload_signature_string() {
        let headers = SignatureHeaders {
            user_id: "user123".to_string(),
            timestamp: 1234567890,
            nonce: "randomstring".to_string(),
            signature: "signature".to_string(),
            development_skip: None,
        };

        let mut queries = HashMap::new();
        queries.insert("param1".to_string(), "value1".to_string());

        let payload = RequestPayload {
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            queries,
            body: Some(serde_json::json!({"key": "value"})),
        };

        let signature_string = payload.to_signature_string(&headers);
        assert!(signature_string.contains("POST,/api/test,{user123,1234567890,randomstring}"));
        assert!(signature_string.contains("param1=value1"));
        assert!(signature_string.contains("{key=value}"));
    }
}

/// 使用示例

pub fn create_signator_middleware_chain(redis_url: &str, key_loader: KeyLoader) -> MiddlewareChain {
    // 定义排除函数
    let exclude_health = |parts: &axum::http::request::Parts| -> bool {
        parts.uri.path() == "/api/health" || parts.uri.path() == "/api/ping"
    };
    
    let exclude_public = |parts: &axum::http::request::Parts| -> bool {
        parts.uri.path().starts_with("/api/public/")
    };

    // 创建签名中间件
    let signator = SignatorMiddleware::new(redis_url, key_loader)
        .with_excludes(vec![exclude_health, exclude_public])
        .with_nonce_lifetime(300); // 5分钟

    // 构建中间件链
    let manager = MiddlewareBuilder::new()
        .add(super::examples::CorsMiddleware::new())           // 优先级: 110
        .add(super::LoggingMiddleware::new(true))              // 优先级: 100
        .add(super::examples::AuthMiddleware::new("secret".to_string())) // 优先级: 90
        .add(signator)                                         // 优先级: 85
        .add(super::examples::RateLimitMiddleware::new(100, 60)) // 优先级: 80
        .build();

    MiddlewareChain::new(manager)
}

/// 创建带有后门的签名中间件链（用于开发环境）
pub fn create_signator_middleware_chain_with_rear(
    redis_url: &str, 
    key_loader: KeyLoader, 
    rear: String
) -> MiddlewareChain {
    let signator = SignatorMiddleware::with_rear(redis_url, key_loader, rear)
        .with_nonce_lifetime(300);

    let manager = MiddlewareBuilder::new()
        .add(super::LoggingMiddleware::new(true))
        .add(signator)
        .build();

    MiddlewareChain::new(manager)
}