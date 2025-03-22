#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Pagination {
    pub page: usize,
    pub size: usize,
}

impl Pagination {
    pub fn new(page: usize, size: usize) -> Self {
        Pagination { page, size }
    }
}
