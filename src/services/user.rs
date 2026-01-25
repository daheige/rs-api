use crate::entity::user::User;
use r2d2::Pool;
use redis::Commands;
use redis::RedisResult;

// 下面的函数为了演示效果就没有单独封装到对应的模块中，实际项目开发中需要做好分层设计，例如：ddd
pub async fn set_user_cache(redis: Pool<redis::Client>, user: &User) -> anyhow::Result<()> {
    let mut conn = redis.get()?;

    // set pool timeout session
    // let mut conn = redis.get_timeout(Duration::from_secs(2))?;
    let key = format!("user:{}", user.id);
    let value = serde_json::to_string(&user)?;
    let res: RedisResult<String> = conn.set(key, value);
    if res.is_err() {
        return Err(anyhow::anyhow!(
            "failed to set user error:{}",
            res.err().unwrap()
        ));
    }

    Ok(())
}

pub async fn get_user_cache(redis: Pool<redis::Client>,id: i64) -> anyhow::Result<User> {
    let mut conn = redis.get()?;

    // set pool timeout session
    // let mut conn = redis.get_timeout(Duration::from_secs(2))?;
    let key = format!("user:{}", id);
    let res:String = conn.get(key)?;
    if res.is_empty() {
        return Err(anyhow::anyhow!("user cache not found"));
    }

    let user : User = serde_json::from_str(&res)?;

    Ok(user)
}

pub async fn query_user(db: sqlx::MySqlPool, id: u64) -> anyhow::Result<User> {
    let sql = "select * from users where id = ?";
    // query_as将其映射到结构体UserEntity中
    let user: User = sqlx::query_as(sql).bind(id).fetch_one(&db).await?;
    // println!("user: {:?}", user);
    println!("id = {} name = {}", user.id, user.username);

    Ok(user)
}

pub async fn get_user(db: sqlx::MySqlPool, username: &str) -> anyhow::Result<User> {
    let sql = "select * from users where username = ?";
    // query_as将其映射到结构体UserEntity中
    let user: User = sqlx::query_as(sql).bind(username).fetch_one(&db).await?;
    // println!("user: {:?}", user);
    println!("id = {} name = {}", user.id, user.username);

    Ok(user)
}

pub async fn get_user_by_id(db: sqlx::MySqlPool, id: i64) -> anyhow::Result<User> {
    let sql = "select * from users where id = ?";
    // query_as将其映射到结构体UserEntity中
    let user: User = sqlx::query_as(sql).bind(id).fetch_one(&db).await?;
    // println!("user: {:?}", user);
    println!("id = {} name = {}", user.id, user.username);

    Ok(user)
}

pub async fn create_user(db: sqlx::MySqlPool, username: &str)-> anyhow::Result<i64> {
    let sql = "insert into users (username) values (?)";
    let res = sqlx::query(sql).bind(username).execute(&db).await?;
    if res.last_insert_id() == 0{
        return Err(anyhow::anyhow!(
            "failed to create user:{}",username
        ));
    }

    Ok(res.last_insert_id() as i64)
}

pub async fn query_user_count(db: sqlx::MySqlPool) -> anyhow::Result<u64> {
    let sql = "select count(*) as cnt from users";
    let result: (i64,) = sqlx::query_as(sql).fetch_one(&db).await?;
    println!("total = {}", result.0);

    Ok(result.0 as u64)
}
