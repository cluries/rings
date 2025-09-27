use crate::erx::{Erx, Layouted};
use crate::web::api::Out;
use crate::web::middleware::{ApplyKind, Context, Middleware, MiddlewareEventErr, MiddlewareFuture, MiddlewareImpl, Pattern};
use crate::web::{define::HttpMethod, request::Parts};
use axum::{
    extract::Request,
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub type ApplyMethod = Arc<dyn Fn(&Parts) -> bool + Send + Sync>;
pub type KeyExtractor = Arc<dyn Fn(&Parts) -> Option<String> + Send + Sync>;

#[derive(Debug, Clone)]
pub enum LimitStrategy {
    TokenBucket,
    FixedWindow,
    SlidingWindow,
}

#[derive(Debug, Clone)]
pub enum LimitKey {
    Ip,
    User,
    Custom(String),
    Path,
    Header(String),
}

#[derive(Debug, Clone)]
pub struct LimitRule {
    pub key: LimitKey,
    pub capacity: u64,
    pub refill_rate: u64,
    pub window_size: Duration,
    pub strategy: LimitStrategy,
    pub methods: Option<Vec<ApplyKind<HttpMethod>>>,
    pub patterns: Option<Vec<ApplyKind<Pattern>>>,
}

#[derive(Clone)]
pub struct LimitorConfig {
    pub priority: i32,
    pub redis_url: String,
    pub rules: Vec<LimitRule>,
    pub apply: Option<ApplyMethod>,
    pub default_limit: Option<(u64, Duration)>,
    pub key_extractor: Option<KeyExtractor>,
    pub block_duration: Duration,
}

impl std::fmt::Debug for LimitorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LimitorConfig")
            .field("priority", &self.priority)
            .field("redis_url", &self.redis_url)
            .field("rules", &self.rules)
            .field("apply", &self.apply.as_ref().map(|_| "Some(Fn)"))
            .field("default_limit", &self.default_limit)
            .field("key_extractor", &self.key_extractor.as_ref().map(|_| "Some(Fn)"))
            .field("block_duration", &self.block_duration)
            .finish()
    }
}

impl LimitorConfig {
    pub fn new(redis_url: String) -> Self {
        Self {
            priority: 0,
            redis_url,
            rules: Vec::new(),
            apply: None,
            default_limit: Some((100, Duration::from_secs(60))),
            key_extractor: None,
            block_duration: Duration::from_secs(300),
        }
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn apply<F>(mut self, apply: F) -> Self
    where
        F: Fn(&Parts) -> bool + Send + Sync + 'static,
    {
        self.apply = Some(Arc::new(apply));
        self
    }

    pub fn add_rule(mut self, rule: LimitRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn default_limit(mut self, limit: u64, duration: Duration) -> Self {
        self.default_limit = Some((limit, duration));
        self
    }

    pub fn key_extractor<F>(mut self, extractor: F) -> Self
    where
        F: Fn(&Parts) -> Option<String> + Send + Sync + 'static,
    {
        self.key_extractor = Some(Arc::new(extractor));
        self
    }

    pub fn block_duration(mut self, duration: Duration) -> Self {
        self.block_duration = duration;
        self
    }

    pub fn validate(&self) -> Result<(), Box<Error>> {
        if self.rules.is_empty() && self.default_limit.is_none() {
            return Err(Box::new(Error::ConfigError("At least one rule or default limit is required".to_string())));
        }

        if !self.redis_url.starts_with("redis://") && !self.redis_url.starts_with("rediss://") {
            return Err(Box::new(Error::ConfigError("Redis URL must start with 'redis://' or 'rediss://'".to_string())));
        }

        for rule in &self.rules {
            if rule.capacity == 0 {
                return Err(Box::new(Error::ConfigError("Rule capacity must be greater than 0".to_string())));
            }
            if rule.refill_rate == 0 {
                return Err(Box::new(Error::ConfigError("Rule refill rate must be greater than 0".to_string())));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    ConfigError(String),
    RedisConnectionFailed(String),
    RedisOperationFailed(String),
    LimitExceeded {
        key: String,
        limit: u64,
        remaining: u64,
        reset_time: u64,
    },
    Blocked {
        key: String,
        remaining_time: u64,
    },
    InternalError(String),
}

impl Error {
    fn make_out(&self) -> Out<()> {
        let message = self.to_string();
        let c = Layouted::middleware("LIMIT", "EROR");
        let mut out = Out::new(c, Some(message), None);

        match self {
            Error::LimitExceeded { limit, remaining, reset_time, .. } => {
                out.add_meta("limit", limit.to_string());
                out.add_meta("remaining", remaining.to_string());
                out.add_meta("reset_time", reset_time.to_string());
            },
            Error::Blocked { remaining_time, .. } => {
                out.add_meta("block_time", remaining_time.to_string());
            },
            _ => {}
        }

        out
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Error::RedisConnectionFailed(msg) => write!(f, "Redis connection failed: {}", msg),
            Error::RedisOperationFailed(msg) => write!(f, "Redis operation failed: {}", msg),
            Error::LimitExceeded { key, limit, remaining, reset_time } => {
                write!(f, "Rate limit exceeded for {}: {}/{} requests, reset in {} seconds", key, remaining, limit, reset_time)
            },
            Error::Blocked { key, remaining_time } => {
                write!(f, "Client {} is blocked, try again in {} seconds", key, remaining_time)
            },
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
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

pub struct Limitor {
    config: Arc<LimitorConfig>,
    redis_client: redis::Client,
}

impl Limitor {
    pub fn new(config: LimitorConfig) -> Result<Self, Box<Error>> {
        config.validate()?;

        let redis_client = redis::Client::open(config.redis_url.as_str()).map_err(|err| {
            Box::new(Error::ConfigError(format!("Invalid Redis URL '{}': {}", config.redis_url, err)))
        })?;

        let config = Arc::new(config);
        Ok(Limitor { config, redis_client })
    }

    async fn extract_limit_key(&self, parts: &Parts) -> Option<String> {
        if let Some(extractor) = &self.config.key_extractor {
            return extractor(parts);
        }

        for rule in &self.config.rules {
            if self.should_apply_rule(rule, parts) {
                match &rule.key {
                    LimitKey::Ip => {
                        if let Some(ip) = self.extract_client_ip(parts) {
                            return Some(format!("ip:{}", ip));
                        }
                    },
                    LimitKey::User => {
                        if let Some(user_id) = self.extract_user_id(parts) {
                            return Some(format!("user:{}", user_id));
                        }
                    },
                    LimitKey::Custom(prefix) => {
                        return Some(prefix.clone());
                    },
                    LimitKey::Path => {
                        return Some(format!("path:{}", parts.uri.path()));
                    },
                    LimitKey::Header(header_name) => {
                        if let Some(header_value) = parts.headers.get(header_name) {
                            if let Ok(value) = header_value.to_str() {
                                return Some(format!("header:{}:{}", header_name, value));
                            }
                        }
                    },
                }
            }
        }

        if let Some((_, _)) = self.config.default_limit {
            if let Some(ip) = self.extract_client_ip(parts) {
                return Some(format!("default:ip:{}", ip));
            }
        }

        None
    }

    fn should_apply_rule(&self, rule: &LimitRule, parts: &Parts) -> bool {
        if let Some(methods) = &rule.methods {
            let method_str = parts.method.as_str();
            let mut applies = false;

            for method in methods {
                if method.apply(method_str) {
                    applies = true;
                    break;
                }
            }

            if !applies {
                return false;
            }
        }

        if let Some(patterns) = &rule.patterns {
            let path = parts.uri.path();
            let mut applies = false;

            for pattern in patterns {
                if pattern.apply(path) {
                    applies = true;
                    break;
                }
            }

            if !applies {
                return false;
            }
        }

        true
    }

    fn extract_client_ip(&self, parts: &Parts) -> Option<String> {
        parts.headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                parts.headers
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
            })
            .or_else(|| {
                parts.headers
                    .get("cf-connecting-ip")
                    .and_then(|h| h.to_str().ok())
            })
            .map(|s| s.to_string())
    }

    fn extract_user_id(&self, parts: &Parts) -> Option<String> {
        parts.headers
            .get("x-u")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    }

    async fn check_rate_limit(&self, key: &str, rule: &LimitRule) -> Result<(), Box<Error>> {
        let mut redis_conn = self.redis_client.get_multiplexed_tokio_connection().await?;

        let block_key = format!("block:{}", key);
        let blocked: Option<i64> = redis_conn.get(block_key.clone()).await?;

        if let Some(block_until) = blocked {
            let now = chrono::Utc::now().timestamp() as u64;
            if block_until > now {
                return Err(Box::new(Error::Blocked {
                    key: key.to_string(),
                    remaining_time: block_until - now,
                }));
            }
        }

        let limit_key = format!("limit:{}", key);
        let current_time = chrono::Utc::now().timestamp() as u64;

        match rule.strategy {
            LimitStrategy::TokenBucket => {
                self.check_token_bucket(&mut redis_conn, &limit_key, rule, current_time).await
            },
            LimitStrategy::FixedWindow => {
                self.check_fixed_window(&mut redis_conn, &limit_key, rule, current_time).await
            },
            LimitStrategy::SlidingWindow => {
                self.check_sliding_window(&mut redis_conn, &limit_key, rule, current_time).await
            },
        }
    }

    async fn check_token_bucket(
        &self,
        redis_conn: &mut redis::aio::MultiplexedConnection,
        key: &str,
        rule: &LimitRule,
        current_time: u64,
    ) -> Result<(), Box<Error>> {
        let script = r#"
            local key = KEYS[1]
            local capacity = tonumber(ARGV[1])
            local refill_rate = tonumber(ARGV[2])
            local current_time = tonumber(ARGV[3])
            local block_key = "block:" .. key
            local block_duration = tonumber(ARGV[4])

            local tokens = tonumber(redis.call('HGET', key, 'tokens')) or capacity
            local last_refill = tonumber(redis.call('HGET', key, 'last_refill')) or current_time

            local elapsed = current_time - last_refill
            local tokens_to_add = math.floor(elapsed * refill_rate / 60)
            tokens = math.min(tokens + tokens_to_add, capacity)

            if tokens >= 1 then
                tokens = tokens - 1
                redis.call('HMSET', key, 'tokens', tokens, 'last_refill', current_time)
                redis.call('EXPIRE', key, math.ceil(capacity / refill_rate * 60) + 1)
                return {1, tokens, capacity}
            else
                redis.call('SET', block_key, current_time + block_duration)
                redis.call('EXPIRE', block_key, block_duration)
                return {0, tokens, capacity}
            end
        "#;

        let result: Vec<i64> = redis::Script::new(script)
            .key(key)
            .arg(rule.capacity)
            .arg(rule.refill_rate)
            .arg(current_time)
            .arg(self.config.block_duration.as_secs())
            .invoke_async(redis_conn)
            .await?;

        if result[0] == 0 {
            Err(Box::new(Error::LimitExceeded {
                key: key.to_string(),
                limit: rule.capacity,
                remaining: result[1] as u64,
                reset_time: (rule.capacity - result[1] as u64) * 60 / rule.refill_rate,
            }))
        } else {
            Ok(())
        }
    }

    async fn check_fixed_window(
        &self,
        redis_conn: &mut redis::aio::MultiplexedConnection,
        key: &str,
        rule: &LimitRule,
        current_time: u64,
    ) -> Result<(), Box<Error>> {
        let window_size = rule.window_size.as_secs();
        let window_key = format!("{}:{}", key, current_time / window_size);

        let count: i64 = redis_conn.incr(&window_key).await?;

        if count == 1 {
            redis_conn.expire(window_key, window_size as usize).await?;
        }

        if count as u64 > rule.capacity {
            let block_key = format!("block:{}", key);
            redis_conn.set(block_key, current_time + self.config.block_duration.as_secs()).await?;
            redis_conn.expire(block_key, self.config.block_duration.as_secs()).await?;

            Err(Box::new(Error::LimitExceeded {
                key: key.to_string(),
                limit: rule.capacity,
                remaining: 0,
                reset_time: window_size - (current_time % window_size),
            }))
        } else {
            Ok(())
        }
    }

    async fn check_sliding_window(
        &self,
        redis_conn: &mut redis::aio::MultiplexedConnection,
        key: &str,
        rule: &LimitRule,
        current_time: u64,
    ) -> Result<(), Box<Error>> {
        let window_size = rule.window_size.as_secs();
        let cutoff = current_time - window_size;

        let script = r#"
            local key = KEYS[1]
            local cutoff = tonumber(ARGV[1])
            local capacity = tonumber(ARGV[2])
            local current_time = tonumber(ARGV[3])
            local block_key = "block:" .. key
            local block_duration = tonumber(ARGV[4])

            redis.call('ZREMRANGEBYSCORE', key, 0, cutoff)
            local count = redis.call('ZCARD', key)

            if count < capacity then
                redis.call('ZADD', key, current_time, current_time)
                redis.call('EXPIRE', key, math.ceil(window_size / 60) + 1)
                return {1, capacity - count - 1, capacity}
            else
                redis.call('SET', block_key, current_time + block_duration)
                redis.call('EXPIRE', block_key, block_duration)
                return {0, 0, capacity}
            end
        "#;

        let result: Vec<i64> = redis::Script::new(script)
            .key(key)
            .arg(cutoff)
            .arg(rule.capacity)
            .arg(current_time)
            .arg(self.config.block_duration.as_secs())
            .invoke_async(redis_conn)
            .await?;

        if result[0] == 0 {
            Err(Box::new(Error::LimitExceeded {
                key: key.to_string(),
                limit: rule.capacity,
                remaining: result[1] as u64,
                reset_time: window_size,
            }))
        } else {
            Ok(())
        }
    }

    async fn apply_limit(&self, request: Request) -> Result<Request, Box<Error>> {
        let (parts, body) = request.into_parts();
        let limit_key = self.extract_limit_key(&parts).await.ok_or_else(|| {
            Box::new(Error::InternalError("Could not extract limit key".to_string()))
        })?;

        for rule in &self.config.rules {
            if self.should_apply_rule(rule, &parts) {
                self.check_rate_limit(&limit_key, rule).await?;
                break;
            }
        }

        if let Some((limit, window)) = self.config.default_limit {
            let default_rule = LimitRule {
                key: LimitKey::Custom("default".to_string()),
                capacity: limit,
                refill_rate: limit,
                window_size: window,
                strategy: LimitStrategy::FixedWindow,
                methods: None,
                patterns: None,
            };
            self.check_rate_limit(&limit_key, &default_rule).await?;
        }

        let request = Request::from_parts(parts, body);
        Ok(request)
    }
}

impl Clone for Limitor {
    fn clone(&self) -> Self {
        Limitor {
            config: self.config.clone(),
            redis_client: self.redis_client.clone(),
        }
    }
}

impl std::fmt::Debug for Limitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Limitor").field("config", &self.config).field("redis_client", &self.redis_client).finish()
    }
}

impl Middleware for Limitor {
    fn name(&self) -> &'static str {
        "Limitor"
    }

    fn on_request(&self, context: Context, request: Request) -> MiddlewareImpl<MiddlewareFuture<Request>, MiddlewareEventErr<Request>> {
        let limitor = self.clone();

        let future = Box::pin(async move {
            match limitor.apply_limit(request).await {
                Ok(req) => Ok((context, req)),
                Err(error) => {
                    let erx = Erx::new(&error.to_string());
                    let out: Out<()> = error.make_out();

                    let mut context = context;
                    context.make_abort_with_response(
                        Limitor::middleware_name(),
                        &error.to_string(),
                        out.into_response(),
                    );
                    Err((context, None, Some(erx)))
                },
            }
        });

        MiddlewareImpl::Implemented(future)
    }

    fn priority(&self) -> i32 {
        self.config.priority
    }

    fn apply(&self, parts: &Parts) -> Option<bool> {
        self.config.apply.as_ref().map(|f| f(parts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::middleware::ApplyTrait;

    #[test]
    fn test_limitor_config_new() {
        let config = LimitorConfig::new("redis://localhost:6379".to_string());
        assert_eq!(config.priority, 0);
        assert!(config.rules.is_empty());
        assert!(config.apply.is_none());
        assert!(config.default_limit.is_some());
    }

    #[test]
    fn test_limitor_config_builder() {
        let rule = LimitRule {
            key: LimitKey::Ip,
            capacity: 100,
            refill_rate: 50,
            window_size: Duration::from_secs(60),
            strategy: LimitStrategy::TokenBucket,
            methods: None,
            patterns: None,
        };

        let config = LimitorConfig::new("redis://localhost:6379".to_string())
            .priority(100)
            .add_rule(rule)
            .block_duration(Duration::from_secs(600));

        assert_eq!(config.priority, 100);
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.block_duration, Duration::from_secs(600));
    }

    #[test]
    fn test_limit_key_matching() {
        let ip_pattern = LimitKey::Ip;
        let user_pattern = LimitKey::User;
        let custom_pattern = LimitKey::Custom("api_key".to_string());

        assert!(matches!(ip_pattern, LimitKey::Ip));
        assert!(matches!(user_pattern, LimitKey::User));
        assert_eq!(custom_pattern, LimitKey::Custom("api_key".to_string()));
    }

    #[test]
    fn test_limit_strategy() {
        assert!(matches!(LimitStrategy::TokenBucket, LimitStrategy::TokenBucket));
        assert!(matches!(LimitStrategy::FixedWindow, LimitStrategy::FixedWindow));
        assert!(matches!(LimitStrategy::SlidingWindow, LimitStrategy::SlidingWindow));
    }

    #[tokio::test]
    async fn test_config_validation() {
        let valid_config = LimitorConfig::new("redis://localhost:6379".to_string())
            .default_limit(100, Duration::from_secs(60));
        assert!(valid_config.validate().is_ok());

        let invalid_config = LimitorConfig::new("invalid://localhost:6379".to_string());
        assert!(invalid_config.validate().is_err());
    }
}
