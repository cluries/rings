//! # JWT 令牌刷新模块
//! 
//! 提供 JWT 令牌的自动刷新和管理功能，包括：
//! - 访问令牌和刷新令牌的管理
//! - 自动令牌刷新机制
//! - 令牌黑名单管理
//! - 安全的令牌轮换

use super::*;
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

/// 令牌类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    /// 访问令牌 - 用于API访问，生命周期较短
    Access,
    /// 刷新令牌 - 用于获取新的访问令牌，生命周期较长
    Refresh,
}

/// 令牌对结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// 访问令牌过期时间
    pub access_expires_in: i64,
    /// 刷新令牌过期时间
    pub refresh_expires_in: i64,
    /// 令牌类型
    pub token_type: String,
}

/// 刷新令牌声明
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    /// 主题（用户ID）
    pub sub: String,
    /// 过期时间
    pub exp: i64,
    /// 签发时间
    pub iat: i64,
    /// 令牌ID（用于撤销）
    pub jti: String,
    /// 令牌类型
    pub token_type: TokenType,
    /// 关联的访问令牌ID
    pub access_token_id: Option<String>,
}

impl RefreshClaims {
    /// 创建新的刷新令牌声明
    pub fn new(user_id: &str, expires_in_seconds: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        RefreshClaims {
            sub: user_id.to_string(),
            exp: now + expires_in_seconds,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
            access_token_id: None,
        }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }
}

/// 令牌黑名单管理器
#[derive(Debug)]
pub struct TokenBlacklist {
    /// 黑名单令牌存储
    blacklisted_tokens: RwLock<HashMap<String, Instant>>,
    /// 清理间隔
    cleanup_interval: Duration,
}

impl TokenBlacklist {
    /// 创建新的令牌黑名单管理器
    pub fn new() -> Self {
        TokenBlacklist {
            blacklisted_tokens: RwLock::new(HashMap::new()),
            cleanup_interval: Duration::from_secs(3600), // 1小时清理一次
        }
    }

    /// 将令牌添加到黑名单
    pub fn blacklist_token(&self, token_id: &str, expires_at: Instant) {
        let mut blacklist = self.blacklisted_tokens.write().unwrap();
        blacklist.insert(token_id.to_string(), expires_at);
    }

    /// 检查令牌是否在黑名单中
    pub fn is_blacklisted(&self, token_id: &str) -> bool {
        let blacklist = self.blacklisted_tokens.read().unwrap();
        blacklist.contains_key(token_id)
    }

    /// 清理过期的黑名单条目
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut blacklist = self.blacklisted_tokens.write().unwrap();
        blacklist.retain(|_, &mut expires_at| expires_at > now);
    }

    /// 启动定期清理任务
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let cleanup_interval = self.cleanup_interval;
        let blacklist = Arc::new(self);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                blacklist.cleanup_expired();
            }
        })
    }

    /// 获取黑名单大小
    pub fn size(&self) -> usize {
        let blacklist = self.blacklisted_tokens.read().unwrap();
        blacklist.len()
    }
}

/// JWT 令牌刷新管理器
pub struct JwtRefreshManager {
    config: JwtConfig,
    blacklist: Arc<TokenBlacklist>,
    /// 访问令牌生命周期（秒）
    access_token_lifetime: i64,
    /// 刷新令牌生命周期（秒）
    refresh_token_lifetime: i64,
}

impl JwtRefreshManager {
    /// 创建新的令牌刷新管理器
    pub fn new(config: JwtConfig) -> Self {
        JwtRefreshManager {
            config,
            blacklist: Arc::new(TokenBlacklist::new()),
            access_token_lifetime: 900,  // 15分钟
            refresh_token_lifetime: 7 * 24 * 3600, // 7天
        }
    }

    /// 设置访问令牌生命周期
    pub fn with_access_token_lifetime(mut self, seconds: i64) -> Self {
        self.access_token_lifetime = seconds;
        self
    }

    /// 设置刷新令牌生命周期
    pub fn with_refresh_token_lifetime(mut self, seconds: i64) -> Self {
        self.refresh_token_lifetime = seconds;
        self
    }

    /// 生成令牌对
    pub fn generate_token_pair(&self, user_id: &str, roles: Vec<String>) -> Result<TokenPair, JwtError> {
        let generator = JwtGenerator::new(self.config.clone());
        
        // 生成访问令牌
        let mut access_claims = Claims::new(user_id);
        access_claims.roles = Some(roles);
        access_claims.set_expiration(self.access_token_lifetime);
        
        // 添加令牌ID用于撤销
        let access_token_id = Uuid::new_v4().to_string();
        access_claims.data = Some(serde_json::json!({
            "jti": access_token_id,
            "token_type": "access"
        }));
        
        let access_token = generator.generate_token(&access_claims)?;
        
        // 生成刷新令牌
        let mut refresh_claims = RefreshClaims::new(user_id, self.refresh_token_lifetime);
        refresh_claims.access_token_id = Some(access_token_id);
        
        let refresh_token = encode(
            &Header::new(self.config.algorithm),
            &refresh_claims,
            &self.config.encoding_key()
        ).map_err(|e| JwtError::ConfigError(format!("Refresh token generation failed: {}", e)))?;
        
        Ok(TokenPair {
            access_token,
            refresh_token,
            access_expires_in: self.access_token_lifetime,
            refresh_expires_in: self.refresh_token_lifetime,
            token_type: "Bearer".to_string(),
        })
    }

    /// 刷新访问令牌
    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenPair, JwtError> {
        // 验证刷新令牌
        let refresh_claims = self.verify_refresh_token(refresh_token)?;
        
        // 检查刷新令牌是否在黑名单中
        if self.blacklist.is_blacklisted(&refresh_claims.jti) {
            return Err(JwtError::TokenInvalid("Refresh token has been revoked".to_string()));
        }
        
        // 检查是否过期
        if refresh_claims.is_expired() {
            return Err(JwtError::TokenExpired);
        }
        
        // 生成新的令牌对
        // 注意：这里需要从某个地方获取用户的角色信息
        // 在实际应用中，你可能需要从数据库或缓存中获取
        let roles = self.get_user_roles(&refresh_claims.sub)?;
        
        let new_token_pair = self.generate_token_pair(&refresh_claims.sub, roles)?;
        
        // 将旧的刷新令牌加入黑名单
        let expires_at = Instant::now() + Duration::from_secs(self.refresh_token_lifetime as u64);
        self.blacklist.blacklist_token(&refresh_claims.jti, expires_at);
        
        Ok(new_token_pair)
    }

    /// 验证刷新令牌
    fn verify_refresh_token(&self, token: &str) -> Result<RefreshClaims, JwtError> {
        let token_data = decode::<RefreshClaims>(
            token,
            &self.config.decoding_key(),
            &self.config.validation
        ).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                _ => JwtError::TokenInvalid(e.to_string()),
            }
        })?;

        // 验证令牌类型
        if token_data.claims.token_type != TokenType::Refresh {
            return Err(JwtError::TokenInvalid("Invalid token type".to_string()));
        }

        Ok(token_data.claims)
    }

    /// 撤销令牌
    pub fn revoke_token(&self, token: &str) -> Result<(), JwtError> {
        // 尝试解析为访问令牌
        if let Ok(claims) = self.verify_access_token(token) {
            if let Some(data) = &claims.data {
                if let Some(jti) = data.get("jti").and_then(|v| v.as_str()) {
                    let expires_at = Instant::now() + Duration::from_secs(self.access_token_lifetime as u64);
                    self.blacklist.blacklist_token(jti, expires_at);
                    return Ok(());
                }
            }
        }
        
        // 尝试解析为刷新令牌
        if let Ok(refresh_claims) = self.verify_refresh_token(token) {
            let expires_at = Instant::now() + Duration::from_secs(self.refresh_token_lifetime as u64);
            self.blacklist.blacklist_token(&refresh_claims.jti, expires_at);
            return Ok(());
        }
        
        Err(JwtError::TokenInvalid("Unable to parse token for revocation".to_string()))
    }

    /// 验证访问令牌（内部使用）
    fn verify_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let generator = JwtGenerator::new(self.config.clone());
        generator.verify_token(token)
    }

    /// 获取用户角色（需要根据实际情况实现）
    fn get_user_roles(&self, _user_id: &str) -> Result<Vec<String>, JwtError> {
        // 这里应该从数据库或缓存中获取用户角色
        // 为了示例，返回默认角色
        Ok(vec!["user".to_string()])
    }

    /// 获取黑名单管理器
    pub fn get_blacklist(&self) -> Arc<TokenBlacklist> {
        Arc::clone(&self.blacklist)
    }

    /// 启动清理任务
    pub fn start_cleanup_tasks(&self) -> tokio::task::JoinHandle<()> {
        self.blacklist.start_cleanup_task()
    }
}

/// 令牌刷新中间件
pub struct JwtRefreshMiddleware {
    refresh_manager: Arc<JwtRefreshManager>,
    /// 自动刷新阈值（秒）- 当令牌剩余时间少于此值时自动刷新
    auto_refresh_threshold: i64,
}

impl JwtRefreshMiddleware {
    /// 创建新的令牌刷新中间件
    pub fn new(refresh_manager: JwtRefreshManager) -> Self {
        JwtRefreshMiddleware {
            refresh_manager: Arc::new(refresh_manager),
            auto_refresh_threshold: 300, // 5分钟
        }
    }

    /// 设置自动刷新阈值
    pub fn with_auto_refresh_threshold(mut self, seconds: i64) -> Self {
        self.auto_refresh_threshold = seconds;
        self
    }

    /// 检查令牌是否需要刷新
    pub fn should_refresh_token(&self, claims: &Claims) -> bool {
        if let Some(exp) = claims.exp {
            let now = chrono::Utc::now().timestamp();
            let remaining = exp - now;
            remaining <= self.auto_refresh_threshold
        } else {
            false
        }
    }

    /// 处理令牌刷新请求
    pub async fn handle_refresh_request(&self, refresh_token: &str) -> Result<TokenPair, JwtError> {
        self.refresh_manager.refresh_access_token(refresh_token)
    }

    /// 处理令牌撤销请求
    pub async fn handle_revoke_request(&self, token: &str) -> Result<(), JwtError> {
        self.refresh_manager.revoke_token(token)
    }
}

impl Clone for JwtRefreshMiddleware {
    fn clone(&self) -> Self {
        JwtRefreshMiddleware {
            refresh_manager: Arc::clone(&self.refresh_manager),
            auto_refresh_threshold: self.auto_refresh_threshold,
        }
    }
}

/// 令牌刷新响应头中间件
/// 当检测到令牌即将过期时，在响应头中添加新的令牌
pub struct TokenRefreshResponseMiddleware {
    refresh_middleware: JwtRefreshMiddleware,
}

impl TokenRefreshResponseMiddleware {
    pub fn new(refresh_middleware: JwtRefreshMiddleware) -> Self {
        TokenRefreshResponseMiddleware {
            refresh_middleware,
        }
    }

    /// 处理响应，如果需要则添加刷新的令牌
    pub async fn process_response(&self, mut response: Response, claims: &Claims) -> Response {
        if self.refresh_middleware.should_refresh_token(claims) {
            // 在实际应用中，你需要从某个地方获取刷新令牌
            // 这里只是示例
            if let Ok(new_token_pair) = self.refresh_middleware.refresh_manager
                .generate_token_pair(&claims.sub, claims.roles.clone().unwrap_or_default()) {
                
                // 在响应头中添加新的访问令牌
                response.headers_mut().insert(
                    "X-New-Access-Token",
                    new_token_pair.access_token.parse().unwrap()
                );
                
                response.headers_mut().insert(
                    "X-Token-Expires-In",
                    new_token_pair.access_expires_in.to_string().parse().unwrap()
                );
            }
        }
        
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_blacklist() {
        let blacklist = TokenBlacklist::new();
        let token_id = "test-token-123";
        let expires_at = Instant::now() + Duration::from_secs(3600);
        
        // 测试添加到黑名单
        assert!(!blacklist.is_blacklisted(token_id));
        blacklist.blacklist_token(token_id, expires_at);
        assert!(blacklist.is_blacklisted(token_id));
        
        // 测试大小
        assert_eq!(blacklist.size(), 1);
    }

    #[tokio::test]
    async fn test_token_pair_generation() {
        let config = JwtConfig::new("test-secret");
        let refresh_manager = JwtRefreshManager::new(config);
        
        let token_pair = refresh_manager.generate_token_pair(
            "test-user",
            vec!["user".to_string(), "editor".to_string()]
        ).unwrap();
        
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.token_type, "Bearer");
        assert!(token_pair.access_expires_in > 0);
        assert!(token_pair.refresh_expires_in > 0);
    }

    #[tokio::test]
    async fn test_refresh_claims() {
        let claims = RefreshClaims::new("test-user", 3600);
        
        assert_eq!(claims.sub, "test-user");
        assert_eq!(claims.token_type, TokenType::Refresh);
        assert!(!claims.jti.is_empty());
        assert!(!claims.is_expired());
        
        // 测试过期的声明
        let expired_claims = RefreshClaims::new("test-user", -10);
        assert!(expired_claims.is_expired());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let config = JwtConfig::new("test-secret");
        let refresh_manager = JwtRefreshManager::new(config);
        
        let token_pair = refresh_manager.generate_token_pair(
            "test-user",
            vec!["user".to_string()]
        ).unwrap();
        
        // 撤销刷新令牌
        let result = refresh_manager.revoke_token(&token_pair.refresh_token);
        assert!(result.is_ok());
        
        // 尝试使用被撤销的令牌刷新
        let refresh_result = refresh_manager.refresh_access_token(&token_pair.refresh_token);
        assert!(refresh_result.is_err());
    }
}