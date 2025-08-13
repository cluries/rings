use chrono::Datelike;
use rand::prelude::ThreadRng;
use rand::Rng;


pub fn rand_bool() -> bool {
    use rand::Rng;
    let mut rng = rand::rng();
    rng.random()
}

pub fn rand_i64(min: i64, max: i64) -> i64 {
    use rand::Rng;
    let mut rng = rand::rng();
    rng.random_range(min..max)
}

pub fn rand_f64(min: f64, max: f64) -> f64 {
    use rand::Rng;
    let mut rng = rand::rng();
    rng.random_range(min..max)
}

pub fn rand_str(len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut s = String::new();
    for _ in 0..len {
        s.push(rng.random_range('a'..='z'));
    }
    s
}

pub fn rand_date() -> String {
    let mut rng = rand::rng();
    let (year, month, day) = _rand_date_numbers(&mut rng);
    format!("{}-{}-{}", year, month, day)
}

pub fn rand_datetime() -> String {
    let mut rng = rand::rng();
    let (year, month, day) = _rand_date_numbers(&mut rng);
    let hour = rng.random_range(0..24);
    let minute = rng.random_range(0..60);
    let second = rng.random_range(0..60);
    format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second)
}

fn _rand_date_numbers(rng: &mut ThreadRng) -> (i32, i32, i32) {
    let year_end = chrono::Utc::now().year();
    let year = rng.random_range(1970..year_end);
    let month = rng.random_range(1..13);
    let day = if month == 2 {
        rng.random_range(1..29)
    } else if month == 4 || month == 6 || month == 9 || month == 11 {
        rng.random_range(1..31)
    } else {
        rng.random_range(1..32)
    };

    (year, month, day)
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;

    const LOOP: i32 = 1000;

    #[test]
    fn test_rand_bool() {
        for _ in 0..LOOP {
            let b = rand_bool();
            assert!(b == true || b == false);
        }
        println!("test_rand_bool passed")
    }

    #[test]
    fn test_rand_i64() {
        for _ in 0..LOOP {
            let i = rand_i64(0, 100);
            assert!(i >= 0 && i < 100);
        }
        println!("test_rand_i64 passed")
    }

    #[test]
    fn test_rand_f64() {
        for _ in 0..LOOP {
            let f = rand_f64(0.0, 1.0);
            assert!(f >= 0.0 && f < 1.0);
        }
        println!("test_rand_f64 passed")
    }

    #[test]
    fn test_rand_str() {
        for _ in 0..LOOP {
            let s = rand_str(10);
            assert_eq!(s.len(), 10);
        }
        println!("test_rand_str passed")
    }

    #[test]
    fn test_rand_date() {
        println!("{} - {}", rand_date(), rand_datetime())
    }
}
