pub enum Lang {
    English,
    Chinese,
}

struct Promptor {
    lang: Lang,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_debug_promptor() {}
}
