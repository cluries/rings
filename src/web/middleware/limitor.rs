//! API Rate Limiting Middleware / API 限流中间件
//!
//! This module provides a comprehensive rate limiting middleware for web APIs with support for
//! multiple strategies, Redis-backed storage, and flexible configuration options.
//!
//! 本模块提供了一个全面的 API 限流中间件，支持多种限流策略、Redis 后端存储和灵活的配置选项。
//!
//! # Features / 特性
//!
//! - Multiple rate limiting strategies (Token Bucket, Fixed Window, Sliding Window)
//!   多种限流策略（令牌桶、固定窗口、滑动窗口）
//! - Redis-based distributed rate limiting
//!   基于 Redis 的分布式限流
//! - IP-based and user-based rate limiting
//!   基于 IP 和用户的限流
//! - Configurable blocking mechanisms
//!   可配置的阻塞机制
//! - Flexible rule matching with HTTP methods and URL patterns
//!   灵活的规则匹配，支持 HTTP 方法和 URL 模式
//!
//! # Usage Example / 使用示例
//!
//! ```rust
//! use std::time::Duration;
//! use crate::web::middleware::limitor::*;
//! use crate::web::middleware::{ApplyKind, Pattern};
//! use crate::web::define::HttpMethod;
//!
//! // Create a limitor configuration / 创建限流器配置
//! let limitor = Limitor::new(
//!     LimitorConfig::new("redis://localhost:6379".to_string())
//!         .priority(100)  // Higher priority runs first / 优先级越高越先执行
//!         .add_rule(LimitRule {
//!             key: LimitKey::Ip,  // Rate limit by IP address / 按 IP 地址限流
//!             capacity: 100,     // Maximum requests / 最大请求数
//!             refill_rate: 50,   // Refill rate per minute / 每分钟补充速率
//!             window_size: Duration::from_secs(60),  // Time window / 时间窗口
//!             strategy: LimitStrategy::TokenBucket,   // Rate limiting strategy / 限流策略
//!             methods: Some(vec![
//!                 ApplyKind::Include(HttpMethod::GET),
//!                 ApplyKind::Include(HttpMethod::POST),
//!             ]),
//!             patterns: Some(vec![
//!                 ApplyKind::Include(Pattern::Prefix("/api/".to_string(), true)),
//!             ]),
//!         })
//!         .block_duration(Duration::from_secs(300))  // Block duration / 阻塞时长
//! )?;
//!
//! // Use the limitor in your middleware stack / 在中间件栈中使用限流器
//!
//! # Configuration Options / 配置选项
//!
//! ## Redis Configuration / Redis 配置
//!
//! ```rust
//! // Redis connection string / Redis 连接字符串
//! let config = LimitorConfig::new("redis://localhost:6379".to_string());
//!
//! // Redis with authentication / 带认证的 Redis
//! let config = LimitorConfig::new("redis://:password@localhost:6379".to_string());
//!
//! // Redis cluster / Redis 集群
//! let config = LimitorConfig::new("redis://cluster-node1:6379,cluster-node2:6379".to_string());
//! ```
//!
//! ## Rate Limiting Strategies / 限流策略
//!
//! ### Token Bucket (令牌桶)
//!
//! Most flexible strategy, suitable for burst traffic control.
//! 最灵活的策略，适合突发流量控制。
//!
//! ```rust
//! LimitRule {
//!     key: LimitKey::Ip,
//!     capacity: 100,        // Bucket size / 桶容量
//!     refill_rate: 50,      // Tokens per minute / 每分钟补充令牌数
//!     window_size: Duration::from_secs(60),  // Not used for token bucket / 令牌桶不使用
//!     strategy: LimitStrategy::TokenBucket,
//!     methods: None,
//!     patterns: None,
//! }
//! ```
//!
//! ### Fixed Window (固定窗口)
//!
//! Simple and predictable, resets counter at fixed intervals.
//! 简单可预测，在固定时间间隔重置计数器。
//!
//! ```rust
//! LimitRule {
//!     key: LimitKey::User,
//!     capacity: 1000,       // Max requests per window / 每个窗口最大请求数
//!     refill_rate: 1000,    // Not used for fixed window / 固定窗口不使用
//!     window_size: Duration::from_secs(3600),  // 1 hour window / 1小时窗口
//!     strategy: LimitStrategy::FixedWindow,
//!     methods: None,
//!     patterns: None,
//! }
//! ```
//!
//! ### Sliding Window (滑动窗口)
//!
//! More accurate than fixed window, provides smoother rate limiting.
//! 比固定窗口更准确，提供更平滑的限流。
//!
//! ```rust
//! LimitRule {
//!     key: LimitKey::Custom("api_key".to_string()),
//!     capacity: 100,        // Max requests in sliding window / 滑动窗口最大请求数
//!     refill_rate: 100,     // Not used for sliding window / 滑动窗口不使用
//!     window_size: Duration::from_secs(60),     // 1 minute sliding window / 1分钟滑动窗口
//!     strategy: LimitStrategy::SlidingWindow,
//!     methods: None,
//!     patterns: None,
//! }
//! ```
//!
//! ## Limit Keys / 限流键
//!
//! Different ways to identify and rate limit clients:
//! 识别和限制客户端的不同方式：
//!
//! ```rust
//! // IP-based limiting / 基于 IP 的限流
//! LimitKey::Ip
//!
//! // User-based limiting (requires x-u header) / 基于用户的限流（需要 x-u 头）
//! LimitKey::User
//!
//! // Custom key limiting / 自定义键限流
//! LimitKey::Custom("api_key".to_string())
//!
//! // Path-based limiting / 基于路径的限流
//! LimitKey::Path
//!
//! // Header-based limiting / 基于请求头的限流
//! LimitKey::Header("Authorization".to_string())
//! ```
//!
//! ## Pattern Matching / 模式匹配
//!
//! Apply rate limiting to specific endpoints:
//! 对特定端点应用限流：
//!
//! ```rust
//! use crate::web::middleware::{ApplyKind, Pattern};
//! use crate::web::define::HttpMethod;
//!
//! // Include specific methods / 包含特定方法
//! methods: Some(vec![
//!     ApplyKind::Include(HttpMethod::GET),
//!     ApplyKind::Include(HttpMethod::POST),
//! ])
//!
//! // Include specific URL patterns / 包含特定 URL 模式
//! patterns: Some(vec![
//!     ApplyKind::Include(Pattern::Prefix("/api/v1/".to_string(), true)),
//!     ApplyKind::Include(Pattern::Exact("/login".to_string())),
//!     ApplyKind::Exclude(Pattern::Prefix("/public/".to_string(), true)),
//! ])
//! ```
//!
//! ## Default Rate Limiting / 默认限流
//!
//! Set a fallback rate limit for requests that don't match any specific rules:
//! 为不匹配任何特定规则的请求设置回退限流：
//!
//! ```rust
//! let config = LimitorConfig::new("redis://localhost:6379".to_string())
//!     .default_limit(100, Duration::from_secs(60))  // 100 requests per minute / 每分钟100次请求
//!     .block_duration(Duration::from_secs(300));   // Block for 5 minutes when exceeded / 超过时阻塞5分钟
//! ```
//!
//! # Error Handling / 错误处理
//!
//! The middleware returns specific error responses when rate limits are exceeded:
//! 当超过限流时，中间件返回特定的错误响应：
//!
//! ```rust
//! match limitor.apply_limit(request).await {
//!     Ok(request) => {
//!         // Request allowed / 请求被允许
//!         // Continue processing / 继续处理
//!     }
//!     Err(error) => {
//!         // Rate limit exceeded / 限流超过
//!         match error.downcast_ref::<limitor::Error>() {
//!             Some(limitor::Error::LimitExceeded { key, limit, remaining, reset_time }) => {
//!                 // Construct error response / 构造错误响应
//!                 let response = Response::builder()
//!                     .status(StatusCode::TOO_MANY_REQUESTS)
//!                     .header("X-RateLimit-Limit", limit.to_string())
//!                     .header("X-RateLimit-Remaining", remaining.to_string())
//!                     .header("X-RateLimit-Reset", reset_time.to_string())
//!                     .header("Retry-After", reset_time.to_string())
//!                     .body("Rate limit exceeded".into())?;
//!                 return Ok(response);
//!             }
//!             Some(limitor::Error::Blocked { key, remaining_time }) => {
//!                 // Client is blocked / 客户端被阻塞
//!                 let response = Response::builder()
//!                     .status(StatusCode::FORBIDDEN)
//!                     .header("Retry-After", remaining_time.to_string())
//!                     .body("Access blocked due to rate limit violations".into())?;
//!                 return Ok(response);
//!             }
//!             _ => {
//!                 // Other errors / 其他错误
//!                 let response = Response::builder()
//!                     .status(StatusCode::INTERNAL_SERVER_ERROR)
//!                     .body("Internal server error".into())?;
//!                 return Ok(response);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Performance Considerations / 性能考虑
//!
//! - Redis Lua scripts are pre-compiled for better performance
//!   Redis Lua 脚本预编译以提高性能
//! - Use multiplexed Redis connections for concurrent request handling
//!   使用多路复用 Redis 连接处理并发请求
//! - Choose appropriate strategies based on your use case
//!   根据用例选择合适的策略
//! - Monitor Redis memory usage for high-traffic applications
//!   监控高流量应用的 Redis 内存使用情况
//!
//! # Security Features / 安全特性
//!
//! - IP address validation to prevent spoofing
//!   IP 地址验证以防止欺骗
//! - Configurable block duration for abusive clients
//!   可配置的阻塞时长用于恶意客户端
//! - Support for various authentication mechanisms
//!   支持各种认证机制
//!
//! # Monitoring and Metrics / 监控和指标
//!
//! The middleware provides detailed error information that can be used for monitoring:
//! 中间件提供详细的错误信息，可用于监控：
//!
//! ```rust
//! // Log rate limit events / 记录限流事件
//! match limitor.apply_limit(request).await {
//!     Ok(req) => Ok(req),
//!     Err(error) => {
//!         match error.downcast_ref::<limitor::Error>() {
//!             Some(limitor::Error::LimitExceeded { key, limit, remaining, reset_time }) => {
//!                 log::warn!(\"Rate limit exceeded for {}: {}/{} requests, reset in {}s\",
//!                           key, limit - remaining, limit, reset_time);
//!             }
//!             Some(limitor::Error::Blocked { key, remaining_time }) => {
//!                 log::warn!(\"Client blocked for {}: {} seconds remaining\", key, remaining_time);
//!             }
//!             _ => {}
//!         }
//!         Err(error)
//!     }
//! }
//! ```

use crate::erx::{Erx, Layouted};
use crate::web::api::Out;
use crate::web::middleware::{ApplyKind, Context, Middleware, MiddlewareEventErr, MiddlewareFuture, MiddlewareImpl, Pattern};
use crate::web::middleware::Parts;
use crate::web::define::HttpMethod;
use axum::extract::Request;
use axum::response::IntoResponse;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;

/// Type alias for custom application logic function / 自定义应用逻辑函数类型别名
///
/// This function type allows users to define custom logic to determine whether
/// the limitor should be applied to a specific request.
///
/// 此函数类型允许用户定义自定义逻辑，以确定限流器是否应该应用于特定请求。
pub type ApplyMethod = Arc<dyn Fn(&Parts) -> bool + Send + Sync>;

/// Type alias for custom key extraction function / 自定义键提取函数类型别名
///
/// This function type allows users to define custom logic to extract
/// rate limiting keys from request parts.
///
/// 此函数类型允许用户定义自定义逻辑，以从请求部分提取限流键。
pub type KeyExtractor = Arc<dyn Fn(&Parts) -> Option<String> + Send + Sync>;

/// Rate limiting strategies / 限流策略
///
/// Defines different algorithms for implementing rate limiting.
///
/// 定义实现限流的不同算法。
#[derive(Debug, Clone, PartialEq)]
pub enum LimitStrategy {
    /// Token Bucket algorithm / 令牌桶算法
    ///
    /// Provides smooth rate limiting with burst capacity.
    /// 提供平滑的限流，支持突发容量。
    TokenBucket,

    /// Fixed Window algorithm / 固定窗口算法
    ///
    /// Simple implementation with fixed time windows.
    /// 使用固定时间窗口的简单实现。
    FixedWindow,

    /// Sliding Window algorithm / 滑动窗口算法
    ///
    /// More accurate rate limiting with sliding time windows.
    /// 使用滑动时间窗口的更准确限流。
    SlidingWindow,
}

/// Rate limiting key types / 限流键类型
///
/// Defines different sources for rate limiting keys.
///
/// 定义限流键的不同来源。
#[derive(Debug, Clone, PartialEq)]
pub enum LimitKey {
    /// Limit by client IP address / 按客户端 IP 地址限流
    ///
    /// Extracts IP from headers like X-Forwarded-For, X-Real-IP, etc.
    /// 从 X-Forwarded-For、X-Real-IP 等头部提取 IP。
    Ip,

    /// Limit by user ID / 按用户 ID 限流
    ///
    /// Extracts user ID from X-U header.
    /// 从 X-U 头部提取用户 ID。
    User,

    /// Custom key prefix / 自定义键前缀
    ///
    /// Uses a custom string as the key prefix.
    /// 使用自定义字符串作为键前缀。
    Custom(String),

    /// Limit by request path / 按请求路径限流
    ///
    /// Uses the request path as the key.
    /// 使用请求路径作为键。
    Path,

    /// Limit by header value / 按头部值限流
    ///
    /// Uses the value of a specific header as the key.
    /// 使用特定头部的值作为键。
    Header(String),
}

/// Rate limiting rule configuration / 限流规则配置
///
/// Defines a specific rate limiting rule with its parameters and conditions.
///
/// 定义特定的限流规则及其参数和条件。
#[derive(Debug, Clone)]
pub struct LimitRule {
    /// The key type for rate limiting / 限流的键类型
    ///
    /// Determines how to identify the client being rate limited.
    /// 决定如何识别被限流的客户端。
    pub key: LimitKey,

    /// Maximum number of requests allowed / 允许的最大请求数
    ///
    /// The maximum number of requests allowed within the time window.
    /// 在时间窗口内允许的最大请求数。
    pub capacity: u64,

    /// Token refill rate per minute / 每分钟令牌补充速率
    ///
    /// Only used for TokenBucket strategy. Defines how many tokens are added per minute.
    /// 仅用于令牌桶策略。定义每分钟添加多少令牌。
    pub refill_rate: u64,

    /// Time window size / 时间窗口大小
    ///
    /// The duration of the rate limiting window.
    /// 限流窗口的持续时间。
    pub window_size: Duration,

    /// Rate limiting strategy / 限流策略
    ///
    /// The algorithm to use for rate limiting.
    /// 用于限流的算法。
    pub strategy: LimitStrategy,

    /// HTTP method filters / HTTP 方法过滤器
    ///
    /// Optional list of HTTP methods this rule applies to.
    /// 此规则适用的 HTTP 方法可选列表。
    pub methods: Option<Vec<ApplyKind<HttpMethod>>>,

    /// URL pattern filters / URL 模式过滤器
    ///
    /// Optional list of URL patterns this rule applies to.
    /// 此规则适用的 URL 模式可选列表。
    pub patterns: Option<Vec<ApplyKind<Pattern>>>,
}

/// Rate limiter configuration / 限流器配置
///
/// Main configuration structure for the rate limiting middleware.
///
/// 限流中间件的主要配置结构。
#[derive(Clone)]
pub struct LimitorConfig {
    /// Middleware priority / 中间件优先级
    ///
    /// Higher priority numbers are executed first. Default is 0.
    /// 优先级数字越大越先执行。默认为 0。
    pub priority: i32,

    /// Redis connection URL / Redis 连接 URL
    ///
    /// URL for connecting to Redis server for distributed rate limiting.
    /// 用于连接 Redis 服务器的 URL，用于分布式限流。
    pub redis_url: String,

    /// List of rate limiting rules / 限流规则列表
    ///
    /// Rules are evaluated in order. First matching rule is applied.
    /// 规则按顺序评估。第一个匹配的规则将被应用。
    pub rules: Vec<LimitRule>,

    /// Custom application logic / 自定义应用逻辑
    ///
    /// Optional function to determine if limitor should be applied.
    /// 可选函数，用于确定是否应该应用限流器。
    pub apply: Option<ApplyMethod>,

    /// Default rate limiting / 默认限流
    ///
    /// Default limit (requests, duration) when no specific rules match.
    /// 当没有特定规则匹配时的默认限制（请求数，持续时间）。
    pub default_limit: Option<(u64, Duration)>,

    /// Custom key extractor / 自定义键提取器
    ///
    /// Optional function to extract custom rate limiting keys.
    /// 可选函数，用于提取自定义限流键。
    pub key_extractor: Option<KeyExtractor>,

    /// Block duration when limit exceeded / 超限时的阻塞时长
    ///
    /// How long to block clients when they exceed rate limits.
    /// 当客户端超过限流时阻塞的时长。
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

/// Rate limiting errors / 限流错误
///
/// Enumeration of all possible errors that can occur during rate limiting.
///
/// 限流过程中可能发生的所有错误的枚举。
#[derive(Debug)]
pub enum Error {
    /// Configuration error / 配置错误
    ///
    /// Errors related to invalid configuration parameters.
    /// 与无效配置参数相关的错误。
    ConfigError(String),

    /// Redis connection failure / Redis 连接失败
    ///
    /// Failed to connect to Redis server.
    /// 连接到 Redis 服务器失败。
    RedisConnectionFailed(String),

    /// Redis operation failure / Redis 操作失败
    ///
    /// Errors during Redis operations (queries, commands, etc.).
    /// Redis 操作期间发生的错误（查询、命令等）。
    RedisOperationFailed(String),

    /// Rate limit exceeded / 限流超出
    ///
    /// Client has exceeded their rate limit.
    /// 客户端已超出其限流限制。
    LimitExceeded {
        /// Rate limiting key identifier / 限流键标识符
        key: String,
        /// Maximum allowed requests / 最大允许请求数
        limit: u64,
        /// Remaining requests allowed / 剩余允许请求数
        remaining: u64,
        /// Time until limit resets / 限制重置时间（秒）
        reset_time: u64,
    },

    /// Client blocked / 客户端被阻塞
    ///
    /// Client is temporarily blocked due to repeated limit violations.
    /// 客户端因重复违反限制而被临时阻塞。
    Blocked {
        /// Rate limiting key identifier / 限流键标识符
        key: String,
        /// Remaining block time / 剩余阻塞时间（秒）
        remaining_time: u64,
    },

    /// Internal error / 内部错误
    ///
    /// Unexpected internal errors.
    /// 意外的内部错误。
    InternalError(String),
}

impl Error {
    fn make_out(&self) -> Out<()> {
        let message = self.to_string();
        let c = Layouted::middleware("LIMIT", "EROR");
        Out::new(c, Some(message), None)
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

/// Rate limiter middleware / 限流中间件
///
/// Main rate limiting implementation with Redis backing and multiple strategies.
///
/// 主要的限流实现，具有 Redis 后端和多种策略。
pub struct Limitor {
    /// Configuration settings / 配置设置
    ///
    /// Arc-shared configuration for thread safety.
    /// 用于线程安全的 Arc 共享配置。
    config: Arc<LimitorConfig>,

    /// Redis client connection / Redis 客户端连接
    ///
    /// Client for connecting to Redis server.
    /// 用于连接 Redis 服务器的客户端。
    redis_client: redis::Client,

    /// Pre-compiled token bucket script / 预编译的令牌桶脚本
    ///
    /// Lua script for token bucket algorithm, pre-compiled for performance.
    /// 令牌桶算法的 Lua 脚本，预编译以提高性能。
    token_bucket_script: redis::Script,

    /// Pre-compiled sliding window script / 预编译的滑动窗口脚本
    ///
    /// Lua script for sliding window algorithm, pre-compiled for performance.
    /// 滑动窗口算法的 Lua 脚本，预编译以提高性能。
    sliding_window_script: redis::Script,
}

impl Limitor {
    /// Create a new rate limiter instance / 创建新的限流器实例
    ///
    /// Initializes a new rate limiter with the given configuration.
    /// This method validates the configuration and pre-compiles Lua scripts.
    ///
    /// 使用给定配置初始化新的限流器。此方法验证配置并预编译 Lua 脚本。
    ///
    /// # Arguments / 参数
    ///
    /// * `config` - Rate limiter configuration / 限流器配置
    ///
    /// # Returns / 返回
    ///
    /// * `Result<Self, Box<Error>>` - New limiter instance or error / 新的限流器实例或错误
    ///
    /// # Errors / 错误
    ///
    /// Returns error if configuration is invalid or Redis connection fails.
    /// 如果配置无效或 Redis 连接失败，则返回错误。
    pub fn new(config: LimitorConfig) -> Result<Self, Box<Error>> {
        config.validate()?;

        let redis_client = redis::Client::open(config.redis_url.as_str())
            .map_err(|err| Box::new(Error::ConfigError(format!("Invalid Redis URL '{}': {}", config.redis_url, err))))?;

        // Token Bucket Algorithm Implementation / 令牌桶算法实现
        //
        // This Lua script implements the token bucket rate limiting algorithm:
        // 此 Lua 脚本实现了令牌桶限流算法：
        //
        // Arguments / 参数:
        // - KEYS[1]: Rate limiting key / 限流键
        // - ARGV[1]: Bucket capacity / 桶容量
        // - ARGV[2]: Token refill rate per minute / 每分钟令牌补充速率
        // - ARGV[3]: Current timestamp / 当前时间戳
        // - ARGV[4]: Block duration in seconds / 阻塞时长（秒）
        //
        // Returns / 返回:
        // - {1, remaining_tokens, capacity}: Success / 成功
        // - {0, remaining_tokens, capacity}: Rate limit exceeded / 限流超出
        let token_bucket_script = redis::Script::new(r#"
            -- Get script arguments / 获取脚本参数
            local key = KEYS[1]                    -- Rate limiting key / 限流键
            local capacity = tonumber(ARGV[1])     -- Maximum tokens / 最大令牌数
            local refill_rate = tonumber(ARGV[2]) -- Tokens per minute / 每分钟令牌数
            local current_time = tonumber(ARGV[3]) -- Current timestamp / 当前时间戳
            local block_key = "block:" .. key      -- Block key / 阻塞键
            local block_duration = tonumber(ARGV[4]) -- Block duration / 阻塞时长

            -- Get current token count or initialize to capacity / 获取当前令牌数或初始化为容量
            local tokens = tonumber(redis.call('HGET', key, 'tokens')) or capacity
            local last_refill = tonumber(redis.call('HGET', key, 'last_refill')) or current_time

            -- Calculate tokens to add based on elapsed time / 根据经过时间计算要添加的令牌数
            local elapsed = current_time - last_refill
            local tokens_to_add = math.floor(elapsed * refill_rate / 60)
            tokens = math.min(tokens + tokens_to_add, capacity)

            -- Check if request can be processed / 检查是否可以处理请求
            if tokens >= 1 then
                -- Consume one token / 消耗一个令牌
                tokens = tokens - 1
                -- Update token state / 更新令牌状态
                redis.call('HMSET', key, 'tokens', tokens, 'last_refill', current_time)
                -- Set expiration for automatic cleanup / 设置过期时间以便自动清理
                redis.call('EXPIRE', key, math.ceil(capacity / refill_rate * 60) + 1)
                -- Return success with remaining tokens / 返回成功和剩余令牌数
                return {1, tokens, capacity}
            else
                -- Block client for specified duration / 阻塞客户端指定时长
                redis.call('SET', block_key, current_time + block_duration)
                redis.call('EXPIRE', block_key, block_duration)
                -- Return failure / 返回失败
                return {0, tokens, capacity}
            end
        "#);

        // 滑动窗口限流 Lua 脚本 / Sliding Window Rate Limiting Lua Script
        // 使用 Redis 有序集合实现精确的滑动窗口限流
        // Implements precise sliding window rate limiting using Redis sorted sets
        //
        // 参数 / Parameters:
        // KEYS[1]: 限流键 / Rate limiting key
        // ARGV[1]: 窗口截止时间戳 / Window cutoff timestamp
        // ARGV[2]: 窗口容量 / Window capacity
        // ARGV[3]: 当前时间戳 / Current timestamp
        // ARGV[4]: 封禁时长 / Block duration
        //
        // 返回值 / Returns:
        // {1, remaining, capacity}: 允许访问，剩余配额 / Access allowed, remaining quota
        // {0, 0, capacity}: 拒绝访问，已封禁 / Access denied, blocked
        let sliding_window_script = redis::Script::new(r#"
            -- 限流键和参数获取 / Get rate limiting key and parameters
            local key = KEYS[1]
            local cutoff = tonumber(ARGV[1])
            local capacity = tonumber(ARGV[2])
            local current_time = tonumber(ARGV[3])
            local block_key = "block:" .. key
            local block_duration = tonumber(ARGV[4])

            -- 清理窗口外的过期请求 / Remove expired requests outside the window
            redis.call('ZREMRANGEBYSCORE', key, 0, cutoff)
            -- 计算当前窗口内的请求数 / Count requests in current window
            local count = redis.call('ZCARD', key)

            -- 检查是否超过容量限制 / Check if capacity limit is exceeded
            if count < capacity then
                -- 允许访问，记录当前请求 / Allow access, record current request
                redis.call('ZADD', key, current_time, current_time)
                -- 设置键过期时间 / Set key expiration time
                redis.call('EXPIRE', key, math.ceil(window_size / 60) + 1)
                -- 返回成功状态和剩余配额 / Return success status and remaining quota
                return {1, capacity - count - 1, capacity}
            else
                -- 超过限制，设置封禁 / Exceeded limit, set block
                redis.call('SET', block_key, current_time + block_duration)
                redis.call('EXPIRE', block_key, block_duration)
                -- 返回拒绝状态 / Return denial status
                return {0, 0, capacity}
            end
        "#);

        let config = Arc::new(config);
        Ok(Limitor {
            config,
            redis_client,
            token_bucket_script,
            sliding_window_script,
        })
    }

    /// 提取限流键 / Extract rate limiting key
    ///
    /// 根据配置的规则和请求信息生成唯一的限流键
    /// Generates a unique rate limiting key based on configured rules and request information
    ///
    /// # 参数 / Parameters
    /// * `parts` - HTTP 请求部分 / HTTP request parts
    ///
    /// # 返回值 / Returns
    /// * `Option<String>` - 限流键，如果没有匹配的规则则返回 None / Rate limiting key, returns None if no matching rule
    async fn extract_limit_key(&self, parts: &Parts) -> Option<String> {
        // 优先使用自定义提取器 / Prefer custom extractor
        if let Some(extractor) = &self.config.key_extractor {
            return extractor(parts);
        }

        // 遍历所有规则寻找匹配项 / Iterate through all rules to find matches
        for rule in &self.config.rules {
            if self.should_apply_rule(rule, parts) {
                match &rule.key {
                    LimitKey::Ip => {
                        // 基于IP的限流 / IP-based rate limiting
                        if let Some(ip) = self.extract_client_ip(parts) {
                            return Some(format!("ip:{}", ip));
                        }
                    },
                    LimitKey::User => {
                        // 基于用户的限流 / User-based rate limiting
                        if let Some(user_id) = self.extract_user_id(parts) {
                            return Some(format!("user:{}", user_id));
                        }
                    },
                    LimitKey::Custom(prefix) => {
                        // 自定义前缀的限流 / Custom prefix rate limiting
                        return Some(prefix.clone());
                    },
                    LimitKey::Path => {
                        // 基于路径的限流 / Path-based rate limiting
                        return Some(format!("path:{}", parts.uri.path()));
                    },
                    LimitKey::Header(header_name) => {
                        // 基于请求头的限流 / Header-based rate limiting
                        if let Some(header_value) = parts.headers.get(header_name) {
                            if let Ok(value) = header_value.to_str() {
                                return Some(format!("header:{}:{}", header_name, value));
                            }
                        }
                    },
                }
            }
        }

        // 使用默认限流配置 / Use default rate limiting configuration
        if let Some((_, _)) = self.config.default_limit {
            if let Some(ip) = self.extract_client_ip(parts) {
                return Some(format!("default:ip:{}", ip));
            }
        }

        None
    }

    /// 检查规则是否适用于当前请求 / Check if rule applies to current request
    ///
    /// 根据HTTP方法和URL路径匹配规则
    /// Matches rules based on HTTP method and URL path
    ///
    /// # 参数 / Parameters
    /// * `rule` - 限流规则 / Rate limiting rule
    /// * `parts` - HTTP 请求部分 / HTTP request parts
    ///
    /// # 返回值 / Returns
    /// * `bool` - 规则是否适用 / Whether the rule applies
    fn should_apply_rule(&self, rule: &LimitRule, parts: &Parts) -> bool {
        // 检查HTTP方法匹配 / Check HTTP method matching
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

        // 检查URL路径匹配 / Check URL path matching
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

    /// 提取客户端IP地址 / Extract client IP address
    ///
    /// 从HTTP请求头中提取真实的客户端IP地址
    /// Extracts real client IP address from HTTP request headers
    ///
    /// 支持的头部 / Supported headers:
    /// - X-Forwarded-For: 代理链中的真实IP / Real IP in proxy chain
    /// - X-Real-IP: Nginx等代理设置的IP / IP set by proxies like Nginx
    /// - CF-Connecting-IP: Cloudflare设置的IP / IP set by Cloudflare
    ///
    /// # 参数 / Parameters
    /// * `parts` - HTTP 请求部分 / HTTP request parts
    ///
    /// # 返回值 / Returns
    /// * `Option<String>` - 客户端IP地址，如果无法提取则返回 None / Client IP address, returns None if extraction fails
    fn extract_client_ip(&self, parts: &Parts) -> Option<String> {
        // 按优先级尝试从不同的头部获取IP / Try to get IP from different headers by priority
        let ip = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .or_else(|| parts.headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
            .or_else(|| parts.headers.get("cf-connecting-ip").and_then(|h| h.to_str().ok()));

        // 验证IP地址格式 / Validate IP address format
        ip.and_then(|s| {
            if self.is_valid_ip(s) {
                Some(s.to_string())
            } else {
                None
            }
        })
    }

    /// 验证IP地址格式 / Validate IP address format
    ///
    /// 使用标准库验证IP地址的有效性
    /// Uses standard library to validate IP address format
    ///
    /// # 参数 / Parameters
    /// * `ip_str` - IP地址字符串 / IP address string
    ///
    /// # 返回值 / Returns
    /// * `bool` - IP地址是否有效 / Whether the IP address is valid
    fn is_valid_ip(&self, ip_str: &str) -> bool {
        ip_str.parse::<std::net::IpAddr>().is_ok()
    }

    /// 提取用户ID / Extract user ID
    ///
    /// 从HTTP请求头中提取用户标识符
    /// Extracts user identifier from HTTP request header
    ///
    /// # 参数 / Parameters
    /// * `parts` - HTTP 请求部分 / HTTP request parts
    ///
    /// # 返回值 / Returns
    /// * `Option<String>` - 用户ID，如果不存在则返回 None / User ID, returns None if not exists
    fn extract_user_id(&self, parts: &Parts) -> Option<String> {
        // 从 x-u 头部获取用户ID / Get user ID from x-u header
        parts.headers.get("x-u").and_then(|h| h.to_str().ok()).map(|s| s.to_string())
    }

    /// 检查限流状态 / Check rate limiting status
    ///
    /// 检查指定键的限流状态，包括封禁检查和具体限流算法检查
    /// Checks rate limiting status for specified key, including block check and specific algorithm check
    ///
    /// # 参数 / Parameters
    /// * `key` - 限流键 / Rate limiting key
    /// * `rule` - 限流规则 / Rate limiting rule
    ///
    /// # 返回值 / Returns
    /// * `Result<(), Box<Error>>` - 成功时允许访问，失败时返回限流错误
    ///   Success allows access, failure returns rate limiting error
    ///
    /// # 错误 / Errors
    /// * `Error::Blocked` - 请求被封禁 / Request is blocked
    /// * `Error::LimitExceeded` - 限流阈值超过 / Rate limit exceeded
    /// * `Error::Redis` - Redis操作失败 / Redis operation failed
    async fn check_rate_limit(&self, key: &str, rule: &LimitRule) -> Result<(), Box<Error>> {
        // 建立Redis连接 / Establish Redis connection
        let mut redis_conn = self.redis_client.get_multiplexed_tokio_connection().await.map_err(|e| Box::new(Error::from(e)))?;

        // 检查是否被封禁 / Check if blocked
        let block_key = format!("block:{}", key);
        let blocked: Option<i64> = redis_conn.get(&block_key).await.map_err(|e| Box::new(Error::from(e)))?;

        if let Some(block_until) = blocked {
            let now = chrono::Utc::now().timestamp();
            if block_until > now {
                return Err(Box::new(Error::Blocked { key: key.to_string(), remaining_time: (block_until - now) as u64 }));
            }
        }

        // 根据策略执行限流检查 / Execute rate limiting check based on strategy
        let limit_key = format!("limit:{}", key);
        let current_time = chrono::Utc::now().timestamp() as u64;

        match rule.strategy {
            LimitStrategy::TokenBucket => self.check_token_bucket(&mut redis_conn, &limit_key, rule, current_time).await,
            LimitStrategy::FixedWindow => self.check_fixed_window(&mut redis_conn, &limit_key, rule, current_time).await,
            LimitStrategy::SlidingWindow => self.check_sliding_window(&mut redis_conn, &limit_key, rule, current_time).await,
        }
    }

    /// 检查令牌桶限流 / Check token bucket rate limiting
    ///
    /// 使用令牌桶算法检查请求是否被允许
    /// Uses token bucket algorithm to check if request is allowed
    ///
    /// # 参数 / Parameters
    /// * `redis_conn` - Redis连接 / Redis connection
    /// * `key` - 限流键 / Rate limiting key
    /// * `rule` - 限流规则 / Rate limiting rule
    /// * `current_time` - 当前时间戳 / Current timestamp
    ///
    /// # 返回值 / Returns
    /// * `Result<(), Box<Error>>` - 成功时允许访问，失败时返回限流错误
    ///   Success allows access, failure returns rate limiting error
    async fn check_token_bucket(
        &self, redis_conn: &mut redis::aio::MultiplexedConnection, key: &str, rule: &LimitRule, current_time: u64,
    ) -> Result<(), Box<Error>> {
        // 执行Lua脚本检查令牌桶 / Execute Lua script to check token bucket
        let result: Vec<i64> = self.token_bucket_script
            .key(key)
            .arg(rule.capacity)
            .arg(rule.refill_rate)
            .arg(current_time)
            .arg(self.config.block_duration.as_secs() as i64)
            .invoke_async(redis_conn)
            .await
            .map_err(|e| Box::new(Error::from(e)))?;

        // 检查结果：0表示拒绝，1表示允许 / Check result: 0 means deny, 1 means allow
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

    /// 检查固定窗口限流 / Check fixed window rate limiting
    ///
    /// 使用固定窗口算法检查请求是否被允许
    /// Uses fixed window algorithm to check if request is allowed
    ///
    /// # 参数 / Parameters
    /// * `redis_conn` - Redis连接 / Redis connection
    /// * `key` - 限流键 / Rate limiting key
    /// * `rule` - 限流规则 / Rate limiting rule
    /// * `current_time` - 当前时间戳 / Current timestamp
    ///
    /// # 返回值 / Returns
    /// * `Result<(), Box<Error>>` - 成功时允许访问，失败时返回限流错误
    ///   Success allows access, failure returns rate limiting error
    async fn check_fixed_window(
        &self, redis_conn: &mut redis::aio::MultiplexedConnection, key: &str, rule: &LimitRule, current_time: u64,
    ) -> Result<(), Box<Error>> {
        let window_size = rule.window_size.as_secs();
        // 生成当前窗口的键 / Generate key for current window
        let window_key = format!("{}:{}", key, current_time / window_size);

        // 增加计数器 / Increment counter
        let count: i64 = redis_conn.incr(&window_key, 1i64).await.map_err(|e| Box::new(Error::from(e)))?;

        // 首次访问时设置过期时间 / Set expiration on first access
        if count == 1 {
            redis_conn.expire::<_, ()>(&window_key, window_size as i64).await.map_err(|e| Box::new(Error::from(e)))?;
        }

        // 检查是否超过限制 / Check if limit is exceeded
        if count as u64 > rule.capacity {
            // 设置封禁 / Set block
            let block_key = format!("block:{}", key);
            redis_conn.set::<_, _, ()>(&block_key, (current_time + self.config.block_duration.as_secs()) as i64).await.map_err(|e| Box::new(Error::from(e)))?;
            redis_conn.expire::<_, ()>(&block_key, self.config.block_duration.as_secs() as i64).await.map_err(|e| Box::new(Error::from(e)))?;

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

    /// 检查滑动窗口限流 / Check sliding window rate limiting
    ///
    /// 使用滑动窗口算法检查请求是否被允许
    /// Uses sliding window algorithm to check if request is allowed
    ///
    /// # 参数 / Parameters
    /// * `redis_conn` - Redis连接 / Redis connection
    /// * `key` - 限流键 / Rate limiting key
    /// * `rule` - 限流规则 / Rate limiting rule
    /// * `current_time` - 当前时间戳 / Current timestamp
    ///
    /// # 返回值 / Returns
    /// * `Result<(), Box<Error>>` - 成功时允许访问，失败时返回限流错误
    ///   Success allows access, failure returns rate limiting error
    async fn check_sliding_window(
        &self, redis_conn: &mut redis::aio::MultiplexedConnection, key: &str, rule: &LimitRule, current_time: u64,
    ) -> Result<(), Box<Error>> {
        let window_size = rule.window_size.as_secs();
        // 计算窗口截止时间 / Calculate window cutoff time
        let cutoff = current_time - window_size;

        // 执行Lua脚本检查滑动窗口 / Execute Lua script to check sliding window
        let result: Vec<i64> = self.sliding_window_script
            .key(key)
            .arg(cutoff)
            .arg(rule.capacity)
            .arg(current_time)
            .arg(self.config.block_duration.as_secs() as i64)
            .invoke_async(redis_conn)
            .await
            .map_err(|e| Box::new(Error::from(e)))?;

        // 检查结果：0表示拒绝，1表示允许 / Check result: 0 means deny, 1 means allow
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
        let limit_key = self
            .extract_limit_key(&parts)
            .await
            .ok_or_else(|| Box::new(Error::InternalError("Could not extract limit key".to_string())))?;

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
            token_bucket_script: self.token_bucket_script.clone(),
            sliding_window_script: self.sliding_window_script.clone(),
        }
    }
}

impl std::fmt::Debug for Limitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Limitor")
            .field("config", &self.config)
            .field("redis_client", &self.redis_client)
            .field("token_bucket_script", &"Script")
            .field("sliding_window_script", &"Script")
            .finish()
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
                    context.make_abort_with_response(Limitor::middleware_name(), error.to_string(), out.into_response());
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
        assert!(matches!(custom_pattern, LimitKey::Custom(_)));
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
        let valid_config = LimitorConfig::new("redis://localhost:6379".to_string()).default_limit(100, Duration::from_secs(60));
        assert!(valid_config.validate().is_ok());

        let invalid_config = LimitorConfig::new("invalid://localhost:6379".to_string());
        assert!(invalid_config.validate().is_err());
    }
}
