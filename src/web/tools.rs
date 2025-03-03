#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Pagination {
    pub page: u32,
    pub size: u32,
}

impl Pagination {
    pub fn new(page: u32, size: u32) -> Self {
        Pagination { page, size }
    }
}

