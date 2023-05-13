use crate::handlers;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

// create api router
pub fn api_router() -> Router {
    let mut app = Router::new()
        .route("/", get(handlers::index::root))
        .route("/empty-array", get(handlers::index::empty_array))
        .route("/users", post(handlers::index::create_user))
        .route("/html", get(handlers::index::html_foo))
        .route("/form", post(handlers::index::accept_form))
        .route("/set-user-cookie", get(handlers::index::set_user_cookie))
        // .route("/set-user-cookie", post(handlers::index::set_user_cookie))
        .route("/get-user-cookie", get(handlers::index::get_user_cookie));

    // handler not found
    app = app.fallback(not_found_handler);
    app
}

// handler not found
async fn not_found_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "this page not found")
}
