//! # 完整的 JWT 中间件使用示例
//! 
//! 这个示例展示了如何在实际项目中使用 JWT 中间件的所有功能，包括：
//! - 基础认证和授权
//! - 角色权限控制
//! - 速率限制
//! - 性能监控
//! - 错误处理
//! - 多种令牌提取方式

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower::ServiceExt;

// 假设这些是项目中的模块
// use rings::web::middleware::jwt::{
//     Claims, JwtConfig, JwtGenerator, JwtMiddleware, JwtMonitor,
//     rate_limit::{RateLimitConfig, JwtRateLimiter},
// };

/// 用户登录请求
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// 用户登录响应
#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    user_info: Option<UserInfo>,
    message: String,
}

/// 用户信息
#[derive(Serialize, Clone)]
struct UserInfo {
    id: String,
    username: String,
    email: String,
    roles: Vec<String>,
    department: String,
    created_at: String,
}

/// 查询参数
#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

/// API 响应包装器
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
    pagination: Option<PaginationInfo>,
}

#[derive(Serialize)]
struct PaginationInfo {
    page: u32,
    limit: u32,
    total: u32,
    total_pages: u32,
}

/// 模拟用户数据库
#[derive(Clone)]
struct UserDatabase {
    users: HashMap<String, (String, UserInfo)>, // username -> (password, user_info)
}

impl UserDatabase {
    fn new() -> Self {
        let mut users = HashMap::new();
        
        // 添加测试用户
        users.insert("admin".to_string(), (
            "admin123".to_string(),
            UserInfo {
                id: "1".to_string(),
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                roles: vec!["admin".to_string(), "user".to_string()],
                department: "IT".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            }
        ));
        
        users.insert("editor".to_string(), (
            "editor123".to_string(),
            UserInfo {
                id: "2".to_string(),
                username: "editor".to_string(),
                email: "editor@example.com".to_string(),
                roles: vec!["editor".to_string(), "user".to_string()],
                department: "Content".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            }
        ));
        
        users.insert("user".to_string(), (
            "user123".to_string(),
            UserInfo {
                id: "3".to_string(),
                username: "user".to_string(),
                email: "user@example.com".to_string(),
                roles: vec!["user".to_string()],
                department: "General".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            }
        ));
        
        users.insert("premium".to_string(), (
            "premium123".to_string(),
            UserInfo {
                id: "4".to_string(),
                username: "premium".to_string(),
                email: "premium@example.com".to_string(),
                roles: vec!["premium".to_string(), "user".to_string()],
                department: "Premium".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            }
        ));
        
        Self { users }
    }
    
    fn authenticate(&self, username: &str, password: &str) -> Option<UserInfo> {
        if let Some((stored_password, user_info)) = self.users.get(username) {
            if stored_password == password {
                return Some(user_info.clone());
            }
        }
        None
    }
    
    fn get_user_by_id(&self, user_id: &str) -> Option<UserInfo> {
        self.users.values()
            .find(|(_, user_info)| user_info.id == user_id)
            .map(|(_, user_info)| user_info.clone())
    }
}

/// 应用状态
#[derive(Clone)]
struct AppState {
    user_db: UserDatabase,
    // jwt_generator: JwtGenerator,
}

/// 处理器函数

/// 健康检查处理器
async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    }))
}

/// 用户登录处理器
async fn login_handler(
    Extension(state): Extension<AppState>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // 验证用户凭据
    if let Some(user_info) = state.user_db.authenticate(&login_req.username, &login_req.password) {
        // 在实际实现中，这里会生成真实的 JWT 令牌
        /*
        let mut claims = Claims::new(&user_info.id);
        for role in &user_info.roles {
            claims.add_role(role);
        }
        claims.set_expiration(3600); // 1小时
        claims.set_issuer("jwt-example-service");
        
        // 添加自定义数据
        claims.data = Some(json!({
            "username": user_info.username,
            "department": user_info.department
        }));
        
        let token = state.jwt_generator.generate_token(&claims)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // 生成刷新令牌（有效期更长）
        let mut refresh_claims = Claims::new(&user_info.id);
        refresh_claims.add_role("refresh");
        refresh_claims.set_expiration(7 * 24 * 3600); // 7天
        
        let refresh_token = state.jwt_generator.generate_token(&refresh_claims)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        */
        
        // 模拟令牌生成
        let token = format!("mock-jwt-token-for-{}", user_info.id);
        let refresh_token = format!("mock-refresh-token-for-{}", user_info.id);
        
        Ok(Json(LoginResponse {
            success: true,
            token: Some(token),
            refresh_token: Some(refresh_token),
            expires_in: Some(3600),
            user_info: Some(user_info),
            message: "Login successful".to_string(),
        }))
    } else {
        Ok(Json(LoginResponse {
            success: false,
            token: None,
            refresh_token: None,
            expires_in: None,
            user_info: None,
            message: "Invalid username or password".to_string(),
        }))
    }
}

/// 获取当前用户信息
async fn get_current_user(
    Extension(state): Extension<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<UserInfo>>, StatusCode> {
    if let Some(user_info) = state.user_db.get_user_by_id(&claims.sub) {
        Ok(Json(ApiResponse {
            success: true,
            data: Some(user_info),
            message: "User information retrieved successfully".to_string(),
            pagination: None,
        }))
    } else {
        Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: "User not found".to_string(),
            pagination: None,
        }))
    }
}

/// 更新用户资料
async fn update_user_profile(
    Extension(_claims): Extension<Claims>,
    Json(update_data): Json<Value>,
) -> Json<ApiResponse<Value>> {
    // 在实际实现中，这里会更新数据库
    Json(ApiResponse {
        success: true,
        data: Some(json!({
            "user_id": claims.sub,
            "updated_fields": update_data,
            "updated_at": chrono::Utc::now().to_rfc3339()
        })),
        message: "Profile updated successfully".to_string(),
        pagination: None,
    })
}

/// 获取用户列表（需要管理员权限）
async fn get_users_list(
    Extension(claims): Extension<Claims>,
    Query(query): Query<PaginationQuery>,
) -> Json<ApiResponse<Vec<UserInfo>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    
    // 模拟用户列表
    let users = vec![
        UserInfo {
            id: "1".to_string(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            roles: vec!["admin".to_string()],
            department: "IT".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        },
        UserInfo {
            id: "2".to_string(),
            username: "editor".to_string(),
            email: "editor@example.com".to_string(),
            roles: vec!["editor".to_string()],
            department: "Content".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        },
    ];
    
    Json(ApiResponse {
        success: true,
        data: Some(users),
        message: "Users retrieved successfully".to_string(),
        pagination: Some(PaginationInfo {
            page,
            limit,
            total: 2,
            total_pages: 1,
        }),
    })
}

/// 创建新用户（需要管理员权限）
async fn create_user(
    Extension(claims): Extension<Claims>,
    Json(user_data): Json<Value>,
) -> Json<ApiResponse<Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(json!({
            "created_by": claims.sub,
            "user_data": user_data,
            "created_at": chrono::Utc::now().to_rfc3339()
        })),
        message: "User created successfully".to_string(),
        pagination: None,
    })
}

/// 删除用户（需要管理员权限）
async fn delete_user(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Json<ApiResponse<Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(json!({
            "deleted_user_id": user_id,
            "deleted_by": claims.sub,
            "deleted_at": chrono::Utc::now().to_rfc3339()
        })),
        message: "User deleted successfully".to_string(),
        pagination: None,
    })
}

/// 获取文章列表（需要编辑权限）
async fn get_posts(
    Extension(claims): Extension<Claims>,
    Query(query): Query<PaginationQuery>,
) -> Json<ApiResponse<Vec<Value>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    
    let posts = vec![
        json!({
            "id": "1",
            "title": "First Post",
            "content": "This is the first post",
            "author": claims.sub,
            "created_at": "2024-01-01T00:00:00Z"
        }),
        json!({
            "id": "2",
            "title": "Second Post",
            "content": "This is the second post",
            "author": claims.sub,
            "created_at": "2024-01-02T00:00:00Z"
        }),
    ];
    
    Json(ApiResponse {
        success: true,
        data: Some(posts),
        message: "Posts retrieved successfully".to_string(),
        pagination: Some(PaginationInfo {
            page,
            limit,
            total: 2,
            total_pages: 1,
        }),
    })
}

/// 创建文章（需要编辑权限）
async fn create_post(
    Extension(claims): Extension<Claims>,
    Json(post_data): Json<Value>,
) -> Json<ApiResponse<Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(json!({
            "author": claims.sub,
            "post_data": post_data,
            "created_at": chrono::Utc::now().to_rfc3339()
        })),
        message: "Post created successfully".to_string(),
        pagination: None,
    })
}

/// 文件上传处理器（有速率限制）
async fn upload_file(
    Extension(claims): Extension<Claims>,
    Json(file_data): Json<Value>,
) -> Json<ApiResponse<Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(json!({
            "upload_id": uuid::Uuid::new_v4().to_string(),
            "uploaded_by": claims.sub,
            "file_info": file_data,
            "uploaded_at": chrono::Utc::now().to_rfc3339()
        })),
        message: "File uploaded successfully".to_string(),
        pagination: None,
    })
}

/// 获取系统统计信息（需要管理员权限）
async fn get_system_stats(
    Extension(claims): Extension<Claims>,
) -> Json<ApiResponse<Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(json!({
            "total_users": 100,
            "active_sessions": 25,
            "total_posts": 500,
            "system_uptime": "5 days, 3 hours",
            "memory_usage": "45%",
            "cpu_usage": "12%",
            "requested_by": claims.sub,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
        message: "System statistics retrieved successfully".to_string(),
        pagination: None,
    })
}

/// 获取性能指标（公开访问，用于监控）
async fn get_metrics() -> Json<Value> {
    // 在实际实现中，这里会从监控器获取报告
    // let report = monitor.get_report();
    Json(json!({
        "total_requests": 1000,
        "success_rate": 95.5,
        "avg_processing_time_ms": 12.3,
        "error_breakdown": {
            "token_missing_errors": 20,
            "token_invalid_errors": 15,
            "token_expired_errors": 10,
            "rate_limit_errors": 5
        }
    }))
}

/// 刷新令牌处理器
async fn refresh_token(
    Extension(_state): Extension<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // 验证这是一个刷新令牌
    if !claims.has_role("refresh") {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 在实际实现中，这里会生成新的访问令牌
    /*
    if let Some(user_info) = state.user_db.get_user_by_id(&claims.sub) {
        let mut new_claims = Claims::new(&user_info.id);
        for role in &user_info.roles {
            new_claims.add_role(role);
        }
        new_claims.set_expiration(3600); // 1小时
        
        let new_token = state.jwt_generator.generate_token(&new_claims)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Json(LoginResponse {
            success: true,
            token: Some(new_token),
            refresh_token: None, // 不返回新的刷新令牌
            expires_in: Some(3600),
            user_info: Some(user_info),
            message: "Token refreshed successfully".to_string(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
    */
    
    // 模拟令牌刷新
    let new_token = format!("new-mock-jwt-token-for-{}", claims.sub);
    
    Ok(Json(LoginResponse {
        success: true,
        token: Some(new_token),
        refresh_token: None,
        expires_in: Some(3600),
        user_info: None,
        message: "Token refreshed successfully".to_string(),
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建应用状态
    let user_db = UserDatabase::new();
    
    // 在实际实现中，取消以下注释：
    /*
    // 1. 创建 JWT 配置
    let jwt_config = JwtConfig::new(&std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-super-secret-jwt-key".to_string()))
        .with_algorithm(Algorithm::HS256)
        .with_issuer("jwt-example-service")
        .with_cookie_extraction(true, "session_token")
        .with_query_extraction(true, "access_token")
        .with_leeway(60); // 1分钟容忍度

    let jwt_generator = JwtGenerator::new(jwt_config.clone());
    
    // 2. 创建速率限制配置
    let rate_config = RateLimitConfig::new()
        .with_default_limit(100, 60)           // 默认每分钟100次请求
        .with_role_limit("premium", 1000, 60)  // 高级用户每分钟1000次
        .with_role_limit("admin", 500, 60)     // 管理员每分钟500次
        .with_endpoint_limit("/api/upload", 10, 60)     // 上传接口每分钟10次
        .with_endpoint_limit("/api/auth/refresh", 5, 60); // 刷新接口每分钟5次

    let rate_limiter = JwtRateLimiter::new(rate_config);
    
    // 3. 创建不同权限级别的中间件
    let base_jwt_middleware = JwtMiddleware::new(jwt_config.clone())
        .with_rate_limiter(rate_limiter.clone())
        .with_excludes(vec![
            |parts| parts.uri.path() == "/health",
            |parts| parts.uri.path() == "/api/auth/login",
            |parts| parts.uri.path() == "/metrics",
            |parts| parts.uri.path().starts_with("/public/"),
        ]);

    let admin_middleware = JwtMiddleware::new(jwt_config.clone())
        .with_rate_limiter(rate_limiter.clone())
        .require_role("admin");

    let editor_middleware = JwtMiddleware::new(jwt_config.clone())
        .with_rate_limiter(rate_limiter.clone())
        .require_any_role(vec!["editor", "admin"]);

    let refresh_middleware = JwtMiddleware::new(jwt_config.clone())
        .require_role("refresh");
    
    // 4. 创建性能监控器
    let monitor = JwtMonitor::new(base_jwt_middleware.clone());
    
    // 启动定期性能报告（每30秒）
    let _report_task = monitor.start_periodic_reporting(30);
    
    let app_state = AppState {
        user_db,
        jwt_generator,
    };
    */
    
    // 模拟应用状态（实际实现时删除）
    let app_state = AppState {
        user_db,
    };
    
    // 5. 构建应用路由
    let app = Router::new()
        // 公开端点（无需认证）
        .route("/health", get(health_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/metrics", get(get_metrics))
        
        // 需要基本认证的端点
        .route("/api/user/profile", get(get_current_user))
        .route("/api/user/profile", put(update_user_profile))
        // .route_layer(base_jwt_middleware.clone())
        
        // 需要编辑权限的端点
        .route("/api/posts", get(get_posts))
        .route("/api/posts", post(create_post))
        // .route_layer(editor_middleware)
        
        // 需要管理员权限的端点
        .route("/api/admin/users", get(get_users_list))
        .route("/api/admin/users", post(create_user))
        .route("/api/admin/users/:id", delete(delete_user))
        .route("/api/admin/stats", get(get_system_stats))
        // .route_layer(admin_middleware)
        
        // 特殊端点（有特定速率限制）
        .route("/api/upload", post(upload_file))
        // .route_layer(base_jwt_middleware.clone())
        
        // 令牌刷新端点
        .route("/api/auth/refresh", post(refresh_token))
        // .route_layer(refresh_middleware)
        
        // 添加应用状态和监控器
        .layer(axum::extract::Extension(app_state));
        // .layer(axum::extract::Extension(monitor));

    // 6. 启动服务器
    println!("🚀 完整 JWT 示例服务器启动中...");
    println!("📍 服务地址: http://localhost:3000");
    println!("\n📋 可用端点:");
    
    println!("\n🔓 公开端点:");
    println!("  GET  /health                    - 健康检查");
    println!("  POST /api/auth/login            - 用户登录");
    println!("  GET  /metrics                   - 性能指标");
    
    println!("\n🔐 需要认证的端点:");
    println!("  GET  /api/user/profile          - 获取当前用户信息");
    println!("  PUT  /api/user/profile          - 更新用户资料");
    println!("  POST /api/upload                - 文件上传（速率限制：10次/分钟）");
    println!("  POST /api/auth/refresh          - 刷新令牌（需要刷新令牌）");
    
    println!("\n✏️  需要编辑权限的端点:");
    println!("  GET  /api/posts                 - 获取文章列表");
    println!("  POST /api/posts                 - 创建文章");
    
    println!("\n👑 需要管理员权限的端点:");
    println!("  GET  /api/admin/users           - 获取用户列表");
    println!("  POST /api/admin/users           - 创建用户");
    println!("  DELETE /api/admin/users/:id     - 删除用户");
    println!("  GET  /api/admin/stats           - 获取系统统计");
    
    println!("\n👥 测试用户账号:");
    println!("  admin/admin123     - 管理员权限");
    println!("  editor/editor123   - 编辑权限");
    println!("  user/user123       - 普通用户权限");
    println!("  premium/premium123 - 高级用户权限（更高速率限制）");
    
    println!("\n🔧 使用方法:");
    println!("1. 首先调用 POST /api/auth/login 获取 JWT 令牌");
    println!("2. 在后续请求中通过以下方式之一提供令牌:");
    println!("   - Authorization 头: 'Authorization: Bearer <token>'");
    println!("   - Cookie: 'session_token=<token>'");
    println!("   - 查询参数: '?access_token=<token>'");
    println!("3. 不同用户角色可以访问不同的端点");
    println!("4. 某些端点有速率限制，超出限制会返回 429 错误");
    
    println!("\n📊 监控功能:");
    println!("- 访问 /metrics 查看实时性能指标");
    println!("- 控制台每30秒输出性能报告");
    println!("- 包含请求统计、错误分析、令牌提取统计等");

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// 占位符结构体，在实际实现中应该从项目中导入
#[derive(Debug, Clone)]
struct Claims {
    pub sub: String,
    pub roles: Option<Vec<String>>,
    pub exp: Option<i64>,
    pub data: Option<Value>,
}

impl Claims {
    fn new(subject: &str) -> Self {
        Self {
            sub: subject.to_string(),
            roles: None,
            exp: None,
            data: None,
        }
    }

    fn add_role(&mut self, role: &str) {
        let mut roles = self.roles.clone().unwrap_or_default();
        roles.push(role.to_string());
        self.roles = Some(roles);
    }

    fn set_expiration(&mut self, seconds_from_now: i64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.exp = Some(now + seconds_from_now);
    }

    fn has_role(&self, role: &str) -> bool {
        if let Some(roles) = &self.roles {
            roles.iter().any(|r| r == role)
        } else {
            false
        }
    }
}

struct JwtMonitor;

impl JwtMonitor {
    fn new(_middleware: ()) -> Self {
        Self
    }
    
    fn get_report(&self) -> Value {
        json!({
            "total_requests": 1000,
            "success_rate": 95.5,
            "avg_processing_time_ms": 12.3,
            "error_breakdown": {
                "token_missing_errors": 20,
                "token_invalid_errors": 15,
                "token_expired_errors": 10,
                "rate_limit_errors": 5
            }
        })
    }
}