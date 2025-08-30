pub struct DynObject {
    pub id: String,
    pub data: serde_json::Value,
}

impl DynObject {
    pub fn new(id: &str, data: serde_json::Value) -> Self {
        Self { id: id.to_string(), data }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_data(&self) -> &serde_json::Value {
        &self.data
    }

    pub fn set_data(&mut self, data: serde_json::Value) {
        self.data = data;
    }
}
