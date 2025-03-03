use serde_derive::{Deserialize, Serialize};

// Initialize = 0
// OK( value > 10)
// Error( value < -10)
// MarkDeleted = -1
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Status {
    Initialize,
    OK(i32, String),
    Error(i32, String), //
    MarkDeleted,
}

const INITIALIZE: i32 = 0;
const MARK_DELETED: i32 = -1;

const BOUNDARY_OK: i32 = 11;
const BOUNDARY_ERROR: i32 = -11;

impl Status {
    pub fn initialize() -> Self {
        Status::Initialize
    }

    pub fn deleted() -> Self {
        Status::MarkDeleted
    }

    pub fn ok(val: i32, message: &str) -> Self {
        if val < BOUNDARY_OK {
            panic!("invalid OK({}) - {} ", val, message);
        }
        Status::OK(val, message.to_string())
    }

    pub fn error(val: i32, message: &str) -> Self {
        if val >= BOUNDARY_ERROR {
            panic!("invalid ERR({}) - {} ", val, message);
        }

        Status::Error(val, message.to_string())
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Initialize => write!(f, "Initialize"),
            Status::OK(id, message) => write!(f, "OK ({}): {}", id, message),
            Status::Error(id, message) => write!(f, "ERR ({}): {}", id, message),
            Status::MarkDeleted => write!(f, "MarkDelete"),
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Initialize
    }
}

impl Into<i32> for Status {
    fn into(self) -> i32 {
        match self {
            Status::Initialize => INITIALIZE,
            Status::OK(v, _) => v,
            Status::Error(v, _) => v,
            Status::MarkDeleted => MARK_DELETED,
        }
    }
}

impl From<i32> for Status {
    fn from(v: i32) -> Self {
        (v, "").into()
    }
}

impl From<(i32, &str)> for Status {
    fn from(value: (i32, &str)) -> Self {
        match value.0 {
            INITIALIZE => Status::Initialize,
            MARK_DELETED => Status::MarkDeleted,
            v => {
                if v >= BOUNDARY_OK {
                    Status::OK(v, String::from(value.1))
                } else if v <= BOUNDARY_ERROR {
                    Status::Error(v, String::from(value.1))
                } else {
                    panic!("invalid status val: {} ", v);
                }
            }
        }
    }
}

#[test]
fn test_status() {
    assert_eq!(Status::initialize(), Status::Initialize);

    println!("{}", crate::tools::json::Enc::en(&Status::initialize()).unwrap());

    let s = crate::tools::json::Enc::en(&Status::ok(1000, "waiting")).unwrap();
    println!("{}", s);
    println!(
        "{:?}",
        crate::tools::json::Dec::de::<Status>(s.as_str()).unwrap()
    );
}
