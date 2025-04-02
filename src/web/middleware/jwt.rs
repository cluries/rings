use std::sync::Arc;
use axum::{extract::Request, middleware::Next, response::Response};
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use crate::web::context::Context;
use crate::erx::{Layouted, LayoutedC};

/// JWT错误类别标识
static JWT_STR: &str = "JWT";

/// JWT配置
#[derive(Clone)]
pub struct JwtConfig {
    /// 密钥
    secret: String,
    /// 令牌过期时间（秒）
    expiration: i64,
    /// 签发者
    issuer: Option<String>,
    /// 算法
    algorithm: Algorithm,
}

impl JwtConfig {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            expiration: 3600, // 默认1小时
            issuer: None,
            algorithm: Algorithm::HS256,
        }
    }

    pub fn with_expiration(mut self, seconds: i64) -> Self {
        self.expiration = seconds;
        self
    }

    pub fn with_issuer(mut self, issuer: String) -> Self {
        self.issuer = Some(issuer);
        self
    }

    pub fn with_algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self
    }
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 主题（通常是用户ID）
    pub sub: String,
    /// 签发者
    pub iss: Option<String>,
    /// 签发时间
    pub iat: i64,
    /// 过期时间
    pub exp: i64,
    /// 自定义数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JWT中间件
pub struct JwtMiddleware {
    config: JwtConfig,
}

impl JwtMiddleware {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// 生成JWT令牌
    pub fn generate_token(&self, subject: &str, data: Option<serde_json::Value>) -> Result<String, LayoutedC> {
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::seconds(self.config.expiration)).timestamp();

        let claims = Claims {
            sub: subject.to_string(),
            iss: self.config.issuer.clone(),
            iat,
            exp,
            data,
        };

        encode(
            &Header::new(self.config.algorithm),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )
            .map_err(|e| Layouted::middleware(JWT_STR, &format!("Token生成失败: {}", e)))
    }

    /// 验证JWT令牌
    pub fn verify_token(&self, token: &str) -> Result<Claims, LayoutedC> {
        let mut validation = Validation::new(self.config.algorithm);

        // 如果配置了签发者，则验证签发者
        if let Some(issuer) = &self.config.issuer {
            validation.set_issuer(&[issuer]);
        }

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.secret.as_bytes()),
            &validation,
        )
            .map(|data| data.claims)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        Layouted::middleware(JWT_STR, "Token已过期")
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        Layouted::middleware(JWT_STR, "无效的Token签名")
                    }
                    _ => Layouted::middleware(JWT_STR, &format!("Token验证失败: {}", e)),
                }
            })
    }

    /// 从请求头中提取JWT令牌
    fn extract_token_from_headers(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|auth_header| {
                if auth_header.starts_with("Bearer ") {
                    Some(auth_header[7..].to_string())
                } else {
                    None
                }
            })
    }

    /// 从Cookie中提取JWT令牌
    fn extract_token_from_cookie(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get("Cookie")
            .and_then(|value| value.to_str().ok())
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .map(|s| s.trim())
                    .find(|s| s.starts_with("jwt="))
                    .map(|s| s[4..].to_string())
            })
    }

    /// 从请求中提取JWT令牌
    fn extract_token(&self, headers: &HeaderMap) -> Option<String> {
        self.extract_token_from_headers(headers)
            .or_else(|| self.extract_token_from_cookie(headers))
    }

    /// 创建中间件处理函数
    pub fn create_middleware(&self) -> crate::web::middleware::Middleware {
        let config = self.config.clone();
        let jwt_middleware = Arc::new(Self { config });

        crate::web::middleware::Middleware {
            focus: |parts| {
                // 可以在这里定义哪些请求需要JWT验证
                // 例如，可以根据路径前缀或其他条件来决定
                true
            },
            work: move |request, next| {
                let jwt_middleware = jwt_middleware.clone();

                // 提取请求头
                let headers = request.headers();

                // 从请求中提取JWT令牌
                match jwt_middleware.extract_token(headers) {
                    Some(token) => {
                        // 验证令牌
                        match jwt_middleware.verify_token(&token) {
                            Ok(claims) => {
                                // 创建或更新上下文
                                let mut context = Context::new(claims.sub.clone());

                                // 如果有自定义数据，将其添加到上下文中
                                if let Some(data) = claims.data {
                                    if let Some(data_obj) = data.as_object() {
                                        for (key, value) in data_obj {
                                            if let Some(value_str) = value.as_str() {
                                                context.set_str(key, value_str);
                                            } else {
                                                context.set_str(key, &value.to_string());
                                            }
                                        }
                                    }
                                }

                                // 将上下文添加到请求的扩展中
                                let mut req = request;
                                req.extensions_mut().insert(context);

                                // 继续处理请求
                                None
                            },
                            Err(e) => {
                                // 令牌验证失败，返回错误响应
                                Some(Response::builder()
                                    .status(StatusCode::UNAUTHORIZED)
                                    .header("Content-Type", "application/json")
                                    .body(serde_json::to_string(&e).unwrap().into())
                                    .unwrap())
                            }
                        }
                    },
                    None => {
                        // 没有找到令牌，返回错误响应
                        let error = Layouted::middleware(JWT_STR, "缺少JWT令牌");
                        Some(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(&error).unwrap().into())
                            .unwrap())
                    }
                }
            },
        }
    }
}

/// 创建JWT中间件的辅助函数
pub fn jwt_middleware(secret: String) -> JwtMiddleware {
    JwtMiddleware::new(JwtConfig::new(secret))
}

/// 创建带有自定义配置的JWT中间件的辅助函数
pub fn jwt_middleware_with_config(config: JwtConfig) -> JwtMiddleware {
    JwtMiddleware::new(config)
}