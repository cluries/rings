use hmac::{Hmac, Mac};
use sha1::{Digest, Sha1};

type HmacSha1 = Hmac<Sha1>;

pub fn hmac_sha1(c: &str, key: &str) -> String {
    let mut mac = HmacSha1::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(c.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn sha1(c: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(c.as_bytes());
    hex::encode(hasher.finalize())
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
        println!("{}", hmac_sha1("key", "value"));
    }

    #[test]
    fn test_ha1() {
        println!("{:?}", sha1("aaa"));
    }
    
    #[test]
    fn test_md5() {
        println!("{:?}", md5("key"));
    }
}
