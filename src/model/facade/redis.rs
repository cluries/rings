use crate::erx;
use redis::Commands;
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

pub type FacadeResult<T> = erx::ResultE<T>;
pub type BoolResult = FacadeResult<bool>;
pub type FloatResult = FacadeResult<f64>;
pub type IntegerResult = FacadeResult<i64>;

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

    redis_c!(exists, (key: &str), BoolResult);
    redis_c!(ttl, (key: &str), FacadeResult<i64>);
    redis_c!(del, (key: &str), BoolResult);
    redis_c!(persist, (key: &str), BoolResult);
    redis_c!(expire, (key: &str, seconds: i64), BoolResult);
    redis_c!(expire_at, (key: &str, expire_at: i64), BoolResult);
    redis_c!(rename, (key: K, nkey: N), BoolResult, generics: [K: redis::ToRedisArgs, N: redis::ToRedisArgs]);
    redis_c!(rename_nx, (key: K, nkey: N), BoolResult, generics: [K: redis::ToRedisArgs, N: redis::ToRedisArgs]);

    redis_c!(get, (key: &str), FacadeResult<RV>, generics: [RV: redis::FromRedisValue]);
    redis_c!(getset, (key: &str, val: V), FacadeResult<V>, generics: [V: redis::ToRedisArgs + redis::FromRedisValue]);
    redis_c!(getdel, redis: get_del, (key: &str), FacadeResult<RV>, generics: [RV: redis::FromRedisValue]);
    redis_c!(hget, (key: &str, field: F), FacadeResult<RV>, generics: [F: redis::ToRedisArgs, RV: redis::FromRedisValue]);
    redis_c!(hgetall, (key: &str), FacadeResult<RV>, generics: [RV: redis::FromRedisValue]);

    redis_c!(append, (key: &str, val: V), BoolResult, generics: [V: redis::ToRedisArgs]);

    redis_c!(set, (key: &str, val: T), BoolResult, generics: [T: redis::ToRedisArgs]);
    redis_c!(set_ex, (key: &str, val: T, ttl: u64), BoolResult, generics: [T: redis::ToRedisArgs]);
    redis_c!(set_nx, (key: &str, val: T), BoolResult, generics: [T: redis::ToRedisArgs]);

    redis_c!(hset, (key: &str, field: F, val: V), BoolResult, generics: [F: redis::ToRedisArgs, V: redis::ToRedisArgs]);
    redis_c!(hset_nx, (key: &str, field: F, val: V), BoolResult, generics: [F: redis::ToRedisArgs, V: redis::ToRedisArgs]);
    redis_c!(hset_multiple, (key: &str, values: &[(F, V)]), BoolResult, generics: [F: redis::ToRedisArgs, V: redis::ToRedisArgs]);
    redis_c!(hdel, (key: &str, field: F), BoolResult, generics: [F: redis::ToRedisArgs]);
    redis_c!(hlen, (key: &str), FacadeResult<RV>, generics: [RV: redis::FromRedisValue]);
    redis_c!(hkeys, (key: &str), FacadeResult<T>, generics: [T: redis::FromRedisValue]);
    redis_c!(hvals, (key: &str), FacadeResult<T>, generics: [T: redis::FromRedisValue]);

    redis_c!(incr, (key: &str, val: T), FacadeResult<T>, generics: [T: redis::ToRedisArgs + redis::FromRedisValue]);
    redis_c!(decr, (key: &str, val: T), FacadeResult<T>, generics: [T: redis::ToRedisArgs + redis::FromRedisValue]);

    redis_c!(mset, (items: &[(K, V)]), BoolResult, generics: [K: redis::ToRedisArgs, V: redis::ToRedisArgs]);
    redis_c!(set_multiple, redis: mset, (items: &[(K, V)]), BoolResult, generics: [K: redis::ToRedisArgs, V: redis::ToRedisArgs]);

    /// Set the value of one or more fields of a given hash key, and optionally set their expiration
    pub fn hset_ex<F: redis::ToRedisArgs, V: redis::ToRedisArgs>(&self, key: &str, ttl: u64, values: &[(F, V)]) -> BoolResult {
        let exo = redis::HashFieldExpirationOptions::default();
        exo.set_expiration(redis::SetExpiry::EX(ttl));
        self.get_connection()?.hset_ex(key, &exo, values).map_err(erx::smp)
    }

    /// Get the value of a key and set expiration
    pub fn get_ex<RV: redis::FromRedisValue>(&self, key: &str, expire_at: u64) -> FacadeResult<RV> {
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

    impl redis::ToRedisArgs for Name {
        fn write_redis_args<W>(&self, out: &mut W)
        where
            W: ?Sized + RedisWrite,
        {
            out.write_arg(serde_json::to_vec(self).unwrap().as_slice());
        }
    }

    impl redis::FromRedisValue for Name {
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
