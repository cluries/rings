/// Trait for providing zero/default values for different types
/// Used for database operations and display representations
pub trait Zero {
    /// Returns the zero value as a SeaORM Value for database operations
    fn seaorm() -> sea_orm::Value;
    /// Returns the zero value as a display string
    fn display() -> &'static str;
}

/// Macro to implement Zero trait for numeric types with consistent behavior
macro_rules! impl_zero_numeric {
    ($($t:ty),*) => {
        $(
            impl Zero for $t {
                #[inline]
                fn seaorm() -> sea_orm::Value {
                    <$t>::default().into()
                }

                #[inline]
                fn display() -> &'static str {
                    "0"
                }
            }
        )*
    };
}

// Implement Zero for all numeric types
impl_zero_numeric!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);

impl<Tz: chrono::TimeZone> Zero for chrono::DateTime<Tz> {
    fn seaorm() -> sea_orm::Value {
        // Use UNIX epoch (1970-01-01 00:00:00 UTC) as zero value
        chrono::DateTime::from_timestamp(0, 0).expect("Valid UNIX epoch timestamp").naive_utc().into()
    }

    fn display() -> &'static str {
        "1970-01-01T00:00:00Z"
    }
}

impl Zero for String {
    #[inline]
    fn seaorm() -> sea_orm::Value {
        String::new().into()
    }

    #[inline]
    fn display() -> &'static str {
        ""
    }
}

impl Zero for bool {
    #[inline]
    fn seaorm() -> sea_orm::Value {
        false.into()
    }

    #[inline]
    fn display() -> &'static str {
        "false"
    }
}
