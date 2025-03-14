//IgnoreCase::Prefix("prefix").matches("content")
pub enum IgnoreCase {
    Contain(String),
    Prefix(String),
    Suffix(String),
}

pub struct Sub;
pub struct Word;
 

impl IgnoreCase {
    pub fn matches(&self, s: &str) -> bool {
        match self {
            IgnoreCase::Contain(val) => {
                s.to_lowercase().contains(&val.to_lowercase())
            }
            IgnoreCase::Prefix(prefix) => {
                let src_len = s.len();
                let prx_len = prefix.len();
                if src_len < prx_len {
                    false
                } else {
                    s[..prx_len].eq_ignore_ascii_case(prefix)
                }
            }
            IgnoreCase::Suffix(suffix) => {
                let src_len = s.len();
                let sfx_len = suffix.len();
                if src_len < sfx_len {
                    false
                } else {
                    let index = src_len - sfx_len;
                    s[index..].eq_ignore_ascii_case(suffix)
                }
            }
        }
    }

    pub fn is_contains(s: &str, val: &str) -> bool {
        IgnoreCase::Contain(s.to_string()).matches(val)
    }

    pub fn is_prefix(s: &str, prefix: &str) -> bool {
        IgnoreCase::Prefix(s.to_string()).matches(prefix)
    }

    pub fn is_suffix(s: &str, suffix: &str) -> bool {
        IgnoreCase::Suffix(s.to_string()).matches(suffix)
    }
}


impl Sub {
    pub fn head(s: &str, size: usize) -> String {
        s.chars().take(size).collect::<String>()
    }

    pub fn tail(s: &str, size: usize) -> String {
        s.chars().skip(s.len() - size).collect::<String>()
    }

    pub fn sub(s: &str, start: usize, end: usize) -> String {
        let len = s.len();
        if start >= len || end >= len {
            return "".to_string();
        }
        s[start..end].to_string()
    }
}

impl Word {
    pub fn count(s: &str) -> usize {
        s.split_whitespace().count()
    }

    pub fn head(s: &str, size: usize) -> String {
        let mut count = 0;
        let mut start = 0;
        for (i, c) in s.char_indices() {
            if c.is_whitespace() {
                count += 1;
                if count >= size {
                    return s[start..i].to_string();
                }
                start = i + 1;
            }
        }
        s[start..].to_string()
    }

    pub fn tail(s: &str, size: usize) -> String {
        let mut count = 0;
        let mut end = s.len();
        for (i, c) in s.char_indices().rev() {
            if c.is_whitespace() {
                count += 1;
                if count >= size {
                    return s[i..end].to_string();
                }
                end = i;
            }
        }
        s[..end].to_string()
    }
    
    pub fn ucfirst(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str(),
        }
    }

    pub fn lcfirst(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + &chars.as_str(),
        }
    }

    pub fn ucwords(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str(),
        }
    }

    pub fn lcwords(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + &chars.as_str(),
        }
    }

    pub fn format(s: &str, size: usize) -> String {
        let mut count = 0;
        let mut start = 0;
        let mut end = s.len();
        for (i, c) in s.char_indices() {
            if c.is_whitespace() {
                count += 1;
                if count >= size {
                    end = i;
                    break;
                }
                start = i + 1;
            }
        }
        s[start..end].to_string()
    }
}

 

#[test]
fn test_start_with_ignore_case() {
    let s = IgnoreCase::Prefix("ABC".to_string());
    assert!(s.matches("abc"));
    assert!(!s.matches("ab"));
    assert!(s.matches("abcde"));
    assert!(!s.matches("Aab"));

    //
    // assert!(start_with_ignore_case("ABC", "ABC"));
    // assert!(start_with_ignore_case("ABC", "ab"));
    // assert!(!start_with_ignore_case("ABC", "abcde"));
    // assert!(!start_with_ignore_case("ABC", "Aab"));
}

// #[test]
// fn test_end_with_ignore_case() {
//     // assert!(end_with_ignore_case("ABC", "ABC"));
//     // assert!(!end_with_ignore_case("ABC", "ab"));
//     // assert!(!end_with_ignore_case("ABC", "abcde"));
//     // assert!(end_with_ignore_case("ABC", "c"));
// }

// #[test]
// fn test_strip_prefix_and_suffix() {
//     assert_eq!(strip_prefix_and_suffix("abc_def_abc", 4, 4), "def");
//     assert_eq!(strip_prefix_and_suffix("abc", 4, 4), "");
//     assert_eq!(strip_prefix_and_suffix("strip_prefix_and_suffix", 0, 100), "");
// }