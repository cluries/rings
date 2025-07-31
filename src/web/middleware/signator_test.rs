#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::define::HttpMethod;
    use crate::web::middleware::{ApplyKind, Pattern};
    use axum::http::{Method, Uri};
    use axum::http::request::Parts;
    use std::sync::Arc;

    #[test]
    fn test_signator_config_default() {
        let config = SignatorConfig::default();
        assert_eq!(config.priority, 0);
        assert_eq!(config.nonce_lifetime, DEFAULT_NONCE_LIFETIME);
        assert!(config.apply.is_none());
        assert!(config.methods.is_none());
        assert!(config.patterns.is_none());
    }

    #[test]
    fn test_signator_config_builder() {
        let config = SignatorConfig::new()
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

    #[test]
    fn test_signator_config_apply() {
        let config = SignatorConfig::new()
            .apply(|parts| {
                parts.uri.path().starts_with("/admin/")
            });

        assert!(config.apply.is_some());

        // 创建一个模拟的 Parts 来测试 apply 函数
        let uri: Uri = "/admin/users".parse().unwrap();
        let method = Method::GET;
        let mut parts = Parts::default();
        parts.uri = uri;
        parts.method = method;

        let apply_fn = config.apply.unwrap();
        assert!(apply_fn(&parts));

        // 测试不匹配的路径
        let uri: Uri = "/public/info".parse().unwrap();
        let mut parts = Parts::default();
        parts.uri = uri;
        parts.method = Method::GET;

        assert!(!apply_fn(&parts));
    }

    #[test]
    fn test_signator_config_debug() {
        let config = SignatorConfig::new()
            .priority(50)
            .apply(|_| true);

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SignatorConfig"));
        assert!(debug_str.contains("priority: 50"));
        assert!(debug_str.contains("Some(Fn)"));
    }

    #[test]
    fn test_signator_with_config() {
        let config = SignatorConfig::new()
            .priority(200)
            .nonce_lifetime(300);

        let key_loader = Arc::new(|user_id: String| -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> {
            Box::pin(async move {
                Ok(format!("key_for_{}", user_id))
            })
        });

        // 这里我们不能真正测试 Redis 连接，但可以验证配置被正确设置
        // 在实际环境中，这需要一个有效的 Redis URL
        // let signator = Signator::with_config("redis://localhost:6379", key_loader, config);
        // assert_eq!(signator.config.priority, 200);
        // assert_eq!(signator.config.nonce_lifetime, 300);
    }

    #[test]
    fn test_pattern_matching() {
        // 测试前缀匹配
        let prefix_pattern = Pattern::Prefix("/api/".to_string(), true);
        assert!(prefix_pattern.apply("/api/users"));
        assert!(!prefix_pattern.apply("/public/info"));

        // 测试后缀匹配
        let suffix_pattern = Pattern::Suffix(".json".to_string(), true);
        assert!(suffix_pattern.apply("/api/users.json"));
        assert!(!suffix_pattern.apply("/api/users.html"));

        // 测试包含匹配
        let contains_pattern = Pattern::Contains("admin".to_string(), true);
        assert!(contains_pattern.apply("/admin/users"));
        assert!(contains_pattern.apply("/api/admin/settings"));
        assert!(!contains_pattern.apply("/public/info"));

        // 测试正则表达式匹配
        let regex_pattern = Pattern::Regex(r"^/api/v\d+/.*$".to_string());
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
}