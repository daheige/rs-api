use crate::entity::user;
use crate::services::user as userService;
use std::process;

mod config;
mod entity;
mod services;

fn main() {
    println!("job exec...");
    println!("app_debug:{:?}", config::APP_CONFIG.app_debug);
    println!("current process pid:{}", process::id());

    let user = user::User {
        id: 1,
        username: "daheige".to_string(),
    };

    let res = userService::set_user(&user);
    if res.is_err() {
        println!("set user error:{}", res.err().unwrap().to_string());
    } else {
        println!("set success");
    }
}
