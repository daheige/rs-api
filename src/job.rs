use redis::Commands;
use redis::RedisResult;
use std::process;

mod config;
mod services;

fn main() {
    println!("job exec...");
    println!("app_debug:{:?}", config::APP_CONFIG.app_debug);
    println!("current process pid:{}", process::id());

    let mut conn = config::REDIS_POOL.get().unwrap();

    // 设置单个pool timeout
    // let mut conn = pool.get_timeout(Duration::from_secs(2)).unwrap();
    let res: RedisResult<String> = conn.set("my_user", "daheige");
    if res.is_err() {
        println!("redis set error:{}", res.err().unwrap().to_string());
    } else {
        println!("set success");
    }
}
