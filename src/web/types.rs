pub static DEFAULT_PAGE_SIZE: usize = 50;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Pagination {
    pub page: usize,
    pub size: usize,
}

impl Pagination {
    pub fn new(page: usize, size: usize) -> Self {
        Pagination { page, size }
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn offset(&self) -> usize {
        if self.page == 0 { 0 } else { (self.page - 1) * self.size }
    }

    pub fn raw_sql(&self) -> String {
        format!(" LIMIT {} OFFSET {}", self.size, self.offset())
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination::new(1, DEFAULT_PAGE_SIZE)
    }
}
