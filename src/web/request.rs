use axum::body::Bytes;
use axum::extract::Request;
use std::sync::Arc;
use tracing::error;

// TODO 2019.10.24
//
// i think this is a stupid method to clone a request
// but i don't know how to do it, so i just do it, i will make it better later
//
pub async fn clone_request(req: axum::extract::Request) -> (axum::extract::Request, axum::extract::Request) {
    const LIMIT: usize = 1024 * 1024 * 32;

    let (parts, body) = req.into_parts();

    let bytes = axum::body::to_bytes(body, LIMIT).await.unwrap_or_else(|e| {
        error!("axum::body::to_bytes error: {}", e);
        Bytes::new()
    });

    (
        axum::extract::Request::from_parts(parts.clone(), axum::body::Body::from(bytes.clone())),
        axum::extract::Request::from_parts(parts.clone(), axum::body::Body::from(bytes.clone())),
    )
}

/// 可重用的请求克隆器 - 适合需要多次克隆的场景
pub struct RequestCloner {
    parts: Arc<axum::http::request::Parts>,
    body_bytes: Arc<Bytes>,
}

impl RequestCloner {
    /// 从请求创建克隆器
    pub async fn from_request(req: Request) -> Result<Self, axum::Error> {
        const LIMIT: usize = 1024 * 1024 * 32;

        let (parts, body) = req.into_parts();
        let bytes = axum::body::to_bytes(body, LIMIT).await?;

        Ok(Self { parts: Arc::new(parts), body_bytes: Arc::new(bytes) })
    }

    /// 生成一个新的请求实例
    pub fn clone_request(&self) -> Request {
        Request::from_parts((*self.parts).clone(), axum::body::Body::from((*self.body_bytes).clone()))
    }

    /// 获取请求体大小
    pub fn body_size(&self) -> usize {
        self.body_bytes.len()
    }

    /// 检查是否为空请求体
    pub fn is_empty_body(&self) -> bool {
        self.body_bytes.is_empty()
    }
}
