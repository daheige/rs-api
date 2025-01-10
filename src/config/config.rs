use crate::config::{mysql, xredis};
use crate::infras::{Config, ConfigTrait};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub redis_conf: xredis::RedisConfig,
    pub mysql_conf: mysql::MysqlConfig,
    pub app_debug: bool,
    pub app_name: String,
    pub app_port: i32,
    pub graceful_wait_time: u64,
}

// config read and init app config
// 项目启动的时候读取配置文件app.yaml
pub static APP_CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let config_dir = std::env::var("CONFIG_DIR").unwrap_or("./".to_string());
    let filename = Path::new(config_dir.as_str()).join("app.yaml");
    println!("filename:{:?}", filename);
    let c = Config::load(filename);

    // read config to struct
    let conf: AppConfig = serde_yaml::from_str(c.content()).unwrap();
    // 开发过程中，可以取消下面的注释
    if conf.app_debug {
        println!("conf:{:?}", conf);
    }

    conf
});
