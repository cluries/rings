use crate::erx;

pub type One<T> = erx::ResultE<Option<T>>;
pub type Many<T> = erx::ResultE<Vec<T>>;

pub struct Related<T> {
    records: Vec<T>,
    total: usize,
    offset: usize,
}

impl<T> Related<T> {
    pub fn records(&self) -> &Vec<T> {
        &self.records
    }

    pub fn records_count(&self) -> usize {
        self.records.len()
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}
