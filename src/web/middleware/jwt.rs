#[allow(unused_imports, dead_code)]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};


#[allow(unused_imports, dead_code)]
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