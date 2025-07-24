use super::*;
use axum::http::{Method, StatusCode};
use axum::response::Response;

/// 认证中间件示例
pub struct AuthMiddleware {
    _secret_key: String, // 使用 _ 前缀表示暂时未使用
    protected_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(secret_key: String) -> Self {
        Self {
            _secret_key: secret_key,
            protected_paths: vec!["/api/".to_string()],
        }
    }

    pub fn with_protected_paths(mut self, paths: Vec<String>) -> Self {
        self.protected_paths = paths;
        self
    }

    fn is_protected_path(&self, path: &str) -> bool {
        self.protected_paths.iter().any(|p| path.starts_with(p))
    }
}

impl Middleware for AuthMiddleware {
    fn focus(&self, parts: &Parts) -> bool {
        self.is_protected_path(parts.uri.path())
    }

    fn priority(&self) -> i32 {
        90 // 高优先级，在日志之后执行
    }

    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            // 检查 Authorization header
            if let Some(auth_header) = request.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        // 这里可以添加实际的 JWT 验证逻辑
                        println!("Validating token: {}", &auth_str[7..]);
                        return Ok(request);
                    }
                }
            }
            
            Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized".into())
                .unwrap())
        })
    }

    fn name(&self) -> &'static str {
        "AuthMiddleware"
    }

    fn path_pattern(&self) -> Option<&str> {
        Some("/api/*")
    }
}

/// CORS 中间件示例
pub struct CorsMiddleware {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<Method>,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![Method::GET, Method::POST, Method::PUT, Method::DELETE],
        }
    }

    pub fn with_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    pub fn with_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods;
        self
    }
}

impl Middleware for CorsMiddleware {
    fn focus(&self, _parts: &Parts) -> bool {
        true // 对所有请求都应用 CORS
    }

    fn priority(&self) -> i32 {
        110 // 最高优先级，最先执行
    }

    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            // 添加 CORS 相关的 header 到请求中（实际应用中可能需要不同的处理）
            // 这里只是示例，实际的 CORS 处理通常在响应阶段
            
            // 检查 preflight 请求
            if request.method() == Method::OPTIONS {
                return Err(Response::builder()
                    .status(StatusCode::OK)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
                    .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
                    .body("".into())
                    .unwrap());
            }

            Ok(request)
        })
    }

    fn name(&self) -> &'static str {
        "CorsMiddleware"
    }

    fn methods(&self) -> Option<&[Method]> {
        Some(&self.allowed_methods)
    }
}

/// 请求限流中间件示例
pub struct RateLimitMiddleware {
    max_requests: u32,
    window_seconds: u64,
    // 在实际应用中，这里应该使用更复杂的存储机制
    // 比如 Redis 或内存中的 HashMap with TTL
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
        }
    }
}

impl Middleware for RateLimitMiddleware {
    fn focus(&self, parts: &Parts) -> bool {
        // 只对 API 路径进行限流
        parts.uri.path().starts_with("/api/")
    }

    fn priority(&self) -> i32 {
        80 // 中等优先级
    }

    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;
        
        Box::pin(async move {
            // 这里应该实现实际的限流逻辑
            // 检查客户端 IP 的请求频率
            
            // 模拟限流检查
            let client_ip = request
                .headers()
                .get("x-forwarded-for")
                .or_else(|| request.headers().get("x-real-ip"))
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown");

            println!("Rate limiting check for IP: {} (max: {}/{}s)", client_ip, max_requests, window_seconds);
            
            // 在实际应用中，这里会查询存储系统
            // 如果超过限制，返回 429 错误
            
            Ok(request)
        })
    }

    fn name(&self) -> &'static str {
        "RateLimitMiddleware"
    }

    fn path_pattern(&self) -> Option<&str> {
        Some("/api/*")
    }
}

/// 使用示例
pub fn create_middleware_chain() -> MiddlewareChain {
    let manager = MiddlewareBuilder::new()
        .add(CorsMiddleware::new())
        .add(LoggingMiddleware::new(true))
        .add(AuthMiddleware::new("my_secret_key".to_string()))
        .add(RateLimitMiddleware::new(100, 60)) // 每分钟最多 100 个请求
        .build();

    MiddlewareChain::new(manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    #[tokio::test]
    async fn test_auth_middleware() {
        let middleware = AuthMiddleware::new("test_secret".to_string());
        
        // 创建一个测试请求
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/users")
            .body(Body::empty())
            .unwrap();

        let (parts, _) = request.into_parts();
        
        // 测试 focus 方法
        assert!(middleware.focus(&parts));
        
        // 测试优先级
        assert_eq!(middleware.priority(), 90);
    }

    #[tokio::test]
    async fn test_cors_middleware() {
        let middleware = CorsMiddleware::new();
        
        let request = Request::builder()
            .method(Method::OPTIONS)
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let result = middleware.call(request).await;
        
        // OPTIONS 请求应该返回 CORS 响应
        assert!(result.is_err()); // 在我们的实现中，CORS preflight 返回 Err(Response)
    }

    #[test]
    fn test_middleware_chain_creation() {
        let chain = create_middleware_chain();
        // 测试链创建成功
        // 在实际测试中，可以验证中间件的执行顺序
    }
}