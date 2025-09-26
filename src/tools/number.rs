// https://crates.io/crates/rust_decimal
// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub enum Number {
//     I8(i8),
//     I16(i16),
//     I32(i32),
//     I64(i64),
//     I128(i128),
//     U8(u8),
//     U16(u16),
//     U32(u32),
//     U64(u64),
//     U128(u128),
//     F32(f32),
//     F64(f64),
// }

pub mod conv {

    use std::str::FromStr;

    pub fn bool(value: &str) -> bool {
        // match value.to_lowercase().as_str() {
        //     "true" | "1" | "yes" | "on" => true,
        //     _ => false,
        // }

        matches!(value.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
    }

    pub fn boold(value: &str, d: bool) -> bool {
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => d,
        }
    }

    pub fn int(value: &str) -> i64 {
        // i64::from_str_radix(value, 10).unwrap_or_default()
        value.parse::<i64>().unwrap_or_default()
    }

    pub fn intd(value: &str, d: i64) -> i64 {
        // i64::from_str_radix(value, 10).unwrap_or(d)
        value.parse::<i64>().unwrap_or(d)
    }

    pub fn float(value: &str) -> f64 {
        f64::from_str(value).unwrap_or_default()
    }

    pub fn floatd(value: &str, d: f64) -> f64 {
        f64::from_str(value).unwrap_or(d)
    }

    pub fn hex_to_int(value: &str) -> i64 {
        let clean_value = value.trim_start_matches("0x").trim_start_matches("0X");
        i64::from_str_radix(clean_value, 16).unwrap_or_default()
    }

    pub fn oct_to_int(value: &str) -> i64 {
        let clean_value = value.trim_start_matches("0o").trim_start_matches("0O");
        i64::from_str_radix(clean_value, 8).unwrap_or_default()
    }

    pub fn bin_to_int(value: &str) -> i64 {
        let clean_value = value.trim_start_matches("0b").trim_start_matches("0B");
        i64::from_str_radix(clean_value, 2).unwrap_or_default()
    }

    pub fn int_to_hex(value: i64) -> String {
        format!("{:X}", value)
    }

    pub fn int_to_oct(value: i64) -> String {
        format!("{:o}", value)
    }

    pub fn int_to_bin(value: i64) -> String {
        format!("{:b}", value)
    }
}
