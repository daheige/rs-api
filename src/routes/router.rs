use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};

// create api router
pub fn api_router() -> Router {
    let app = Router::new()
        .route("/", get(handlers::index::root))
        .route("/empty-array", get(handlers::index::empty_array))
        .route("/users", post(handlers::index::create_user));
    app
}
