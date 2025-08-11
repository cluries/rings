use serde::{Deserialize, Serialize};

/// 验证错误
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: Option<String>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self { field: field.into(), message: message.into(), code: None }
    }

    pub fn with_code(field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) -> Self {
        Self { field: field.into(), message: message.into(), code: Some(code.into()) }
    }
}

/// 验证结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self { is_valid: true, errors: Vec::new() }
    }

    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self { is_valid: false, errors }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }

    pub fn has_field_error(&self, field: &str) -> bool {
        self.errors.iter().any(|e| e.field == field)
    }

    pub fn get_field_errors(&self, field: &str) -> Vec<&ValidationError> {
        self.errors.iter().filter(|e| e.field == field).collect()
    }
}

/// 验证规则
pub trait ValidationRule<T> {
    fn validate(&self, value: &T, field: &str) -> ValidationResult;
}
