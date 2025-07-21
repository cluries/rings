//! # 签名验证中间件模块
//! 
//! 本模块提供基于 HMAC-SHA1 的 HTTP 请求签名验证功能，包括：
//! 
//! ## 核心功能
//! 
//! - **请求签名验证**: 使用 HMAC-SHA1 算法验证请求签名的完整性
//! - **防重放攻击**: 通过 nonce（随机数）机制防止请求重放攻击
//! - **时间戳验证**: 验证请求时间戳，防止过期请求
//! - **性能监控**: 全面的性能指标收集和监控功能
//! - **灵活配置**: 支持排除规则、自定义超时时间等配置
//! - **开发支持**: 提供开发后门功能，便于开发和测试
//! 
//! ## 签名算法
//! 
//! 签名基于以下信息生成：
//! ```text
//! 签名字符串 = HTTP方法,请求路径,{用户ID,时间戳,随机数}[,{查询参数}][,请求体JSON]
//! 签名 = HMAC-SHA1(签名字符串, 用户密钥)
//! ```
//! 
//! ## 请求头格式
//! 
//! 客户端需要在请求头中包含以下信息：
//! - `X-U`: 用户ID
//! - `X-T`: Unix 时间戳
//! - `X-R`: 随机数（nonce）
//! - `X-S`: 签名值（40位十六进制字符串）
//! - `X-DEVELOPMENT-SKIP`: 开发后门密钥（可选）
//! 
//! ## 快速开始
//! 
//! ```rust
//! use std::sync::Arc;
//! use crate::web::middleware::signator::{SignatorMiddleware, KeyLoader};
//! 
//! // 1. 创建密钥加载器
//! let key_loader: KeyLoader = Arc::new(|user_id: String| {
//!     Box::pin(async move {
//!         // 从数据库或配置中加载用户密钥
//!         Ok(format!("secret_key_for_{}", user_id))
//!     })
//! });
//! 
//! // 2. 创建签名验证中间件
//! let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?
//!     .with_nonce_lifetime(300)  // 5分钟 nonce 有效期
//!     .with_excludes(vec![
//!         |parts| parts.uri.path() == "/health",
//!         |parts| parts.uri.path().starts_with("/public/"),
//!     ]);
//! 
//! // 3. 创建性能监控器
//! let monitor = SignatorMonitor::new(middleware.clone());
//! let _report_task = monitor.start_periodic_reporting(60); // 每分钟报告
//! 
//! // 4. 在 Axum 应用中使用
//! let app = Router::new()
//!     .route("/api/users", get(get_users))
//!     .layer(middleware);
//! ```
//! 
//! ## 客户端签名示例
//! 
//! ```javascript
//! // JavaScript 客户端签名示例
//! function generateSignature(method, path, userId, timestamp, nonce, query, body, secretKey) {
//!     let signatureString = `${method.toUpperCase()},${path},{${userId},${timestamp},${nonce}}`;
//!     
//!     // 添加查询参数（按键名排序）
//!     if (query && Object.keys(query).length > 0) {
//!         const sortedKeys = Object.keys(query).sort();
//!         const queryString = sortedKeys.map(key => `${key}=${query[key]}`).join(',');
//!         signatureString += `,{${queryString}}`;
//!     }
//!     
//!     // 添加请求体
//!     if (body) {
//!         signatureString += `,${JSON.stringify(body)}`;
//!     }
//!     
//!     return hmacSha1(signatureString, secretKey);
//! }
//! 
//! // 发送签名请求
//! const headers = {
//!     'X-U': userId,
//!     'X-T': timestamp.toString(),
//!     'X-R': nonce,
//!     'X-S': signature,
//!     'Content-Type': 'application/json'
//! };
//! ```
//! 
//! ## 性能监控
//! 
//! ```rust
//! let monitor = SignatorMonitor::new(middleware);
//! 
//! // 获取性能报告
//! let report = monitor.get_performance_report();
//! println!("成功率: {:.2}%", report.success_rate);
//! println!("平均处理时间: {:.2}ms", report.avg_processing_time_ms);
//! 
//! // 健康检查
//! let health = monitor.health_check();
//! if !health.is_healthy() {
//!     eprintln!("系统存在性能问题: {:?}", health.get_issues());
//! }
//! ```
//! 
//! ## 安全注意事项
//! 
//! - 密钥应该安全存储，不要硬编码在代码中
//! - nonce 有效期不宜过长，建议 5-30 分钟
//! - 时间戳验证范围建议在 5 分钟内
//! - 开发后门功能仅应在开发环境使用
//! - 定期监控性能指标，及时发现异常

use super::*;
use crate::erx::{Erx, Layouted, LayoutedC};
use crate::tools::hash;
use crate::web::api::Out;
use crate::web::request::clone_request;
use crate::web::url::parse_query;

use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::fmt;
use tokio::time::{timeout, Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// 签名验证相关的错误类型
/// 
/// # 用法示例
/// ```rust
/// use crate::web::middleware::signator::SignatorError;
/// 
/// // 创建不同类型的错误
/// let parse_error = SignatorError::PayloadParse("Invalid JSON format".to_string());
/// let format_error = SignatorError::SignatureFormat("Missing required headers".to_string());
/// let key_error = SignatorError::KeyLoad("Failed to load user key".to_string());
/// 
/// // 错误可以转换为 HTTP 响应
/// let response = parse_error.into_response();
/// 
/// // 错误实现了 Display trait，可以直接打印
/// println!("Error: {}", format_error);
/// ```
#[derive(Debug, Clone)]
pub enum SignatorError {
    /// 载荷解析错误 - 当请求体无法解析为有效 JSON 或格式不正确时
    PayloadParse(String),
    /// 签名格式错误 - 当签名头部缺失或格式不正确时
    SignatureFormat(String),
    /// 密钥加载错误 - 当无法从密钥加载器获取用户密钥时
    KeyLoad(String),
    /// 签名验证失败 - 当计算的签名与客户端提供的签名不匹配时
    SignatureInvalid { error: String, debug: Debug },
    /// 随机数重复使用 - 当检测到 nonce 重放攻击时
    NonceReplay(String),
    /// Redis 连接错误 - 当 Redis 操作失败时
    RedisConnection(String),
    /// 时间戳验证失败 - 当请求时间戳超出允许范围时
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

/// 密钥加载器类型定义
/// 
/// 这是一个异步函数类型，用于根据用户ID加载对应的签名密钥。
/// 
/// # 用法示例
/// ```rust
/// use std::sync::Arc;
/// use std::pin::Pin;
/// use std::future::Future;
/// use crate::web::middleware::signator::KeyLoader;
/// 
/// // 创建一个简单的密钥加载器
/// let key_loader: KeyLoader = Arc::new(|user_id: String| {
///     Box::pin(async move {
///         // 从数据库或配置中加载用户密钥
///         match user_id.as_str() {
///             "user123" => Ok("secret_key_123".to_string()),
///             "user456" => Ok("secret_key_456".to_string()),
///             _ => Err(crate::erx::Erx::new("User not found")),
///         }
///     })
/// });
/// 
/// // 创建一个从 Redis 加载密钥的加载器
/// let redis_key_loader: KeyLoader = Arc::new(|user_id: String| {
///     Box::pin(async move {
///         let mut conn = redis_client.get_connection().await?;
///         let key: String = conn.get(format!("user:{}:key", user_id)).await?;
///         Ok(key)
///     })
/// });
/// ```
pub type KeyLoader = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output=Result<String, Erx>> + Send>> + Send + Sync>;

/// 性能指标收集器
/// 
/// 用于收集和统计签名验证中间件的各种性能指标，包括请求统计、错误统计、
/// 性能指标和缓存统计等。所有操作都是线程安全的。
/// 
/// # 用法示例
/// ```rust
/// use crate::web::middleware::signator::{SignatorMetrics, SignatorError};
/// 
/// let metrics = SignatorMetrics::new();
/// 
/// // 记录请求开始
/// metrics.record_request_start();
/// 
/// // 记录成功请求（处理时间100ms）
/// metrics.record_request_success(100);
/// 
/// // 记录失败请求
/// let error = SignatorError::PayloadParse("Invalid JSON".to_string());
/// metrics.record_request_failure(&error, 150);
/// 
/// // 记录各种性能指标
/// metrics.record_key_load_time(50);
/// metrics.record_redis_operation_time(30);
/// metrics.record_signature_validation_time(20);
/// 
/// // 记录缓存统计
/// metrics.record_nonce_cache_hit();
/// metrics.record_nonce_cache_miss();
/// 
/// // 获取性能报告
/// let report = metrics.get_performance_report();
/// println!("Success rate: {:.2}%", report.success_rate);
/// println!("Average processing time: {:.2}ms", report.avg_processing_time_ms);
/// println!("Cache hit rate: {:.2}%", report.cache_hit_rate);
/// ```
#[derive(Debug, Default)]
pub struct SignatorMetrics {
    // 请求计数器
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    
    // 错误类型计数
    pub payload_parse_errors: AtomicU32,
    pub signature_format_errors: AtomicU32,
    pub key_load_errors: AtomicU32,
    pub signature_invalid_errors: AtomicU32,
    pub nonce_replay_errors: AtomicU32,
    pub redis_connection_errors: AtomicU32,
    pub timestamp_invalid_errors: AtomicU32,
    
    // 性能指标
    pub total_processing_time_ms: AtomicU64,
    pub key_load_time_ms: AtomicU64,
    pub redis_operation_time_ms: AtomicU64,
    pub signature_validation_time_ms: AtomicU64,
    
    // Redis 操作统计
    pub redis_script_executions: AtomicU64,
    pub redis_connection_acquisitions: AtomicU64,
    pub redis_timeouts: AtomicU32,
    
    // 缓存命中率
    pub nonce_cache_hits: AtomicU64,
    pub nonce_cache_misses: AtomicU64,
}

impl SignatorMetrics {
    /// 创建新的性能指标收集器实例
    /// 
    /// # 返回值
    /// 返回一个 Arc 包装的 SignatorMetrics 实例，可以在多个线程间安全共享
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// let metrics_clone = Arc::clone(&metrics);
    /// // 可以在不同线程中使用 metrics 和 metrics_clone
    /// ```
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    
    /// 记录请求开始
    /// 
    /// 增加总请求计数器。通常在请求处理开始时调用。
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// metrics.record_request_start();
    /// ```
    pub fn record_request_start(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录请求成功
    /// 
    /// 增加成功请求计数器并累加处理时间。
    /// 
    /// # 参数
    /// * `processing_time_ms` - 请求处理时间（毫秒）
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// let start_time = std::time::Instant::now();
    /// // ... 处理请求 ...
    /// let processing_time = start_time.elapsed().as_millis() as u64;
    /// metrics.record_request_success(processing_time);
    /// ```
    pub fn record_request_success(&self, processing_time_ms: u64) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time_ms.fetch_add(processing_time_ms, Ordering::Relaxed);
    }
    
    /// 记录请求失败
    /// 
    /// 增加失败请求计数器，累加处理时间，并根据错误类型增加相应的错误计数器。
    /// 
    /// # 参数
    /// * `error` - 发生的错误
    /// * `processing_time_ms` - 请求处理时间（毫秒）
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// let error = SignatorError::PayloadParse("Invalid JSON".to_string());
    /// metrics.record_request_failure(&error, 150);
    /// ```
    pub fn record_request_failure(&self, error: &SignatorError, processing_time_ms: u64) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time_ms.fetch_add(processing_time_ms, Ordering::Relaxed);
        
        // 记录具体错误类型
        match error {
            SignatorError::PayloadParse(_) => {
                self.payload_parse_errors.fetch_add(1, Ordering::Relaxed);
            }
            SignatorError::SignatureFormat(_) => {
                self.signature_format_errors.fetch_add(1, Ordering::Relaxed);
            }
            SignatorError::KeyLoad(_) => {
                self.key_load_errors.fetch_add(1, Ordering::Relaxed);
            }
            SignatorError::SignatureInvalid { .. } => {
                self.signature_invalid_errors.fetch_add(1, Ordering::Relaxed);
            }
            SignatorError::NonceReplay(_) => {
                self.nonce_replay_errors.fetch_add(1, Ordering::Relaxed);
            }
            SignatorError::RedisConnection(_) => {
                self.redis_connection_errors.fetch_add(1, Ordering::Relaxed);
            }
            SignatorError::TimestampInvalid(_) => {
                self.timestamp_invalid_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    /// 记录密钥加载时间
    /// 
    /// 累加密钥加载操作的总时间，用于计算平均密钥加载时间。
    /// 
    /// # 参数
    /// * `time_ms` - 密钥加载时间（毫秒）
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// let start = std::time::Instant::now();
    /// // ... 加载密钥 ...
    /// let load_time = start.elapsed().as_millis() as u64;
    /// metrics.record_key_load_time(load_time);
    /// ```
    pub fn record_key_load_time(&self, time_ms: u64) {
        self.key_load_time_ms.fetch_add(time_ms, Ordering::Relaxed);
    }
    
    /// 记录 Redis 操作时间
    /// 
    /// 累加 Redis 操作的总时间并增加脚本执行计数器。
    /// 
    /// # 参数
    /// * `time_ms` - Redis 操作时间（毫秒）
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// let start = std::time::Instant::now();
    /// // ... 执行 Redis 操作 ...
    /// let redis_time = start.elapsed().as_millis() as u64;
    /// metrics.record_redis_operation_time(redis_time);
    /// ```
    pub fn record_redis_operation_time(&self, time_ms: u64) {
        self.redis_operation_time_ms.fetch_add(time_ms, Ordering::Relaxed);
        self.redis_script_executions.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录签名验证时间
    /// 
    /// 累加签名验证操作的总时间。
    /// 
    /// # 参数
    /// * `time_ms` - 签名验证时间（毫秒）
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// let start = std::time::Instant::now();
    /// // ... 验证签名 ...
    /// let validation_time = start.elapsed().as_millis() as u64;
    /// metrics.record_signature_validation_time(validation_time);
    /// ```
    pub fn record_signature_validation_time(&self, time_ms: u64) {
        self.signature_validation_time_ms.fetch_add(time_ms, Ordering::Relaxed);
    }
    
    /// 记录 Redis 连接获取
    /// 
    /// 增加 Redis 连接获取次数计数器。每次从连接池获取连接时调用。
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// // 在获取 Redis 连接前调用
    /// metrics.record_redis_connection_acquisition();
    /// let conn = redis_manager.get_connection().await?;
    /// ```
    pub fn record_redis_connection_acquisition(&self) {
        self.redis_connection_acquisitions.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录 Redis 超时
    /// 
    /// 增加 Redis 操作超时次数计数器。当 Redis 操作超时时调用。
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// match timeout(Duration::from_secs(5), redis_operation).await {
    ///     Ok(result) => { /* 处理结果 */ },
    ///     Err(_) => {
    ///         metrics.record_redis_timeout();
    ///         // 处理超时
    ///     }
    /// }
    /// ```
    pub fn record_redis_timeout(&self) {
        self.redis_timeouts.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录 nonce 缓存命中
    /// 
    /// 增加 nonce 缓存命中次数计数器。当在 Redis 中找到重复的 nonce 时调用。
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// // 当检测到重复 nonce 时
    /// if nonce_exists_in_cache {
    ///     metrics.record_nonce_cache_hit();
    /// }
    /// ```
    pub fn record_nonce_cache_hit(&self) {
        self.nonce_cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录 nonce 缓存未命中
    /// 
    /// 增加 nonce 缓存未命中次数计数器。当 nonce 是新的（未在缓存中找到）时调用。
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// // 当 nonce 是新的时
    /// if !nonce_exists_in_cache {
    ///     metrics.record_nonce_cache_miss();
    /// }
    /// ```
    pub fn record_nonce_cache_miss(&self) {
        self.nonce_cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取性能统计报告
    /// 
    /// 生成包含所有性能指标的详细报告，包括成功率、平均处理时间、
    /// 缓存命中率、错误分类统计和性能分类统计。
    /// 
    /// # 返回值
    /// 返回 `PerformanceReport` 结构体，包含所有统计信息
    /// 
    /// # 用法示例
    /// ```rust
    /// let metrics = SignatorMetrics::new();
    /// // ... 记录一些指标 ...
    /// 
    /// let report = metrics.get_performance_report();
    /// println!("总请求数: {}", report.total_requests);
    /// println!("成功率: {:.2}%", report.success_rate);
    /// println!("平均处理时间: {:.2}ms", report.avg_processing_time_ms);
    /// println!("缓存命中率: {:.2}%", report.cache_hit_rate);
    /// 
    /// // 查看错误分类
    /// println!("载荷解析错误: {}", report.error_breakdown.payload_parse_errors);
    /// println!("签名格式错误: {}", report.error_breakdown.signature_format_errors);
    /// 
    /// // 查看性能分类
    /// println!("平均密钥加载时间: {:.2}ms", report.performance_breakdown.avg_key_load_time_ms);
    /// println!("平均Redis操作时间: {:.2}ms", report.performance_breakdown.avg_redis_operation_time_ms);
    /// ```
    pub fn get_performance_report(&self) -> PerformanceReport {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.successful_requests.load(Ordering::Relaxed);
        let failed_requests = self.failed_requests.load(Ordering::Relaxed);
        let total_processing_time = self.total_processing_time_ms.load(Ordering::Relaxed);
        
        let success_rate = if total_requests > 0 {
            (successful_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_processing_time = if total_requests > 0 {
            total_processing_time as f64 / total_requests as f64
        } else {
            0.0
        };
        
        let cache_hit_rate = {
            let hits = self.nonce_cache_hits.load(Ordering::Relaxed);
            let misses = self.nonce_cache_misses.load(Ordering::Relaxed);
            let total_cache_ops = hits + misses;
            if total_cache_ops > 0 {
                (hits as f64 / total_cache_ops as f64) * 100.0
            } else {
                0.0
            }
        };
        
        PerformanceReport {
            total_requests,
            successful_requests,
            failed_requests,
            success_rate,
            avg_processing_time_ms: avg_processing_time,
            cache_hit_rate,
            error_breakdown: ErrorBreakdown {
                payload_parse_errors: self.payload_parse_errors.load(Ordering::Relaxed),
                signature_format_errors: self.signature_format_errors.load(Ordering::Relaxed),
                key_load_errors: self.key_load_errors.load(Ordering::Relaxed),
                signature_invalid_errors: self.signature_invalid_errors.load(Ordering::Relaxed),
                nonce_replay_errors: self.nonce_replay_errors.load(Ordering::Relaxed),
                redis_connection_errors: self.redis_connection_errors.load(Ordering::Relaxed),
                timestamp_invalid_errors: self.timestamp_invalid_errors.load(Ordering::Relaxed),
            },
            performance_breakdown: PerformanceBreakdown {
                avg_key_load_time_ms: if successful_requests > 0 {
                    self.key_load_time_ms.load(Ordering::Relaxed) as f64 / successful_requests as f64
                } else {
                    0.0
                },
                avg_redis_operation_time_ms: if self.redis_script_executions.load(Ordering::Relaxed) > 0 {
                    self.redis_operation_time_ms.load(Ordering::Relaxed) as f64 / 
                    self.redis_script_executions.load(Ordering::Relaxed) as f64
                } else {
                    0.0
                },
                avg_signature_validation_time_ms: if successful_requests > 0 {
                    self.signature_validation_time_ms.load(Ordering::Relaxed) as f64 / successful_requests as f64
                } else {
                    0.0
                },
                redis_timeouts: self.redis_timeouts.load(Ordering::Relaxed),
            },
        }
    }
}

/// 性能报告结构
#[derive(Debug, Serialize)]
pub struct PerformanceReport {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_processing_time_ms: f64,
    pub cache_hit_rate: f64,
    pub error_breakdown: ErrorBreakdown,
    pub performance_breakdown: PerformanceBreakdown,
}

/// 错误分类统计
#[derive(Debug, Serialize)]
pub struct ErrorBreakdown {
    pub payload_parse_errors: u32,
    pub signature_format_errors: u32,
    pub key_load_errors: u32,
    pub signature_invalid_errors: u32,
    pub nonce_replay_errors: u32,
    pub redis_connection_errors: u32,
    pub timestamp_invalid_errors: u32,
}

/// 性能分类统计
#[derive(Debug, Serialize)]
pub struct PerformanceBreakdown {
    pub avg_key_load_time_ms: f64,
    pub avg_redis_operation_time_ms: f64,
    pub avg_signature_validation_time_ms: f64,
    pub redis_timeouts: u32,
}

/// Redis 连接池管理器
/// 
/// 管理 Redis 连接的创建和复用，提供高效的连接池功能。
/// 
/// # 用法示例
/// ```rust
/// use crate::web::middleware::signator::RedisConnectionManager;
/// 
/// // 创建连接管理器
/// let manager = RedisConnectionManager::new("redis://localhost:6379")?;
/// 
/// // 获取连接
/// let mut conn = manager.get_connection().await?;
/// 
/// // 使用连接执行 Redis 操作
/// let result: String = redis::cmd("GET").arg("key").query_async(&mut conn).await?;
/// ```
#[derive(Clone)]
pub struct RedisConnectionManager {
    client: redis::Client,
}

impl RedisConnectionManager {
    /// 创建新的 Redis 连接管理器
    /// 
    /// # 参数
    /// * `redis_url` - Redis 连接 URL，格式如 "redis://localhost:6379"
    /// 
    /// # 返回值
    /// 成功时返回 `RedisConnectionManager` 实例，失败时返回 `redis::RedisError`
    /// 
    /// # 用法示例
    /// ```rust
    /// let manager = RedisConnectionManager::new("redis://localhost:6379")?;
    /// let manager_with_auth = RedisConnectionManager::new("redis://:password@localhost:6379")?;
    /// let manager_with_db = RedisConnectionManager::new("redis://localhost:6379/1")?;
    /// ```
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(RedisConnectionManager { client })
    }

    /// 获取 Redis 连接
    /// 
    /// 从连接池获取一个多路复用的异步连接。连接是可复用的，
    /// 多个操作可以并发使用同一个连接。
    /// 
    /// # 返回值
    /// 成功时返回 `MultiplexedConnection`，失败时返回 `redis::RedisError`
    /// 
    /// # 用法示例
    /// ```rust
    /// let manager = RedisConnectionManager::new("redis://localhost:6379")?;
    /// let mut conn = manager.get_connection().await?;
    /// 
    /// // 执行 Redis 命令
    /// let _: () = redis::cmd("SET").arg("key").arg("value").query_async(&mut conn).await?;
    /// let value: String = redis::cmd("GET").arg("key").query_async(&mut conn).await?;
    /// ```
    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
        self.client.get_multiplexed_tokio_connection().await
    }
}

/// 签名验证中间件
/// 
/// 提供基于 HMAC-SHA1 的请求签名验证功能，包括：
/// - 请求签名验证
/// - nonce 防重放攻击
/// - 时间戳验证
/// - 性能监控
/// - 灵活的排除规则
/// 
/// # 用法示例
/// ```rust
/// use std::sync::Arc;
/// use crate::web::middleware::signator::{SignatorMiddleware, KeyLoader};
/// 
/// // 创建密钥加载器
/// let key_loader: KeyLoader = Arc::new(|user_id: String| {
///     Box::pin(async move {
///         // 从数据库加载用户密钥
///         Ok(format!("secret_key_for_{}", user_id))
///     })
/// });
/// 
/// // 创建基本的签名中间件
/// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?;
/// 
/// // 创建带有排除规则的中间件
/// let exclude_health = |parts: &axum::http::request::Parts| -> bool {
///     parts.uri.path() == "/health"
/// };
/// 
/// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?
///     .with_excludes(vec![exclude_health])
///     .with_nonce_lifetime(600); // 10分钟
/// 
/// // 创建带有开发后门的中间件
/// let dev_middleware = SignatorMiddleware::with_rear(
///     "redis://localhost:6379", 
///     key_loader, 
///     "dev_backdoor_key".to_string()
/// )?;
/// ```
pub struct SignatorMiddleware {
    rear: String, // 后门，开发时候方便用
    excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>,
    nonce_lifetime: i64,
    key_loader: KeyLoader,
    redis_manager: RedisConnectionManager,
    metrics: Arc<SignatorMetrics>,
}

impl Clone for SignatorMiddleware {
    fn clone(&self) -> Self {
        SignatorMiddleware {
            rear: self.rear.clone(),
            excludes: self.excludes.clone(),
            nonce_lifetime: self.nonce_lifetime,
            key_loader: Arc::clone(&self.key_loader),
            redis_manager: self.redis_manager.clone(),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

impl SignatorMiddleware {
    /// 创建新的签名验证中间件
    /// 
    /// 使用默认配置创建中间件实例，不包含开发后门。
    /// 
    /// # 参数
    /// * `redis_url` - Redis 连接 URL
    /// * `key_loader` - 密钥加载器，用于根据用户ID获取签名密钥
    /// 
    /// # 返回值
    /// 成功时返回中间件实例，失败时返回 `SignatorError`
    /// 
    /// # 用法示例
    /// ```rust
    /// let key_loader: KeyLoader = Arc::new(|user_id: String| {
    ///     Box::pin(async move {
    ///         Ok(format!("secret_key_for_{}", user_id))
    ///     })
    /// });
    /// 
    /// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?;
    /// ```
    pub fn new(redis_url: &str, key_loader: KeyLoader) -> Result<Self, SignatorError> {
        Self::with_rear(redis_url, key_loader, String::default())
    }

    /// 创建带有开发后门的签名验证中间件
    /// 
    /// 创建包含开发后门的中间件实例。当请求头中包含指定的后门密钥时，
    /// 即使签名验证失败也会允许请求通过。
    /// 
    /// # 参数
    /// * `redis_url` - Redis 连接 URL
    /// * `key_loader` - 密钥加载器
    /// * `rear` - 开发后门密钥，通过 X-DEVELOPMENT-SKIP 头部传递
    /// 
    /// # 返回值
    /// 成功时返回中间件实例，失败时返回 `SignatorError`
    /// 
    /// # 用法示例
    /// ```rust
    /// let middleware = SignatorMiddleware::with_rear(
    ///     "redis://localhost:6379",
    ///     key_loader,
    ///     "dev_secret_123".to_string()
    /// )?;
    /// 
    /// // 客户端可以通过以下方式跳过签名验证：
    /// // curl -H "X-DEVELOPMENT-SKIP: dev_secret_123" http://api.example.com/endpoint
    /// ```
    pub fn with_rear(redis_url: &str, key_loader: KeyLoader, rear: String) -> Result<Self, SignatorError> {
        let redis_manager = RedisConnectionManager::new(redis_url)
            .map_err(|e| SignatorError::RedisConnection(format!("Failed to create Redis manager: {}", e)))?;

        Ok(SignatorMiddleware {
            rear,
            excludes: vec![],
            nonce_lifetime: DEFAULT_RAND_LIFE,
            key_loader,
            redis_manager,
            metrics: SignatorMetrics::new(),
        })
    }

    /// 获取性能指标收集器
    /// 
    /// 返回性能指标收集器的共享引用，可用于外部监控和统计。
    /// 
    /// # 返回值
    /// 返回 `Arc<SignatorMetrics>` 共享指针
    /// 
    /// # 用法示例
    /// ```rust
    /// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?;
    /// let metrics = middleware.get_metrics();
    /// 
    /// // 在其他地方使用指标
    /// let report = metrics.get_performance_report();
    /// println!("Success rate: {:.2}%", report.success_rate);
    /// ```
    pub fn get_metrics(&self) -> Arc<SignatorMetrics> {
        Arc::clone(&self.metrics)
    }

    /// 获取性能报告
    /// 
    /// 生成当前的性能统计报告，包含所有性能指标。
    /// 
    /// # 返回值
    /// 返回 `PerformanceReport` 结构体
    /// 
    /// # 用法示例
    /// ```rust
    /// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?;
    /// let report = middleware.get_performance_report();
    /// 
    /// println!("总请求数: {}", report.total_requests);
    /// println!("成功率: {:.2}%", report.success_rate);
    /// ```
    pub fn get_performance_report(&self) -> PerformanceReport {
        self.metrics.get_performance_report()
    }

    /// 添加排除规则
    /// 
    /// 向现有的中间件实例添加一个排除规则。匹配规则的请求将跳过签名验证。
    /// 
    /// # 参数
    /// * `exclude` - 排除规则函数，接收请求部分并返回是否排除
    /// 
    /// # 返回值
    /// 返回自身的可变引用，支持链式调用
    /// 
    /// # 用法示例
    /// ```rust
    /// let mut middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?;
    /// 
    /// middleware.add_exclude(|parts| parts.uri.path() == "/health")
    ///           .add_exclude(|parts| parts.uri.path().starts_with("/public/"));
    /// ```
    pub fn add_exclude(&mut self, exclude: fn(parts: &axum::http::request::Parts) -> bool) -> &mut Self {
        self.excludes.push(exclude);
        self
    }

    /// 设置排除规则列表
    /// 
    /// 一次性设置多个排除规则。匹配任一规则的请求将跳过签名验证。
    /// 
    /// # 参数
    /// * `excludes` - 排除规则函数列表
    /// 
    /// # 返回值
    /// 返回修改后的中间件实例
    /// 
    /// # 用法示例
    /// ```rust
    /// let exclude_health = |parts: &axum::http::request::Parts| -> bool {
    ///     parts.uri.path() == "/health"
    /// };
    /// 
    /// let exclude_public = |parts: &axum::http::request::Parts| -> bool {
    ///     parts.uri.path().starts_with("/public/")
    /// };
    /// 
    /// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?
    ///     .with_excludes(vec![exclude_health, exclude_public]);
    /// ```
    pub fn with_excludes(mut self, excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>) -> Self {
        self.excludes.extend(excludes);
        self
    }

    /// 设置 nonce 生命周期
    /// 
    /// 设置 nonce（随机数）的有效期，用于防重放攻击。
    /// 超过此时间的 nonce 将被自动清理。
    /// 
    /// # 参数
    /// * `lifetime` - nonce 生命周期（秒）
    /// 
    /// # 返回值
    /// 返回修改后的中间件实例
    /// 
    /// # 用法示例
    /// ```rust
    /// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?
    ///     .with_nonce_lifetime(600); // 10分钟
    /// 
    /// // 常用的生命周期设置：
    /// // .with_nonce_lifetime(300)   // 5分钟（默认）
    /// // .with_nonce_lifetime(600)   // 10分钟
    /// // .with_nonce_lifetime(1800)  // 30分钟
    /// ```
    pub fn with_nonce_lifetime(mut self, lifetime: i64) -> Self {
        self.nonce_lifetime = lifetime;
        self
    }

    fn should_exclude(&self, parts: &Parts) -> bool {
        self.excludes.iter().any(|exclude| exclude(parts))
    }

    async fn validate_signature(&self, request: Request) -> Result<Request, Response> {
        let start_time = Instant::now();
        self.metrics.record_request_start();

        let result = self.validate_signature_internal(request).await;
        
        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        match &result {
            Ok(_) => {
                self.metrics.record_request_success(processing_time_ms);
            }
            Err(response) => {
                // 尝试从响应中提取错误信息
                if let Some(error) = self.extract_error_from_response(response) {
                    self.metrics.record_request_failure(&error, processing_time_ms);
                } else {
                    // 如果无法提取错误，记录为通用失败
                    self.metrics.record_request_failure(
                        &SignatorError::SignatureFormat("Unknown error".to_string()), 
                        processing_time_ms
                    );
                }
            }
        }
        
        result
    }

    async fn validate_signature_internal(&self, request: Request) -> Result<Request, Response> {
        let (payload_request, mut request) = clone_request(request).await;

        // 并行处理载荷解析和验证
        let payload = Payload::from_request(payload_request).await
            .map_err(|e| e.into_response())?;
        
        // 同步验证，无需 await
        payload.guard()
            .map_err(|e| e.into_response())?;

        let user_id = payload.user_id();
        let nonce = payload.nonce();

        // 并行执行密钥加载和随机数检查，带性能监控
        let key_start = Instant::now();
        let key_future = {
            let loader = Arc::clone(&self.key_loader);
            let user_id = user_id.to_string();
            timeout(
                Duration::from_millis(KEY_LOAD_TIMEOUT_MS),
                async move { loader(user_id).await }
            )
        };

        let redis_start = Instant::now();
        let nonce_future = timeout(
            Duration::from_millis(REDIS_TIMEOUT_MS),
            self.rand_guard(user_id, nonce)
        );

        // 使用 tokio::join! 并行执行
        let (key_timeout_result, nonce_timeout_result) = tokio::join!(
            key_future,
            nonce_future
        );

        // 记录密钥加载时间
        let key_load_time = key_start.elapsed().as_millis() as u64;
        self.metrics.record_key_load_time(key_load_time);

        // 记录 Redis 操作时间
        let redis_time = redis_start.elapsed().as_millis() as u64;
        self.metrics.record_redis_operation_time(redis_time);

        let key = match key_timeout_result {
            Ok(Ok(key)) => key,
            Ok(Err(e)) => {
                return Err(SignatorError::KeyLoad(e.message_string()).into_response());
            }
            Err(_) => {
                self.metrics.record_redis_timeout();
                return Err(SignatorError::KeyLoad("Key loading timeout".to_string()).into_response());
            }
        };

        match nonce_timeout_result {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                return Err(e.into_response());
            }
            Err(_) => {
                self.metrics.record_redis_timeout();
                return Err(SignatorError::RedisConnection("Redis operation timeout".to_string()).into_response());
            }
        }

        // 验证签名（同步操作）
        let signature_start = Instant::now();
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
        
        // 记录签名验证时间
        let signature_time = signature_start.elapsed().as_millis() as u64;
        self.metrics.record_signature_validation_time(signature_time);

        // 设置上下文
        use crate::web::context::Context;
        let context = Context::new(user_id.to_string());
        request.extensions_mut().insert(context);

        Ok(request)
    }

    /// 从响应中提取错误信息（用于指标收集）
    fn extract_error_from_response(&self, _response: &Response) -> Option<SignatorError> {
        // 这里可以根据响应状态码或内容来推断错误类型
        // 为了简化，我们返回 None，让调用者处理
        None
    }

    async fn rand_guard(&self, user_id: &str, nonce: &str) -> Result<(), SignatorError> {
        // 记录 Redis 连接获取
        self.metrics.record_redis_connection_acquisition();
        
        let mut conn = self.redis_manager.get_connection().await
            .map_err(|e| SignatorError::RedisConnection(format!("Failed to get Redis connection: {}", e)))?;

        let key = format!("XR:{}", user_id);
        let current = chrono::Local::now().timestamp();

        // 使用 Redis 事务来确保原子性和性能
        let script = redis::Script::new(r#"
            local key = KEYS[1]
            local nonce = ARGV[1]
            local current = tonumber(ARGV[2])
            local lifetime = tonumber(ARGV[3])
            
            -- 检查是否存在重复的 nonce
            local score = redis.call('ZSCORE', key, nonce)
            if score then
                local diff = math.abs(current - tonumber(score))
                if diff < lifetime then
                    return {"err", "duplicate nonce", "cache_hit"}
                end
            end
            
            -- 添加新的 nonce 并清理过期数据
            redis.call('ZADD', key, current, nonce)
            redis.call('ZREMRANGEBYSCORE', key, '-inf', current - lifetime)
            redis.call('EXPIRE', key, lifetime)
            
            return {"ok", "success", "cache_miss"}
        "#);

        let result: redis::Value = script
            .key(&key)
            .arg(nonce)
            .arg(current)
            .arg(self.nonce_lifetime)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| SignatorError::RedisConnection(format!("Redis script execution failed: {}", e)))?;

        // 检查脚本执行结果并记录缓存统计
        match result {
            redis::Value::Array(ref values) => {
                if let Some(redis::Value::BulkString(ref data)) = values.get(0) {
                    if data == b"err" {
                        // 记录缓存命中（因为找到了重复的 nonce）
                        self.metrics.record_nonce_cache_hit();
                        if let Some(redis::Value::BulkString(ref msg)) = values.get(1) {
                            if msg == b"duplicate nonce" {
                                return Err(SignatorError::NonceReplay("duplicate nonce detected".to_string()));
                            }
                        }
                    } else if data == b"ok" {
                        // 记录缓存未命中（新的 nonce）
                        self.metrics.record_nonce_cache_miss();
                    }
                }
            }
            _ => {
                // 默认记录为缓存未命中
                self.metrics.record_nonce_cache_miss();
            }
        }

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
    
    // 异步操作超时时间
    pub const REDIS_TIMEOUT_MS: u64 = 5000; // 5秒
    pub const KEY_LOAD_TIMEOUT_MS: u64 = 3000; // 3秒
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
    #[allow(dead_code)]
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

    /// 生成用于签名的载荷字符串（优化版本）
    fn to_signature_string(&self, headers: &SignatureHeaders) -> String {
        // 预估容量以减少重新分配
        let estimated_capacity = self.method.len() + self.path.len() + 
            headers.user_id.len() + 20 + // timestamp 和其他字符
            self.queries.iter().map(|(k, v)| k.len() + v.len() + 2).sum::<usize>() +
            self.body.as_ref().map_or(0, |_| 100); // 估算 JSON 大小

        let mut buffer = String::with_capacity(estimated_capacity);
        
        // 构建基础载荷
        buffer.push_str(&self.method);
        buffer.push(',');
        buffer.push_str(&self.path);
        buffer.push_str(",{");
        buffer.push_str(&headers.user_id);
        buffer.push(',');
        buffer.push_str(&headers.timestamp.to_string());
        buffer.push(',');
        buffer.push_str(&headers.nonce);
        buffer.push('}');

        // 添加查询参数
        if !self.queries.is_empty() {
            let mut query_keys: Vec<&String> = self.queries.keys().collect();
            query_keys.sort_unstable();

            buffer.push_str(",{");
            for (i, key) in query_keys.iter().enumerate() {
                if i > 0 {
                    buffer.push(',');
                }
                buffer.push_str(key);
                buffer.push('=');
                buffer.push_str(self.queries.get(*key).unwrap());
            }
            buffer.push('}');
        }

        // 添加请求体
        if let Some(body) = &self.body {
            buffer.push(',');
            JsonFormatter::format_into(&mut buffer, body);
        }

        buffer
    }
}

/// 高性能 JSON 格式化器
struct JsonFormatter;

impl JsonFormatter {
    #[allow(dead_code)]
    fn format(value: &serde_json::Value) -> String {
        let mut buffer = String::new();
        Self::format_into(&mut buffer, value);
        buffer
    }

    fn format_into(buffer: &mut String, value: &serde_json::Value) {
        match value {
            serde_json::Value::Null => buffer.push_str("null"),
            serde_json::Value::Bool(b) => buffer.push_str(&b.to_string()),
            serde_json::Value::Number(n) => buffer.push_str(&n.to_string()),
            serde_json::Value::String(s) => buffer.push_str(s),
            serde_json::Value::Array(array) => Self::format_array_into(buffer, array),
            serde_json::Value::Object(object) => Self::format_object_into(buffer, object),
        }
    }

    fn format_array_into(buffer: &mut String, array: &[serde_json::Value]) {
        buffer.push('[');
        for (i, item) in array.iter().enumerate() {
            if i > 0 {
                buffer.push(',');
            }
            Self::format_into(buffer, item);
        }
        buffer.push(']');
    }

    fn format_object_into(buffer: &mut String, object: &serde_json::Map<String, serde_json::Value>) {
        let mut keys: Vec<&String> = object.keys().collect();
        keys.sort_unstable();
        
        buffer.push('{');
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                buffer.push(',');
            }
            buffer.push_str(key);
            buffer.push('=');
            Self::format_into(buffer, object.get(*key).unwrap());
        }
        buffer.push('}');
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
        let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)
            .expect("Failed to create middleware");
        
        assert_eq!(middleware.priority(), 85);
        assert_eq!(middleware.name(), "SignatorMiddleware");
        assert_eq!(middleware.path_pattern(), Some("/api/*"));
        assert_eq!(middleware.nonce_lifetime, DEFAULT_RAND_LIFE);
    }

    #[tokio::test]
    async fn test_signator_middleware_focus() {
        let key_loader = create_test_key_loader();
        let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)
            .expect("Failed to create middleware");
        
        // 创建测试请求
        let request = Request::builder()
            .method(axum::http::Method::GET)
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
            .expect("Failed to create middleware")
            .with_excludes(vec![exclude_health]);
        
        // 测试排除的路径
        let request = Request::builder()
            .method(axum::http::Method::GET)
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();

        let (parts, _) = request.into_parts();
        
        // 应该被排除，不处理
        assert!(!middleware.focus(&parts));
        
        // 测试非排除的路径
        let request = Request::builder()
            .method(axum::http::Method::GET)
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
        ).expect("Failed to create middleware")
        .with_nonce_lifetime(600);
        
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

    #[test]
    fn test_signator_metrics() {
        let metrics = SignatorMetrics::new();
        
        // 测试初始状态
        assert_eq!(metrics.total_requests.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.successful_requests.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.failed_requests.load(Ordering::Relaxed), 0);
        
        // 测试记录请求
        metrics.record_request_start();
        assert_eq!(metrics.total_requests.load(Ordering::Relaxed), 1);
        
        metrics.record_request_success(100);
        assert_eq!(metrics.successful_requests.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_processing_time_ms.load(Ordering::Relaxed), 100);
        
        // 测试记录错误
        let error = SignatorError::PayloadParse("test error".to_string());
        metrics.record_request_failure(&error, 50);
        assert_eq!(metrics.failed_requests.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.payload_parse_errors.load(Ordering::Relaxed), 1);
        
        // 测试性能报告
        let report = metrics.get_performance_report();
        assert_eq!(report.total_requests, 1); // 只有一个请求被记录为开始
        assert_eq!(report.successful_requests, 1);
        assert_eq!(report.failed_requests, 1);
        assert_eq!(report.error_breakdown.payload_parse_errors, 1);
    }

    #[test]
    fn test_performance_report() {
        let metrics = SignatorMetrics::new();
        
        // 模拟一些请求
        for _ in 0..10 {
            metrics.record_request_start();
            metrics.record_request_success(100);
        }
        
        for _ in 0..2 {
            metrics.record_request_start();
            let error = SignatorError::SignatureInvalid { 
                error: "test".to_string(), 
                debug: Debug::default() 
            };
            metrics.record_request_failure(&error, 150);
        }
        
        // 记录一些性能指标
        metrics.record_key_load_time(50);
        metrics.record_redis_operation_time(30);
        metrics.record_signature_validation_time(20);
        
        // 记录缓存统计
        metrics.record_nonce_cache_hit();
        metrics.record_nonce_cache_hit();
        metrics.record_nonce_cache_miss();
        
        let report = metrics.get_performance_report();
        
        assert_eq!(report.total_requests, 12);
        assert_eq!(report.successful_requests, 10);
        assert_eq!(report.failed_requests, 2);
        assert!((report.success_rate - 83.33).abs() < 0.1); // 10/12 ≈ 83.33%
        assert!((report.cache_hit_rate - 66.67).abs() < 0.1); // 2/3 ≈ 66.67%
        assert_eq!(report.error_breakdown.signature_invalid_errors, 2);
    }

    #[test]
    fn test_signator_monitor_health_check() {
        let key_loader = create_test_key_loader();
        let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)
            .expect("Failed to create middleware");
        
        let monitor = SignatorMonitor::new(middleware);
        let metrics = monitor.get_metrics();
        
        // 初始状态应该是健康的（没有足够的请求来判断）
        let initial_report = metrics.get_performance_report();
        println!("Initial report: total={}, success_rate={:.2}%, cache_hit_rate={:.2}%", 
                 initial_report.total_requests, initial_report.success_rate, initial_report.cache_hit_rate);
        
        let health = monitor.health_check();
        if !health.is_healthy() {
            if let Some(issues) = health.get_issues() {
                println!("Initial health issues: {:?}", issues);
            }
        }
        assert!(health.is_healthy());
        
        // 模拟一些成功请求和缓存操作
        for _ in 0..150 {
            metrics.record_request_start();
            metrics.record_request_success(100);
        }
        
        // 添加一些缓存统计以避免低缓存命中率问题
        for _ in 0..50 {
            metrics.record_nonce_cache_hit();
        }
        for _ in 0..10 {
            metrics.record_nonce_cache_miss();
        }
        
        // 健康状态应该仍然良好
        let health = monitor.health_check();
        assert!(health.is_healthy());
        
        // 添加一些失败请求，使成功率降到 75%
        for _ in 0..50 {
            metrics.record_request_start();
            let error = SignatorError::PayloadParse("test".to_string());
            metrics.record_request_failure(&error, 100);
        }
        
        // 现在成功率应该是 150/(150+50) = 75% < 95%
        let health = monitor.health_check();
        assert!(!health.is_healthy());
        
        if let Some(issues) = health.get_issues() {
            assert!(!issues.is_empty());
            assert!(issues.iter().any(|issue| issue.contains("Low success rate")));
        }
        
        // 测试高处理时间检查
        let metrics2 = SignatorMetrics::new();
        for _ in 0..10 {
            metrics2.record_request_start();
            metrics2.record_request_success(2000); // 2秒处理时间
        }
        
        let monitor2 = SignatorMonitor { middleware: SignatorMiddleware {
            rear: String::new(),
            excludes: vec![],
            nonce_lifetime: 300,
            key_loader: create_test_key_loader(),
            redis_manager: RedisConnectionManager::new("redis://localhost:6379").unwrap(),
            metrics: metrics2,
        }};
        
        let health = monitor2.health_check();
        assert!(!health.is_healthy());
        
        if let Some(issues) = health.get_issues() {
            assert!(issues.iter().any(|issue| issue.contains("High average processing time")));
        }
    }
}

/// 创建完整的签名验证中间件链
/// 
/// 创建一个包含多个中间件的完整中间件链，包括 CORS、日志记录、
/// 认证、签名验证和限流等功能。
/// 
/// # 参数
/// * `redis_url` - Redis 连接 URL
/// * `key_loader` - 密钥加载器
/// 
/// # 返回值
/// 成功时返回配置好的中间件链，失败时返回 `SignatorError`
/// 
/// # 用法示例
/// ```rust
/// use std::sync::Arc;
/// use crate::web::middleware::signator::{create_signator_middleware_chain, KeyLoader};
/// 
/// // 创建密钥加载器
/// let key_loader: KeyLoader = Arc::new(|user_id: String| {
///     Box::pin(async move {
///         // 从数据库或配置中加载密钥
///         match user_id.as_str() {
///             "user1" => Ok("secret1".to_string()),
///             "user2" => Ok("secret2".to_string()),
///             _ => Err(crate::erx::Erx::new("User not found")),
///         }
///     })
/// });
/// 
/// // 创建中间件链
/// let middleware_chain = create_signator_middleware_chain(
///     "redis://localhost:6379",
///     key_loader
/// )?;
/// 
/// // 在 Axum 应用中使用
/// let app = Router::new()
///     .route("/api/users", get(get_users))
///     .layer(middleware_chain);
/// ```
pub fn create_signator_middleware_chain(redis_url: &str, key_loader: KeyLoader) -> Result<MiddlewareChain, SignatorError> {
    // 定义排除函数
    let exclude_health = |parts: &axum::http::request::Parts| -> bool {
        parts.uri.path() == "/api/health" || parts.uri.path() == "/api/ping"
    };
    
    let exclude_public = |parts: &axum::http::request::Parts| -> bool {
        parts.uri.path().starts_with("/api/public/")
    };

    // 创建签名中间件
    let signator = SignatorMiddleware::new(redis_url, key_loader)?
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

    Ok(MiddlewareChain::new(manager))
}

/// 创建带有后门的签名中间件链（用于开发环境）
/// 
/// 创建一个包含开发后门的简化中间件链，主要用于开发和测试环境。
/// 当请求包含正确的后门密钥时，可以跳过签名验证。
/// 
/// # 参数
/// * `redis_url` - Redis 连接 URL
/// * `key_loader` - 密钥加载器
/// * `rear` - 开发后门密钥
/// 
/// # 返回值
/// 成功时返回配置好的中间件链，失败时返回 `SignatorError`
/// 
/// # 安全警告
/// 此函数创建的中间件链包含开发后门，仅应在开发和测试环境中使用，
/// 绝不应在生产环境中使用！
/// 
/// # 用法示例
/// ```rust
/// // 仅在开发环境中使用
/// #[cfg(debug_assertions)]
/// let middleware_chain = create_signator_middleware_chain_with_rear(
///     "redis://localhost:6379",
///     key_loader,
///     "dev_backdoor_secret_123".to_string()
/// )?;
/// 
/// // 客户端可以通过以下方式跳过签名验证：
/// // curl -H "X-DEVELOPMENT-SKIP: dev_backdoor_secret_123" \
/// //      http://localhost:3000/api/protected-endpoint
/// ```
pub fn create_signator_middleware_chain_with_rear(
    redis_url: &str, 
    key_loader: KeyLoader, 
    rear: String
) -> Result<MiddlewareChain, SignatorError> {
    let signator = SignatorMiddleware::with_rear(redis_url, key_loader, rear)?
        .with_nonce_lifetime(300);

    let manager = MiddlewareBuilder::new()
        .add(super::LoggingMiddleware::new(true))
        .add(signator)
        .build();

    Ok(MiddlewareChain::new(manager))
}

/// 性能监控工具
/// 
/// 提供签名验证中间件的性能监控功能，包括实时指标获取、
/// 性能报告生成、日志记录和健康检查等功能。
/// 
/// # 用法示例
/// ```rust
/// use crate::web::middleware::signator::{SignatorMiddleware, SignatorMonitor};
/// 
/// // 创建中间件和监控器
/// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?;
/// let monitor = SignatorMonitor::new(middleware);
/// 
/// // 获取性能指标
/// let metrics = monitor.get_metrics();
/// let report = monitor.get_performance_report();
/// 
/// // 记录性能指标到日志
/// monitor.log_performance_metrics();
/// 
/// // 启动定期报告（每60秒）
/// let _task = monitor.start_periodic_reporting(60);
/// 
/// // 检查健康状况
/// let health = monitor.health_check();
/// if !health.is_healthy() {
///     eprintln!("Health issues detected: {:?}", health.get_issues());
/// }
/// ```
pub struct SignatorMonitor {
    middleware: SignatorMiddleware,
}

impl SignatorMonitor {
    /// 创建新的性能监控器
    /// 
    /// # 参数
    /// * `middleware` - 要监控的签名验证中间件实例
    /// 
    /// # 返回值
    /// 返回新的监控器实例
    /// 
    /// # 用法示例
    /// ```rust
    /// let middleware = SignatorMiddleware::new("redis://localhost:6379", key_loader)?;
    /// let monitor = SignatorMonitor::new(middleware);
    /// ```
    pub fn new(middleware: SignatorMiddleware) -> Self {
        Self { middleware }
    }

    /// 获取实时性能指标
    /// 
    /// 返回性能指标收集器的共享引用，可用于实时监控。
    /// 
    /// # 返回值
    /// 返回 `Arc<SignatorMetrics>` 共享指针
    /// 
    /// # 用法示例
    /// ```rust
    /// let monitor = SignatorMonitor::new(middleware);
    /// let metrics = monitor.get_metrics();
    /// 
    /// // 实时查看指标
    /// println!("Total requests: {}", metrics.total_requests.load(Ordering::Relaxed));
    /// println!("Success requests: {}", metrics.successful_requests.load(Ordering::Relaxed));
    /// ```
    pub fn get_metrics(&self) -> Arc<SignatorMetrics> {
        self.middleware.get_metrics()
    }

    /// 获取性能报告
    /// 
    /// 生成包含所有性能统计信息的详细报告。
    /// 
    /// # 返回值
    /// 返回 `PerformanceReport` 结构体
    /// 
    /// # 用法示例
    /// ```rust
    /// let monitor = SignatorMonitor::new(middleware);
    /// let report = monitor.get_performance_report();
    /// 
    /// println!("Success rate: {:.2}%", report.success_rate);
    /// println!("Average processing time: {:.2}ms", report.avg_processing_time_ms);
    /// println!("Cache hit rate: {:.2}%", report.cache_hit_rate);
    /// ```
    pub fn get_performance_report(&self) -> PerformanceReport {
        self.middleware.get_performance_report()
    }

    /// 记录性能指标到日志
    /// 
    /// 将当前的性能指标以结构化日志的形式记录到日志系统中。
    /// 使用不同的日志级别记录不同类型的信息。
    /// 
    /// # 日志级别
    /// - `info`: 记录关键性能指标（总请求数、成功率、平均处理时间等）
    /// - `debug`: 记录详细的错误分类和性能分类信息
    /// 
    /// # 用法示例
    /// ```rust
    /// let monitor = SignatorMonitor::new(middleware);
    /// 
    /// // 手动记录一次性能指标
    /// monitor.log_performance_metrics();
    /// 
    /// // 在定时任务中使用
    /// tokio::spawn(async move {
    ///     let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5分钟
    ///     loop {
    ///         interval.tick().await;
    ///         monitor.log_performance_metrics();
    ///     }
    /// });
    /// ```
    pub fn log_performance_metrics(&self) {
        let report = self.get_performance_report();
        
        tracing::info!(
            total_requests = report.total_requests,
            successful_requests = report.successful_requests,
            failed_requests = report.failed_requests,
            success_rate = %format!("{:.2}%", report.success_rate),
            avg_processing_time_ms = %format!("{:.2}ms", report.avg_processing_time_ms),
            cache_hit_rate = %format!("{:.2}%", report.cache_hit_rate),
            "Signator middleware performance metrics"
        );

        tracing::debug!(
            payload_parse_errors = report.error_breakdown.payload_parse_errors,
            signature_format_errors = report.error_breakdown.signature_format_errors,
            key_load_errors = report.error_breakdown.key_load_errors,
            signature_invalid_errors = report.error_breakdown.signature_invalid_errors,
            nonce_replay_errors = report.error_breakdown.nonce_replay_errors,
            redis_connection_errors = report.error_breakdown.redis_connection_errors,
            timestamp_invalid_errors = report.error_breakdown.timestamp_invalid_errors,
            "Signator middleware error breakdown"
        );

        tracing::debug!(
            avg_key_load_time_ms = %format!("{:.2}ms", report.performance_breakdown.avg_key_load_time_ms),
            avg_redis_operation_time_ms = %format!("{:.2}ms", report.performance_breakdown.avg_redis_operation_time_ms),
            avg_signature_validation_time_ms = %format!("{:.2}ms", report.performance_breakdown.avg_signature_validation_time_ms),
            redis_timeouts = report.performance_breakdown.redis_timeouts,
            "Signator middleware performance breakdown"
        );
    }

    /// 启动定期性能报告任务
    /// 
    /// 启动一个后台任务，定期将性能指标记录到日志中。
    /// 任务会持续运行直到被取消或程序退出。
    /// 
    /// # 参数
    /// * `interval_seconds` - 报告间隔（秒）
    /// 
    /// # 返回值
    /// 返回任务句柄，可用于取消任务
    /// 
    /// # 用法示例
    /// ```rust
    /// let monitor = SignatorMonitor::new(middleware);
    /// 
    /// // 每60秒记录一次性能指标
    /// let report_task = monitor.start_periodic_reporting(60);
    /// 
    /// // 每5分钟记录一次（适合生产环境）
    /// let report_task = monitor.start_periodic_reporting(300);
    /// 
    /// // 如果需要停止定期报告
    /// report_task.abort();
    /// 
    /// // 等待任务完成
    /// match report_task.await {
    ///     Ok(_) => println!("Report task completed"),
    ///     Err(e) if e.is_cancelled() => println!("Report task was cancelled"),
    ///     Err(e) => println!("Report task failed: {}", e),
    /// }
    /// ```
    pub fn start_periodic_reporting(&self, interval_seconds: u64) -> tokio::task::JoinHandle<()> {
        let monitor = SignatorMonitor::new(self.middleware.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                monitor.log_performance_metrics();
            }
        })
    }

    /// 检查性能健康状况
    /// 
    /// 根据当前的性能指标评估系统健康状况，检查多个维度的指标
    /// 并返回健康状态和潜在问题列表。
    /// 
    /// # 检查项目
    /// - 成功率：低于 95% 且请求数超过 100 时报警
    /// - 平均处理时间：超过 1000ms 时报警
    /// - Redis 超时次数：超过 10 次时报警
    /// - 缓存命中率：低于 10% 且请求数超过 100 时报警
    /// 
    /// # 返回值
    /// 返回 `HealthStatus` 枚举，包含健康状态和问题列表
    /// 
    /// # 用法示例
    /// ```rust
    /// let monitor = SignatorMonitor::new(middleware);
    /// let health = monitor.health_check();
    /// 
    /// match health {
    ///     HealthStatus::Healthy => {
    ///         println!("System is healthy");
    ///     }
    ///     HealthStatus::Degraded(issues) => {
    ///         println!("System has issues:");
    ///         for issue in issues {
    ///             println!("  - {}", issue);
    ///         }
    ///     }
    /// }
    /// 
    /// // 在监控系统中使用
    /// if !health.is_healthy() {
    ///     // 发送告警
    ///     send_alert("Signator middleware health check failed", health.get_issues());
    /// }
    /// 
    /// // 在健康检查端点中使用
    /// async fn health_endpoint(monitor: SignatorMonitor) -> impl IntoResponse {
    ///     let health = monitor.health_check();
    ///     if health.is_healthy() {
    ///         (StatusCode::OK, "Healthy")
    ///     } else {
    ///         (StatusCode::SERVICE_UNAVAILABLE, "Degraded")
    ///     }
    /// }
    /// ```
    pub fn health_check(&self) -> HealthStatus {
        let report = self.get_performance_report();
        
        let mut issues = Vec::new();
        
        // 检查成功率
        if report.success_rate < 95.0 && report.total_requests > 100 {
            issues.push(format!("Low success rate: {:.2}%", report.success_rate));
        }
        
        // 检查平均处理时间
        if report.avg_processing_time_ms > 1000.0 {
            issues.push(format!("High average processing time: {:.2}ms", report.avg_processing_time_ms));
        }
        
        // 检查 Redis 超时
        if report.performance_breakdown.redis_timeouts > 10 {
            issues.push(format!("High Redis timeout count: {}", report.performance_breakdown.redis_timeouts));
        }
        
        // 检查缓存命中率
        if report.cache_hit_rate < 10.0 && report.total_requests > 100 {
            issues.push(format!("Low cache hit rate: {:.2}%", report.cache_hit_rate));
        }
        
        if issues.is_empty() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded(issues)
        }
    }
}

/// 健康状况枚举
/// 
/// 表示系统的健康状态，包括健康和降级两种状态。
/// 
/// # 用法示例
/// ```rust
/// let health = monitor.health_check();
/// 
/// // 检查是否健康
/// if health.is_healthy() {
///     println!("System is running normally");
/// } else {
///     println!("System has performance issues");
/// }
/// 
/// // 获取具体问题列表
/// if let Some(issues) = health.get_issues() {
///     for issue in issues {
///         println!("Issue: {}", issue);
///     }
/// }
/// ```
#[derive(Debug)]
pub enum HealthStatus {
    /// 系统健康，所有指标都在正常范围内
    Healthy,
    /// 系统降级，存在性能问题，包含问题描述列表
    Degraded(Vec<String>),
}

impl HealthStatus {
    /// 检查是否健康
    /// 
    /// # 返回值
    /// 健康时返回 `true`，有问题时返回 `false`
    /// 
    /// # 用法示例
    /// ```rust
    /// let health = monitor.health_check();
    /// if health.is_healthy() {
    ///     // 系统正常，继续处理
    /// } else {
    ///     // 系统有问题，可能需要告警或降级处理
    /// }
    /// ```
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
    
    /// 获取问题列表
    /// 
    /// # 返回值
    /// 健康时返回 `None`，有问题时返回问题描述列表的引用
    /// 
    /// # 用法示例
    /// ```rust
    /// let health = monitor.health_check();
    /// if let Some(issues) = health.get_issues() {
    ///     // 记录或发送告警
    ///     for issue in issues {
    ///         log::warn!("Health issue: {}", issue);
    ///         // 发送到监控系统
    ///         metrics_client.increment("health.issues", &[("issue", issue)]);
    ///     }
    /// }
    /// ```
    pub fn get_issues(&self) -> Option<&Vec<String>> {
        match self {
            HealthStatus::Healthy => None,
            HealthStatus::Degraded(issues) => Some(issues),
        }
    }
}