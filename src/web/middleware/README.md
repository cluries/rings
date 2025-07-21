# 中间件系统使用指南

这个中间件系统提供了一个灵活、可扩展的方式来处理 HTTP 请求。

## 核心概念

### Middleware Trait
所有中间件都需要实现 `Middleware` trait：

```rust
pub trait Middleware: Send + Sync {
    fn focus(&self, parts: &Parts) -> bool;
    fn priority(&self) -> i32;
    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>>;
    fn name(&self) -> &'static str { "UnnamedMiddleware" }
    fn path_pattern(&self) -> Option<&str> { None }
    fn methods(&self) -> Option<&[Method]> { None }
}
```

### 关键方法说明

- `focus()`: 判断中间件是否应该处理特定请求
- `priority()`: 中间件优先级（数值越大优先级越高）
- `call()`: 异步处理请求的核心方法

## 使用示例

### 1. 创建自定义中间件

```rust
use crate::web::middleware::*;

pub struct AuthMiddleware {
    secret_key: String,
    protected_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            protected_paths: vec!["/api/".to_string()],
        }
    }
}

impl Middleware for AuthMiddleware {
    fn focus(&self, parts: &Parts) -> bool {
        self.protected_paths.iter().any(|p| parts.uri.path().starts_with(p))
    }

    fn priority(&self) -> i32 {
        90 // 高优先级，在日志之后执行
    }

    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            if let Some(auth_header) = request.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
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
}
```

### 2. 构建中间件链

```rust
use crate::web::middleware::*;
use crate::web::middleware::examples::*;

// 使用构建器模式
let manager = MiddlewareBuilder::new()
    .add(CorsMiddleware::new())                    // 优先级: 110
    .add(LoggingMiddleware::new(true))             // 优先级: 100
    .add(AuthMiddleware::new("secret".to_string())) // 优先级: 90
    .add(RateLimitMiddleware::new(100, 60))        // 优先级: 80
    .build();

let chain = MiddlewareChain::new(manager);
```

### 3. 执行中间件链

```rust
use axum::{extract::Request, body::Body};

// 创建测试请求
let request = Request::builder()
    .method(Method::GET)
    .uri("/api/users")
    .header("Authorization", "Bearer token123")
    .body(Body::empty())
    .unwrap();

// 执行中间件链
match chain.execute(request).await {
    Ok(processed_request) => {
        // 请求通过所有中间件，继续处理
        println!("Request processed successfully");
    }
    Err(response) => {
        // 某个中间件拒绝了请求
        println!("Request rejected: {:?}", response.status());
    }
}
```

## 内置中间件示例

### 1. 日志中间件
```rust
let logging = LoggingMiddleware::new(true);
```

### 2. 认证中间件
```rust
let auth = AuthMiddleware::new("my_secret_key".to_string())
    .with_protected_paths(vec!["/api/".to_string(), "/admin/".to_string()]);
```

### 3. CORS 中间件
```rust
let cors = CorsMiddleware::new()
    .with_origins(vec!["https://example.com".to_string()])
    .with_methods(vec![Method::GET, Method::POST]);
```

### 4. 限流中间件
```rust
let rate_limit = RateLimitMiddleware::new(100, 60); // 每分钟100个请求
```

## 高级功能

### 路径匹配
使用内置的路径匹配函数：

```rust
// 匹配所有路径
path_matches("*", "/any/path") // true

// 匹配前缀
path_matches("/api/*", "/api/users") // true

// 精确匹配
path_matches("/api/users", "/api/users") // true
```

### 方法过滤
```rust
impl Middleware for MyMiddleware {
    fn methods(&self) -> Option<&[Method]> {
        Some(&[Method::GET, Method::POST])
    }
    
    fn focus(&self, parts: &Parts) -> bool {
        if let Some(methods) = self.methods() {
            method_matches(methods, &parts.method)
        } else {
            true
        }
    }
}
```

### 中间件上下文
使用 `MiddlewareContext` 在中间件之间传递数据：

```rust
let mut context = MiddlewareContext::new();
context.set_metadata("user_id".to_string(), "123".to_string());

// 获取请求 ID
println!("Request ID: {}", context.request_id);

// 获取处理时间
let elapsed = context.start_time.elapsed();
println!("Processing time: {:?}", elapsed);
```

## 最佳实践

### 1. 优先级设计
- **CORS 中间件**: 110+ (最高优先级)
- **日志中间件**: 100-109
- **认证中间件**: 90-99
- **业务逻辑中间件**: 50-89
- **限流中间件**: 70-89
- **错误处理中间件**: 10-49

### 2. 错误处理
```rust
fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
    Box::pin(async move {
        match self.validate_request(&request) {
            Ok(_) => Ok(request),
            Err(err) => Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Validation error: {}", err).into())
                .unwrap())
        }
    })
}
```

### 3. 性能考虑
- 在 `focus()` 方法中进行快速过滤
- 避免在不必要的请求上执行重型操作
- 使用异步操作避免阻塞

### 4. 测试策略
```rust
#[tokio::test]
async fn test_middleware_chain() {
    let chain = create_middleware_chain();
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/test")
        .body(Body::empty())
        .unwrap();

    let result = chain.execute(request).await;
    assert!(result.is_ok());
}
```

## 与 Axum 集成

虽然当前实现还没有完全集成到 Axum 的 layer 系统中，但可以通过以下方式使用：

```rust
use axum::{Router, middleware::from_fn};

async fn middleware_handler(
    request: Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::response::Response> {
    let chain = create_middleware_chain();
    
    match chain.execute(request).await {
        Ok(processed_request) => {
            Ok(next.run(processed_request).await)
        }
        Err(response) => Err(response)
    }
}

let app = Router::new()
    .route("/api/users", get(get_users))
    .layer(from_fn(middleware_handler));
```

这个中间件系统提供了强大的请求处理能力，支持异步操作、优先级排序和灵活的条件执行。