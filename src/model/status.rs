use serde::{Deserialize, Serialize};

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

const MARK_DELETED_STR: &'static str = "MarkDelete";
const INITIALIZE_STR: &'static str = "Initialize";
static OKP: &str = "OK(";
static ERP: &str = "ERR(";

impl Status {

    /// parse from string
    pub fn parse(formated: &str) -> crate::erx::ResultE<Self> {
        let formated = formated.trim();
        if formated.len() < 1 {
            return Err("Empty formated status".into());
        }

        fn inner_parse(s: String) -> crate::erx::ResultE<(i32, String)> {
            let splits: Vec<&str> = s.splitn(2, " ").collect();
            if splits.len() < 1 {
                return Err("Empty formated status".into());
            }

            if !splits[0].ends_with(")") {
                return Err("Invalid formated status".into());
            }

            let code = splits[0][..splits[0].len() - 1].parse::<i32>().map_err(crate::erx::smp)?;
            Ok((code, if splits.len() > 1 { splits[1] } else { "" }.to_string()))
        }

        match formated {
            INITIALIZE_STR => Ok(Status::Initialize),
            MARK_DELETED_STR => Ok(Status::MarkDeleted),
            formated => {
                if formated.starts_with(OKP) {
                    let parsed = inner_parse(formated[OKP.len()..].to_string())?;
                    Self::ok(parsed.0, &parsed.1)
                } else if formated.starts_with(ERP) {
                    let parsed = inner_parse(formated[ERP.len()..].to_string())?;
                    Self::error(parsed.0, &parsed.1)
                } else {
                    Err(format!("Unknown status: {}", formated).into())
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

    pub fn initialize() -> Self {
        Status::Initialize
    }

    pub fn deleted() -> Self {
        Status::MarkDeleted
    }

    pub fn ok(val: i32, message: &str) -> crate::erx::ResultE<Self> {
        if Self::valid_ok_code(val) {
            Ok(Status::OK(val, message.to_string()))
        } else {
            Err(format!("invalid ok code:{}  code must GTE(>=) BOUNDARY_OK:{}", val, BOUNDARY_OK).into())
        }
    }

    pub fn error(val: i32, message: &str) -> crate::erx::ResultE<Self> {
        if Self::valid_error_code(val) {
            Ok(Status::Error(val, message.to_string()))
        } else {
            Err(format!("invalid error code:{}, code must LTE(<=) BOUNDARY_ERROR:{}", val, BOUNDARY_ERROR).into())
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

impl Into<(i32, String)> for Status {
    fn into(self) -> (i32, String) {
        match self {
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

        let s = crate::tools::json::Enc::en(&Status::ok(1000, "waiting")).unwrap();
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
