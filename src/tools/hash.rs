use hmac::{Hmac, Mac};
use sha1::{Digest, Sha1};
use sha2::Sha256;

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;

pub fn hmac_sha1(c: &str, key: &str) -> Result<String, String> {
    match HmacSha1::new_from_slice(key.as_bytes()) {
        Ok(mut mac) => {
            mac.update(c.as_bytes());
            Ok(hex::encode(mac.finalize().into_bytes()))
        },
        Err(e) => Err(format!("Failed to create HMAC-SHA1: {}", e)),
    }
}

pub fn sha1(c: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(c.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn sha256(c: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(c.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hmac_sha256(c: &str, key: &str) -> Result<String, String> {
    match HmacSha256::new_from_slice(key.as_bytes()) {
        Ok(mut mac) => {
            mac.update(c.as_bytes());
            Ok(hex::encode(mac.finalize().into_bytes()))
        },
        Err(e) => Err(format!("Failed to create HMAC-SHA256: {}", e)),
    }
}

pub fn md5(c: &str) -> String {
    use md5 as md5lib;
    let digest = md5lib::compute(c.as_bytes());
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha1() {
        match hmac_sha1("key", "value") {
            Ok(hash) => println!("{}", hash),
            Err(e) => println!("Error: {}", e),
        }
    }

    #[test]
    fn test_ha1() {
        println!("{:?}", sha1("aaa"));
    }

    #[test]
    fn test_sha256() {
        println!("{}", sha256("hello world"));
    }

    #[test]
    fn test_hmac_sha256() {
        match hmac_sha256("hello world", "secret") {
            Ok(hash) => println!("{}", hash),
            Err(e) => println!("Error: {}", e),
        }
    }

    #[test]
    fn test_md5() {
        println!("{:?}", md5("key"));
    }
}
