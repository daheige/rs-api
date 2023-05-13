use crate::config::xredis;
use once_cell::sync::Lazy;
use rs_infras::config::{Config, ConfigTrait};
use serde::{Deserialize, Serialize};

// config read and init app config
pub static APP_CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let mut c = Config::new("app.yaml");
    c.load().expect("read file failed");

    // read config to struct
    let conf: AppConfig = serde_yaml::from_str(c.content()).unwrap();
    if conf.app_debug {
        println!("{:?}", conf);
    }

    conf
});

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub redis_conf: xredis::RedisConfig,
    pub app_debug: bool,
    pub app_name: String,
    pub app_port: i32,
    pub graceful_wait_time: u64,
}
