pub trait Zero {
    fn zero() -> sea_orm::Value;
}

impl<Tz: chrono::TimeZone> Zero for chrono::DateTime<Tz> {
    fn zero() -> sea_orm::Value {
        chrono::DateTime::from_timestamp_nanos(0).naive_local().into()
    }
}

impl Zero for String {
    fn zero() -> sea_orm::Value {
        "".into()
    }
}

impl Zero for i64 {
    fn zero() -> sea_orm::Value {
        0i64.into()
    }
}

impl Zero for u64 {
    fn zero() -> sea_orm::Value {
        0u64.into()
    }
}

impl Zero for f64 {
    fn zero() -> sea_orm::Value {
        0f64.into()
    }
}

impl Zero for bool {
    fn zero() -> sea_orm::Value {
        false.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        println!("{}", chrono::DateTime::<chrono::FixedOffset>::zero());
    }
}
