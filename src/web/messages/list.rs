// Define common structures frequently used in web development APIs

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List<T> {
    total: usize,
    values: Vec<T>,
}

impl<T> List<T> {
    pub fn new(total: usize) -> Self {
        Self { total, values: Vec::new() }
    }

    pub fn with_values(values: Vec<T>) -> Self {
        let total = values.len();
        Self { total, values }
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }
    
    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }

    pub fn add(&mut self, value: T) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn into_values(self) -> Vec<T> {
        self.values
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(values: Vec<T>) -> Self {
        Self::with_values(values)
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

/// 分页列表结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PagedList<T> {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub values: Vec<T>,
}

impl<T> PagedList<T> {
    pub fn new(total: usize, page: usize, page_size: usize, values: Vec<T>) -> Self {
        Self { total, page, page_size, values }
    }

    pub fn total_pages(&self) -> usize {
        if self.page_size == 0 {
            0
        } else {
            (self.total + self.page_size - 1) / self.page_size
        }
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages()
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

/// 排序方向
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Asc
    }
}

/// 排序参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SortBy {
    pub field: String,
    pub direction: SortDirection,
}

impl SortBy {
    pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }

    pub fn asc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Asc)
    }

    pub fn desc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Desc)
    }
}
