use r2d2::Pool;

#[derive(Clone)]
pub struct AppState {
    pub mysql_pool: sqlx::MySqlPool,
    pub redis_pool: Pool<redis::Client>,
}
