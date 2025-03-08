use std::ops::{Add, AddAssign, Index};

pub struct DictAny {
    boxed: std::collections::HashMap<String, Box<dyn std::any::Any>>,
}


impl DictAny {
    pub fn new() -> Self {
        DictAny {
            boxed: std::collections::HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.boxed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.boxed.is_empty()
    }

    pub fn keys(&self) -> Vec<String> {
        self.boxed.iter().map(|(k, _)| k.clone()).collect()
    }

    pub fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        self.boxed.values().cloned().collect()
    }

    pub fn values_sorted_by_key(&self) -> Vec<Box<dyn std::any::Any>> {
        let mut keys = self.keys();
        keys.sort();
        keys.iter().map(|k| self.boxed.get(k)).collect()
    }

    pub fn insert<T: std::any::Any>(&mut self, key: &str, val: T) {
        self.boxed.insert(key.to_string(), Box::new(val));
    }

    pub fn get<T: std::any::Any>(&self, key: &str) -> Option<&T> {
        self.boxed.get(key)
    }

    pub fn get_mut<T: std::any::Any>(&mut self, key: &str) -> Option<&mut T> {
        self.boxed.get_mut(key)
    }

    pub fn remove<T: std::any::Any>(&mut self, key: &str) -> Option<T> {
        self.boxed.remove(key)
    }

    pub fn contains<T: std::any::Any>(&self, key: &str) -> bool {
        self.boxed.contains_key(key)
    }
}


impl<T: std::any::Any> Add for DictAny {
    type Output = DictAny;
    fn add(self, other: DictAny) -> Self::Output {
        let mut result = DictAny::new();

        result.boxed.extend(self.boxed);
        result.boxed.extend(other.boxed);

        result
    }
}

impl<T: std::any::Any> AddAssign for DictAny {
    fn add_assign(&mut self, other: DictAny) {
        self.boxed.extend(other.boxed);
    }
}
