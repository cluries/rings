use hmac::{Hmac, Mac};
use sha1::{Sha1, Digest};
 
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



#[test]
fn test_hmac_sha1() {
    println!("{}", hmac_sha1("key", "value"));
}

#[test]
fn test_ha1() {
    println!("{:?}", sha1("aaa"));
}



