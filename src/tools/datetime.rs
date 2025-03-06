use chrono::TimeZone;

use crate::erx;

pub struct Now;
pub struct Is;

pub struct Timestamp {
    pub nanos: i64,
}

pub enum Format {
    Date,
    Time,
    DateTime,
    DatetimeWithTimeZone,
}

pub static MONTHS: [&'static str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub static MONTHS_LONG: [&'static str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December",
];

pub static WEEKDAYS: [&'static str; 7] = [
    "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun",
];

pub static WEEKDAYS_LONG: [&'static str; 7] = [
    "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
];


pub static FORMAT_DATE: &'static str = "%Y-%m-%d";
pub static FORMAT_TIME: &'static str = "%H:%M:%S";
pub static FORMAT_DATETIME: &'static str = "%Y-%m-%d %H:%M:%S";
pub static FORMAT_DATETIME_WITH_TIMEZONE: &'static str = "%Y-%m-%d %H:%M:%S %Z";


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


impl Timestamp {
    pub fn with(timestamp: i64) -> Self {
        // 根据timestamp的大小推断时间单位并转换为纳秒
        let nanos = if timestamp > 1_000_000_000_000_000_000 {
            // 已经是纳秒
            timestamp
        } else if timestamp > 1_000_000_000_000_000 {
            // 微秒
            timestamp * 1_000
        } else if timestamp > 1_000_000_000_000 {
            // 毫秒
            timestamp * 1_000_000
        } else {
            // 秒
            timestamp
        };

        Self {
            nanos,
        }
    }

    pub fn with_now() -> Self {
        Self {
            nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        }
    }

    // pub fn with_nanos(nanos: i64) -> Self {
    //     Self {
    //         nanos,
    //     }
    // }
    //
    // pub fn with_micros(micros: i64) -> Self {
    //     Self {
    //         nanos: micros * 1000,
    //     }
    // }
    //
    // pub fn with_millis(millis: i64) -> Self {
    //     Self {
    //         nanos: millis * 1000 * 1000,
    //     }
    // }
    //
    // pub fn with_seconds(seconds: i64) -> Self {
    //     Self {
    //         nanos: seconds * 1000 * 1000 * 1000,
    //     }
    // }

    pub fn micros(&self) -> i64 {
        self.nanos / 1000
    }

    pub fn millis(&self) -> i64 {
        self.nanos / 1000 / 1000
    }
    pub fn seconds(&self) -> i64 {
        self.nanos / 1000 / 1000 / 1000
    }

    pub fn date_utc(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.timestamp_nanos(self.nanos)
    }

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
#[test]
fn test_local_datetime() {
    // println!("{}", Now::local_datetime_with_zone_string());
}


