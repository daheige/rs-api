use redis::Commands;
use redis::RedisResult;

// 定义项目相关module
mod config;
mod handlers;
mod middleware;
mod routes;
mod services;

fn main() {
    println!("Hello, world!");
    println!("app_debug:{:?}", config::APP_CONFIG.app_debug);
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
