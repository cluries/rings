pub mod suber {

    pub fn contains_ignore_case(s: &str, val: &str) -> bool {
        s.to_ascii_lowercase().contains(&val.to_ascii_lowercase())
    }

    pub fn is_prefix_ignore_case(s: &str, prefix: &str) -> bool {
        fn cmi<T: Iterator<Item = char>>(sources: &mut T, prefixs: &mut T) -> bool {
            loop {
                match (sources.next(), prefixs.next()) {
                    (Some(s), Some(p)) => {
                        if !s.eq_ignore_ascii_case(&p) {
                            return false;
                        }
                    },
                    (_, None) => return true,        // 所有 prefix 字符已匹配
                    (None, Some(_)) => return false, // source 比 prefix 短
                }
            }
        }

        let mut sources = s.chars();
        let mut prefixs = prefix.chars();
        cmi(&mut sources, &mut prefixs)
    }

    pub fn is_suffix_ignore_case(s: &str, suffix: &str) -> bool {
        fn cmi<T: Iterator<Item = char>>(sources: &mut T, prefixs: &mut T) -> bool {
            loop {
                match (sources.next(), prefixs.next()) {
                    (Some(s), Some(p)) => {
                        if !s.eq_ignore_ascii_case(&p) {
                            return false;
                        }
                    },
                    (_, None) => return true,        // 所有 prefix 字符已匹配
                    (None, Some(_)) => return false, // source 比 prefix 短
                }
            }
        }

        let mut sources = s.chars().rev();
        let mut suffixs = suffix.chars().rev();
        cmi(&mut sources, &mut suffixs)
    }

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

pub mod word {
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
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    pub fn lcfirst(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + chars.as_str(),
        }
    }

    pub fn ucwords(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    pub fn lcwords(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + chars.as_str(),
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
        let s = "china is huge and strong";
        let start = "is";
        let end = "and";
        let result = suber::extract(s, start, end);
        println!("{:?}", result);
    }
}
