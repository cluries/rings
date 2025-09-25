use serde::{Deserialize, Serialize};

/// 创建请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateRequest<T> {
    pub data: T,
    pub options: Option<CreateOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateOptions {
    pub return_created: bool,
    pub validate_only: bool,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self { return_created: true, validate_only: false }
    }
}

/// 更新请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateRequest<T> {
    pub id: String,
    pub data: T,
    pub options: Option<UpdateOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateOptions {
    pub partial: bool,
    pub return_updated: bool,
    pub validate_only: bool,
    pub version: Option<i64>, // 乐观锁版本号
}

impl Default for UpdateOptions {
    fn default() -> Self {
        Self { partial: true, return_updated: true, validate_only: false, version: None }
    }
}

/// 删除请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub id: String,
    pub options: Option<DeleteOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeleteOptions {
    pub soft_delete: bool,
    pub cascade: bool,
    pub return_deleted: bool,
}

/// 批量创建请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchCreateRequest<T> {
    pub data: Vec<T>,
    pub options: Option<BatchCreateOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchCreateOptions {
    pub continue_on_error: bool,
    pub return_created: bool,
    pub validate_only: bool,
    pub batch_size: Option<usize>,
}

impl Default for BatchCreateOptions {
    fn default() -> Self {
        Self { continue_on_error: true, return_created: false, validate_only: false, batch_size: Some(100) }
    }
}

/// 批量更新请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchUpdateRequest<T> {
    pub updates: Vec<BatchUpdateItem<T>>,
    pub options: Option<BatchUpdateOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchUpdateItem<T> {
    pub id: String,
    pub data: T,
    pub version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchUpdateOptions {
    pub continue_on_error: bool,
    pub partial: bool,
    pub return_updated: bool,
    pub validate_only: bool,
}

impl Default for BatchUpdateOptions {
    fn default() -> Self {
        Self { continue_on_error: true, partial: true, return_updated: false, validate_only: false }
    }
}

/// 批量删除请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
    pub options: Option<BatchDeleteOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchDeleteOptions {
    pub continue_on_error: bool,
    pub soft_delete: bool,
    pub cascade: bool,
    pub return_deleted: bool,
}

impl Default for BatchDeleteOptions {
    fn default() -> Self {
        Self { continue_on_error: true, soft_delete: false, cascade: false, return_deleted: false }
    }
}

/// CRUD操作结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrudResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub affected_rows: usize,
    pub message: Option<String>,
}

impl<T> CrudResult<T> {
    pub fn success(data: T, affected_rows: usize) -> Self {
        Self { success: true, data: Some(data), affected_rows, message: None }
    }

    pub fn success_with_message(data: T, affected_rows: usize, message: impl Into<String>) -> Self {
        Self { success: true, data: Some(data), affected_rows, message: Some(message.into()) }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self { success: false, data: None, affected_rows: 0, message: Some(message.into()) }
    }
}
