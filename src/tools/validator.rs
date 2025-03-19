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

/// Check if a string matches a regex.
pub fn regex_match(regex: &str, string: &str) -> bool {
    match regex::Regex::new(regex) {
        Ok(re) => re.is_match(string),
        Err(_) => false,
    }
}

/// Extract all matches of a regex from a string.
pub fn regex_extract(regex: &str, string: &str) -> Vec<String> {
    let re = match regex::Regex::new(regex) {
        Ok(re) => re,
        Err(_) => return vec![],
    };

    let mut vec: Vec<String> = Vec::new();
    for cap in re.captures_iter(string) {
        if let Some(m) = cap.get(0) {
            vec.push(m.as_str().to_string());
        }
    }
    vec
}

/// Replace all matches of a regex in a string.
pub fn regex_replace(regex: &str, string: &str, replace: &str) -> String {
    let re = match regex::Regex::new(regex) {
        Ok(re) => re,
        Err(_) => return string.to_string(),
    };
    re.replace_all(string, replace).to_string()
}

/// Split a string by a regex.
pub fn regex_split(regex: &str, string: &str) -> Vec<String> {
    let re = match regex::Regex::new(regex) {
        Ok(re) => re,
        Err(_) => return vec![],
    };
    re.split(string).map(|s| s.to_string()).collect()
}

/// Find the first match of a regex in a string.
pub fn regex_find(regex: &str, string: &str) -> Option<String> {
    let re = match regex::Regex::new(regex) {
        Ok(re) => re,
        Err(_) => return None,
    };
    re.find(string).map(|m| m.as_str().to_string())
}


///
impl Net {
    pub fn email(email: &str) -> bool {
        let r = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        regex_match(r, email)
    }

    pub fn china_mobile(mobile: &str) -> bool {
        mobile.len() == 11 && mobile.starts_with("1") && mobile.chars().all(|c| c.is_digit(10))
    }

    pub fn chinese(s: &str) -> bool {
        let r = r"[\u4e00-\u9fa5]";
        regex_match(r, s)
    }

    pub fn url(s: &str) -> bool {
        let r = r"^(http|https|ftp)://[^\s]+$";
        regex_match(r, s)
    }

    pub fn ip4(s: &str) -> bool {
        let r = r"^\d+\.\d+\.\d+\.\d+$";
        regex_match(r, s)
    }

    pub fn ip6(s: &str) -> bool {
        let r = r"^\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}\:\d{1,4}$";
        regex_match(r, s)
    }

    pub fn mac(s: &str) -> bool {
        let r = r"^[0-9A-Fa-f]{2}(:[0-9A-Fa-f]{2}){5}$";
        regex_match(r, s)
    }
}


impl Len {
    /// Check if a string is between min and max length.
    pub fn range(s: &str, min: usize, max: usize) -> bool {
        let l = s.len();
        l >= min && l <= max
    }

    /// Check if a string is at least min length.
    pub fn min(s: &str, min: usize) -> bool {
        s.len() >= min
    }

    /// Check if a string is at most max length.
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
        const R: &str = r"^[a-zA-Z]+$";
        regex_match(R, s)
    }

    pub fn alphanumeric(s: &str) -> bool {
        regex_match(r"^[a-zA-Z0-9]+$", s)
    }

    pub fn base64(s: &str) -> bool {
        regex_match(r"^[A-Za-z0-9+/=]+$", s)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_regex_extract() {
        let s = "Hello, world! My name is John.";
        let r = regex_extract(r"\w+", s);
        println!("{:?}", r);
        assert_eq!(r, vec!["Hello", "world", "My", "name", "is", "John"]);
        
        let s = "Hello, world! My name is John. i am 18 years old. i am a student. my math score is 90.";
        let r = regex_extract(r"\d+", s);
        println!("{:?}", r);
    }
}