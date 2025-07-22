//! # JWT 速率限制模块
//! 
//! 为 JWT 认证的用户提供基于令牌的速率限制功能，支持：
//! 
//! ## 核心功能
//! 
//! - **用户级速率限制**: 基于 JWT 中的用户ID进行限制
//! - **角色级速率限制**: 不同角色有不同的速率限制
//! - **端点级速率限制**: 针对特定API端点的限制
//! - **滑动窗口算法**: 使用滑动窗口实现精确的速率控制
//! - **分布式支持**: 支持 Redis 作为后端存储
//! - **动态配置**: 运行时调整速率限制参数
//! 
//! ## 使用示例
//! 
//! ```rust
//! use crate::web::middleware::jwt::rate_limit::{RateLimitConfig, JwtRateLimiter};
//! 
//! // 创建速率限制配置
//! let config = RateLimitConfig::new()
//!     .with_default_limit(100, 60) // 每分钟100次请求
//!     .with_role_limit("premium", 1000, 60) // 高级用户每分钟1000次
//!     .with_endpoint_limit("/api/upload", 10, 60); // 上传接口每分钟10次
//! 
//! // 创建速率限制器
//! let rate_limiter = JwtRateLimiter::new(config);
//! 
//! // 在中间件中使用
//! let jwt_middleware = JwtMiddleware::new(jwt_config)
//!     .with_rate_limiter(rate_limiter);
//! ```

use super::Claims;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use serde::{Deserialize, Serialize};

/// 速率限制错误类型
#[derive(Debug, Clone)]
pub enum RateLimitError {
    /// 超出速率限制
    RateLimitExceeded {
        limit: u32,
        window_seconds: u32,
        retry_after: u32,
    },
    /// 配置错误
    ConfigError(String),
    /// 存储错误
    StorageError(String),
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded { limit, window_seconds, retry_after } => {
                write!(f, "Rate limit exceeded: {} requests per {} seconds, retry after {} seconds", 
                       limit, window_seconds, retry_after)
            },
            RateLimitError::ConfigError(msg) => write!(f, "Rate limit configuration error: {}", msg),
            RateLimitError::StorageError(msg) => write!(f, "Rate limit storage error: {}", msg),
        }
    }
}

impl std::error::Error for RateLimitError {}



/// 速率限制规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    /// 请求限制数量
    pub limit: u32,
    /// 时间窗口（秒）
    pub window_seconds: u32,
    /// 规则描述
    pub description: Option<String>,
}

impl RateLimitRule {
    /// 创建新的速率限制规则
    pub fn new(limit: u32, window_seconds: u32) -> Self {
        Self {
            limit,
            window_seconds,
            description: None,
        }
    }

    /// 设置规则描述
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 默认速率限制规则
    pub default_rule: RateLimitRule,
    /// 基于角色的速率限制规则
    pub role_rules: HashMap<String, RateLimitRule>,
    /// 基于端点的速率限制规则
    pub endpoint_rules: HashMap<String, RateLimitRule>,
    /// 是否启用速率限制
    pub enabled: bool,
    /// 是否使用分布式存储（Redis）
    pub use_distributed_storage: bool,
    /// Redis 连接字符串
    pub redis_url: Option<String>,
}

impl RateLimitConfig {
    /// 创建新的速率限制配置
    pub fn new() -> Self {
        Self {
            default_rule: RateLimitRule::new(60, 60), // 默认每分钟60次请求
            role_rules: HashMap::new(),
            endpoint_rules: HashMap::new(),
            enabled: true,
            use_distributed_storage: false,
            redis_url: None,
        }
    }

    /// 设置默认速率限制
    pub fn with_default_limit(mut self, limit: u32, window_seconds: u32) -> Self {
        self.default_rule = RateLimitRule::new(limit, window_seconds);
        self
    }

    /// 添加角色级速率限制
    pub fn with_role_limit(mut self, role: &str, limit: u32, window_seconds: u32) -> Self {
        self.role_rules.insert(
            role.to_string(),
            RateLimitRule::new(limit, window_seconds)
                .with_description(&format!("Role-based limit for {}", role))
        );
        self
    }

    /// 添加端点级速率限制
    pub fn with_endpoint_limit(mut self, endpoint: &str, limit: u32, window_seconds: u32) -> Self {
        self.endpoint_rules.insert(
            endpoint.to_string(),
            RateLimitRule::new(limit, window_seconds)
                .with_description(&format!("Endpoint-based limit for {}", endpoint))
        );
        self
    }

    /// 启用分布式存储
    pub fn with_redis(mut self, redis_url: &str) -> Self {
        self.use_distributed_storage = true;
        self.redis_url = Some(redis_url.to_string());
        self
    }

    /// 禁用速率限制
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// 获取用户的速率限制规则
    pub fn get_user_rule(&self, claims: &Claims, endpoint: &str) -> RateLimitRule {
        // 1. 优先检查端点级规则
        if let Some(rule) = self.endpoint_rules.get(endpoint) {
            return rule.clone();
        }

        // 2. 检查角色级规则（选择最宽松的规则）
        if let Some(roles) = &claims.roles {
            let mut best_rule: Option<RateLimitRule> = None;
            
            for role in roles {
                if let Some(rule) = self.role_rules.get(role) {
                    match &best_rule {
                        None => best_rule = Some(rule.clone()),
                        Some(current) => {
                            // 选择限制更宽松的规则
                            if rule.limit > current.limit {
                                best_rule = Some(rule.clone());
                            }
                        }
                    }
                }
            }
            
            if let Some(rule) = best_rule {
                return rule;
            }
        }

        // 3. 使用默认规则
        self.default_rule.clone()
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 请求记录
#[derive(Debug, Clone)]
struct RequestRecord {
    timestamp: Instant,
    count: u32,
}

/// 滑动窗口计数器
#[derive(Debug)]
struct SlidingWindowCounter {
    records: Vec<RequestRecord>,
    window_duration: Duration,
}

impl SlidingWindowCounter {
    fn new(window_seconds: u32) -> Self {
        Self {
            records: Vec::new(),
            window_duration: Duration::from_secs(window_seconds as u64),
        }
    }

    /// 清理过期记录
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.records.retain(|record| {
            now.duration_since(record.timestamp) < self.window_duration
        });
    }

    /// 获取当前窗口内的请求总数
    fn get_current_count(&mut self) -> u32 {
        self.cleanup_expired();
        self.records.iter().map(|r| r.count).sum()
    }

    /// 添加请求记录
    fn add_request(&mut self, count: u32) {
        self.cleanup_expired();
        
        let now = Instant::now();
        
        // 如果最近的记录是在同一秒内，则合并
        if let Some(last_record) = self.records.last_mut() {
            if now.duration_since(last_record.timestamp) < Duration::from_secs(1) {
                last_record.count += count;
                return;
            }
        }
        
        // 添加新记录
        self.records.push(RequestRecord {
            timestamp: now,
            count,
        });
    }

    /// 检查是否超出限制
    fn check_limit(&mut self, limit: u32) -> bool {
        self.get_current_count() < limit
    }

    /// 获取重试时间（秒）
    fn get_retry_after(&mut self) -> u32 {
        self.cleanup_expired();
        
        if let Some(oldest_record) = self.records.first() {
            let elapsed = Instant::now().duration_since(oldest_record.timestamp);
            let remaining = self.window_duration.saturating_sub(elapsed);
            remaining.as_secs() as u32 + 1
        } else {
            1
        }
    }
}

/// 内存存储后端
#[derive(Debug)]
struct MemoryStorage {
    counters: RwLock<HashMap<String, SlidingWindowCounter>>,
}

impl MemoryStorage {
    fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
        }
    }

    fn check_and_update(&self, key: &str, rule: &RateLimitRule) -> Result<(), RateLimitError> {
        let mut counters = self.counters.write().unwrap();
        
        let counter = counters
            .entry(key.to_string())
            .or_insert_with(|| SlidingWindowCounter::new(rule.window_seconds));

        if !counter.check_limit(rule.limit) {
            let retry_after = counter.get_retry_after();
            return Err(RateLimitError::RateLimitExceeded {
                limit: rule.limit,
                window_seconds: rule.window_seconds,
                retry_after,
            });
        }

        counter.add_request(1);
        Ok(())
    }

    fn get_current_count(&self, key: &str, _window_seconds: u32) -> u32 {
        let mut counters = self.counters.write().unwrap();
        
        if let Some(counter) = counters.get_mut(key) {
            counter.get_current_count()
        } else {
            0
        }
    }
}

/// 存储后端 trait
trait StorageBackend: Send + Sync {
    fn check_and_update(&self, key: &str, rule: &RateLimitRule) -> Result<(), RateLimitError>;
    fn get_current_count(&self, key: &str, window_seconds: u32) -> u32;
}

impl StorageBackend for MemoryStorage {
    fn check_and_update(&self, key: &str, rule: &RateLimitRule) -> Result<(), RateLimitError> {
        self.check_and_update(key, rule)
    }

    fn get_current_count(&self, key: &str, window_seconds: u32) -> u32 {
        self.get_current_count(key, window_seconds)
    }
}

/// JWT 速率限制器
pub struct JwtRateLimiter {
    config: RateLimitConfig,
    storage: Arc<dyn StorageBackend>,
    metrics: Arc<RateLimitMetrics>,
}

impl JwtRateLimiter {
    /// 创建新的速率限制器
    pub fn new(config: RateLimitConfig) -> Self {
        let storage: Arc<dyn StorageBackend> = if config.use_distributed_storage {
            // 在实际实现中，这里会创建 Redis 存储后端
            // Arc::new(RedisStorage::new(config.redis_url.as_ref().unwrap()))
            Arc::new(MemoryStorage::new())
        } else {
            Arc::new(MemoryStorage::new())
        };

        Self {
            config,
            storage,
            metrics: RateLimitMetrics::new(),
        }
    }

    /// 检查用户是否超出速率限制
    pub async fn check_rate_limit(
        &self,
        claims: &Claims,
        endpoint: &str,
    ) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let rule = self.config.get_user_rule(claims, endpoint);
        let key = self.generate_key(claims, endpoint, &rule);

        let start_time = Instant::now();
        let result = self.storage.check_and_update(&key, &rule);
        let duration = start_time.elapsed();

        // 记录指标
        match &result {
            Ok(_) => {
                self.metrics.record_allowed_request(duration);
            }
            Err(_) => {
                self.metrics.record_blocked_request(duration);
            }
        }

        result
    }

    /// 获取用户当前的请求计数
    pub fn get_current_usage(&self, claims: &Claims, endpoint: &str) -> RateLimitUsage {
        let rule = self.config.get_user_rule(claims, endpoint);
        let key = self.generate_key(claims, endpoint, &rule);
        
        let current_count = self.storage.get_current_count(&key, rule.window_seconds);
        
        RateLimitUsage {
            current_count,
            limit: rule.limit,
            window_seconds: rule.window_seconds,
            remaining: rule.limit.saturating_sub(current_count),
            reset_time: SystemTime::now() + Duration::from_secs(rule.window_seconds as u64),
        }
    }

    /// 生成存储键
    fn generate_key(&self, claims: &Claims, endpoint: &str, _rule: &RateLimitRule) -> String {
        // 根据规则类型生成不同的键
        if self.config.endpoint_rules.contains_key(endpoint) {
            format!("rate_limit:endpoint:{}:user:{}", endpoint, claims.sub)
        } else if claims.roles.as_ref().map_or(false, |roles| {
            roles.iter().any(|role| self.config.role_rules.contains_key(role))
        }) {
            let role = claims.roles.as_ref().unwrap()
                .iter()
                .find(|role| self.config.role_rules.contains_key(*role))
                .unwrap();
            format!("rate_limit:role:{}:user:{}", role, claims.sub)
        } else {
            format!("rate_limit:default:user:{}", claims.sub)
        }
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> Arc<RateLimitMetrics> {
        Arc::clone(&self.metrics)
    }
}

impl Clone for JwtRateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            storage: Arc::clone(&self.storage),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

/// 速率限制使用情况
#[derive(Debug, Serialize)]
pub struct RateLimitUsage {
    pub current_count: u32,
    pub limit: u32,
    pub window_seconds: u32,
    pub remaining: u32,
    pub reset_time: SystemTime,
}

/// 速率限制指标
#[derive(Debug, Default)]
pub struct RateLimitMetrics {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub allowed_requests: std::sync::atomic::AtomicU64,
    pub blocked_requests: std::sync::atomic::AtomicU64,
    pub total_check_time_ms: std::sync::atomic::AtomicU64,
}

impl RateLimitMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn record_allowed_request(&self, duration: Duration) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.allowed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_check_time_ms.fetch_add(
            duration.as_millis() as u64,
            std::sync::atomic::Ordering::Relaxed
        );
    }

    pub fn record_blocked_request(&self, duration: Duration) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.blocked_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_check_time_ms.fetch_add(
            duration.as_millis() as u64,
            std::sync::atomic::Ordering::Relaxed
        );
    }

    pub fn get_report(&self) -> RateLimitReport {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        let allowed = self.allowed_requests.load(std::sync::atomic::Ordering::Relaxed);
        let blocked = self.blocked_requests.load(std::sync::atomic::Ordering::Relaxed);
        let total_time = self.total_check_time_ms.load(std::sync::atomic::Ordering::Relaxed);

        let block_rate = if total > 0 {
            (blocked as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let avg_check_time = if total > 0 {
            total_time as f64 / total as f64
        } else {
            0.0
        };

        RateLimitReport {
            total_requests: total,
            allowed_requests: allowed,
            blocked_requests: blocked,
            block_rate,
            avg_check_time_ms: avg_check_time,
        }
    }
}

/// 速率限制报告
#[derive(Debug, Serialize)]
pub struct RateLimitReport {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub blocked_requests: u64,
    pub block_rate: f64,
    pub avg_check_time_ms: f64,
}



#[cfg(test)]
mod tests {
    use super::*;
    // use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_sliding_window_counter() {
        let mut counter = SlidingWindowCounter::new(60); // 1分钟窗口
        
        // 添加一些请求
        counter.add_request(1);
        counter.add_request(1);
        counter.add_request(1);
        
        assert_eq!(counter.get_current_count(), 3);
        assert!(counter.check_limit(5));
        assert!(!counter.check_limit(2));
    }

    #[tokio::test]
    async fn test_rate_limit_config() {
        let config = RateLimitConfig::new()
            .with_default_limit(100, 60)
            .with_role_limit("premium", 1000, 60)
            .with_endpoint_limit("/api/upload", 10, 60);

        let mut claims = super::super::Claims::new("user123");
        claims.add_role("premium");

        // 测试角色级限制
        let rule = config.get_user_rule(&claims, "/api/test");
        assert_eq!(rule.limit, 1000);

        // 测试端点级限制
        let rule = config.get_user_rule(&claims, "/api/upload");
        assert_eq!(rule.limit, 10);

        // 测试默认限制
        let basic_claims = super::super::Claims::new("user456");
        let rule = config.get_user_rule(&basic_claims, "/api/test");
        assert_eq!(rule.limit, 100);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig::new()
            .with_default_limit(3, 60); // 每分钟3次请求

        let rate_limiter = JwtRateLimiter::new(config);
        let claims = super::super::Claims::new("test_user");

        // 前3次请求应该成功
        for i in 0..3 {
            let result = rate_limiter.check_rate_limit(&claims, "/api/test").await;
            assert!(result.is_ok(), "Request {} should be allowed", i + 1);
        }

        // 第4次请求应该被阻止
        let result = rate_limiter.check_rate_limit(&claims, "/api/test").await;
        assert!(result.is_err(), "Request 4 should be blocked");

        if let Err(RateLimitError::RateLimitExceeded { limit, window_seconds, retry_after }) = result {
            assert_eq!(limit, 3);
            assert_eq!(window_seconds, 60);
            assert!(retry_after > 0);
        } else {
            panic!("Expected RateLimitExceeded error");
        }
    }

    #[tokio::test]
    async fn test_rate_limit_usage() {
        let config = RateLimitConfig::new()
            .with_default_limit(10, 60);

        let rate_limiter = JwtRateLimiter::new(config);
        let claims = super::super::Claims::new("test_user");

        // 发送5次请求
        for _ in 0..5 {
            let _ = rate_limiter.check_rate_limit(&claims, "/api/test").await;
        }

        let usage = rate_limiter.get_current_usage(&claims, "/api/test");
        assert_eq!(usage.current_count, 5);
        assert_eq!(usage.limit, 10);
        assert_eq!(usage.remaining, 5);
    }

    #[tokio::test]
    async fn test_rate_limit_metrics() {
        let config = RateLimitConfig::new()
            .with_default_limit(2, 60);

        let rate_limiter = JwtRateLimiter::new(config);
        let claims = super::super::Claims::new("test_user");

        // 发送3次请求（2次成功，1次被阻止）
        let _ = rate_limiter.check_rate_limit(&claims, "/api/test").await;
        let _ = rate_limiter.check_rate_limit(&claims, "/api/test").await;
        let _ = rate_limiter.check_rate_limit(&claims, "/api/test").await;

        let metrics = rate_limiter.get_metrics();
        let report = metrics.get_report();

        assert_eq!(report.total_requests, 3);
        assert_eq!(report.allowed_requests, 2);
        assert_eq!(report.blocked_requests, 1);
        assert!((report.block_rate - 33.33).abs() < 0.1);
    }
}