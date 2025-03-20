use std::collections::HashMap;

pub enum Value {
    Int(i64),
    Bool(bool),
    Float(f64),
    Str(String),
    Object(HashMap<String, Value>),
}

