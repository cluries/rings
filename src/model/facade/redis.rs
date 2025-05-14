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
        let logit = self.logit;

        let mut conn = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                if logit {
                    tracing::error!("Redis {}", e);
                }
                return false;
            },
        };
        conn.del(key).unwrap_or_else(unwra(self.logit, false))
    }

    pub fn expire(&self, key: &str, ttl: i64) -> Option<u64> {
        let logit = self.logit;

        let mut conn = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                if logit {
                    tracing::error!("Redis {}", e);
                }
                return None;
            },
        };

        conn.expire(key, ttl).unwrap_or_else(unwra(self.logit, None))
    }

    pub fn ttl(&self, key: &str) -> Option<i64> {
        let logit = self.logit;
        self.client.get_connection().ok().and_then(|mut conn| match conn.ttl(key) {
            Ok(c) => Some(c),
            Err(e) => {
                if logit {
                    tracing::error!("Redis {}", e);
                }
                None
            },
        })
    }

    pub fn get<T: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<T> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.get(key).map_err(erx::smp)
    }

    pub fn hget<F: redis::ToRedisArgs, V: redis::FromRedisValue>(&self, key: &str, field: F) -> erx::ResultE<V> {
        let mut conn = self.client.get_connection().map_err(erx::smp)?;
        conn.hget(key, field).map_err(erx::smp)
    }

    pub fn hgetall<T: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<T> {
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

    pub fn hlen<T: redis::FromRedisValue>(&self, key: &str) -> erx::ResultE<T> {
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
        let mut conn = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Redis {}", e);
                return None;
            },
        };
        conn.incr(key, val).unwrap_or_else(|e| {
            tracing::error!("Redis {}", e);
            None
        })
    }

    pub fn decr<T: redis::ToRedisArgs + redis::FromRedisValue>(&self, key: &str, val: T) -> Option<T> {
        let mut conn = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Redis {}", e);
                return None;
            },
        };
        conn.decr(key, val).unwrap_or_else(|e| {
            tracing::error!("Redis {}", e);
            None
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_value() {
        let c = rds();
        println!("{:?}", c.set_ex::<_, String>("test_ttl", 1024, 10));
        println!("{:?}", c.ttl("test_ttl"));
    }

    fn rds() -> Redis {
        Redis::new(redis::Client::open("redis://127.0.0.1").unwrap())
    }
}
