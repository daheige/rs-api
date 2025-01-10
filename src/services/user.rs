use crate::entity::user::User;
use crate::{config, entity};
use r2d2::Pool;
use redis::RedisResult;
use redis::{Commands, RedisError};

pub fn set_user(redis: Pool<redis::Client>, user: &entity::user::User) -> RedisResult<String> {
    let mut conn = redis.get().unwrap();
    // set pool timeout session
    // let mut conn = config::REDIS_POOL.get_timeout(Duration::from_secs(2)).unwrap();
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

pub async fn query_user(db: sqlx::MySqlPool, id: u64) -> Result<User, sqlx::Error> {
    let sql = "select * from users where id = ?";
    // query_as将其映射到结构体UserEntity中
    let user: User = sqlx::query_as(sql).bind(id).fetch_one(&db).await?;
    // println!("user: {:?}", user);
    println!("id = {} name = {}", user.id, user.username);

    Ok(user)
}

pub async fn query_user_count(db: sqlx::MySqlPool) -> Result<u64, sqlx::Error> {
    let sql = "select count(*) as cnt from users";
    let result: (i64,) = sqlx::query_as(sql).fetch_one(&db).await?;
    println!("total = {}", result.0);
    Ok(result.0 as u64)
}
