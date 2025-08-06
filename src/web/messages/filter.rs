use serde::{Deserialize, Serialize};

/// 过滤操作符
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FilterOperator {
    Eq,      // 等于
    Ne,      // 不等于
    Gt,      // 大于
    Gte,     // 大于等于
    Lt,      // 小于
    Lte,     // 小于等于
    Like,    // 模糊匹配
    In,      // 在列表中
    NotIn,   // 不在列表中
    IsNull,  // 为空
    IsNotNull, // 不为空
    Between, // 在范围内
}

/// 过滤条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

impl FilterCondition {
    pub fn eq(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::Eq,
            value: value.into(),
        }
    }

    pub fn ne(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::Ne,
            value: value.into(),
        }
    }

    pub fn gt(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::Gt,
            value: value.into(),
        }
    }

    pub fn like(field: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::Like,
            value: serde_json::Value::String(pattern.into()),
        }
    }

    pub fn is_null(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::IsNull,
            value: serde_json::Value::Null,
        }
    }
}

/// 查询过滤器
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryFilter {
    pub conditions: Vec<FilterCondition>,
    pub logic: LogicOperator,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogicOperator {
    And,
    Or,
}

impl QueryFilter {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            logic: LogicOperator::And,
        }
    }

    pub fn and(mut self, condition: FilterCondition) -> Self {
        self.conditions.push(condition);
        self.logic = LogicOperator::And;
        self
    }

    pub fn or(mut self, condition: FilterCondition) -> Self {
        self.conditions.push(condition);
        self.logic = LogicOperator::Or;
        self
    }

    pub fn add_condition(&mut self, condition: FilterCondition) {
        self.conditions.push(condition);
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// 搜索查询参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub keyword: Option<String>,
    pub filters: QueryFilter,
    pub fields: Option<Vec<String>>, // 指定搜索字段
}

impl SearchQuery {
    pub fn new() -> Self {
        Self {
            keyword: None,
            filters: QueryFilter::new(),
            fields: None,
        }
    }

    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keyword = Some(keyword.into());
        self
    }

    pub fn with_filter(mut self, filter: QueryFilter) -> Self {
        self.filters = filter;
        self
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self::new()
    }
}