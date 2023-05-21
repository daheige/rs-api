use crate::handlers;
use crate::middleware as ware;
use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

// create api router
pub fn api_router() -> Router {
    let mut app = Router::new()
        .route("/", get(handlers::index::root))
        .route("/empty-array", get(handlers::index::empty_array))
        .route("/empty-object", get(handlers::index::empty_object))
        .route("/users", post(handlers::index::create_user))
        .route("/html", get(handlers::index::html_foo))
        .route("/set-user-cookie", get(handlers::index::set_user_cookie))
        // .route("/set-user-cookie", post(handlers::index::set_user_cookie))
        .route("/get-user-cookie", get(handlers::index::get_user_cookie))
        .route("/form", post(handlers::index::accept_form))
        .route("/user/:id", get(handlers::index::user_info))
        .route("/repo/:repo/:name", get(handlers::index::repo_info))
        .route("/query_user", get(handlers::index::query_user))
        .route("/query_user_opt", get(handlers::index::query_user_opt))
        .route(
            "/query_user_opt_done",
            get(handlers::index::query_user_opt_done),
        )
        .route("/all-query", get(handlers::index::all_query))
        .route("/validate", get(handlers::index::validate_name))
        .layer(middleware::from_fn(ware::access_log))
        .layer(middleware::from_fn(ware::no_cache_header));

    // handler not found
    app = app
        .fallback(not_found_handler)
        .layer(middleware::from_fn(ware::access_log));
    app
}

// handler not found
async fn not_found_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "this page not found")
}
