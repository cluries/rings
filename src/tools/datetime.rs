pub struct Now;

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


impl Now {
    pub fn timestamp() -> i64 {
        chrono::Utc::now().timestamp()
    }

    pub fn timestamp_nanos() -> i64 {
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    }

    pub fn timestamp_milliseconds() -> i64 {
        chrono::Utc::now().timestamp_millis()
    }

    pub fn timestamp_microseconds() -> i64 {
        chrono::Utc::now().timestamp_micros()
    }

    pub fn local_date_string() -> String {
        chrono::Local::now().format(Format::Date.into()).to_string()
    }

    pub fn local_time_string() -> String {
        chrono::Local::now().format(Format::Time.into()).to_string()
    }

    pub fn local_datetime_string() -> String {
        chrono::Local::now().format(Format::DateTime.into()).to_string()
    }

    pub fn local_datetime_with_zone_string() -> String {
        chrono::Local::now().format(Format::DatetimeWithTimeZone.into()).to_string()
    }
}


#[cfg(test)]
#[test]
fn test_local_datetime() {
    println!("{}", Now::local_datetime_with_zone_string());
}


