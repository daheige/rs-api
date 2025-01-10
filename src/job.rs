use crate::config::mysql;
use crate::config::xredis;
use crate::entity::user;
use crate::services::user as userService;
use std::process;

mod config;
mod entity;
mod infras;
mod services;

#[tokio::main]
async fn main() {
    println!("job exec...");
    println!("app_debug:{:?}", config::APP_CONFIG.app_debug);
    println!("current process pid:{}", process::id());

    // create mysql pool
    let mysql_pool = mysql::pool(&config::APP_CONFIG.mysql_conf)
        .await
        .expect("mysql pool init failed");

    // create redis pool
    let redis_pool = xredis::pool(&config::APP_CONFIG.redis_conf);
    let app_state = config::app::AppState {
        redis_pool,
        mysql_pool,
    };
    // get user
    let result = userService::query_user(app_state.mysql_pool.clone(), 1).await;
    println!("result:{:?}", result);

    let result = userService::query_user_count(app_state.mysql_pool.clone()).await;
    println!("result:{:?}", result);

    let user = user::User {
        id: 1,
        username: "daheige".to_string(),
    };

    let res = userService::set_user(app_state.redis_pool.clone(), &user);
    if res.is_err() {
        println!("set user error:{}", res.err().unwrap().to_string());
    } else {
        println!("set success");
    }
}
