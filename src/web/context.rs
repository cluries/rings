use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Context {
    ident: String,
    born: i64,
    vals: HashMap<String, String>,
}

impl Context {
    pub fn new(ident: String) -> Context {
        Context { ident, born: chrono::Utc::now().timestamp_micros(), vals: HashMap::new() }
    }

    pub fn get_born(&self) -> i64 {
        self.born
    }

    pub fn set_ident(&mut self, ident: String) {
        self.ident = ident;
    }

    pub fn get_ident(&self) -> String {
        self.ident.clone()
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.vals.get(key).cloned()
    }

    pub fn get_str_or(&self, key: &str, default: &str) -> String {
        self.vals.get(key).cloned().unwrap_or(default.to_string())
    }

    pub fn set_str(&mut self, key: &str, val: &str) {
        self.vals.insert(key.to_string(), val.to_string());
    }
}
