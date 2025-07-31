use rings::web::middleware::signator::{Signator, SignatorConfig};
use rings::web::middleware::Pattern;
use rings::web::define::HttpMethod;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

// 示例：如何配置 Signator 中间件
fn main() {
    // 1. 基本配置
    let basic_config = SignatorConfig::new()
        .priority(100)
        .nonce_lifetime(600); // 10分钟

    // 2. 配置 HTTP 方法过滤
    let method_config = SignatorConfig::new()
        .include_method(HttpMethod::POST)
        .include_method(HttpMethod::PUT)
        .exclude_method(HttpMethod::GET);

    // 3. 配置路径匹配模式
    let pattern_config = SignatorConfig::new()
        .include_prefix("/api/", true)  // 包含以 /api/ 开头的路径，区分大小写
        .exclude_suffix(".html", false) // 排除以 .html 结尾的路径，不区分大小写
        .include_regex(r"^/admin/.*$");  // 包含匹配正则表达式的路径

    // 4. 自定义应用逻辑
    let custom_apply_config = SignatorConfig::new()
        .apply(|parts| {
            // 自定义逻辑：只对特定的用户代理应用签名验证
            parts.headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|ua| ua.contains("MyApp"))
                .unwrap_or(false)
        });

    // 5. 综合配置示例
    let comprehensive_config = SignatorConfig::new()
        .priority(200)
        .nonce_lifetime(300)
        .include_method(HttpMethod::POST)
        .include_method(HttpMethod::PUT)
        .include_method(HttpMethod::DELETE)
        .include_prefix("/api/v1/", true)
        .exclude_pattern(Pattern::Contains("public".to_string(), false))
        .apply(|parts| {
            // 复杂的自定义逻辑
            let path = parts.uri.path();
            let method = parts.method.as_str();
            
            // 对管理员路径总是应用签名验证
            if path.starts_with("/admin/") {
                return true;
            }
            
            // 对 GET 请求不应用签名验证
            if method == "GET" {
                return false;
            }
            
            // 其他情况根据路径判断
            path.starts_with("/api/") && !path.contains("/public/")
        });

    // 创建 key loader
    let key_loader = Arc::new(|user_id: String| -> Pin<Box<dyn Future<Output = Result<String, rings::erx::Erx>> + Send>> {
        Box::pin(async move {
            // 这里应该从数据库或其他存储中加载用户的签名密钥
            Ok(format!("secret_key_for_{}", user_id))
        })
    });

    // 使用配置创建 Signator 实例
    let _signator1 = Signator::with_config("redis://localhost:6379", key_loader.clone(), basic_config);
    let _signator2 = Signator::with_config("redis://localhost:6379", key_loader.clone(), method_config);
    let _signator3 = Signator::with_config("redis://localhost:6379", key_loader.clone(), pattern_config);
    let _signator4 = Signator::with_config("redis://localhost:6379", key_loader.clone(), custom_apply_config);
    let _signator5 = Signator::with_config("redis://localhost:6379", key_loader.clone(), comprehensive_config);

    // 也可以同时设置配置和后门
    let backdoor_config = SignatorConfig::new()
        .priority(50)
        .include_method(HttpMethod::POST);
    
    let _signator_with_backdoor = Signator::with_config_and_backdoor(
        "redis://localhost:6379", 
        key_loader, 
        backdoor_config, 
        "dev_skip_token".to_string()
    );

    println!("Signator 配置示例完成！");
}