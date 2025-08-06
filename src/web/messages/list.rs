// Define common structures frequently used in web development APIs

use serde::{Deserialize, Serialize};
 
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
