/// generate id
/// format:
use crate::erx;


use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::atomic::{AtomicI64, Ordering};

/// id
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Id {
    val: i64,
}

/// id factory
pub struct Factory {
    name: String,
    sharding: i64,
    millis: AtomicI64,
    sequence: AtomicI64, // milli, sequence at milli
}

lazy_static! {
    static ref _shared_factory: Factory = Factory::new("SHARED", 0);
}

/// generate id
/// actually, it is call shared().make()
#[macro_export]
macro_rules! id {
    () => {
        shared().make()
    };
}

// #[macro_export]
// macro_rules! factory {
//     () => {
//
//         Factory::new(rand::random::<i64>());
//     };
//     ($x:expr) => {
//         if let Ok(n) = $x.parse::<i64>() {
//             Factory::new(n)
//         } else {
//             panic!("The argument provided is not a valid number.");
//         }
//     };
// }

/// max sequence
const MAX_SEQUENCE: i64 = 9999;

/// max sharding
const MAX_SHARDING: i64 = 99;

/// millis base
const MILLIS_BASE: i64 = 10_000_000;

/// sequence base
const SEQUENCE_BASE: i64 = 100;

/// maybe min id value
const MIN_VALUE: i64 = 1_000_000_000_000_000_000;

/// gets shared id factory
pub fn shared() -> &'static Factory {
    &_shared_factory
}

struct ShorterMills {
    mills: i64,
    shorter: i64,
}

impl ShorterMills {
    const START: i64 = 1650;
    const DIVBASE: i64 = 1_000_000_000;

    pub fn with_mills(mills: i64) -> ShorterMills {
        let angel = mills / Self::DIVBASE - Self::START;
        if angel > 999 {
            panic!("mills out of range");
        }

        let shorter = angel * Self::DIVBASE + mills % Self::DIVBASE;
        ShorterMills { mills, shorter }
    }

    pub fn with_shorter(shorter: i64) -> ShorterMills {
        let angel = shorter / Self::DIVBASE;
        if angel > 999 {
            panic!("Shorter mills too high");
        }

        let mills = (angel + Self::START) * Self::DIVBASE + shorter % Self::DIVBASE;
        ShorterMills { mills, shorter }
    }

    pub fn mills(&self) -> i64 {
        self.mills
    }

    pub fn shorter(&self) -> i64 {
        self.shorter
    }
}

fn current_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

impl Factory {
    pub fn new(name: &str, sharding: i64) -> Factory {
        Factory { name: name.to_owned(), sharding: sharding % MAX_SHARDING, 
            millis: AtomicI64::new(0),sequence: AtomicI64::new(0) }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sharding(&self) -> i64 {
        self.sharding
    }

    pub fn sequence(&self) -> i64 {
        self.sequence.load(Ordering::Relaxed)
    }

    pub fn millis(&self) -> i64 {
        self.sequence.load(Ordering::Relaxed)
    }

    pub fn make(&self) -> erx::ResultE<Id> {
        let millis = current_millis();
        let old_millis = self.millis.load(Ordering::Relaxed);

        if millis != old_millis {
            match self.millis.compare_exchange(old_millis, millis, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_old) => {
                    self.sequence.store(0, Ordering::Release);
                }
                Err(_current) => {
                    // Another thread updated millis, reload and continue
                }
            }
        }
 
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        if seq > MAX_SEQUENCE {
            return Err("beyond sequence limits".into());
        }

        let shorter = ShorterMills::with_mills(millis).shorter();
        let val = MILLIS_BASE * shorter + seq * SEQUENCE_BASE + self.sharding;

        Ok(Id { val })
    }

    pub fn make_n(&self, n: u16) -> erx::ResultE<Vec<Id>> {
        if n < 1 {
            return Ok(vec![]);
        }

        let millis = current_millis();
         

        let old_millis = self.millis.load(Ordering::Relaxed);

        if millis != old_millis {
            match self.millis.compare_exchange(old_millis, millis, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_old) => {
                    self.sequence.store(0, Ordering::Release);
                }
                Err(_current) => {
                    // Another thread updated millis, reload and continue
                }
            }
        }

        let seq = self.sequence.load(Ordering::Relaxed);
        let n = i64::from(n);
        if seq + n > MAX_SEQUENCE {
            return Err("beyond sequence limits".into());
        }

        self.sequence.fetch_add(n, Ordering::AcqRel);

        let mut ids: Vec<Id> = Vec::new();
        let shorter = ShorterMills::with_mills(millis).shorter();
        for i in seq..seq+n {
            let val = MILLIS_BASE * shorter + i * SEQUENCE_BASE + self.sharding;
            ids.push(Id { val });
        }

        Ok(ids)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        let val: i64 = value.parse().unwrap_or(0);
        val.into()
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        if value >= MIN_VALUE { Id { val: value } } else { panic!("less than min value") }
    }
}

impl Into<i64> for Id {
    fn into(self) -> i64 {
        self.val
    }
}

impl Into<String> for Id {
    fn into(self) -> String {
        self.val.to_string()
    }
}

impl Id {
    pub fn new(val: i64) -> Option<Self> {
        if val < MIN_VALUE {
            return None;
        }

        Some(Self { val })
    }

    pub fn from_short(short: String) -> Option<Self> {
        let val = base62_to_decimal(short.as_str());
        if val < MIN_VALUE {
            return None;
        }
        Some(Id { val })
    }

    pub fn millis(self) -> i64 {
        ShorterMills::with_shorter(self.val / MILLIS_BASE).mills()
    }

    pub fn second(self) -> i64 {
        self.millis() / 1_000
    }

    pub fn sharding(self) -> i64 {
        self.val % 1_00
    }

    pub fn sequence(self) -> i64 {
        (self.val - self.val / MILLIS_BASE * MILLIS_BASE) / SEQUENCE_BASE
    }

    pub fn description(self) -> String {
        format!(
            "{} shard:{:02} seq:{:03} millis:{} second:{}",
            self.val.to_string(),
            self.sharding(),
            self.sequence(),
            self.millis(),
            self.second()
        )
    }

    pub fn value(self) -> i64 {
        self.val
    }

    pub fn short(self) -> String {
        decimal_to_base62(self.val)
    }
}

const BASE62: i64 = 62;

const BASE62_CHARS: [u8; 62] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l',
    b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H',
    b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
];

const BASE62_MAP: [u8; 128] = {
    let mut map = [255; 128];
    let mut i = 0;
    while i < BASE62_CHARS.len() {
        map[BASE62_CHARS[i] as usize] = i as u8;
        i += 1;
    }
    map
};

fn decimal_to_base62(val: i64) -> String {
    if val == 0 {
        return "0".to_string();
    }

    let mut num = val;
    let mut result = Vec::new();
    while num > 0 {
        result.push(BASE62_CHARS[(num % BASE62) as usize]);
        num /= 62;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

fn base62_to_decimal(base62: &str) -> i64 {
    let mut decimal = 0;
    let mut i = 0;
    for c in base62.chars().rev() {
        decimal += (BASE62_MAP[c as usize] as i64) * BASE62.pow(i);
        i += 1;
    }

    decimal
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[test]
    fn test_id() {
        let mut v = Vec::new();
        for i in 1..9999 {
            v.push(id!().unwrap());
        }
        for id in v.iter() {
            println!("{}", id.description());
        }
    }

    #[test]
    fn test_id_n() {
        let mut v = Vec::new();
        for i in 1..5 {
            v.push(shared().make_n(1299).unwrap())
        }

        for list in v.iter() {
            for id in list {
                println!("{}", id.description());
            }
        }
    }

    #[test]
    fn test_try() {
        println!("{}", i64::MAX);

        println!("{}", Id::from(i64::MAX).description());

        for i in 4..9 {
            println!("{}", id!().unwrap().description());
        }
    }

    #[test]
    fn test_12() {
        let i = |mills| {
            let s = ShorterMills::with_mills(mills);
            assert_eq!(s.shorter().to_string().len(), 12);
            assert_eq!(mills, ShorterMills::with_shorter(s.shorter()).mills());
        };

        i(current_millis());
        i(current_millis() + 120312412);
    }
}
