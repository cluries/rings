/*
在不同时区，同一时刻获取的 Unix 时间戳（timestamp）是相同的。
Unix 时间戳是以 UTC（协调世界时）为基准，记录从 1970 年 1 月 1 日 00:00:00 UTC 开始的秒数（或毫秒数）。
它与时区无关，表示的是一个绝对的时刻。

时间戳的本质：时间戳是一个整数，表示某个具体时刻与 Unix 纪元（1970-01-01 00:00:00 UTC）的时间差。
它不包含时区信息。例如，2025 年 8 月 30 日 04:04:00 PDT（太平洋夏令时）与同一时刻的北京时间（CST，2025-08-30 20:04:00）对应的时间戳是相同的。

时区只影响时间的显示格式，例如在不同时区下，同一时间戳会被转换为当地时间（如 PDT 显示为 04:04，CST 显示为 20:04）。但底层时间戳值保持一致。
*/

use chrono::TimeZone;

use crate::erx;

pub struct Now;
pub struct Is;

/// Yearmonth
///  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Yearmonth {
    pub year: i32,
    pub month: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    pub nanos: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Date,                 // %Y-%m-%d
    Time,                 // %H:%M:%S
    DateTime,             // %Y-%m-%d %H:%M:%S
    DatetimeWithTimeZone, // %Y-%m-%d %H:%M:%S %Z
}

pub const FORMAT_DATE: &'static str = "%Y-%m-%d";
pub const FORMAT_TIME: &'static str = "%H:%M:%S";
pub const FORMAT_DATETIME: &'static str = "%Y-%m-%d %H:%M:%S";
pub const FORMAT_DATETIME_WITH_TIMEZONE: &'static str = "%Y-%m-%d %H:%M:%S %Z";

impl Format {
    pub fn layout(&self) -> &'static str {
        match self {
            Format::Date => FORMAT_DATE,
            Format::Time => FORMAT_TIME,
            Format::DateTime => FORMAT_DATETIME,
            Format::DatetimeWithTimeZone => FORMAT_DATETIME_WITH_TIMEZONE,
        }
    }
}

// impl Into<&'static str> for Format {
//     fn into(self) -> &'static str {
//         self.layout()
//     }
// }

impl From<Format> for &'static str {
    fn from(f: Format) -> Self {
        f.layout()
    }
}

impl Format {
    //utc timestamp nanos to datetime string
    pub fn format(&self, timestamp_nanos: i64) -> String {
        let datetime = chrono::Utc.timestamp_nanos(timestamp_nanos);
        datetime.format(self.layout()).to_string()
    }

    // parse datetime string to utc timestamp nanos
    pub fn parse(&self, datetime: &str) -> i64 {
        let datetime = chrono::DateTime::parse_from_str(datetime, self.layout());
        match datetime {
            Ok(datetime) => datetime.timestamp(),
            Err(_) => 0,
        }
    }

    pub fn parse_to_utc(&self, datetime: &str) -> erx::ResultBoxedE<chrono::DateTime<chrono::Utc>> {
        let datetime = chrono::DateTime::parse_from_str(datetime, self.layout());
        match datetime {
            Ok(datetime) => Ok(datetime.with_timezone(&chrono::Utc)),
            Err(e) => Err(erx::Erx::boxed(e.to_string().as_str())),
        }
    }

    pub fn parse_to_local(&self, datetime: &str) -> erx::ResultBoxedE<chrono::DateTime<chrono::Local>> {
        let datetime = chrono::DateTime::parse_from_str(datetime, self.layout());
        match datetime {
            Ok(datetime) => Ok(datetime.with_timezone(&chrono::Local)),
            Err(e) => Err(erx::Erx::boxed(e.to_string().as_str())),
        }
    }
}

impl Now {
    /// current timestamp
    pub fn timestamp() -> Timestamp {
        Timestamp { nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default() }
    }

    /// utc datetime
    pub fn utc() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    /// local datetime
    pub fn local() -> chrono::DateTime<chrono::Local> {
        chrono::Local::now()
    }

    /// fixed timezone datetime
    ///# Arguments
    ///* `z` - The timezone offset, range -12 to 12
    ///# Returns
    ///* `chrono::DateTime<chrono::FixedOffset>` - The fixed timezone datetime
    pub fn fixed(z: i32) -> chrono::DateTime<chrono::FixedOffset> {
        let offset_seconds = z.clamp(-12, 12) * 3600; // 将时区转换为秒数，并限制在-12到12之间
        let zone = if offset_seconds >= 0 {
            chrono::FixedOffset::east_opt(offset_seconds).unwrap()
        } else {
            chrono::FixedOffset::west_opt(-offset_seconds).unwrap()
        };
        chrono::Utc::now().with_timezone(&zone)
    }
}

impl Yearmonth {
    pub fn new(year: i32, month: i32) -> Self {
        Self { year, month }
    }

    /// get days of month
    pub fn month_days(&self) -> i32 {
        let year = self.year;
        let month = self.month;
        if Is::leap(year) {
            if month == 2 {
                29
            } else if month == 4 || month == 6 || month == 9 || month == 11 {
                30
            } else {
                31
            }
        } else if month == 2 {
            28
        } else if month == 4 || month == 6 || month == 9 || month == 11 {
            30
        } else {
            31
        }
    }

    /// get days of year
    pub fn year_days(&self) -> i32 {
        let year = self.year;
        if Is::leap(year) {
            366
        } else {
            365
        }
    }
}

impl std::fmt::Display for Yearmonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

/// Timestamp
///
///
/// # Fields
///
/// * `nanos` - The nanos.          
///     
/// # Methods
///     
/// * `with_now` - with now to timestamp.
/// * `with_nanos` - with nanos to timestamp.
/// * `with_micros` - with micros to timestamp.
/// * `with_millis` - with millis to timestamp.
/// * `with_seconds` - with seconds to timestamp.
/// * `micros` - to micros.
/// * `millis` - to millis.
/// * `seconds` - to seconds.
/// * `date_utc` - to utc datetime.
/// * `date_local` - to local datetime.
///     
impl Timestamp {
    /// with now to timestamp
    ///
    /// # Returns
    ///
    /// * `Timestamp` - The timestamp.
    pub fn with_now() -> Self {
        Self { nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default() }
    }

    /// with nanos to timestamp
    ///
    /// # Arguments
    ///
    /// * `nanos` - The nanos.
    ///
    /// # Returns           
    ///
    /// * `Timestamp` - The timestamp.
    pub fn with_nanos(nanos: i64) -> Self {
        Self { nanos }
    }

    /// with micros to timestamp
    ///       
    /// # Arguments
    ///
    /// * `micros` - The micros.
    ///
    /// # Returns
    ///
    /// * `Timestamp` - The timestamp.
    pub fn with_micros(micros: i64) -> Self {
        Self { nanos: micros * 1000 }
    }

    /// with millis to timestamp
    ///
    /// # Arguments
    ///
    /// * `millis` - The millis.
    ///
    /// # Returns
    ///
    /// * `Timestamp` - The timestamp.
    pub fn with_millis(millis: i64) -> Self {
        Self { nanos: millis * 1000 * 1000 }
    }

    /// with seconds to timestamp
    ///
    /// # Arguments
    ///
    /// * `seconds` - The seconds.
    ///
    /// # Returns
    ///
    /// * `Timestamp` - The timestamp.
    pub fn with_seconds(seconds: i64) -> Self {
        Self { nanos: seconds * 1000 * 1000 * 1000 }
    }

    /// to micros
    pub fn micros(&self) -> i64 {
        self.nanos / 1000
    }

    /// to millis
    pub fn millis(&self) -> i64 {
        self.nanos / (1000 * 1000)
    }

    /// to seconds
    pub fn seconds(&self) -> i64 {
        self.nanos / (1000 * 1000 * 1000)
    }

    /// to utc datetime
    pub fn date_utc(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.timestamp_nanos(self.nanos)
    }

    /// to local datetime
    ///
    pub fn date_local(&self) -> chrono::DateTime<chrono::Local> {
        chrono::Local.timestamp_nanos(self.nanos)
    }
}

impl Is {
    pub fn leap(year: i32) -> bool {
        // 判断是否为闰年:
        // 1. 能被4整除但不能被100整除
        // 2. 能被400整除
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    pub fn valid_date(year: i32, month: u32, day: u32) -> bool {
        chrono::NaiveDate::from_ymd_opt(year, month, day).is_some()
    }

    pub fn valid_time(hour: u32, minute: u32, second: u32) -> bool {
        chrono::NaiveTime::from_hms_opt(hour, minute, second).is_some()
    }

    pub fn valid_datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> bool {
        chrono::NaiveDate::from_ymd_opt(year, month, day).and_then(|date| date.and_hms_opt(hour, minute, second)).is_some()
    }

    pub fn valid_yearmonth(year: i32, month: i32) -> bool {
        year >= 0 && (1..=12).contains(&month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_datetime() {
        println!("{} {} {}", Now::fixed(1).to_utc(), Now::fixed(2).to_utc(), Now::local().to_utc());
        println!("{} {} {}", Now::fixed(1).timestamp(), Now::fixed(2).timestamp(), Now::local().timestamp());
    }
}
