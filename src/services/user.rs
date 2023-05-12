use crate::{config, entity};
use redis::RedisResult;
use redis::{Commands, RedisError};

pub fn set_user(user: &entity::user::User) -> RedisResult<String> {
    let mut conn = config::REDIS_POOL.get().unwrap();

    // 设置单个pool timeout
    // let mut conn = pool.get_timeout(Duration::from_secs(2)).unwrap();
    let res: RedisResult<String> = conn.set(
        "my_user:".to_string() + &user.id.to_string(),
        user.username.as_str(),
    );

    if res.is_err() {
        return Err(RedisError::from((
            redis::ErrorKind::ResponseError,
            "redis set user failed",
            format!("{}", res.err().unwrap().to_string()),
        )));
    }

    Ok("ok".to_string())
}
