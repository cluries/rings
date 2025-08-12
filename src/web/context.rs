use std::collections::HashMap;


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ident {
    pub ident: String,
    pub by: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Context {
    ident: Option<Ident>,
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

    pub fn set_ident(&mut self, ident: String, by:String) -> &mut Self {
        self.ident = Some(Ident { ident: ident, by: by });
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
}

impl Default for Context {
    fn default() -> Self {
        Self { ident: None, born_micros: chrono::Utc::now().timestamp_micros(), vals: HashMap::new() }
    }
}
