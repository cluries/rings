use crate::erx;
use redis::{Commands, FromRedisValue, ToRedisArgs};
use std::fmt::Display;

#[allow(dead_code)]
pub struct Redis {
    logit: bool,
    client: redis::Client,
}

#[allow(unused)]
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

pub mod rs {
    use redis::FromRedisValue;

    #[derive(Debug)]
    pub struct Zpoped {
        pub member: String,
        pub score: f64,
    }

    #[derive(Debug)]
    pub struct Bzpoped {
        pub key: String,
        pub member: String,
        pub score: f64,
    }
    impl FromRedisValue for Zpoped {
        fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
            match v {
                redis::Value::Array(ref items) if items.len() == 2 => {
                    let member: String = redis::from_redis_value(&items[0])?;
                    let score: f64 = redis::from_redis_value(&items[1])?;
                    Ok(Zpoped { member, score })
                },
                _ => Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Expected a bulk response with 3 elements for BZPOPMAX"))),
            }
        }
    }
    impl FromRedisValue for Bzpoped {
        fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
            match v {
                redis::Value::Array(ref items) if items.len() == 3 => {
                    let key: String = redis::from_redis_value(&items[0])?;
                    let member: String = redis::from_redis_value(&items[1])?;
                    let score: f64 = redis::from_redis_value(&items[2])?;
                    Ok(Bzpoped { key, member, score })
                },
                _ => Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Expected a bulk response with 3 elements for BZPOPMAX"))),
            }
        }
    }
}

macro_rules! redis_c {
    // 基本形式：方法名、额外参数（不包括 key）、返回类型
    ($method_name:ident, ($($arg_name:ident: $arg_type:ty $(=> $transform:expr)?),*), $return_type:ty) => {
        pub fn $method_name<K: ToRedisArgs>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$method_name(key, $(redis_c!(@process_arg $arg_name $($transform)?)),*).map_err(erx::smp)
        }
    };

    // 支持额外泛型参数
    ($method_name:ident, ($($arg_name:ident: $arg_type:ty $(=> $transform:expr)?),*), $return_type:ty, generics: [$($generic:tt)*]) => {
        pub fn $method_name<K: ToRedisArgs, $($generic)*>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$method_name(key, $(redis_c!(@process_arg $arg_name $($transform)?)),*).map_err(erx::smp)
        }
    };

    // 支持 no_key：不添加 key: K
    ($method_name:ident, no_key, ($($arg_name:ident: $arg_type:ty $(=> $transform:expr)?),*), $return_type:ty) => {
        pub fn $method_name(&self, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$method_name($(redis_c!(@process_arg $arg_name $($transform)?)),*).map_err(erx::smp)
        }
    };

    // 支持 no_key 和 generics
    ($method_name:ident, no_key, ($($arg_name:ident: $arg_type:ty $(=> $transform:expr)?),*), $return_type:ty, generics: [$($generic:tt)*]) => {
        pub fn $method_name<$($generic)*>(&self, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$method_name($(redis_c!(@process_arg $arg_name $($transform)?)),*).map_err(erx::smp)
        }
    };

    // 支持显式指定 Redis 方法名
    ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty $(=> $transform:expr)?),*), $return_type:ty) => {
        pub fn $method_name<K: ToRedisArgs>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$redis_method(key, $(redis_c!(@process_arg $arg_name $($transform)?)),*).map_err(erx::smp)
        }
    };

    // 支持显式指定 Redis 方法名和 generics
    ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty $(=> $transform:expr)?),*), $return_type:ty, generics: [$($generic:tt)*]) => {
        pub fn $method_name<K: ToRedisArgs, $($generic)*>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
            self.get_connection()?.$redis_method(key, $(redis_c!(@process_arg $arg_name $($transform)?)),*).map_err(erx::smp)
        }
    };

    // 辅助宏：处理单个参数，决定是使用转换表达式还是原始参数名
    (@process_arg $arg_name:ident $transform:expr) => { $transform };
    (@process_arg $arg_name:ident) => { $arg_name };
}

macro_rules! redis_i {
    ($($method_name:ident),*) => {
        $(
            redis_c!($method_name, (), FacadeInt);
        )*
    };
}

macro_rules! redis_b {
    ($($method_name:ident),*) => {
        $(
            redis_c!($method_name, (), FacadeBool);
        )*
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

    redis_b!(exists, del, unlink, persist);
    redis_i!(ttl, expire_time, strlen);

    redis_c!(expire, (seconds: i64), FacadeBool);
    redis_c!(expire_at, (expire_at: i64), FacadeBool);
    redis_c!(rename, (new_key: N), FacadeBool, generics: [N: ToRedisArgs]);
    redis_c!(rename_nx, (new_key: N), FacadeBool, generics: [N: ToRedisArgs]);
    redis_c!(get, (), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(mget, no_key, (key: &[K]), Facade<RV>, generics: [K:ToRedisArgs, RV: FromRedisValue]);
    redis_c!(get_ex, (expire_at: u64 => redis::Expiry::EX(expire_at)), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(getset,(val:V), Facade<RV>, generics: [V: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(getdel, redis: get_del, (), Facade<RV>,  generics: [RV: FromRedisValue]);
    redis_c!(get_del, (), Facade<RV>, generics:[RV: FromRedisValue]);
    redis_c!(getrange, (from:isize, to:isize), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(append, (val: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(set, (val: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(mset, no_key, (items: &[(K, V)]), FacadeBool, generics: [K:ToRedisArgs, V: ToRedisArgs]);
    // redis_c!(set_multiple, no_key, (items: &[(K, V)]), FacadeBool, generics: [K: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(set_options, (value:V, options:redis::SetOptions), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(set_ex, (val: V,  seconds: u64), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(set_nx, (val: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(setrange, (offset:isize, value:V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(mset_nx, no_key, (items: &[(K, V)]),FacadeInt, generics: [K:ToRedisArgs, V: ToRedisArgs]);
    redis_c!(incr, (delta: D), Facade<V>, generics: [D: ToRedisArgs , V: FromRedisValue]);
    redis_c!(decr, (delta: D), Facade<V>, generics: [D: ToRedisArgs , V: FromRedisValue]);

    //hash
    redis_c!(hexists, (field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(hget, (field: F), Facade<RV>, generics: [F: ToRedisArgs, RV: FromRedisValue]);

    redis_c!(hget_ex, (fields: F, expire_at: u64 => {
        redis::Expiry::EX(expire_at)
    }), Facade<RV>, generics: [F: ToRedisArgs, RV: FromRedisValue]);

    redis_c!(hgetall, ( ), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(hget_del, ( field: F), Facade<RV>, generics: [F: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(hset, ( field: F, val: V), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);

    redis_c!(hset_ex, (
            expire_at: u64 => &{
                redis::HashFieldExpirationOptions::default()
                .set_expiration(
                    redis::SetExpiry::EX(expire_at)
                )
            },
            values: &[(F, V)]
        ), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);

    redis_c!(hset_nx, (field: F, val: V), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(hset_multiple, (values: &[(F, V)]), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(hdel, (field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(hpersist, (field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(hkeys, (), Facade<T>, generics: [T: FromRedisValue]);
    redis_c!(hvals, (), Facade<T>, generics: [T: FromRedisValue]);
    redis_c!(hincr, (field: F, delta: D), Facade<RV>, generics: [F: ToRedisArgs, D:  ToRedisArgs ,RV:FromRedisValue]);
    redis_i!(hlen);
    redis_c!(httl, (field:F), FacadeInt, generics: [F: ToRedisArgs]);
    redis_c!(hpttl, (field:F), FacadeInt, generics: [F: ToRedisArgs]);
    redis_c!(hexpire_time, (field:F), FacadeInt, generics: [F: ToRedisArgs]);

    //bit
    redis_c!(getbit, (offset:usize), FacadeBool);
    redis_i!(bitcount);
    redis_c!(bitcount_range, (start:usize, end:usize), FacadeInt);
    redis_c!(setbit, (offset:usize, value:bool), FacadeBool);

    // list operations
    redis_c!(blmove, (dstkey: D, src_dir: redis::Direction, dst_dir: redis::Direction, timeout: f64), Facade<RV>, generics: [D:ToRedisArgs, RV: FromRedisValue]);
    redis_c!(blmpop, no_key, (timeout: f64, numkeys: usize, key: K, dir: redis::Direction, count: usize), Facade<RV>, generics: [K:ToRedisArgs, RV: FromRedisValue]);
    redis_c!(blpop, (timeout: f64), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(brpop, (timeout: f64), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(brpoplpush, (dstkey: D, timeout: f64), Facade<RV>, generics: [D:ToRedisArgs, RV: FromRedisValue]);
    redis_c!(lindex, (index: isize), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(linsert_before, (pivot: P, value: V), FacadeBool, generics: [P: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(linsert_after, (pivot: P, value: V), FacadeBool, generics: [P: ToRedisArgs, V: ToRedisArgs]);
    redis_i!(llen);
    redis_c!(lmove, (dstkey: D, src_dir: redis::Direction, dst_dir: redis::Direction), FacadeBool, generics: [D: ToRedisArgs]);
    redis_c!(lmpop, no_key, (numkeys: usize, key: K, dir: redis::Direction, count: usize), Facade<RV>, generics: [K: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(lpop, (count: Option<core::num::NonZeroUsize>), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(lpos, (value: V, options: redis::LposOptions), Facade<RV>, generics: [V:ToRedisArgs, RV: FromRedisValue]);
    redis_c!(lpush, (value: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(lpush_exists, (value: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(lrange, (start: isize, stop: isize),  Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(lrem, (count: isize, value: V),  Facade<RV>, generics: [V: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(ltrim, (start: isize, stop: isize),  Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(lset, (index: isize, value: V),  Facade<RV>, generics: [V: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(rpop, (count: Option<core::num::NonZeroUsize>), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(rpoplpush, (dstkey: D), Facade<RV>, generics: [D:ToRedisArgs, RV: FromRedisValue]);
    redis_c!(rpush, (value: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(rpush_exists, (value: V), FacadeBool, generics: [V: ToRedisArgs]);

    //set commands
    redis_c!(sadd, (member: M), FacadeBool, generics: [M: ToRedisArgs]);
    redis_i!(scard);
    redis_c!(sdiff, (), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(sdiffstore, no_key, (dest: D, keys:K), FacadeInt, generics: [D: ToRedisArgs, K: ToRedisArgs]);
    redis_c!(sinter, (), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(sinterstore, no_key, (dest: D, keys:K), FacadeInt, generics: [D: ToRedisArgs, K: ToRedisArgs]);
    redis_c!(sismember, (member:M), FacadeBool,  generics: [M: ToRedisArgs]);
    redis_c!(smismember, (member:M), Facade<Vec<i8>>,  generics: [M: ToRedisArgs]);
    redis_c!(smembers, (), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(smove, no_key, (srckey: S, dstkey: D, member: M), FacadeBool, generics: [S: ToRedisArgs, D: ToRedisArgs, M: ToRedisArgs]);
    redis_c!(spop, (), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(srandmember, (), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(srandmember_multiple, (count: usize), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(srem, (member: M), FacadeBool, generics: [M: ToRedisArgs]);
    redis_c!(sunion, (), Facade<RV>,  generics: [RV: FromRedisValue]);
    redis_c!(sunionstore, no_key, (dstkey: D, keys: K), Facade<RV>, generics: [ D: ToRedisArgs, K: ToRedisArgs, RV: FromRedisValue]);

    // sorted set commands
    redis_c!(zadd, (score: S, member: M), FacadeBool, generics: [S: ToRedisArgs, M: ToRedisArgs]);
    redis_c!(zadd_multiple, (items: &[(S, M)]), FacadeBool, generics: [S: ToRedisArgs, M: ToRedisArgs]);
    redis_i!(zcard);
    redis_c!(zcount, (min: M, max: M), FacadeInt, generics: [M: ToRedisArgs]);
    redis_c!(zincr, (member: M, delta:D), Facade<RV>, generics: [M: ToRedisArgs, D: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(zinterstore, no_key, (dest: D, keys:K), FacadeInt, generics: [D: ToRedisArgs, K: ToRedisArgs]);
    redis_c!(zinterstore_min, no_key, (dest: D, keys:K), FacadeInt, generics: [D: ToRedisArgs, K: ToRedisArgs]);
    redis_c!(zinterstore_max, no_key, (dest: D, keys:K), FacadeInt, generics: [D: ToRedisArgs, K: ToRedisArgs]);
    redis_c!(zinterstore_weights, no_key, (dest: D, keys:&[(K, W)]), FacadeInt, generics: [D: ToRedisArgs, K: ToRedisArgs, W: ToRedisArgs]);
    redis_c!(zinterstore_max_weights, no_key, (dest: D, keys:&[(K, W)]), FacadeInt, generics: [D: ToRedisArgs, K: ToRedisArgs, W: ToRedisArgs]);
    redis_c!(zlexcount, (min: M, max: MM), FacadeInt, generics: [M: ToRedisArgs, MM: ToRedisArgs]);

    redis_c!(bzpopmax, (timeout:f64), Facade<rs::Bzpoped>);
    redis_c!(bzpopmin, (timeout:f64), Facade<rs::Bzpoped>);
    redis_c!(zpopmax, (count:isize),  Facade<rs::Zpoped> );
    redis_c!(zpopmin, (count:isize),  Facade<rs::Zpoped> );
}


#[allow(dead_code)]
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
        println!("{:?}", c.exists("test_ttl"));
        println!("{:?}", c.set("test_ttl", "value"));
        println!("{:?}", c.exists("test_ttl"));

        // println!("{:?}", c.set_ex("test_ttl", 1024, 10));
        println!("{:?}", c.ttl("test_ttl"));
        // println!("{:?}", c.incr("test_ttl", 24));
        // println!("{:?}", c.set::<_, Name>("LJ", name_it()));
        // println!("{:?}", c.get::<_, Name>("LJ"));

        // println!("hset {:?}", c.hset("LJHash", "d1age", 102410));
        // println!("hset_multiple {:?}", c.hset_multiple("LJHash", &[("age", 1811), ("lastage", 24)]));
    }

    fn rds() -> Redis {
        Redis::new(redis::Client::open("redis://127.0.0.1").unwrap())
    }
}
