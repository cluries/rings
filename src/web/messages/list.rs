#[derive(Clone, Debug)]
pub struct List<T> {
    total: usize,
    values: Vec<T>,
}

impl<T> List<T> {
    pub fn new(total: usize) -> Self {
        Self { total, values: Vec::new() }
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
}
