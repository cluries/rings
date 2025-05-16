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

// macro_rules! redis_c {
//     // 基本形式：方法名、参数列表、返回类型（默认调用参数与参数名一致）
//     ($method_name:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
//         pub fn $method_name(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$method_name($($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持显式指定 Redis 方法名
//     ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
//         pub fn $method_name(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$redis_method($($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持泛型参数的方法
//     ($method_name:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
//         pub fn $method_name<$($generic)*>(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$method_name($($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持泛型参数且显式指定 Redis 方法名
//     ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
//         pub fn $method_name<$($generic)*>(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$redis_method($($arg_name),*).map_err(erx::smp)
//         }
//     };
// }
// macro_rules! redis_c {
//     // 基本形式：方法名、额外参数（不包括 key）、返回类型，默认 key: K 和 generics: [K: ToRedisArgs]
//     ($method_name:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
//         pub fn $method_name<K: ToRedisArgs>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$method_name(key, $($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 基本形式 + no_key：不添加 key: K
//     ($method_name:ident, no_key, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
//         pub fn $method_name(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$method_name($($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持显式指定 Redis 方法名
//     ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
//         pub fn $method_name<K: ToRedisArgs>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$redis_method(key, $($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持显式指定 Redis 方法名 + no_key：不添加 key: K
//     ($method_name:ident, redis: $redis_method:ident, no_key, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty) => {
//         pub fn $method_name(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$redis_method($($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持额外泛型参数（例如 RV: FromRedisValue）
//     ($method_name:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
//         pub fn $method_name<K: ToRedisArgs, $($generic)*>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$method_name(key, $($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持额外泛型参数 + no_key：不添加 key: K
//     ($method_name:ident, no_key, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
//         pub fn $method_name<$($generic)*>(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$method_name($($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持额外泛型参数且显式指定 Redis 方法名
//     ($method_name:ident, redis: $redis_method:ident, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
//         pub fn $method_name<K: ToRedisArgs, $($generic)*>(&self, key: K, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$redis_method(key, $($arg_name),*).map_err(erx::smp)
//         }
//     };
//
//     // 支持额外泛型参数且显式指定 Redis 方法名 + no_key：不添加 key: K
//     ($method_name:ident, redis: $redis_method:ident, no_key, ($($arg_name:ident: $arg_type:ty),*), $return_type:ty, generics: [$($generic:tt)*]) => {
//         pub fn $method_name<$($generic)*>(&self, $($arg_name: $arg_type),*) -> $return_type {
//             self.get_connection()?.$redis_method($($arg_name),*).map_err(erx::smp)
//         }
//     };
// }

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
    redis_c!(append, (val: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(set, (val: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(mset, no_key, (items: &[(K, V)]), FacadeBool, generics: [K:ToRedisArgs, V: ToRedisArgs]);
    redis_c!(set_multiple, no_key, (items: &[(K, V)]), FacadeBool, generics: [K: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(set_ex, (val: V,  seconds: u64), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(set_nx, (val: V), FacadeBool, generics: [V: ToRedisArgs]);
    redis_c!(incr, (delta: D), Facade<V>, generics: [D: ToRedisArgs , V: FromRedisValue]);
    redis_c!(decr, (delta: D), Facade<V>, generics: [D: ToRedisArgs , V: FromRedisValue]);

    //hash
    redis_c!(hexists, (field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(hget, (field: F), Facade<RV>, generics: [F: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(hgetall, ( ), Facade<RV>, generics: [RV: FromRedisValue]);
    redis_c!(hget_del, ( field: F), Facade<RV>, generics: [F: ToRedisArgs, RV: FromRedisValue]);
    redis_c!(httl, ( field: F), FacadeInt, generics: [F: ToRedisArgs]);
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

    redis_c!(hpersist, (field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(hset_nx, (field: F, val: V), FacadeBool, generics: [FV: ToRedisArgs]);
    redis_c!(hset_multiple, (values: &[(F, V)]), FacadeBool, generics: [F: ToRedisArgs, V: ToRedisArgs]);
    redis_c!(hdel, (field: F), FacadeBool, generics: [F: ToRedisArgs]);
    redis_c!(hkeys, ( ), Facade<T>, generics: [T: FromRedisValue]);
    redis_c!(hvals, ( ), Facade<T>, generics: [T: FromRedisValue]);
    redis_c!(hincr, (field: F, delta: D), Facade<RV>, generics: [F: ToRedisArgs, D:  ToRedisArgs ,RV:FromRedisValue]);
    redis_i!(hlen);

    //bit
    redis_c!(getbit, (offset:usize), FacadeBool);
    redis_i!(bitcount);
    redis_c!(bitcount_range, (start:usize, end:usize), FacadeInt);
    redis_c!(setbit, (offset:usize, value:bool), FacadeBool);

    //set commands
    redis_c!(sadd, (member: M), FacadeBool, generics: [M: ToRedisArgs]);
    redis_i!(scard, sdiff);
    redis_c!(sdiffstore, (dest: &str), FacadeInt);
    redis_c!(smove, (dst: DK, member: M), FacadeBool,  generics: [DK:ToRedisArgs, M: ToRedisArgs]);
    redis_b!(sunion);
    redis_c!(sunionstore, (dst: DK), FacadeBool,  generics: [DK:ToRedisArgs] );

    // redis_c!(sismember, (key: &str, member:M), FacadeInt,  generics: [M: ToRedisArgs]);
    // redis_c!(smembers, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    // redis_c!(spop, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    // redis_c!(srandmember, (key: &str), Facade<RV>, generics: [RV: FromRedisValue]);
    // redis_c!(srandmember_multiple, (key: &str, count: usize), Facade<RV>, generics: [RV: FromRedisValue]);
    // redis_c!(srem, (key: &str, member: M), FacadeBool, generics: [M: ToRedisArgs]);
    //
    // // sorted set commands
    // redis_c!(zadd, (key: &str, score: S, member: M), FacadeBool, generics: [S: ToRedisArgs, M: ToRedisArgs]);
    // redis_c!(zadd_multiple, (key: &str, items: &[(S, M)]), FacadeBool, generics: [S: ToRedisArgs, M: ToRedisArgs]);
    // redis_c!(zcard, (key: &str), FacadeInt);
    // redis_c!(zcount, (key: &str, min: M, max: M), FacadeInt, generics: [M: ToRedisArgs]);
    // redis_c!(zincr, (key: &str, member: M, delta: D), Facade<D>, generics: [M: ToRedisArgs, D: ToRedisArgs + FromRedisValue]);
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
