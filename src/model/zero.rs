pub trait Zero {
    fn seaorm() -> sea_orm::Value;
    fn human() -> String;
}

impl<Tz: chrono::TimeZone> Zero for chrono::DateTime<Tz> {
    fn seaorm() -> sea_orm::Value {
        chrono::DateTime::from_timestamp_nanos(0).naive_utc().into()
    }
    fn human() -> String {
        "0".to_string()
    }
}

impl Zero for String {
    fn seaorm() -> sea_orm::Value {
        "".into()
    }
    fn human() -> String {
        "".to_string()
    }
}

impl Zero for i64 {
    fn seaorm() -> sea_orm::Value {
        0i64.into()
    }
    fn human() -> String {
        "0".to_string()
    }
}

impl Zero for u64 {
    fn seaorm() -> sea_orm::Value {
        0u64.into()
    }
    fn human() -> String {
        "0".to_string()
    }
}

impl Zero for f64 {
    fn seaorm() -> sea_orm::Value {
        0f64.into()
    }
    fn human() -> String {
        "0".to_string()
    }
}

impl Zero for bool {
    fn seaorm() -> sea_orm::Value {
        false.into()
    }
    fn human() -> String {
        "false".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        println!("{}", chrono::DateTime::<chrono::FixedOffset>::seaorm());
    }
}
