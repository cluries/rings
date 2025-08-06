use serde::{Deserialize, Serialize};


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


/// 分页查询参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

impl PaginationQuery {
    pub fn page(&self) -> usize {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> usize {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> usize {
        (self.page() - 1) * self.page_size()
    }

    pub fn limit(&self) -> usize {
        self.page_size()
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            sort_by: None,
            sort_direction: None,
        }
    }
}

/// 分页元数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub current_page: usize,
    pub page_size: usize,
    pub total_items: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationMeta {
    pub fn new(current_page: usize, page_size: usize, total_items: usize) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            (total_items + page_size - 1) / page_size
        };

        Self {
            current_page,
            page_size,
            total_items,
            total_pages,
            has_next: current_page < total_pages,
            has_prev: current_page > 1,
        }
    }
}