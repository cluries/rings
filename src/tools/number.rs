use std::str::FromStr;

#[derive(Debug)]
pub enum Number {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
}

// String Convert
pub struct StrC;

impl StrC {
    pub fn i(value: &str) -> i64 {
        i64::from_str_radix(value, 10).unwrap_or_default()
    }

    pub fn i_d(value: &str, d: i64) -> i64 {
        i64::from_str_radix(value, 10).unwrap_or(d)
    }

    pub fn f(value: &str) -> f64 {
        f64::from_str(value).unwrap_or_default()
    }

    pub fn f_d(value: &str, d: f64) -> f64 {
        f64::from_str(value).unwrap_or(d)
    }
}
