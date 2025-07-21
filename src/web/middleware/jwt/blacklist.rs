//! # JWT 令牌黑名单模块
//! 
//! 提供 JWT 令牌黑名单功能，用于撤销已发布的令牌和防止恶意令牌使用。

use super::*;
use std::collections::HashSet;
use std::sync::RwLock;
use tokio::time::{Duration, Instant};

/// 黑名单条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    /// 令牌ID (jti claim)
    pub token_id: String,
    /// 用户ID
    pub user_id: String,
    /// 加入黑名单的时间
    pub blacklisted_at: i64,
    /// 令牌过期时间
    pub expires_at: i64,
    /// 黑名单原因
    pub reason: String,
}

impl BlacklistEntry {
    /// 创建新的黑名单条目
    pub fn new(token_id: &str, user_id: &str, expires_at: i64, reason: &str) -> Self {
        BlacklistEntry {
            token_id: token_id.to_string(),
            user_id: user_id.to_string(),
            blacklisted_at: chrono::Utc::now().timestamp(),
            expires_at,
            reason: reason.to_string(),
        }
    }

    /// 检查条目是否已过期
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now > self.expires_at
    }
}

/// 黑名单存储接口
pub trait BlacklistStore: Send + Sync {
    /// 添加令牌到黑名单
    fn add_to_blacklist(&self, entry: BlacklistEntry) -> Result<(), JwtError>;
    
    /// 检查令牌是否在黑名单中
    fn is_blacklisted(&self, token_id: &str) -> Result<bool, JwtError>;
    
    /// 从黑名单中移除令牌
    fn remove_from_blacklist(&self, token_id: &str) -> Result<(), JwtError>;
    
    /// 将用户的所有令牌加入黑名单
    fn blacklist_user_tokens(&self, user_id: &str, reason: &str) -> Result<u64, JwtError>;
    
    /// 清理过期的黑名单条目
    fn cleanup_expired_entries(&self) -> Result<u64, JwtError>;
    
    /// 获取黑名单统计信息
    fn get_blacklist_stats(&self) -> Result<BlacklistStats, JwtError>;
}

/// 内存黑名单存储
pub struct InMemoryBlacklistStore {
    entries: RwLock<HashSet<String>>, // 只存储token_id，简化实现
    detailed_entries: RwLock<std::collections::HashMap<String, BlacklistEntry>>,
}

impl InMemoryBlacklistStore {
    /// 创建新的内存黑名单存储
    pub fn new() -> Self {
        InMemoryBlacklistStore {
            entries: RwLock::new(HashSet::new()),
            detailed_entries: RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for InMemoryBlacklistStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BlacklistStore for InMemoryBlacklistStore {
    fn add_to_blacklist(&self, entry: BlacklistEntry) -> Result<(), JwtError> {
        let mut entries = self.entries.write().map_err(|_| {
            JwtError::ConfigError("Failed to acquire write lock".to_string())
        })?;
        let mut detailed = self.detailed_entries.write().map_err(|_| {
            JwtError::ConfigError("Failed to acquire write lock".to_string())
        })?;
        
        entries.insert(entry.token_id.clone());
        detailed.insert(entry.token_id.clone(), entry);
        Ok(())
    }

    fn is_blacklisted(&self, token_id: &str) -> Result<bool, JwtError> {
        let entries = self.entries.read().map_err(|_| {
            JwtError::ConfigError("Failed to acquire read lock".to_string())
        })?;
        Ok(entries.contains(token_id))
    }

    fn remove_from_blacklist(&self, token_id: &str) -> Result<(), JwtError> {
        let mut entries = self.entries.write().map_err(|_| {
            JwtError::ConfigError("Failed to acquire write lock".to_string())
        })?;
        let mut detailed = self.detailed_entries.write().map_err(|_| {
            JwtError::ConfigError("Failed to acquire write lock".to_string())
        })?;
        
        entries.remove(token_id);
        detailed.remove(token_id);
        Ok(())
    }

    fn blacklist_user_tokens(&self, user_id: &str, reason: &str) -> Result<u64, JwtError> {
        // 在实际实现中，这里需要查找用户的所有活跃令牌
        // 为了简化，这里只是一个占位符实现
        let mut count = 0;
        
        // 这里应该有逻辑来查找和黑名单用户的所有令牌
        // 例如：从数据库或缓存中查找用户的活跃令牌
        
        Ok(count)
    }

    fn cleanup_expired_entries(&self) -> Result<u64, JwtError> {
        let mut entries = self.entries.write().map_err(|_| {
            JwtError::ConfigError("Failed to acquire write lock".to_string())
        })?;
        let mut detailed = self.detailed_entries.write().map_err(|_| {
            JwtError::ConfigError("Failed to acquire write lock".to_string())
        })?;
        
        let initial_count = detailed.len();
        let expired_tokens: Vec<String> = detailed
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(token_id, _)| token_id.clone())
            .collect();
        
        for token_id in &expired_tokens {
            entries.remove(token_id);
            detailed.remove(token_id);
        }
        
        Ok((initial_count - detailed.len()) as u64)
    }

    fn get_blacklist_stats(&self) -> Result<BlacklistStats, JwtError> {
        let detailed = self.detailed_entries.read().map_err(|_| {
            JwtError::ConfigError("Failed to acquire read lock".to_string())
        })?;
        
        let total_entries = detailed.len() as u64;
        let expired_entries = detailed
            .values()
            .filter(|entry| entry.is_expired())
            .count() as u64;
        let active_entries = total_entries - expired_entries;
        
        Ok(BlacklistStats {
            total_entries,
            active_entries,
            expired_entries,
        })
    }
}

/// 黑名单统计信息
#[derive(Debug, Serialize)]
pub struct BlacklistStats {
    pub total_entries: u64,
    pub active_entries: u64,
    pub expired_entries: u64,
}

/// JWT 黑名单管理器
pub struct JwtBlacklistManager {
    store: Arc<dyn BlacklistStore>,
}

impl JwtBlacklistManager {
    /// 创建新的黑名单管理器
    pub fn new(store: Arc<dyn BlacklistStore>) -> Self {
        JwtBlacklistManager { store }
    }

    /// 创建带有默认内存存储的管理器
    pub fn with_memory_store() -> Self {
        let store = Arc::new(InMemoryBlacklistStore::new());
        Self::new(store)
    }

    /// 将令牌加入黑名单
    pub fn blacklist_token(&self, token_id: &str, user_id: &str, expires_at: i64, reason: &str) -> Result<(), JwtError> {
        let entry = BlacklistEntry::new(token_id, user_id, expires_at, reason);
        self.store.add_to_blacklist(entry)
    }

    /// 检查令牌是否被黑名单
    pub fn is_token_blacklisted(&self, token_id: &str) -> Result<bool, JwtError> {
        self.store.is_blacklisted(token_id)
    }

    /// 从黑名单中移除令牌
    pub fn remove_token_from_blacklist(&self, token_id: &str) -> Result<(), JwtError> {
        self.store.remove_from_blacklist(token_id)
    }

    /// 将用户的所有令牌加入黑名单
    pub fn blacklist_user(&self, user_id: &str, reason: &str) -> Result<u64, JwtError> {
        self.store.blacklist_user_tokens(user_id, reason)
    }

    /// 清理过期的黑名单条目
    pub fn cleanup_expired(&self) -> Result<u64, JwtError> {
        self.store.cleanup_expired_entries()
    }

    /// 获取黑名单统计信息
    pub fn get_stats(&self) -> Result<BlacklistStats, JwtError> {
        self.store.get_blacklist_stats()
    }

    /// 启动定期清理任务
    pub fn start_cleanup_task(&self, interval_seconds: u64) -> tokio::task::JoinHandle<()> {
        let store = Arc::clone(&self.store);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                
                match store.cleanup_expired_entries() {
                    Ok(cleaned_count) => {
                        if cleaned_count > 0 {
                            println!("Cleaned up {} expired blacklist entries", cleaned_count);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to cleanup expired blacklist entries: {}", e);
                    }
                }
            }
        })
    }
}

/// 黑名单检查中间件
pub struct BlacklistMiddleware {
    blacklist_manager: Arc<JwtBlacklistManager>,
}

impl BlacklistMiddleware {
    /// 创建新的黑名单中间件
    pub fn new(blacklist_manager: Arc<JwtBlacklistManager>) -> Self {
        BlacklistMiddleware { blacklist_manager }
    }

    /// 从JWT声明中提取令牌ID
    fn extract_token_id(&self, claims: &Claims) -> Option<String> {
        // 如果JWT包含jti (JWT ID) claim，使用它
        // 否则可以使用其他唯一标识符
        claims.data
            .as_ref()
            .and_then(|data| data.get("jti"))
            .and_then(|jti| jti.as_str())
            .map(|s| s.to_string())
    }
}

impl Middleware for BlacklistMiddleware {
    fn focus(&self, _parts: &axum::http::request::Parts) -> bool {
        true // 对所有请求都检查黑名单
    }

    fn priority(&self) -> i32 {
        75 // 高优先级，在JWT验证之后立即执行
    }

    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            // 检查是否有JWT声明
            if let Some(claims) = request.extensions().get::<Claims>() {
                // 尝试提取令牌ID
                if let Some(token_id) = self.extract_token_id(claims) {
                    // 检查令牌是否在黑名单中
                    match self.blacklist_manager.is_token_blacklisted(&token_id) {
                        Ok(true) => {
                            return Err(JwtError::TokenInvalid("Token has been revoked".to_string()).into_response());
                        }
                        Ok(false) => {
                            // 令牌不在黑名单中，继续处理
                        }
                        Err(e) => {
                            eprintln!("Error checking blacklist: {}", e);
                            // 在黑名单检查失败时，可以选择拒绝请求或继续处理
                            // 这里选择继续处理，但在生产环境中可能需要更严格的处理
                        }
                    }
                }
            }
            Ok(request)
        })
    }

    fn name(&self) -> &'static str {
        "BlacklistMiddleware"
    }
}

/// 令牌撤销服务
pub struct TokenRevocationService {
    blacklist_manager: Arc<JwtBlacklistManager>,
    jwt_generator: JwtGenerator,
}

impl TokenRevocationService {
    /// 创建新的令牌撤销服务
    pub fn new(blacklist_manager: Arc<JwtBlacklistManager>, jwt_generator: JwtGenerator) -> Self {
        TokenRevocationService {
            blacklist_manager,
            jwt_generator,
        }
    }

    /// 撤销单个令牌
    pub fn revoke_token(&self, token: &str, reason: &str) -> Result<(), JwtError> {
        // 验证并解析令牌
        let claims = self.jwt_generator.verify_token(token)?;
        
        // 提取令牌信息
        let token_id = claims.data
            .as_ref()
            .and_then(|data| data.get("jti"))
            .and_then(|jti| jti.as_str())
            .unwrap_or(&claims.sub); // 如果没有jti，使用sub作为标识符
        
        let expires_at = claims.exp.unwrap_or_else(|| {
            chrono::Utc::now().timestamp() + 3600 // 默认1小时后过期
        });
        
        // 将令牌加入黑名单
        self.blacklist_manager.blacklist_token(token_id, &claims.sub, expires_at, reason)
    }

    /// 撤销用户的所有令牌
    pub fn revoke_user_tokens(&self, user_id: &str, reason: &str) -> Result<u64, JwtError> {
        self.blacklist_manager.blacklist_user(user_id, reason)
    }

    /// 批量撤销令牌
    pub fn revoke_tokens_batch(&self, tokens: Vec<&str>, reason: &str) -> Result<(u64, u64), JwtError> {
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for token in tokens {
            match self.revoke_token(token, reason) {
                Ok(_) => success_count += 1,
                Err(_) => failure_count += 1,
            }
        }
        
        Ok((success_count, failure_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist_entry_creation() {
        let entry = BlacklistEntry::new("token123", "user456", 1234567890, "User logout");
        
        assert_eq!(entry.token_id, "token123");
        assert_eq!(entry.user_id, "user456");
        assert_eq!(entry.expires_at, 1234567890);
        assert_eq!(entry.reason, "User logout");
    }

    #[test]
    fn test_blacklist_entry_expiration() {
        let expired_entry = BlacklistEntry::new("token123", "user456", 1, "Test");
        assert!(expired_entry.is_expired());
        
        let future_time = chrono::Utc::now().timestamp() + 3600;
        let active_entry = BlacklistEntry::new("token456", "user789", future_time, "Test");
        assert!(!active_entry.is_expired());
    }

    #[tokio::test]
    async fn test_in_memory_blacklist_store() {
        let store = InMemoryBlacklistStore::new();
        let entry = BlacklistEntry::new("token123", "user456", 1234567890, "Test");
        
        // 添加到黑名单
        store.add_to_blacklist(entry).unwrap();
        
        // 检查是否在黑名单中
        assert!(store.is_blacklisted("token123").unwrap());
        assert!(!store.is_blacklisted("token456").unwrap());
        
        // 从黑名单中移除
        store.remove_from_blacklist("token123").unwrap();
        assert!(!store.is_blacklisted("token123").unwrap());
    }

    #[tokio::test]
    async fn test_blacklist_manager() {
        let manager = JwtBlacklistManager::with_memory_store();
        
        // 将令牌加入黑名单
        manager.blacklist_token("token123", "user456", 1234567890, "Test").unwrap();
        
        // 检查令牌是否被黑名单
        assert!(manager.is_token_blacklisted("token123").unwrap());
        assert!(!manager.is_token_blacklisted("token456").unwrap());
        
        // 从黑名单中移除令牌
        manager.remove_token_from_blacklist("token123").unwrap();
        assert!(!manager.is_token_blacklisted("token123").unwrap());
    }

    #[tokio::test]
    async fn test_blacklist_stats() {
        let manager = JwtBlacklistManager::with_memory_store();
        
        // 添加一些测试条目
        let future_time = chrono::Utc::now().timestamp() + 3600;
        manager.blacklist_token("token1", "user1", future_time, "Test").unwrap();
        manager.blacklist_token("token2", "user2", 1, "Test").unwrap(); // 过期的
        
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.expired_entries, 1);
        assert_eq!(stats.active_entries, 1);
    }
}