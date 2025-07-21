# JWT 认证中间件完整文档

## 概述

JWT 认证中间件为你的 Axum 应用提供了企业级的基于 JWT (JSON Web Token) 的完整认证解决方案。它不仅包含基础的令牌验证功能，还提供了令牌刷新、黑名单管理、速率限制等高级功能。

## 🚀 核心特性

### 基础功能
- ✅ **JWT 令牌验证**: 完整的 JWT 令牌验证和解析
- ✅ **多种提取方式**: 支持从 Authorization 头、Cookie、查询参数提取令牌
- ✅ **角色权限控制**: 灵活的基于角色的访问控制 (RBAC)
- ✅ **排除规则**: 灵活的请求排除机制
- ✅ **超时保护**: 防止令牌验证操作超时
- ✅ **详细错误处理**: 分类的错误类型和响应
- ✅ **线程安全**: 所有操作都是线程安全的

### 高级功能
- ✅ **令牌刷新机制**: 自动令牌刷新和轮换
- ✅ **黑名单管理**: 令牌撤销和黑名单功能
- ✅ **智能速率限制**: 基于用户角色的动态速率限制
- ✅ **性能监控**: 全面的性能指标收集和实时报告
- ✅ **后台任务**: 自动清理过期数据
- ✅ **综合报告**: 多维度的系统状态报告

## 🚀 快速开始

### 1. 标准设置（推荐）

```rust
use crate::web::middleware::jwt::{create_standard_jwt_middleware, Claims};

// 创建完整的JWT中间件套件（包含所有功能）
let jwt_suite = create_standard_jwt_middleware("your-jwt-secret-key");

// 启动后台清理任务
let _background_tasks = jwt_suite.start_background_tasks();

// 在 Axum 应用中使用
let app = Router::new()
    .route("/api/users", get(get_users))
    .layer(jwt_suite.auth_middleware().clone())
    .layer(Extension(jwt_suite));
```

### 2. 自定义配置

```rust
use crate::web::middleware::jwt::{JwtMiddlewareBuilder, RateLimitConfig};

// 使用构建器模式创建自定义配置
let jwt_suite = JwtMiddlewareBuilder::new("your-jwt-secret")
    .enable_refresh(3600, 7 * 24 * 3600) // 1小时访问令牌，7天刷新令牌
    .enable_blacklist()
    .enable_rate_limit(RateLimitConfig::new(60, 100)) // 每分钟100请求
    .require_any_role(vec!["user", "admin"])
    .with_excludes(vec![
        |parts| parts.uri.path() == "/health",
        |parts| parts.uri.path().starts_with("/public/"),
    ])
    .build();
```

### 3. 基础设置（仅认证）

```rust
use crate::web::middleware::jwt::{JwtMiddleware, JwtConfig};

// 仅使用基础认证功能
let config = JwtConfig::new("your-jwt-secret")
    .with_cookie_extraction(true, "session")
    .with_query_extraction(true, "token");

let jwt_middleware = JwtMiddleware::new(config);
```

### 4. 生成令牌对（推荐）

```rust
use crate::web::middleware::jwt::{Claims};

// 创建用户声明
let mut claims = Claims::new("user123");
claims.add_role("user");
claims.add_role("editor");
claims.set_expiration(3600);

// 生成令牌对（访问令牌 + 刷新令牌）
let token_pair = jwt_suite.generate_token_pair(&claims)?;

println!("Access Token: {}", token_pair.access_token);
println!("Refresh Token: {}", token_pair.refresh_token);
```

### 5. 令牌刷新

```rust
// 使用刷新令牌获取新的访问令牌
if let Some(refresh_manager) = jwt_suite.refresh_manager() {
    let new_tokens = refresh_manager.refresh_access_token(&refresh_token)?;
    println!("New Access Token: {}", new_tokens.access_token);
}
```

### 3. 在处理器中获取用户信息

```rust
use axum::extract::Extension;

async fn protected_handler(Extension(claims): Extension<Claims>) -> Json<Value> {
    Json(json!({
        "user_id": claims.sub,
        "roles": claims.roles.unwrap_or_default()
    }))
}
```

## 详细配置

### JWT 配置选项

```rust
let config = JwtConfig::new("your-jwt-secret")
    // 设置签名算法
    .with_algorithm(Algorithm::HS256)
    
    // 设置令牌签发者
    .with_issuer("my-auth-service")
    
    // 启用 Cookie 提取
    .with_cookie_extraction(true, "session_token")
    
    // 启用查询参数提取
    .with_query_extraction(true, "access_token")
    
    // 设置令牌前缀
    .with_token_prefix("Bearer ")
    
    // 设置时间偏差容忍度
    .with_leeway(60); // 60秒
```

### 中间件配置

```rust
let middleware = JwtMiddleware::new(config)
    // 添加排除规则
    .with_excludes(vec![
        |parts| parts.uri.path() == "/health",
        |parts| parts.uri.path().starts_with("/public/"),
    ])
    
    // 要求特定角色
    .require_role("admin")
    
    // 要求任一角色
    .require_any_role(vec!["user", "editor", "admin"])
    
    // 要求所有角色
    .require_all_roles(vec!["admin", "superuser"]);
```

## 角色权限控制

### 单一角色要求

```rust
// 创建要求 admin 角色的中间件
let admin_middleware = JwtMiddleware::new(config)
    .require_role("admin");

let app = Router::new()
    .route("/admin/dashboard", get(admin_dashboard))
    .route_layer(admin_middleware);
```

### 多角色要求

```rust
// 用户需要具有任一指定角色
let user_middleware = JwtMiddleware::new(config)
    .require_any_role(vec!["user", "editor", "admin"]);

// 用户需要具有所有指定角色
let super_admin_middleware = JwtMiddleware::new(config)
    .require_all_roles(vec!["admin", "superuser"]);
```

### 分层权限控制

```rust
let app = Router::new()
    // 公开端点
    .route("/health", get(health_check))
    
    // 需要基本认证
    .route("/api/profile", get(get_profile))
    .route_layer(base_jwt_middleware)
    
    // 需要编辑权限
    .route("/api/posts", post(create_post))
    .route_layer(editor_middleware)
    
    // 需要管理员权限
    .route("/api/admin", get(admin_panel))
    .route_layer(admin_middleware);
```

## 令牌提取方式

### 1. Authorization 头（默认）

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 2. Cookie

```rust
let config = JwtConfig::new("secret")
    .with_cookie_extraction(true, "jwt_token");
```

```http
Cookie: jwt_token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 3. 查询参数

```rust
let config = JwtConfig::new("secret")
    .with_query_extraction(true, "token");
```

```http
GET /api/users?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 性能监控

### 基本监控

```rust
// 创建监控器
let monitor = JwtMonitor::new(jwt_middleware.clone());

// 获取性能报告
let report = monitor.get_report();
println!("Success rate: {:.2}%", report.success_rate);

// 打印详细报告
monitor.print_detailed_report();
```

### 定期报告

```rust
// 启动定期性能报告（每60秒）
let _report_task = monitor.start_periodic_reporting(60);
```

### 性能指标

监控器收集以下指标：

- **请求统计**: 总请求数、成功数、失败数、成功率
- **性能指标**: 平均处理时间、令牌验证时间、令牌提取时间
- **错误分类**: 按错误类型分类的统计
- **提取统计**: 各种提取方式的使用统计

## 错误处理

### 错误类型

```rust
pub enum JwtError {
    TokenMissing,                    // 令牌缺失
    TokenInvalid(String),           // 令牌无效
    TokenExpired,                   // 令牌过期
    InsufficientPermission(String), // 权限不足
    ConfigError(String),            // 配置错误
}
```

### 错误响应

所有错误都会自动转换为适当的 HTTP 响应：

- `TokenMissing` → 401 Unauthorized
- `TokenInvalid` → 401 Unauthorized  
- `TokenExpired` → 401 Unauthorized
- `InsufficientPermission` → 403 Forbidden
- `ConfigError` → 500 Internal Server Error

## 最佳实践

### 1. 安全配置

```rust
// 使用强密钥
let config = JwtConfig::new("your-super-secret-jwt-key-at-least-32-chars")
    // 使用更安全的算法
    .with_algorithm(Algorithm::HS256)
    // 设置合理的过期时间
    .with_leeway(60); // 1分钟容忍度

// 在生产环境中从环境变量读取密钥
let secret = std::env::var("JWT_SECRET")
    .expect("JWT_SECRET environment variable must be set");
```

### 2. 排除规则

```rust
let middleware = JwtMiddleware::new(config)
    .with_excludes(vec![
        // 健康检查
        |parts| parts.uri.path() == "/health",
        // 静态文件
        |parts| parts.uri.path().starts_with("/static/"),
        // 认证端点
        |parts| parts.uri.path().starts_with("/auth/"),
        // API 文档
        |parts| parts.uri.path().starts_with("/docs/"),
    ]);
```

### 3. 令牌生命周期管理

```rust
// 短期访问令牌
let mut access_claims = Claims::new(user_id);
access_claims.set_expiration(900); // 15分钟

// 长期刷新令牌
let mut refresh_claims = Claims::new(user_id);
refresh_claims.set_expiration(7 * 24 * 3600); // 7天
refresh_claims.add_role("refresh");
```

### 4. 性能优化

```rust
// 使用合理的超时设置
const TOKEN_VALIDATION_TIMEOUT_MS: u64 = 1000; // 1秒

// 定期监控性能
let monitor = JwtMonitor::new(middleware.clone());
let _task = monitor.start_periodic_reporting(300); // 每5分钟报告
```

## 完整示例

查看 `examples/jwt_middleware_usage.rs` 文件获取完整的使用示例，包括：

- 用户登录和令牌生成
- 不同权限级别的路由保护
- 性能监控设置
- 测试用例

## 故障排除

### 常见问题

1. **令牌验证失败**
   - 检查密钥是否正确
   - 确认算法设置匹配
   - 验证令牌格式

2. **权限不足错误**
   - 检查用户角色设置
   - 确认中间件角色要求
   - 验证角色匹配逻辑

3. **性能问题**
   - 检查令牌验证超时设置
   - 监控性能指标
   - 考虑令牌缓存策略

### 调试技巧

```rust
// 启用详细日志
let monitor = JwtMonitor::new(middleware);
monitor.print_detailed_report();

// 检查特定错误类型
let report = monitor.get_report();
if report.error_breakdown.token_invalid_errors > 0 {
    println!("检测到令牌无效错误，请检查密钥配置");
}
```

## 依赖项

确保在 `Cargo.toml` 中包含以下依赖：

```toml
[dependencies]
axum = "0.7"
jsonwebtoken = "9.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

## 许可证

本中间件遵循项目的整体许可证。