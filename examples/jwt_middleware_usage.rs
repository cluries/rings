//! # JWT 中间件完整使用示例
//! 
//! 本文件展示了如何在实际项目中使用 JWT 认证中间件的所有功能，包括：
//! - 基础JWT认证
//! - 令牌刷新机制
//! - 黑名单管理
//! - 速率限制
//! - 性能监控

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::net::TcpListener;

// 假设这些是你的项目模块
// use rings::web::middleware::jwt::{
//     Claims, JwtConfig, JwtGenerator, JwtMiddleware, JwtMonitor,
//     JwtMiddlewareSuite, JwtMiddlewareBuilder, RateLimitConfig,
//     TokenPair, TokenRevocationService, create_standard_jwt_middleware,
// };

/// 登录请求结构
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// 用户登录处理器（支持令牌对生成）
async fn login_handler(
    Extension(jwt_suite): Extension<JwtMiddlewareSuite>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<Value>, StatusCode> {
    // 这里应该验证用户凭据
    // 为了示例，我们假设验证成功
    
    if login_req.username.is_empty() || login_req.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 创建用户声明
    let mut claims = Claims::new(&login_req.username);
    
    // 根据用户设置角色（这里是示例逻辑）
    match login_req.username.as_str() {
        "admin" => {
            claims.add_role("admin");
            claims.add_role("user");
        }
        "editor" => {
            claims.add_role("editor");
            claims.add_role("user");
        }
        _ => {
            claims.add_role("user");
        }
    }
    
    claims.set_issuer("my-auth-service");
    
    // 生成令牌对（如果启用了刷新功能）
    match jwt_suite.generate_token_pair(&claims) {
        Ok(token_pair) => Ok(Json(json!({
            "success": true,
            "access_token": token_pair.access_token,
            "refresh_token": token_pair.refresh_token,
            "access_expires_in": token_pair.access_expires_in,
            "refresh_expires_in": token_pair.refresh_expires_in,
            "token_type": token_pair.token_type
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 令牌刷新请求结构
#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

/// 令牌刷新处理器
async fn refresh_token_handler(
    Extension(jwt_suite): Extension<JwtMiddlewareSuite>,
    Json(refresh_req): Json<RefreshRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Some(refresh_manager) = jwt_suite.refresh_manager() {
        match refresh_manager.refresh_access_token(&refresh_req.refresh_token) {
            Ok(token_pair) => Ok(Json(json!({
                "success": true,
                "access_token": token_pair.access_token,
                "refresh_token": token_pair.refresh_token,
                "access_expires_in": token_pair.access_expires_in,
                "refresh_expires_in": token_pair.refresh_expires_in,
                "token_type": token_pair.token_type
            }))),
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

/// 令牌撤销请求结构
#[derive(Deserialize)]
struct RevokeRequest {
    token: String,
    reason: Option<String>,
}

/// 令牌撤销处理器
async fn revoke_token_handler(
    Extension(jwt_suite): Extension<JwtMiddlewareSuite>,
    Json(revoke_req): Json<RevokeRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Some(revocation_service) = jwt_suite.create_revocation_service() {
        let reason = revoke_req.reason.unwrap_or_else(|| "User requested".to_string());
        
        match revocation_service.revoke_token(&revoke_req.token, &reason) {
            Ok(_) => Ok(Json(json!({
                "success": true,
                "message": "Token revoked successfully"
            }))),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        }
    } else {
        Err(StatusCode::NOT_IMPLEMENTED)
    }
}

/// 需要认证的用户信息处理器
async fn user_info_handler(Extension(claims): Extension<Claims>) -> Json<Value> {
    Json(json!({
        "user_id": claims.sub,
        "roles": claims.roles.unwrap_or_default(),
        "issued_at": claims.iat,
        "expires_at": claims.exp
    }))
}

/// 需要管理员权限的处理器
async fn admin_handler(Extension(claims): Extension<Claims>) -> Json<Value> {
    Json(json!({
        "message": "Welcome, admin!",
        "user_id": claims.sub,
        "admin_access": true
    }))
}

/// 需要编辑权限的处理器
async fn editor_handler(Extension(claims): Extension<Claims>) -> Json<Value> {
    Json(json!({
        "message": "Editor dashboard",
        "user_id": claims.sub,
        "can_edit": true
    }))
}

/// 公开访问的健康检查处理器
async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// 性能监控处理器
async fn metrics_handler(Extension(jwt_suite): Extension<JwtMiddlewareSuite>) -> Json<Value> {
    let report = jwt_suite.get_comprehensive_report();
    Json(json!(report))
}

/// 用户状态查询参数
#[derive(Deserialize)]
struct UserStatusQuery {
    user_id: Option<String>,
}

/// 用户状态处理器（显示速率限制状态）
async fn user_status_handler(
    Extension(jwt_suite): Extension<JwtMiddlewareSuite>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<UserStatusQuery>,
) -> Json<Value> {
    let user_id = query.user_id.unwrap_or_else(|| claims.sub.clone());
    
    let mut status = json!({
        "user_id": user_id,
        "roles": claims.roles.unwrap_or_default(),
        "issued_at": claims.iat,
        "expires_at": claims.exp
    });
    
    // 添加速率限制状态
    if let Some(rate_limit_manager) = jwt_suite.rate_limit_manager() {
        if let Ok(Some(rate_stats)) = rate_limit_manager.get_user_stats(&user_id) {
            status["rate_limit"] = json!(rate_stats);
        }
    }
    
    Json(status)
}

/// 管理员面板处理器（显示系统统计）
async fn admin_dashboard_handler(
    Extension(jwt_suite): Extension<JwtMiddlewareSuite>,
    Extension(claims): Extension<Claims>,
) -> Json<Value> {
    let mut dashboard = json!({
        "admin_user": claims.sub,
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    // 添加综合报告
    let report = jwt_suite.get_comprehensive_report();
    dashboard["system_stats"] = json!(report);
    
    Json(dashboard)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动完整的JWT认证服务器...");
    
    // 1. 创建完整的JWT中间件套件
    let jwt_suite = create_standard_jwt_middleware("your-super-secret-jwt-key-at-least-32-chars");
    
    // 2. 启动后台任务
    let _background_tasks = jwt_suite.start_background_tasks();
    
    // 3. 创建不同权限级别的中间件
    let base_config = JwtConfig::new("your-super-secret-jwt-key-at-least-32-chars")
        .with_algorithm(jsonwebtoken::Algorithm::HS256)
        .with_issuer("my-auth-service")
        .with_cookie_extraction(true, "session_token")
        .with_query_extraction(true, "access_token");

    let user_middleware = JwtMiddleware::new(base_config.clone())
        .require_any_role(vec!["user", "editor", "admin"]);

    let editor_middleware = JwtMiddleware::new(base_config.clone())
        .require_any_role(vec!["editor", "admin"]);

    let admin_middleware = JwtMiddleware::new(base_config.clone())
        .require_role("admin");

    // 4. 构建应用路由
    let app = Router::new()
        // 公开端点（无需认证）
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        
        // 认证端点
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh", post(refresh_token_handler))
        .route("/auth/revoke", post(revoke_token_handler))
        
        // 需要基本用户认证的端点
        .route("/api/user/info", get(user_info_handler))
        .route("/api/user/status", get(user_status_handler))
        .route_layer(user_middleware)
        
        // 需要编辑权限的端点
        .route("/api/editor/dashboard", get(editor_handler))
        .route_layer(editor_middleware)
        
        // 需要管理员权限的端点
        .route("/api/admin/dashboard", get(admin_dashboard_handler))
        .route_layer(admin_middleware)
        
        // 添加JWT套件到应用扩展
        .layer(axum::extract::Extension(jwt_suite.clone()));

    // 5. 启动定期报告
    let suite_clone = jwt_suite.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            suite_clone.print_comprehensive_report();
        }
    });

    // 6. 启动服务器
    println!("📍 服务地址: http://localhost:3000");
    println!("\n📋 可用端点:");
    println!("  POST /auth/login           - 用户登录（获取令牌对）");
    println!("  POST /auth/refresh         - 刷新访问令牌");
    println!("  POST /auth/revoke          - 撤销令牌");
    println!("  GET  /health               - 健康检查（公开访问）");
    println!("  GET  /metrics              - 综合性能指标（公开访问）");
    println!("  GET  /api/user/info        - 用户信息（需要用户认证）");
    println!("  GET  /api/user/status      - 用户状态（包含速率限制信息）");
    println!("  GET  /api/editor/dashboard - 编辑器面板（需要编辑权限）");
    println!("  GET  /api/admin/dashboard  - 管理员面板（需要管理员权限）");
    
    println!("\n🔧 功能特性:");
    println!("  ✅ JWT令牌认证和验证");
    println!("  ✅ 自动令牌刷新机制");
    println!("  ✅ 令牌黑名单和撤销");
    println!("  ✅ 基于用户角色的速率限制");
    println!("  ✅ 实时性能监控");
    println!("  ✅ 多种令牌提取方式");
    
    println!("\n🔧 使用方法:");
    println!("1. 登录获取令牌对:");
    println!("   curl -X POST http://localhost:3000/auth/login \\");
    println!("        -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"username\":\"admin\",\"password\":\"password\"}}'");
    
    println!("\n2. 使用访问令牌访问受保护资源:");
    println!("   curl -H 'Authorization: Bearer <access_token>' \\");
    println!("        http://localhost:3000/api/user/info");
    
    println!("\n3. 刷新过期的访问令牌:");
    println!("   curl -X POST http://localhost:3000/auth/refresh \\");
    println!("        -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"refresh_token\":\"<refresh_token>\"}}'");

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 测试用例模块
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_jwt_middleware_flow() {
        // 创建测试应用
        let jwt_config = JwtConfig::new("test-secret");
        let middleware = JwtMiddleware::new(jwt_config.clone());
        
        let app = Router::new()
            .route("/protected", get(user_info_handler))
            .route_layer(middleware)
            .route("/login", post(login_handler));

        // 1. 测试未认证访问
        let request = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // 2. 测试登录获取令牌
        let login_request = Request::builder()
            .method("POST")
            .uri("/login")
            .body(Body::empty())
            .unwrap();
        
        let login_response = app.clone().oneshot(login_request).await.unwrap();
        assert_eq!(login_response.status(), StatusCode::OK);

        // 在实际测试中，你需要解析响应获取令牌，然后用它来测试认证访问
    }

    #[tokio::test]
    async fn test_role_based_access() {
        let jwt_config = JwtConfig::new("test-secret");
        let generator = JwtGenerator::new(jwt_config.clone());
        
        // 创建用户令牌
        let mut user_claims = Claims::new("user123");
        user_claims.add_role("user");
        user_claims.set_expiration(3600);
        let user_token = generator.generate_token(&user_claims).unwrap();
        
        // 创建管理员令牌
        let mut admin_claims = Claims::new("admin123");
        admin_claims.add_role("admin");
        admin_claims.set_expiration(3600);
        let admin_token = generator.generate_token(&admin_claims).unwrap();
        
        // 创建需要管理员权限的中间件
        let admin_middleware = JwtMiddleware::new(jwt_config)
            .require_role("admin");
        
        let app = Router::new()
            .route("/admin", get(admin_handler))
            .route_layer(admin_middleware);
        
        // 测试用户访问管理员端点（应该失败）
        let user_request = Request::builder()
            .uri("/admin")
            .header("Authorization", format!("Bearer {}", user_token))
            .body(Body::empty())
            .unwrap();
        
        let user_response = app.clone().oneshot(user_request).await.unwrap();
        assert_eq!(user_response.status(), StatusCode::FORBIDDEN);
        
        // 测试管理员访问管理员端点（应该成功）
        let admin_request = Request::builder()
            .uri("/admin")
            .header("Authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap();
        
        let admin_response = app.clone().oneshot(admin_request).await.unwrap();
        assert_eq!(admin_response.status(), StatusCode::OK);
    }
}

/// 性能测试模块
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_jwt_performance() {
        let jwt_config = JwtConfig::new("performance-test-secret");
        let generator = JwtGenerator::new(jwt_config.clone());
        let middleware = JwtMiddleware::new(jwt_config);
        
        // 生成测试令牌
        let mut claims = Claims::new("perf_user");
        claims.set_expiration(3600);
        let token = generator.generate_token(&claims).unwrap();
        
        // 创建测试应用
        let app = Router::new()
            .route("/test", get(user_info_handler))
            .route_layer(middleware.clone());
        
        // 性能测试：发送1000个请求
        let start = Instant::now();
        let mut tasks = vec![];
        
        for _ in 0..1000 {
            let app_clone = app.clone();
            let token_clone = token.clone();
            
            let task = tokio::spawn(async move {
                let request = Request::builder()
                    .uri("/test")
                    .header("Authorization", format!("Bearer {}", token_clone))
                    .body(Body::empty())
                    .unwrap();
                
                app_clone.oneshot(request).await
            });
            
            tasks.push(task);
        }
        
        // 等待所有请求完成
        for task in tasks {
            let response = task.await.unwrap().unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        
        let duration = start.elapsed();
        println!("1000 requests completed in: {:?}", duration);
        println!("Average request time: {:?}", duration / 1000);
        
        // 打印性能报告
        let monitor = JwtMonitor::new(middleware);
        monitor.print_detailed_report();
    }
}