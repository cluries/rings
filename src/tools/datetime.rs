use chrono::TimeZone;

use crate::erx;

pub struct Now;
pub struct Is;

/// Yearmonth
///  
pub struct Yearmonth {
    pub year: i32,
    pub month: i32,
}

pub struct Timestamp {
    pub nanos: i64,
}

pub enum Format {
    Date,
    Time,
    DateTime,
    DatetimeWithTimeZone,
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

impl Into<&'static str> for Format {
    fn into(self) -> &'static str {
        self.layout()
    }
}

impl Into<String> for Format {
    fn into(self) -> String {
        self.layout().to_string()
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

    pub fn parse_to_utc(&self, datetime: &str) -> erx::ResultE<chrono::DateTime<chrono::Utc>> {
        let datetime = chrono::DateTime::parse_from_str(datetime, self.layout());
        match datetime {
            Ok(datetime) => Ok(datetime.with_timezone(&chrono::Utc)),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn parse_to_local(&self, datetime: &str) -> erx::ResultE<chrono::DateTime<chrono::Local>> {
        let datetime = chrono::DateTime::parse_from_str(datetime, self.layout());
        match datetime {
            Ok(datetime) => Ok(datetime.with_timezone(&chrono::Local)),
            Err(e) => Err(e.to_string().into()),
        }
    }
}

impl Now {
    pub fn timestamp() -> Timestamp {
        Timestamp { nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default() }
    }

    pub fn utc() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    pub fn local() -> chrono::DateTime<chrono::Local> {
        chrono::Local::now()
    }

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

    pub fn month_days(&self) -> i32 {
        let year = self.year;
        let month = self.month;
        let days = if Is::leap(year) {
            if month == 2 {
                29
            } else if month == 4 || month == 6 || month == 9 || month == 11 {
                30
            } else {
                31
            }
        } else {
            if month == 2 {
                28
            } else if month == 4 || month == 6 || month == 9 || month == 11 {
                30
            } else {
                31
            }
        };
        days
    }

    pub fn year_days(&self) -> i32 {
        let year = self.year;
        let days = if Is::leap(year) { 366 } else { 365 };
        days
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
        self.nanos / 1000 / 1000
    }

    /// to seconds
    pub fn seconds(&self) -> i64 {
        self.nanos / 1000 / 1000 / 1000
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
