pub struct Obj;
pub struct Net;
pub struct Len;
pub struct Num;
pub struct Enc;

impl Obj {
    pub fn defaults<T: Default + PartialEq>(val: &T) -> bool {
        *val == T::default()
    }

    pub fn empty(s: &str) -> bool {
        s.trim().is_empty()
    }
}

impl Net {
    pub fn email(email: &str) -> bool {
        regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap().is_match(email)
    }

    pub fn china_mobile(mobile: &str) -> bool {
        mobile.len() == 11 && mobile.starts_with("1") && mobile.chars().all(|c| c.is_digit(10))
    }

    pub fn chinese(s: &str) -> bool {
        match regex::Regex::new(r"[\u4e00-\u9fa5]") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }

    pub fn url(s: &str) -> bool {
        match regex::Regex::new(r"^(http|https|ftp)://[^\s]+$") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }

    pub fn ip4(s: &str) -> bool {
        match regex::Regex::new(r"^\d+\.\d+\.\d+\.\d+$") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }

    pub fn ip6(s: &str) -> bool {
        match regex::Regex::new(r"^\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}$") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }

    pub fn mac(s: &str) -> bool {
        match regex::Regex::new(r"^[0-9A-Fa-f]{2}(:[0-9A-Fa-f]{2}){5}$") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }
}


impl Len {
    pub fn range(s: &str, min: usize, max: usize) -> bool {
        let l = s.len();
        l >= min && l <= max
    }

    pub fn min(s: &str, min: usize) -> bool {
        s.len() >= min
    }

    pub fn max(s: &str, max: usize) -> bool {
        s.len() <= max
    }

    pub fn equal(s: &str, len: usize) -> bool {
        s.len() == len
    }
}


impl Num {
    pub fn number(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit())
    }

    pub fn float(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit() || c == '.')
    }

    pub fn int(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit())
    }

    pub fn hex(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_hexdigit())
    }

    pub fn oct(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit() && c < '8')
    }

    pub fn bin(s: &str) -> bool {
        s.chars().all(|c| c == '0' || c == '1')
    }
}

impl Enc {
    pub fn ascii(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii())
    }

    pub fn alpha(s: &str) -> bool {
        match regex::Regex::new(r"^[a-zA-Z]+$") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }

    pub fn alphanumeric(s: &str) -> bool {
        match regex::Regex::new(r"^[a-zA-Z0-9]+$") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }

    pub fn base64(s: &str) -> bool {
        match regex::Regex::new(r"^[A-Za-z0-9+/=]+$") {
            Ok(regex) => regex.is_match(s),
            Err(_) => false,
        }
    }
}

 


