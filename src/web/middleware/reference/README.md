# 中间件系统文档

## 概述

本项目提供了一个灵活、高性能的中间件系统，支持多种认证、授权、监控和安全功能。中间件系统采用模块化设计，易于扩展和维护。

## 🏗️ 架构设计

### 核心组件

```
src/web/middleware/
├── mod.rs              # 中间件系统核心
├── jwt.rs              # JWT 认证中间件
├── jwt/
│   └── rate_limit.rs   # JWT 速率限制
├── profile.rs          # 性能分析中间件
├── signature.rs        # 签名验证中间件
├── signator.rs         # 签名生成中间件
└── examples.rs         # 示例中间件
```

### 中间件 Trait

```rust
pub trait Middleware: Send + Sync {
    /// 判断中间件是否应该处理这个请求
    fn focus(&self, parts: &Parts) -> bool;
    
    /// 中间件优先级，数值越大优先级越高
    fn priority(&self) -> i32;
    
    /// 处理请求的核心方法
    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>>;
    
    /// 可选：中间件名称
    fn name(&self) -> &'static str { "UnnamedMiddleware" }
}
```

## 🔐 JWT 认证中间件

### 特性

- ✅ **完整的 JWT 支持**: 令牌生成、验证、刷新
- ✅ **多源令牌提取**: Authorization 头、Cookie、查询参数
- ✅ **角色权限控制**: 灵活的 RBAC 系统
- ✅ **性能监控**: 详细的性能指标和报告
- ✅ **速率限制**: 基于用户和角色的速率控制
- ✅ **安全特性**: 超时保护、错误处理、审计日志

### 基本使用

```rust
use crate::web::middleware::jwt::{JwtConfig, JwtMiddleware};

// 1. 创建配置
let config = JwtConfig::new("your-jwt-secret")
    .with_algorithm(Algorithm::HS256)
    .with_cookie_extraction(true, "session")
    .with_query_extraction(true, "token")
    .with_issuer("my-service");

// 2. 创建中间件
let jwt_middleware = JwtMiddleware::new(config)
    .with_excludes(vec![
        |parts| parts.uri.path() == "/health",
        |parts| parts.uri.path().starts_with("/public/"),
    ])
    .require_any_role(vec!["user", "admin"]);

// 3. 应用到路由
let app = Router::new()
    .route("/api/protected", get(protected_handler))
    .layer(jwt_middleware);
```

### 高级配置

#### 角色权限控制

```rust
// 单一角色要求
let admin_middleware = JwtMiddleware::new(config.clone())
    .require_role("admin");

// 多角色要求（任一）
let user_middleware = JwtMiddleware::new(config.clone())
    .require_any_role(vec!["user", "editor", "admin"]);

// 多角色要求（全部）
let super_admin_middleware = JwtMiddleware::new(config.clone())
    .require_all_roles(vec!["admin", "superuser"]);
```

#### 速率限制

```rust
use crate::web::middleware::jwt::rate_limit::{RateLimitConfig, JwtRateLimiter};

// 创建速率限制配置
let rate_config = RateLimitConfig::new()
    .with_default_limit(100, 60)           // 默认每分钟100次
    .with_role_limit("premium", 1000, 60)  // 高级用户每分钟1000次
    .with_endpoint_limit("/api/upload", 10, 60); // 上传接口每分钟10次

// 创建速率限制器
let rate_limiter = JwtRateLimiter::new(rate_config);

// 集成到 JWT 中间件
let jwt_middleware = JwtMiddleware::new(jwt_config)
    .with_rate_limiter(rate_limiter);
```

#### 性能监控

```rust
use crate::web::middleware::jwt::JwtMonitor;

// 创建监控器
let monitor = JwtMonitor::new(jwt_middleware.clone());

// 启动定期报告
let _report_task = monitor.start_periodic_reporting(60); // 每分钟

// 获取实时报告
let report = monitor.get_report();
println!("Success rate: {:.2}%", report.success_rate);
println!("Avg processing time: {:.2}ms", report.avg_processing_time_ms);

// 打印详细报告
monitor.print_detailed_report();
```

### JWT 令牌管理

#### 生成令牌

```rust
use crate::web::middleware::jwt::{JwtGenerator, Claims};

let generator = JwtGenerator::new(config);

// 创建用户声明
let mut claims = Claims::new("user123");
claims.add_role("user");
claims.add_role("editor");
claims.set_expiration(3600); // 1小时后过期
claims.set_issuer("my-service");

// 添加自定义数据
claims.data = Some(json!({
    "department": "engineering",
    "permissions": ["read", "write"]
}));

// 生成令牌
let token = generator.generate_token(&claims)?;
```

#### 验证令牌

```rust
// 验证令牌
match generator.verify_token(&token) {
    Ok(claims) => {
        println!("Valid token for user: {}", claims.sub);
        println!("Roles: {:?}", claims.roles);
    },
    Err(e) => {
        println!("Invalid token: {}", e);
    }
}
```

#### 在处理器中使用

```rust
use axum::extract::Extension;

async fn protected_handler(
    Extension(claims): Extension<Claims>
) -> Result<Json<Value>, StatusCode> {
    // 检查用户权限
    if !claims.has_role("admin") {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "user_id": claims.sub,
        "roles": claims.roles.unwrap_or_default(),
        "message": "Access granted"
    })))
}
```

## 📊 性能分析中间件

性能分析中间件提供详细的请求性能监控和分析功能。

```rust
use crate::web::middleware::profile::ProfileMiddleware;

let profile_middleware = ProfileMiddleware::new()
    .with_detailed_timing(true)
    .with_memory_tracking(true)
    .with_custom_metrics(true);

let app = Router::new()
    .route("/api/test", get(test_handler))
    .layer(profile_middleware);
```

## 🔏 签名验证中间件

提供请求签名验证功能，确保请求的完整性和来源可信。

```rust
use crate::web::middleware::signature::SignatureMiddleware;

let signature_middleware = SignatureMiddleware::new("your-secret-key")
    .with_algorithm("HMAC-SHA256")
    .with_timestamp_validation(300); // 5分钟时间窗口

let app = Router::new()
    .route("/api/webhook", post(webhook_handler))
    .layer(signature_middleware);
```

## 🔧 中间件管理器

中间件管理器提供了统一的中间件管理和执行功能。

```rust
use crate::web::middleware::{MiddlewareManager, MiddlewareBuilder, MiddlewareChain};

// 使用构建器模式
let manager = MiddlewareBuilder::new()
    .add(LoggingMiddleware::new(true))
    .add(jwt_middleware)
    .add(profile_middleware)
    .build();

// 创建执行链
let chain = MiddlewareChain::new(manager);

// 在请求处理中使用
async fn handle_request(request: Request) -> Result<Request, Response> {
    chain.execute(request).await
}
```

## 🚀 最佳实践

### 1. 中间件顺序

中间件的执行顺序很重要，建议按以下优先级排序：

```rust
let manager = MiddlewareBuilder::new()
    .add(LoggingMiddleware::new(true))        // 优先级: 100 (最高)
    .add(SignatureMiddleware::new("secret"))  // 优先级: 90
    .add(JwtMiddleware::new(jwt_config))      // 优先级: 80
    .add(RateLimitMiddleware::new(config))    // 优先级: 70
    .add(ProfileMiddleware::new())            // 优先级: 60 (最低)
    .build();
```

### 2. 错误处理

```rust
impl Middleware for CustomMiddleware {
    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            match self.process_request(&request).await {
                Ok(processed_request) => Ok(processed_request),
                Err(e) => {
                    // 记录错误
                    tracing::error!("Middleware error: {}", e);
                    
                    // 返回适当的错误响应
                    Err(self.create_error_response(e))
                }
            }
        })
    }
}
```

### 3. 性能优化

```rust
// 使用条件检查避免不必要的处理
impl Middleware for OptimizedMiddleware {
    fn focus(&self, parts: &Parts) -> bool {
        // 只处理特定路径
        parts.uri.path().starts_with("/api/") &&
        // 只处理特定方法
        matches!(parts.method, Method::POST | Method::PUT | Method::DELETE)
    }
    
    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            // 使用超时保护
            match timeout(Duration::from_secs(5), self.process(request)).await {
                Ok(result) => result,
                Err(_) => Err(self.timeout_response()),
            }
        })
    }
}
```

### 4. 配置管理

```rust
// 使用环境变量进行配置
let jwt_secret = std::env::var("JWT_SECRET")
    .expect("JWT_SECRET must be set");

let jwt_config = JwtConfig::new(&jwt_secret)
    .with_algorithm(Algorithm::HS256)
    .with_leeway(60);

// 使用配置文件
#[derive(Deserialize)]
struct MiddlewareConfig {
    jwt: JwtConfigFile,
    rate_limit: RateLimitConfigFile,
}

let config: MiddlewareConfig = toml::from_str(&config_content)?;
```

### 5. 测试策略

```rust
#[tokio::test]
async fn test_middleware_integration() {
    let middleware = TestMiddleware::new();
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/test")
        .body(Body::empty())
        .unwrap();
    
    let result = middleware.call(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_middleware_performance() {
    let middleware = PerformanceTestMiddleware::new();
    let start = Instant::now();
    
    // 发送1000个请求
    for _ in 0..1000 {
        let request = create_test_request();
        let _ = middleware.call(request).await;
    }
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(1), "Performance too slow");
}
```

## 📈 监控和指标

### 内置指标

所有中间件都提供以下基础指标：

- **请求计数**: 总请求数、成功数、失败数
- **响应时间**: 平均、最小、最大、P95、P99
- **错误率**: 按错误类型分类的统计
- **吞吐量**: 每秒请求数 (RPS)

### 自定义指标

```rust
use crate::web::middleware::metrics::{MetricsCollector, CustomMetric};

let metrics = MetricsCollector::new();

// 记录自定义指标
metrics.record_custom("user_login_attempts", 1.0);
metrics.record_histogram("database_query_time", duration.as_millis() as f64);
metrics.record_gauge("active_connections", connection_count as f64);

// 获取指标报告
let report = metrics.get_report();
```

### Prometheus 集成

```rust
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram};

// 注册 Prometheus 指标
let request_counter = register_counter!(
    "http_requests_total",
    "Total number of HTTP requests"
)?;

let request_duration = register_histogram!(
    "http_request_duration_seconds",
    "HTTP request duration in seconds"
)?;

// 在中间件中使用
impl Middleware for PrometheusMiddleware {
    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            let start = Instant::now();
            request_counter.inc();
            
            let result = self.process(request).await;
            
            let duration = start.elapsed().as_secs_f64();
            request_duration.observe(duration);
            
            result
        })
    }
}
```

## 🔍 调试和故障排除

### 启用调试日志

```rust
// 在环境变量中设置
RUST_LOG=debug

// 或在代码中配置
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### 常见问题

1. **中间件不执行**
   - 检查 `focus()` 方法是否返回 `true`
   - 确认中间件已正确添加到管理器中
   - 验证路径匹配逻辑

2. **性能问题**
   - 使用性能分析中间件识别瓶颈
   - 检查是否有阻塞操作
   - 考虑使用异步操作

3. **内存泄漏**
   - 检查是否正确清理资源
   - 使用 Arc 和 Weak 引用避免循环引用
   - 定期清理过期数据

### 调试工具

```rust
// 中间件执行跟踪
#[derive(Debug)]
struct TracingMiddleware {
    name: String,
}

impl Middleware for TracingMiddleware {
    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        let name = self.name.clone();
        Box::pin(async move {
            tracing::info!("Middleware {} starting", name);
            let start = Instant::now();
            
            let result = Ok(request); // 实际处理逻辑
            
            let duration = start.elapsed();
            tracing::info!("Middleware {} completed in {:?}", name, duration);
            
            result
        })
    }
}
```

## 📚 扩展开发

### 创建自定义中间件

```rust
use crate::web::middleware::Middleware;

pub struct CustomMiddleware {
    config: CustomConfig,
    metrics: Arc<CustomMetrics>,
}

impl CustomMiddleware {
    pub fn new(config: CustomConfig) -> Self {
        Self {
            config,
            metrics: CustomMetrics::new(),
        }
    }
}

impl Middleware for CustomMiddleware {
    fn focus(&self, parts: &Parts) -> bool {
        // 实现焦点逻辑
        true
    }
    
    fn priority(&self) -> i32 {
        // 设置优先级
        50
    }
    
    fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
        Box::pin(async move {
            // 实现处理逻辑
            Ok(request)
        })
    }
    
    fn name(&self) -> &'static str {
        "CustomMiddleware"
    }
}
```

### 中间件模板

```rust
// 使用宏简化中间件创建
macro_rules! create_middleware {
    ($name:ident, $priority:expr, $focus:expr, $process:expr) => {
        pub struct $name;
        
        impl Middleware for $name {
            fn focus(&self, parts: &Parts) -> bool {
                $focus(parts)
            }
            
            fn priority(&self) -> i32 {
                $priority
            }
            
            fn call(&self, request: Request) -> Pin<Box<dyn Future<Output = Result<Request, Response>> + Send + '_>> {
                Box::pin(async move {
                    $process(request).await
                })
            }
            
            fn name(&self) -> &'static str {
                stringify!($name)
            }
        }
    };
}

// 使用宏创建中间件
create_middleware!(
    SimpleLoggingMiddleware,
    100,
    |_parts| true,
    |request| async move {
        println!("Processing request: {}", request.uri());
        Ok(request)
    }
);
```

这个中间件系统提供了强大而灵活的功能，支持各种认证、授权、监控和安全需求。通过模块化设计，你可以轻松地添加新功能或自定义现有功能。