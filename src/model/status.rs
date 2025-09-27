use crate::erx::{Erx, ResultBoxedE};
use serde::{Deserialize, Serialize};

// Initialize = 0
// OK( value > 10)
// Error( value < -10)
// MarkDeleted = -1
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub enum Status {
    #[default]
    Initialize,
    OK(i32, String),
    Error(i32, String), //
    MarkDeleted,
}

const INITIALIZE: i32 = 0;
const MARK_DELETED: i32 = -1;

const BOUNDARY_OK: i32 = 11;
const BOUNDARY_ERROR: i32 = -11;

const MARK_DELETED_STR: &str = "MarkDelete";
const INITIALIZE_STR: &str = "Initialize";
const PREFIX_OK: &str = "OK(";
const PREFIX_ERROR: &str = "ERR(";

impl Status {
    /// parse from string
    pub fn parse(formated: &str) -> ResultBoxedE<Self> {
        let formated = formated.trim();
        if formated.is_empty() {
            return Err(Erx::boxed("Empty formated status"));
        }

        fn inner_parse(s: String) -> ResultBoxedE<(i32, String)> {
            let splits: Vec<&str> = s.splitn(2, " ").collect();
            if splits.is_empty() {
                return Err(Erx::boxed("Empty formated status"));
            }

            if !splits[0].ends_with(")") {
                return Err(Erx::boxed("Invalid formated status"));
            }

            let code = splits[0][..splits[0].len() - 1].parse::<i32>().map_err(crate::erx::simple_conv_boxed)?;
            Ok((code, if splits.len() > 1 { splits[1] } else { "" }.to_string()))
        }

        match formated {
            INITIALIZE_STR => Ok(Status::Initialize),
            MARK_DELETED_STR => Ok(Status::MarkDeleted),
            formated => {
                if let Some(stripped) = formated.strip_prefix(PREFIX_OK) {
                    let parsed = inner_parse(stripped.to_string())?;
                    Self::ok(parsed.0, &parsed.1)
                } else if let Some(stripped) = formated.strip_prefix(PREFIX_ERROR) {
                    let parsed = inner_parse(stripped.to_string())?;
                    Self::error(parsed.0, &parsed.1)
                } else {
                    Err(Erx::boxed(&format!("Unknown status: {}", formated)))
                }
            },
        }
    }

    pub fn valid(&self) -> bool {
        match self {
            Status::Initialize => true,
            Status::OK(c, _) => *c >= BOUNDARY_OK,
            Status::Error(c, _) => *c <= BOUNDARY_ERROR,
            Status::MarkDeleted => true,
        }
    }

    /// 检查是否为成功状态
    pub fn is_ok(&self) -> bool {
        matches!(self, Status::OK(_, _))
    }

    /// 检查是否为错误状态
    pub fn is_error(&self) -> bool {
        matches!(self, Status::Error(_, _))
    }

    /// 获取状态码
    pub fn code(&self) -> i32 {
        match self {
            Status::Initialize => INITIALIZE,
            Status::OK(code, _) | Status::Error(code, _) => *code,
            Status::MarkDeleted => MARK_DELETED,
        }
    }

    /// 获取消息
    pub fn message(&self) -> &str {
        match self {
            Status::OK(_, msg) | Status::Error(_, msg) => msg,
            _ => "",
        }
    }

    pub fn initialize() -> Self {
        Status::Initialize
    }

    pub fn deleted() -> Self {
        Status::MarkDeleted
    }

    pub fn ok(val: i32, message: &str) -> ResultBoxedE<Self> {
        if Self::valid_ok_code(val) {
            Ok(Status::OK(val, message.to_string()))
        } else {
            Err(Erx::boxed(&format!("invalid ok code:{}  code must GTE(>=) BOUNDARY_OK:{}", val, BOUNDARY_OK)))
        }
    }

    pub fn error(val: i32, message: &str) -> ResultBoxedE<Self> {
        if Self::valid_error_code(val) {
            Ok(Status::Error(val, message.to_string()))
        } else {
            Err(Erx::boxed(&format!("invalid error code:{}, code must LTE(<=) BOUNDARY_ERROR:{}", val, BOUNDARY_ERROR)))
        }
    }

    fn valid_ok_code(code: i32) -> bool {
        code >= BOUNDARY_OK
    }

    fn valid_error_code(code: i32) -> bool {
        code <= BOUNDARY_ERROR
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Initialize => write!(f, "{}", INITIALIZE_STR),
            Status::OK(id, message) => write!(f, "OK({}) {}", id, message),
            Status::Error(id, message) => write!(f, "ERR({}) {}", id, message),
            Status::MarkDeleted => write!(f, "{}", MARK_DELETED_STR),
        }
    }
}

impl From<Status> for i32 {
    fn from(s: Status) -> Self {
        match s {
            Status::Initialize => INITIALIZE,
            Status::OK(v, _) => v,
            Status::Error(v, _) => v,
            Status::MarkDeleted => MARK_DELETED,
        }
    }
}

impl From<Status> for (i32, String) {
    fn from(s: Status) -> Self {
        match s {
            Status::Initialize => (INITIALIZE, String::new()),
            Status::OK(v, m) => (v, m),
            Status::Error(v, m) => (v, m),
            Status::MarkDeleted => (MARK_DELETED, String::new()),
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
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status() {
        assert_eq!(Status::initialize(), Status::Initialize);

        println!("{}", crate::tools::json::Enc::en(&Status::initialize()).unwrap());

        let s = crate::tools::json::Enc::en(&Status::ok(1000, "waiting").unwrap()).unwrap();
        println!("{}", s);
        println!("{:?}", crate::tools::json::Dec::de::<Status>(s.as_str()).unwrap());
    }

    #[test]
    fn test_parse() {
        let init = Status::initialize();
        println!("{:?}", Status::parse(&*init.to_string()));

        let ok = Status::OK(100, String::from("i am ok status"));
        println!("{:?}", Status::parse(&*ok.to_string()));

        let err = Status::Error(100, String::from("i am err status"));
        println!("{:?}", Status::parse(&*err.to_string()));
    }
}
