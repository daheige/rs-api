// redis
use crate::infras::RedisService;
use r2d2::Pool;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// redis config
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct RedisConfig {
    dsn: String,
    max_size: u32,
    min_idle: u32,
    max_lifetime: u64,
    idle_timeout: u64,
    connection_timeout: u64,
}

// redis pool init
pub fn pool(redis_conf: &RedisConfig) -> Pool<Client> {
    let pool = RedisService::builder()
        .with_dsn(redis_conf.dsn.as_str())
        .with_max_size(redis_conf.max_size)
        .with_max_lifetime(Duration::from_secs(redis_conf.max_lifetime))
        .with_idle_timeout(Duration::from_secs(redis_conf.idle_timeout))
        .with_min_idle(redis_conf.min_idle)
        .with_connect_timeout(Duration::from_secs(redis_conf.connection_timeout))
        .pool();
    pool
}
