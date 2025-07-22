//! # JWT 认证中间件模块
//! 
//! 本模块提供基于 JWT (JSON Web Token) 的请求认证功能，包括：
//! 
//! ## 核心功能
//! 
//! - **JWT 令牌验证**: 验证请求中的 JWT 令牌的有效性和完整性
//! - **多种令牌提取方式**: 支持从请求头、Cookie 或查询参数中提取令牌
//! - **角色权限控制**: 支持基于角色的访问控制
//! - **性能监控**: 全面的性能指标收集和监控功能
//! - **速率限制**: 基于用户和角色的速率控制
//! - **灵活配置**: 支持排除规则、自定义验证选项等配置

pub mod rate_limit;

use super::*;
use crate::erx::{Layouted, LayoutedC};
use crate::web::api::Out;
use crate::web::context::Context;
use crate::web::cookie::CookieJar;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::fmt;
use tokio::time::{timeout, Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

// 重新导出速率限制相关类型
pub use rate_limit::{RateLimitConfig, JwtRateLimiter, RateLimitError, RateLimitUsage};

/// JWT 认证相关的错误类型
#[derive(Debug, Clone)]
pub enum JwtError {
    /// 令牌缺失 - 当请求中未找到 JWT 令牌时
    TokenMissing,
    /// 令牌无效 - 当 JWT 令牌格式错误或签名验证失败时
    TokenInvalid(String),
    /// 令牌过期 - 当 JWT 令牌已过期时
    TokenExpired,
    /// 权限不足 - 当用户没有所需角色时
    InsufficientPermission(String),
    /// 配置错误 - 当中间件配置有误时
    ConfigError(String),
    /// 速率限制错误 - 当请求超出速率限制时
    RateLimitExceeded(String),
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JwtError::TokenMissing => write!(f, "JWT token is missing"),
            JwtError::TokenInvalid(msg) => write!(f, "JWT token is invalid: {}", msg),
            JwtError::TokenExpired => write!(f, "JWT token has expired"),
            JwtError::InsufficientPermission(msg) => write!(f, "Insufficient permission: {}", msg),
            JwtError::ConfigError(msg) => write!(f, "JWT configuration error: {}", msg),
            JwtError::RateLimitExceeded(msg) => write!(f, "Rate limit exceeded: {}", msg),
        }
    }
}

impl std::error::Error for JwtError {}

impl From<rate_limit::RateLimitError> for JwtError {
    fn from(err: rate_limit::RateLimitError) -> Self {
        JwtError::RateLimitExceeded(err.to_string())
    }
}



/// 常量定义
mod constants {
    pub const JWT_STR: &str = "JWT";
    pub const MISS_STR: &str = "MISS";
    pub const INVD_STR: &str = "INVD";
    pub const EXPR_STR: &str = "EXPR";
    pub const PERM_STR: &str = "PERM";
    pub const CONF_STR: &str = "CONF";
    pub const RATE_STR: &str = "RATE";
    pub const DEFAULT_COOKIE_NAME: &str = "jwt";
    pub const DEFAULT_QUERY_PARAM: &str = "token";
    pub const DEFAULT_TOKEN_PREFIX: &str = "Bearer ";
    pub const TOKEN_VALIDATION_TIMEOUT_MS: u64 = 1000; // 1秒
}

use constants::*;

/// 向后兼容的错误代码生成函数
fn make_code(detail: &str) -> LayoutedC {
    Layouted::middleware(JWT_STR, detail)
}

impl JwtError {
    /// 转换为 HTTP 响应
    pub fn into_response(self) -> Response {
        let (code_detail, message, status_code) = match &self {
            JwtError::TokenMissing => (MISS_STR, self.to_string(), StatusCode::UNAUTHORIZED),
            JwtError::TokenInvalid(_) => (INVD_STR, self.to_string(), StatusCode::UNAUTHORIZED),
            JwtError::TokenExpired => (EXPR_STR, self.to_string(), StatusCode::UNAUTHORIZED),
            JwtError::InsufficientPermission(_) => (PERM_STR, self.to_string(), StatusCode::FORBIDDEN),
            JwtError::ConfigError(_) => (CONF_STR, self.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
            JwtError::RateLimitExceeded(_) => (RATE_STR, self.to_string(), StatusCode::TOO_MANY_REQUESTS),
        };

        (
            status_code,
            Out::<()> {
                code: make_code(code_detail).into(),
                message: Some(message),
                data: None,
                debug: None,
                profile: None,
            }
        ).into_response()
    }
}

/// JWT 令牌声明（Payload）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// 主题（通常是用户ID）
    pub sub: String,
    /// 过期时间（Unix 时间戳）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    /// 签发时间（Unix 时间戳）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    /// 签发者
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// 用户角色列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// 自定义数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Claims {
    /// 创建新的 JWT 声明
    pub fn new(subject: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        Claims {
            sub: subject.to_string(),
            exp: None,
            iat: Some(now),
            iss: None,
            roles: None,
            data: None,
        }
    }

    /// 设置过期时间
    pub fn set_expiration(&mut self, seconds_from_now: i64) {
        let now = chrono::Utc::now().timestamp();
        self.exp = Some(now + seconds_from_now);
    }

    /// 设置签发者
    pub fn set_issuer(&mut self, issuer: &str) {
        self.iss = Some(issuer.to_string());
    }

    /// 添加角色
    pub fn add_role(&mut self, role: &str) {
        let mut roles = self.roles.clone().unwrap_or_default();
        roles.push(role.to_string());
        self.roles = Some(roles);
    }

    /// 检查是否具有指定角色
    pub fn has_role(&self, role: &str) -> bool {
        if let Some(roles) = &self.roles {
            roles.iter().any(|r| r == role)
        } else {
            false
        }
    }

    /// 检查是否具有任一指定角色
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        if let Some(user_roles) = &self.roles {
            roles.iter().any(|role| user_roles.iter().any(|r| r == role))
        } else {
            false
        }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.exp {
            let now = chrono::Utc::now().timestamp();
            exp < now
        } else {
            false
        }
    }
}

/// JWT 配置选项
#[derive(Clone)]
pub struct JwtConfig {
    /// JWT 密钥
    secret: String,
    /// 签名算法
    algorithm: Algorithm,
    /// 是否从 Cookie 中提取令牌
    extract_from_cookie: bool,
    /// Cookie 名称
    cookie_name: String,
    /// 是否从查询参数中提取令牌
    extract_from_query: bool,
    /// 查询参数名称
    query_param: String,
    /// 令牌前缀（如 "Bearer "）
    token_prefix: String,
    /// 验证选项
    validation: Validation,
}

impl JwtConfig {
    /// 创建新的 JWT 配置
    pub fn new(secret: &str) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.leeway = 0; // 允许的时间偏差（秒）

        JwtConfig {
            secret: secret.to_string(),
            algorithm: Algorithm::HS256,
            extract_from_cookie: false,
            cookie_name: DEFAULT_COOKIE_NAME.to_string(),
            extract_from_query: false,
            query_param: DEFAULT_QUERY_PARAM.to_string(),
            token_prefix: DEFAULT_TOKEN_PREFIX.to_string(),
            validation,
        }
    }

    /// 设置签名算法
    pub fn with_algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self.validation.algorithms = vec![algorithm];
        self
    }

    /// 设置是否从 Cookie 中提取令牌
    pub fn with_cookie_extraction(mut self, enabled: bool, cookie_name: &str) -> Self {
        self.extract_from_cookie = enabled;
        self.cookie_name = cookie_name.to_string();
        self
    }

    /// 设置是否从查询参数中提取令牌
    pub fn with_query_extraction(mut self, enabled: bool, param_name: &str) -> Self {
        self.extract_from_query = enabled;
        self.query_param = param_name.to_string();
        self
    }

    /// 设置令牌前缀
    pub fn with_token_prefix(mut self, prefix: &str) -> Self {
        self.token_prefix = prefix.to_string();
        self
    }

    /// 设置令牌签发者
    pub fn with_issuer(mut self, issuer: &str) -> Self {
        self.validation.set_issuer(&[issuer]);
        self
    }

    /// 设置时间偏差容忍度
    pub fn with_leeway(mut self, leeway_seconds: u64) -> Self {
        self.validation.leeway = leeway_seconds;
        self
    }

    /// 创建编码密钥
    fn encoding_key(&self) -> EncodingKey {
        EncodingKey::from_secret(self.secret.as_bytes())
    }

    /// 创建解码密钥
    fn decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(self.secret.as_bytes())
    }
}

/// JWT 令牌生成器
#[derive(Clone)]
pub struct JwtGenerator {
    config: JwtConfig,
}

impl JwtGenerator {
    /// 创建新的 JWT 生成器
    pub fn new(config: JwtConfig) -> Self {
        JwtGenerator { config }
    }

    /// 生成 JWT 令牌
    pub fn generate_token(&self, claims: &Claims) -> Result<String, JwtError> {
        encode(
            &Header::new(self.config.algorithm),
            claims,
            &self.config.encoding_key()
        ).map_err(|e| JwtError::ConfigError(format!("Token generation failed: {}", e)))
    }

    /// 验证 JWT 令牌
    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(
            token,
            &self.config.decoding_key(),
            &self.config.validation
        ).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                _ => JwtError::TokenInvalid(e.to_string()),
            }
        })?;

        Ok(token_data.claims)
    }
}

/// 性能指标收集器
#[derive(Debug, Default)]
pub struct JwtMetrics {
    // 请求计数器
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    
    // 错误类型计数
    pub token_missing_errors: AtomicU32,
    pub token_invalid_errors: AtomicU32,
    pub token_expired_errors: AtomicU32,
    pub insufficient_permission_errors: AtomicU32,
    pub config_errors: AtomicU32,
    pub rate_limit_errors: AtomicU32,
    
    // 性能指标
    pub total_processing_time_ms: AtomicU64,
    pub token_validation_time_ms: AtomicU64,
    pub token_extraction_time_ms: AtomicU64,
    
    // 令牌提取统计
    pub header_extractions: AtomicU64,
    pub cookie_extractions: AtomicU64,
    pub query_extractions: AtomicU64,
    pub extraction_failures: AtomicU64,
}

impl JwtMetrics {
    /// 创建新的性能指标收集器实例
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// 记录请求开始
    pub fn record_request_start(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录请求成功
    pub fn record_request_success(&self, processing_time_ms: u64) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time_ms.fetch_add(processing_time_ms, Ordering::Relaxed);
    }

    /// 记录请求失败
    pub fn record_request_failure(&self, error: &JwtError, processing_time_ms: u64) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time_ms.fetch_add(processing_time_ms, Ordering::Relaxed);
        
        // 记录具体错误类型
        match error {
            JwtError::TokenMissing => {
                self.token_missing_errors.fetch_add(1, Ordering::Relaxed);
            },
            JwtError::TokenInvalid(_) => {
                self.token_invalid_errors.fetch_add(1, Ordering::Relaxed);
            },
            JwtError::TokenExpired => {
                self.token_expired_errors.fetch_add(1, Ordering::Relaxed);
            },
            JwtError::InsufficientPermission(_) => {
                self.insufficient_permission_errors.fetch_add(1, Ordering::Relaxed);
            },
            JwtError::ConfigError(_) => {
                self.config_errors.fetch_add(1, Ordering::Relaxed);
            },
            JwtError::RateLimitExceeded(_) => {
                self.rate_limit_errors.fetch_add(1, Ordering::Relaxed);
            },
        }
    }

    /// 记录令牌验证时间
    pub fn record_token_validation_time(&self, time_ms: u64) {
        self.token_validation_time_ms.fetch_add(time_ms, Ordering::Relaxed);
    }

    /// 记录令牌提取时间
    pub fn record_token_extraction_time(&self, time_ms: u64) {
        self.token_extraction_time_ms.fetch_add(time_ms, Ordering::Relaxed);
    }

    /// 记录从请求头提取令牌
    pub fn record_header_extraction(&self) {
        self.header_extractions.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录从 Cookie 提取令牌
    pub fn record_cookie_extraction(&self) {
        self.cookie_extractions.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录从查询参数提取令牌
    pub fn record_query_extraction(&self) {
        self.query_extractions.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录令牌提取失败
    pub fn record_extraction_failure(&self) {
        self.extraction_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取性能统计报告
    pub fn get_performance_report(&self) -> JwtPerformanceReport {
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

        let total_extractions = self.header_extractions.load(Ordering::Relaxed) +
                               self.cookie_extractions.load(Ordering::Relaxed) +
                               self.query_extractions.load(Ordering::Relaxed);

        let header_extraction_rate = if total_extractions > 0 {
            (self.header_extractions.load(Ordering::Relaxed) as f64 / total_extractions as f64) * 100.0
        } else {
            0.0
        };

        let cookie_extraction_rate = if total_extractions > 0 {
            (self.cookie_extractions.load(Ordering::Relaxed) as f64 / total_extractions as f64) * 100.0
        } else {
            0.0
        };

        let query_extraction_rate = if total_extractions > 0 {
            (self.query_extractions.load(Ordering::Relaxed) as f64 / total_extractions as f64) * 100.0
        } else {
            0.0
        };

        JwtPerformanceReport {
            total_requests,
            successful_requests,
            failed_requests,
            success_rate,
            avg_processing_time_ms: avg_processing_time,
            error_breakdown: JwtErrorBreakdown {
                token_missing_errors: self.token_missing_errors.load(Ordering::Relaxed),
                token_invalid_errors: self.token_invalid_errors.load(Ordering::Relaxed),
                token_expired_errors: self.token_expired_errors.load(Ordering::Relaxed),
                insufficient_permission_errors: self.insufficient_permission_errors.load(Ordering::Relaxed),
                config_errors: self.config_errors.load(Ordering::Relaxed),
                rate_limit_errors: self.rate_limit_errors.load(Ordering::Relaxed),
            },
            performance_breakdown: JwtPerformanceBreakdown {
                avg_token_validation_time_ms: if successful_requests > 0 {
                    self.token_validation_time_ms.load(Ordering::Relaxed) as f64 / successful_requests as f64
                } else {
                    0.0
                },
                avg_token_extraction_time_ms: if total_requests > 0 {
                    self.token_extraction_time_ms.load(Ordering::Relaxed) as f64 / total_requests as f64
                } else {
                    0.0
                },
            },
            extraction_stats: JwtExtractionStats {
                header_extractions: self.header_extractions.load(Ordering::Relaxed),
                cookie_extractions: self.cookie_extractions.load(Ordering::Relaxed),
                query_extractions: self.query_extractions.load(Ordering::Relaxed),
                extraction_failures: self.extraction_failures.load(Ordering::Relaxed),
                header_extraction_rate,
                cookie_extraction_rate,
                query_extraction_rate,
            },
        }
    }
}

/// 性能报告结构
#[derive(Debug, Serialize)]
pub struct JwtPerformanceReport {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_processing_time_ms: f64,
    pub error_breakdown: JwtErrorBreakdown,
    pub performance_breakdown: JwtPerformanceBreakdown,
    pub extraction_stats: JwtExtractionStats,
}

/// 错误分类统计
#[derive(Debug, Serialize)]
pub struct JwtErrorBreakdown {
    pub token_missing_errors: u32,
    pub token_invalid_errors: u32,
    pub token_expired_errors: u32,
    pub insufficient_permission_errors: u32,
    pub config_errors: u32,
    pub rate_limit_errors: u32,
}

/// 性能分类统计
#[derive(Debug, Serialize)]
pub struct JwtPerformanceBreakdown {
    pub avg_token_validation_time_ms: f64,
    pub avg_token_extraction_time_ms: f64,
}

/// 令牌提取统计
#[derive(Debug, Serialize)]
pub struct JwtExtractionStats {
    pub header_extractions: u64,
    pub cookie_extractions: u64,
    pub query_extractions: u64,
    pub extraction_failures: u64,
    pub header_extraction_rate: f64,
    pub cookie_extraction_rate: f64,
    pub query_extraction_rate: f64,
}

/// JWT 认证中间件
pub struct JwtMiddleware {
    config: JwtConfig,
    excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>,
    required_roles: Option<HashSet<String>>,
    require_all_roles: bool,
    metrics: Arc<JwtMetrics>,
    rate_limiter: Option<JwtRateLimiter>,
}

impl Clone for JwtMiddleware {
    fn clone(&self) -> Self {
        JwtMiddleware {
            config: self.config.clone(),
            excludes: self.excludes.clone(),
            required_roles: self.required_roles.clone(),
            require_all_roles: self.require_all_roles,
            metrics: Arc::clone(&self.metrics),
            rate_limiter: self.rate_limiter.clone(),
        }
    }
}

impl JwtMiddleware {
    /// 创建新的 JWT 认证中间件
    pub fn new(config: JwtConfig) -> Self {
        JwtMiddleware {
            config,
            excludes: vec![],
            required_roles: None,
            require_all_roles: false,
            metrics: JwtMetrics::new(),
            rate_limiter: None,
        }
    }

    /// 添加速率限制器
    pub fn with_rate_limiter(mut self, rate_limiter: JwtRateLimiter) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> Arc<JwtMetrics> {
        Arc::clone(&self.metrics)
    }

    /// 获取性能报告
    pub fn get_performance_report(&self) -> JwtPerformanceReport {
        self.metrics.get_performance_report()
    }

    /// 添加排除规则
    pub fn add_exclude(&mut self, exclude: fn(parts: &axum::http::request::Parts) -> bool) -> &mut Self {
        self.excludes.push(exclude);
        self
    }

    /// 设置排除规则列表
    pub fn with_excludes(mut self, excludes: Vec<fn(parts: &axum::http::request::Parts) -> bool>) -> Self {
        self.excludes.extend(excludes);
        self
    }

    /// 要求特定角色
    pub fn require_role(mut self, role: &str) -> Self {
        let mut roles = HashSet::new();
        roles.insert(role.to_string());
        self.required_roles = Some(roles);
        self.require_all_roles = true;
        self
    }

    /// 要求多个角色（全部）
    pub fn require_all_roles(mut self, roles: Vec<&str>) -> Self {
        let roles_set: HashSet<String> = roles.into_iter().map(String::from).collect();
        self.required_roles = Some(roles_set);
        self.require_all_roles = true;
        self
    }

    /// 要求任一角色
    pub fn require_any_role(mut self, roles: Vec<&str>) -> Self {
        let roles_set: HashSet<String> = roles.into_iter().map(String::from).collect();
        self.required_roles = Some(roles_set);
        self.require_all_roles = false;
        self
    }

    /// 从请求中提取 JWT 令牌
    async fn extract_token(&self, headers: &HeaderMap, cookies: &CookieJar, query: Option<&str>) -> Result<String, JwtError> {
        let extraction_start = Instant::now();
        let mut token = None;

        // 1. 从 Authorization 头提取
        if let Some(auth_header) = headers.get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with(&self.config.token_prefix) {
                    token = Some(auth_str[self.config.token_prefix.len()..].to_string());
                    self.metrics.record_header_extraction();
                }
            }
        }

        // 2. 从 Cookie 提取
        if token.is_none() && self.config.extract_from_cookie {
            if let Some(cookie) = cookies.get(&self.config.cookie_name) {
                token = Some(cookie.value().to_string());
                self.metrics.record_cookie_extraction();
            }
        }

        // 3. 从查询参数提取
        if token.is_none() && self.config.extract_from_query && query.is_some() {
            let query_str = query.unwrap();
            // 简单的查询参数解析
            for param in query_str.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    if key == self.config.query_param {
                        token = Some(value.to_string());
                        self.metrics.record_query_extraction();
                        break;
                    }
                }
            }
        }

        // 记录提取时间
        let extraction_time = extraction_start.elapsed().as_millis() as u64;
        self.metrics.record_token_extraction_time(extraction_time);

        // 返回结果
        match token {
            Some(t) => Ok(t),
            None => {
                self.metrics.record_extraction_failure();
                Err(JwtError::TokenMissing)
            }
        }
    }

    /// 验证 JWT 令牌
    async fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation_start = Instant::now();

        // 使用超时保护验证操作
        let validation_result = timeout(
            Duration::from_millis(TOKEN_VALIDATION_TIMEOUT_MS),
            async {
                let generator = JwtGenerator::new(self.config.clone());
                generator.verify_token(token)
            }
        ).await;

        // 记录验证时间
        let validation_time = validation_start.elapsed().as_millis() as u64;
        self.metrics.record_token_validation_time(validation_time);

        // 处理超时和验证结果
        let claims = match validation_result {
            Ok(result) => result?,
            Err(_) => return Err(JwtError::TokenInvalid("Token validation timed out".to_string())),
        };

        // 检查是否过期
        if claims.is_expired() {
            return Err(JwtError::TokenExpired);
        }

        // 检查角色要求
        if let Some(required_roles) = &self.required_roles {
            if self.require_all_roles {
                // 必须具有所有角色
                for role in required_roles {
                    if !claims.has_role(role) {
                        return Err(JwtError::InsufficientPermission(
                            format!("Missing required role: {}", role)
                        ));
                    }
                }
            } else {
                // 具有任一角色即可
                if !required_roles.iter().any(|role| claims.has_role(role)) {
                    let roles_str = required_roles.iter().cloned().collect::<Vec<_>>().join(", ");
                    return Err(JwtError::InsufficientPermission(
                        format!("User does not have any of the required roles: {}", roles_str)
                    ));
                }
            }
        }

        Ok(claims)
    }

    /// 检查速率限制
    async fn check_rate_limit(&self, claims: &Claims, endpoint: &str) -> Result<(), JwtError> {
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.check_rate_limit(claims, endpoint).await?;
        }
        Ok(())
    }

    /// 验证请求
    async fn validate_request(&self, request: Request) -> Result<Request, Response> {
        let start_time = Instant::now();
        self.metrics.record_request_start();

        let result = self.validate_request_internal(request).await;
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => {
                self.metrics.record_request_success(processing_time_ms);
            }
            Err(_) => {
                // 记录为通用失败
                self.metrics.record_request_failure(
                    &JwtError::TokenInvalid("Request validation failed".to_string()), 
                    processing_time_ms
                );
            }
        }

        result
    }

    /// 验证请求的内部实现
    async fn validate_request_internal(&self, mut request: Request) -> Result<Request, Response> {
        // 获取请求头和查询参数
        let headers = request.headers();
        let query = request.uri().query();
        let endpoint = request.uri().path();

        // 获取 Cookie
        let cookies = CookieJar::from_headers(headers);

        // 提取令牌
        let token = self.extract_token(headers, &cookies, query).await
            .map_err(|e| e.into_response())?;

        // 验证令牌
        let claims = self.validate_token(&token).await
            .map_err(|e| e.into_response())?;

        // 检查速率限制
        self.check_rate_limit(&claims, endpoint).await
            .map_err(|e| e.into_response())?;

        // 将用户信息添加到请求上下文
        let context = Context::new(claims.sub.clone());
        request.extensions_mut().insert(context);

        // 将完整的声明添加到请求扩展中，以便后续处理
        request.extensions_mut().insert(claims);

        Ok(request)
    }

    /// 检查是否应该排除请求
    fn should_exclude(&self, parts: &axum::http::request::Parts) -> bool {
        self.excludes.iter().any(|exclude| exclude(parts))
    }
}

impl Middleware for JwtMiddleware {
    fn focus(&self, parts: &axum::http::request::Parts) -> bool {
        !self.should_exclude(parts)
    }

    fn priority(&self) -> i32 {
        80 // 高优先级，在日志之后，在业务逻辑之前
    }

    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(self.validate_request(request))
    }

    fn name(&self) -> &'static str {
        "JwtMiddleware"
    }
}

/// JWT 性能监控器
pub struct JwtMonitor {
    middleware: JwtMiddleware,
}

impl JwtMonitor {
    /// 创建新的 JWT 监控器
    pub fn new(middleware: JwtMiddleware) -> Self {
        JwtMonitor { middleware }
    }

    /// 获取当前性能报告
    pub fn get_report(&self) -> JwtPerformanceReport {
        self.middleware.get_performance_report()
    }

    /// 启动定期性能报告
    pub fn start_periodic_reporting(&self, interval_seconds: u64) -> tokio::task::JoinHandle<()> {
        let metrics = self.middleware.get_metrics();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                let report = metrics.get_performance_report();
                
                println!("=== JWT Middleware Performance Report ===");
                println!("Total Requests: {}", report.total_requests);
                println!("Success Rate: {:.2}%", report.success_rate);
                println!("Average Processing Time: {:.2}ms", report.avg_processing_time_ms);
                println!("Token Missing Errors: {}", report.error_breakdown.token_missing_errors);
                println!("Token Invalid Errors: {}", report.error_breakdown.token_invalid_errors);
                println!("Token Expired Errors: {}", report.error_breakdown.token_expired_errors);
                println!("Permission Errors: {}", report.error_breakdown.insufficient_permission_errors);
                println!("Rate Limit Errors: {}", report.error_breakdown.rate_limit_errors);
                println!("Header Extractions: {} ({:.1}%)", 
                    report.extraction_stats.header_extractions,
                    report.extraction_stats.header_extraction_rate);
                println!("Cookie Extractions: {} ({:.1}%)", 
                    report.extraction_stats.cookie_extractions,
                    report.extraction_stats.cookie_extraction_rate);
                println!("Query Extractions: {} ({:.1}%)", 
                    report.extraction_stats.query_extractions,
                    report.extraction_stats.query_extraction_rate);
                println!("==========================================");
            }
        })
    }

    /// 打印详细的性能报告
    pub fn print_detailed_report(&self) {
        let report = self.get_report();
        
        println!("\n=== JWT Middleware Detailed Performance Report ===");
        
        // 基本统计
        println!("📊 Request Statistics:");
        println!("  Total Requests: {}", report.total_requests);
        println!("  Successful: {} ({:.2}%)", report.successful_requests, report.success_rate);
        println!("  Failed: {} ({:.2}%)", report.failed_requests, 100.0 - report.success_rate);
        
        // 性能指标
        println!("\n⏱️  Performance Metrics:");
        println!("  Average Processing Time: {:.2}ms", report.avg_processing_time_ms);
        println!("  Average Token Validation Time: {:.2}ms", report.performance_breakdown.avg_token_validation_time_ms);
        println!("  Average Token Extraction Time: {:.2}ms", report.performance_breakdown.avg_token_extraction_time_ms);
        
        // 错误分析
        println!("\n❌ Error Breakdown:");
        println!("  Token Missing: {}", report.error_breakdown.token_missing_errors);
        println!("  Token Invalid: {}", report.error_breakdown.token_invalid_errors);
        println!("  Token Expired: {}", report.error_breakdown.token_expired_errors);
        println!("  Insufficient Permission: {}", report.error_breakdown.insufficient_permission_errors);
        println!("  Configuration Errors: {}", report.error_breakdown.config_errors);
        println!("  Rate Limit Errors: {}", report.error_breakdown.rate_limit_errors);
        
        // 提取统计
        println!("\n🔍 Token Extraction Statistics:");
        println!("  Header Extractions: {} ({:.1}%)", 
            report.extraction_stats.header_extractions,
            report.extraction_stats.header_extraction_rate);
        println!("  Cookie Extractions: {} ({:.1}%)", 
            report.extraction_stats.cookie_extractions,
            report.extraction_stats.cookie_extraction_rate);
        println!("  Query Extractions: {} ({:.1}%)", 
            report.extraction_stats.query_extractions,
            report.extraction_stats.query_extraction_rate);
        println!("  Extraction Failures: {}", report.extraction_stats.extraction_failures);
        
        println!("=====================================================\n");
    }
}