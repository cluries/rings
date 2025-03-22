pub struct Like {
    val: String,
}

impl Like {
    pub fn new(val: String) -> Self {
        Self { val }
    }

    pub fn full(&self) -> String {
        format!("%{}%", self.val)
    }

    pub fn left(&self) -> String {
        format!("%{}", self.val)
    }

    pub fn right(&self) -> String {
        format!("%{}", self.val)
    }
}

impl From<&str> for Like {
    fn from(val: &str) -> Self {
        Self { val: String::from(val) }
    }
}

impl From<String> for Like {
    fn from(val: String) -> Self {
        Self { val }
    }
}
