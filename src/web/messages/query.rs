use serde::{Deserialize, Serialize};
use super::{pagination::PaginationQuery, filter::QueryFilter};

/// 通用查询参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryParams {
    pub pagination: PaginationQuery,
    pub filter: QueryFilter,
    pub search: Option<String>,
    pub fields: Option<Vec<String>>, // 指定返回字段
    pub include: Option<Vec<String>>, // 包含关联数据
}

impl QueryParams {
    pub fn new() -> Self {
        Self {
            pagination: PaginationQuery::default(),
            filter: QueryFilter::default(),
            search: None,
            fields: None,
            include: None,
        }
    }

    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_include(mut self, include: Vec<String>) -> Self {
        self.include = Some(include);
        self
    }

    pub fn page(&self) -> usize {
        self.pagination.page()
    }

    pub fn page_size(&self) -> usize {
        self.pagination.page_size()
    }

    pub fn offset(&self) -> usize {
        self.pagination.offset()
    }

    pub fn limit(&self) -> usize {
        self.pagination.limit()
    }
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

/// ID查询参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdQuery {
    pub id: String,
    pub fields: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
}

impl IdQuery {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            fields: None,
            include: None,
        }
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_include(mut self, include: Vec<String>) -> Self {
        self.include = Some(include);
        self
    }
}

/// 批量ID查询参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchIdQuery {
    pub ids: Vec<String>,
    pub fields: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
}

impl BatchIdQuery {
    pub fn new(ids: Vec<String>) -> Self {
        Self {
            ids,
            fields: None,
            include: None,
        }
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_include(mut self, include: Vec<String>) -> Self {
        self.include = Some(include);
        self
    }
}