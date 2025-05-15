use crate::erx;
use redis::{Commands, FromRedisValue, ToRedisArgs};
use std::fmt::Display;

pub struct Redis {
    logit: bool,
    client: redis::Client,
}

macro_rules! conn_mut {
    ($i:expr, $s:expr) => {
        match $i.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                if $i.logit {
                    tracing::error!("RedisClient::get_connection {}", e);
                }
                return $s;
            },
        }
    };
}

#[allow(unused)]
fn unwra<T, E>(logit: bool, value: T) -> impl FnOnce(E) -> T
where
    E: Display,
{
    move |e: E| {
        if logit {
            tracing::error!("Redis error: {}", e);
        }
        value
    }
}

pub type Facade<T> = erx::ResultE<T>;
pub type FacadeBool = Facade<bool>;
pub type FacadeFloat = Facade<f64>;
pub type FacadeInt = Facade<i64>;

macro_rules! redis_c {
    // 基本形式：方法名、参数列表、返回类型（默认调用参数与参数名一致）
    ($method_name:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
        pub fn $method_name(&self, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$method_name($($arg_name),*).map_err(erx::smp)
        }
    };

    // 支持显式指定 Redis 方法名
    ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
        pub fn $method_name(&self, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$redis_method($($arg_name),*).map_err(erx::smp)
        }
    };

    // 支持泛型参数的方法
    ($method_name:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
        pub fn $method_name<$($generic)*>(&self, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$method_name($($arg_name),*).map_err(erx::smp)
        }
    };

    // 支持泛型参数且显式指定 Redis 方法名
    ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
        pub fn $method_name<$($generic)*>(&self, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$redis_method($($arg_name),*).map_err(erx::smp)
        }
    };
}

impl Redis {
    pub fn shared() -> Self {
        Redis { logit: true, client: crate::model::make_redis_client().unwrap() }
    }

    pub fn new(c: redis::Client) -> Self {
        Redis { logit: true, client: c }
    }

    pub fn get_connection(&self) -> erx::ResultE<redis::Connection> {
        self.client.get_connection().map_err(erx::smp)
    }

    redis_c!(exists, (key: &str), FacadeBool);
    redis_c!(ttl, (key: &str), FacadeInt);
    redis_c!(del, (key: &str), FacadeBool);
    redis_c!(persist, (key: &str), FacadeBool);
    redis_c!(expire, (key: &str, seconds: i64), FacadeBool);
    redis_c!(expire_at, (key: &str, expire_at: i64), FacadeBool);
    redis_c!(expire_time,  (key: &str), FacadeInt);
    redis_c!(rename, (key: K, nkey: N), FacadeBool, generics: [K: ToRedisArgs, N: ToRedisArgs]);
    redis_c!(rename_nx, (key: K, nkey: N), FacadeBool, generics: [K: ToRedisArgs, N: ToRedisArgs]);

    redis_c!(get, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(getset, (key: &str, val: V), Facade<V>, generics: [V: ToRedisArgs + FromRedisValue]);
    redis_c!(getdel, redis: get_del, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(hget, (key: &str, field: F), Facade<RV>, generics: [F: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(hgetall, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);

    redis_c!(append, (key: &str, val: V), FacadeBool, generics: [V: ToRedisArgs]);

    redis_c!(set, (key: &str, val: T), FacadeBool, generics: [T: ToRedisArgs]);
    redis_c!(set_ex, (key: &str, val: T, seconds: u64), FacadeBool, generics: [T: ToRedisArgs]);
    redis_c!(set_nx, (key: &str, val: T), FacadeBool, generics: [T: ToRedisArgs]);

    redis_c!(hexists, (key: &str,field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(httl, (key: &str,field: F), FacadeInt, generics: [F: ToRedisArgs]);
    redis_c!(hset, (key: &str, field: F, val: V), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(hset_nx, (key: &str, field: F, val: V), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(hset_multiple, (key: &str, values: &[(F, V)]), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(hdel, (key: &str, field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(hlen, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(hkeys, (key: &str), Facade<T>, generics: [T: FromRedisValue]);
    redis_c!(hvals, (key: &str), Facade<T>, generics: [T: FromRedisValue]);

    redis_c!(incr, (key: &str, delta: D), Facade<D>, generics: [D: ToRedisArgs + FromRedisValue]);
    redis_c!(decr, (key: &str, delta: D), Facade<D>, generics: [D: ToRedisArgs + FromRedisValue]);

    redis_c!(mset, (items: &[(K, V)]), FacadeBool, generics: [K: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(set_multiple, redis: mset, (items: &[(K, V)]), FacadeBool, generics: [K: ToRedisArgs, V: ToRedisArgs]);

    redis_c!(getbit, (key: &str, offset:usize), FacadeBool);
    redis_c!(bitcount, (key: &str), FacadeInt);
    redis_c!(bitcount_range, (key: &str, start:usize, end:usize), FacadeInt);
    redis_c!(setbit, (key: &str, offset:usize, value:bool), FacadeBool);

    redis_c!(strlen, (key: &str), FacadeInt);

    redis_c!(sadd, (key: &str, member: M), FacadeBool, generics: [M: ToRedisArgs]);
    redis_c!(scard, (key: &str), FacadeInt);
    redis_c!(smembers, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(srandmember, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(srandmember_multiple, (key: &str, count: usize), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(srem, (key: &str, member: M), FacadeBool, generics: [M: ToRedisArgs]);


    /// Set the value of one or more fields of a given hash key, and optionally set their expiration
    pub fn hset_ex<F: ToRedisArgs, V: ToRedisArgs>(&self, key: &str, ttl: u64, values: &[(F, V)]) -> FacadeBool {
        let exo = redis::HashFieldExpirationOptions::default();
        exo.set_expiration(redis::SetExpiry::EX(ttl));
        self.get_connection()?.hset_ex(key, &exo, values).map_err(erx::smp)
    }

    /// Get the value of a key and set expiration
    pub fn get_ex<RV: FromRedisValue>(&self, key: &str, expire_at: u64) -> Facade<RV> {
        self.get_connection()?.get_ex(key, redis::Expiry::EX(expire_at)).map_err(erx::smp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis::{RedisError, RedisResult, RedisWrite, Value};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Name {
        key: String,
        first: String,
        middle: String,
        last: String,
    }

    impl ToRedisArgs for Name {
        fn write_redis_args<W>(&self, out: &mut W)
        where
            W: ?Sized + RedisWrite,
        {
            out.write_arg(serde_json::to_vec(self).unwrap().as_slice());
        }
    }

    impl FromRedisValue for Name {
        fn from_redis_value(v: &Value) -> RedisResult<Self> {
            match v {
                Value::BulkString(d) => Ok(serde_json::from_slice(d)?),
                Value::Array(d) => {
                    println!("----- {:?}", d);
                    let e = RedisError::from((redis::ErrorKind::TypeError, "invalid type"));
                    Err(e)
                },
                _ => {
                    println!("==== {:?}", v);
                    let e = RedisError::from((redis::ErrorKind::TypeError, "invalid type"));
                    Err(e)
                },
            }
        }
    }

    fn name_it() -> Name {
        Name { key: "LJ".to_string(), first: "luo".to_string(), middle: "-".to_string(), last: "jing".to_string() }
    }

    #[test]
    fn test_redis_value() {
        let c = rds();
        println!("{:?}", c.set_ex("test_ttl", 1024, 10));
        println!("{:?}", c.ttl("test_ttl"));
        println!("{:?}", c.incr("test_ttl", 24));
        println!("{:?}", c.set::<Name>("LJ", name_it()));
        println!("{:?}", c.get::<Name>("LJ"));

        println!("hset {:?}", c.hset("LJHash", "d1age", 102410));
        println!("hset_multiple {:?}", c.hset_multiple("LJHash", &[("age", 1811), ("lastage", 24)]));
    }

    fn rds() -> Redis {
        Redis::new(redis::Client::open("redis://127.0.0.1").unwrap())
    }
}
