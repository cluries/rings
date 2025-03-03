use lazy_static::lazy_static;
use tracing::error;
use std::fmt::Display;
use std::sync::RwLock;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    val: i64,
}

pub struct Factory {
    sharding: i64,
    sequence: RwLock<(i64, i64)>, // milli, sequence at milli
}

lazy_static! {
    static ref _shared_factory: Factory = Factory::new(0);
}

#[macro_export]
macro_rules! id {
    () => {
        shared().make().unwrap()
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

const MAX_SEQUENCE: i64 = 999;
const MAX_SHARDING: i64 = 99;
const MILLIS_BASE: i64 = 1000000;
const SEQUENCE_BASE: i64 = 100;
const SECOND_DIV: i64 = SEQUENCE_BASE * 100;
const MIN_VALUE: i64 = 1728747205481002100;

pub fn shared() -> &'static Factory {
    &_shared_factory
}

impl Factory {
    pub fn new(sharding: i64) -> Factory {
        Factory {
            sharding: sharding % MAX_SHARDING,
            sequence: RwLock::new((0, 0)),
        }
    }

    pub fn sharding(&self) -> i64 {
        self.sharding
    }

    pub fn sequence(&self) -> i64 {
        self.sequence.read().unwrap().1
    }

    pub fn millis(&self) -> i64 {
        self.sequence.read().unwrap().0
    }

    pub fn make(&self) -> Option<Id> {
        let millis = chrono::Local::now().timestamp_millis();
        let mut seq: i64 = 0;
        let mut sequence = self.sequence.write().unwrap();

        if millis != sequence.0 {
            *sequence = (millis, seq);
        } else {
            seq = sequence.1 + 1;
            sequence.1 = seq;
        }

        if seq > MAX_SEQUENCE {
            error!("out of sequence range");
            return None;
        }

        let val = MILLIS_BASE * millis + seq * SEQUENCE_BASE + self.sharding;
        Some(Id { val })
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        let val: i64 = value.parse().unwrap();
        val.into()
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        if value >= MIN_VALUE {
            Id { val: value }
        } else {
            panic!("less than min value")
        }
    }
}

impl From<Id> for i64 {
    fn from(value: Id) -> Self {
        value.val
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.val.to_string()
    }
}

impl Id {
    pub fn millis(self) -> i64 {
        self.val / MILLIS_BASE
    }

    pub fn second(self) -> i64 {
        self.val / SECOND_DIV
    }

    pub fn sharding(self) -> i64 {
        self.val % SEQUENCE_BASE
    }

    pub fn sequence(self) -> i64 {
        (self.val - self.val / MILLIS_BASE * MILLIS_BASE) / SEQUENCE_BASE
    }

    pub fn description(self) -> String {
        format!(
            "{} shard:{:02} seq:{:03} millis:{}",
            self.val.to_string(),
            self.sharding(),
            self.sequence(),
            self.millis()
        )
    }

    pub fn value(self) -> i64 {
        self.val
    }

    pub fn short(self) -> String {
        decimal_to_base62(self.val as u64)
    }

    pub fn from_short(short: String) -> Option<Self> {
        let val = base62_to_decimal(short.as_str());
        if val > i64::MAX as u64 || val < MIN_VALUE as u64 {
            return None;
        }
        Some(Id { val: val as i64 })
    }
}

const BASE62: u64 = 62;

const BASE62_CHARS: [u8; 62] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L',
    b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
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

fn decimal_to_base62(val: u64) -> String {
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

fn base62_to_decimal(base62: &str) -> u64 {
    let mut decimal = 0;
    let mut i = 0;
    for c in base62.chars().rev() {
        decimal += (BASE62_MAP[c as usize] as u64) * BASE62.pow(i);
        i += 1;
    }

    decimal
}
