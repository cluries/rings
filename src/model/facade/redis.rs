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

impl Redis {
    pub fn shared() -> Self {
        Redis { logit: true, client: crate::model::make_redis_client().unwrap() }
    }

    pub fn new(c: redis::Client) -> Self {
        Redis { logit: true, client: c }
    }

    pub fn exists(&self, key: &str) -> bool {
        let mut conn = conn_mut!(self, false);
        conn.exists(key).unwrap_or_else(unwra(self.logit, false))
    }

    pub fn del(&self, key: &str) -> bool {
        let mut conn = conn_mut!(self, false);
        conn.del(key).unwrap_or_else(unwra(self.logit, false))
    }

    pub fn expire(&self, key: &str, ttl: i64) -> Option<u64> {
        let mut conn = conn_mut!(self, None);
        conn.expire(key, ttl).unwrap_or_else(unwra(self.logit, None))
    }

    /// Set the expiration for a key as a UNIX timestamp.
    pub fn expire_at(&self, key: &str, timestamp: i64) -> Option<i64> {
        let mut conn = conn_mut!(self, None);
        conn.expire_at(key, timestamp).unwrap_or_else(unwra(self.logit, None))
    }

    /// Remove the expiration from a key.
    pub fn persist(&self, key: &str) -> Option<bool> {
        let mut conn = conn_mut!(self, None);
        conn.persist(key).unwrap_or_else(unwra(self.logit, None))
    }

    pub fn ttl(&self, key: &str) -> Option<i64> {
        let mut conn = conn_mut!(self, None);
        conn.ttl(key).unwrap_or_else(unwra(self.logit, None))
    }

    pub fn get<RV: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.get(key).map_err(erx::smp)
    }

    pub fn getset<V: redis::ToRedisArgs + redis::FromRedisValue>(&self, key: &str, val: V) -> erx::ResultE<V> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.getset(key, val).map_err(erx::smp)
    }

    /// Get the value of a key and delete it
    pub fn getdel<RV: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.get_del(key).map_err(erx::smp)
    }

    pub fn hget<F: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: &str, field: F) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hget(key, field).map_err(erx::smp)
    }

    pub fn hgetall<RV: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hgetall(key).map_err(erx::smp)
    }

    pub fn set<T: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: &str, val: T) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.set(key, val).map_err(erx::smp)
    }

    pub fn set_ex<T: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: &str, val: T, ttl: u64) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.set_ex(key, val, ttl).map_err(erx::smp)
    }

    pub fn set_nx<T: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: &str, val: T) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.set_nx(key, val).map_err(erx::smp)
    }

    pub fn set_multiple<K: redis::ToRedisArgs, V: redis::ToRedisArgs, RV: redis::FromRedisValue>(
        &self, items: &[(K, V)],
    ) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.mset(items).map_err(erx::smp)
    }

    pub fn rename<K: redis::ToRedisArgs, N: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: K, nkey: N) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.rename(key, nkey).map_err(erx::smp)
    }

    /// Rename a key, only if the new key does not exist.
    pub fn rename_nx<K: redis::ToRedisArgs, N: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: K, nkey: N) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.rename_nx(key, nkey).map_err(erx::smp)
    }

    /// Append a value to a key.
    pub fn append<V: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: &str, val: V) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.append(key, val).map_err(erx::smp)
    }

    pub fn hset<F: redis::ToRedisArgs, V: redis::ToRedisArgs, RV: redis::FromRedisValue>(
        &self, key: &str, field: F, val: V,
    ) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hset(key, field, val).map_err(erx::smp)
    }

    pub fn hdel<F: redis::ToRedisArgs, RV: redis::FromRedisValue>(&self, key: &str, field: F) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hdel(key, field).map_err(erx::smp)
    }

    pub fn hlen<RV: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<RV> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hlen(key).map_err(erx::smp)
    }

    pub fn hkeys<T: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<T> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hkeys(key).map_err(erx::smp)
    }

    pub fn hvals<T: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<T> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hvals(key).map_err(erx::smp)
    }

    pub fn incr<T: redis::ToRedisArgs + redis::FromRedisValue>(&self, key: &str, val: T) -> Option<T> {
        let mut conn = conn_mut!(self, None);
        conn.incr(key, val).unwrap_or_else(unwra(self.logit, None))
    }

    ///
    pub fn decr<T: redis::ToRedisArgs + redis::FromRedisValue>(&self, key: &str, val: T) -> Option<T> {
        let mut conn = conn_mut!(self, None);
        conn.decr(key, val).unwrap_or_else(unwra(self.logit, None))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use redis::{RedisResult, RedisWrite, Value};

    #[derive(Debug)]
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
        }
    }

    impl redis::FromRedisValue for Name {
        fn from_redis_value(v: &Value) -> RedisResult<Self> {
            todo!()
        }
    }

    fn name_it() -> Name {
        Name { key: "LJ".to_string(), first: "luo".to_string(), middle: "-".to_string(), last: "jing".to_string() }
    }

    #[test]
    fn test_redis_value() {
        let c = rds();
        println!("{:?}", c.set_ex::<_, String>("test_ttl", 1024, 10));
        println!("{:?}", c.ttl("test_ttl"));
        println!("{:?}", c.incr("test_ttl", 24));
        // println!("{:?}", c.set::<Name, Name>("LJ", name_it()));
    }

    fn rds() -> Redis {
        Redis::new(redis::Client::open("redis://127.0.0.1").unwrap())
    }
}
