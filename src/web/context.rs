use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ident {
    /// user ident
    pub ident: String,
    /// who provide it, maybe some middleware
    pub by: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Context {
    ident: Option<Ident>,
    ident_history: Vec<Ident>,
    born_micros: i64,
    vals: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_born_micros(&self) -> i64 {
        self.born_micros
    }

    pub fn set_ident(&mut self, ident: String, by: String) -> &mut Self {
        if let Some(ident) = self.ident.take() {
            self.ident_history.push(ident);
        }

        self.ident = Some(Ident { ident, by });
        self
    }

    pub fn ident_direct(&self) -> Option<String> {
        self.ident.as_ref().map(|ident| ident.ident.clone())
    }

    pub fn ident_must(&self) -> String {
        self.ident.as_ref().map(|ident| ident.ident.clone()).unwrap_or_default()
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

    pub fn get_ident_by(&self) -> Option<String> {
        self.ident.as_ref().map(|ident| ident.by.clone())
    }

    pub fn get_ident_history(&self) -> &Vec<Ident> {
        &self.ident_history
    }

    pub fn clear_ident(&mut self) -> &mut Self {
        if let Some(ident) = self.ident.take() {
            self.ident_history.push(ident);
        }
        self
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.vals.contains_key(key)
    }

    pub fn remove_val(&mut self, key: &str) -> Option<String> {
        self.vals.remove(key)
    }

    pub fn clear_vals(&mut self) -> &mut Self {
        self.vals.clear();
        self
    }

    pub fn get_all_vals(&self) -> &HashMap<String, String> {
        &self.vals
    }
}

impl Default for Context {
    fn default() -> Self {
        Self { ident: None, ident_history: vec![], born_micros: chrono::Utc::now().timestamp_micros(), vals: HashMap::new() }
    }
}
