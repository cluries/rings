/// IgnoreCase::Prefix("prefix").matches("content")
pub enum IgnoreCase {
    Contain(String),
    Prefix(String),
    Suffix(String),
}

/// Sub string tools.
pub struct Sub;

/// Word tools.
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

    /// Extract all strings between two strings.
    /// 提取字符串中间的字符串
    pub fn extract(s: &str, start: &str, end: &str) -> Vec<String> {
        let mut vec = vec![];
        let mut pos = 0;
        while pos < s.len() {
            // 查找起始字符串
            if let Some(start_pos) = s[pos..].find(start) {
                let start_index = pos + start_pos + start.len();
                // 从起始位置开始查找结束字符串
                if let Some(end_pos) = s[start_index..].find(end) {
                    let end_index = start_index + end_pos;
                    // 提取中间的字符串
                    vec.push(s[start_index..end_index].to_string());
                    pos = end_index + end.len();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        vec
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_extract() {
        let s = "china is very big and strong";
        let start = "is";
        let end = "and";
        let result = Sub::extract(s, start, end);
        println!("{:?}", result);
    }
}
 
 