use crate::middleware as ware;
use crate::{config::app::AppState, handlers};
use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

// create api router
pub fn api_router(state: Arc<AppState>) -> Router {
    let router = Router::new()
        .route("/", get(handlers::index::root))
        .route("/empty-array", get(handlers::index::empty_array))
        .route("/empty-object", get(handlers::index::empty_object))
        .route("/users", post(handlers::index::create_user))
        .route("/html", get(handlers::index::html_foo))
        .route("/set-user-cookie", get(handlers::index::set_user_cookie))
        // .route("/set-user-cookie", post(handlers::index::set_user_cookie))
        .route("/get-user-cookie", get(handlers::index::get_user_cookie))
        .route("/form", post(handlers::index::accept_form))
        .route("/user/{id}", get(handlers::index::user_info))
        .route("/repo/{repo}/{name}", get(handlers::index::repo_info))
        .route("/query_user", get(handlers::index::query_user))
        .route("/query_user_opt", get(handlers::index::query_user_opt))
        .route(
            "/query_user_opt_done",
            get(handlers::index::query_user_opt_done),
        )
        .route("/all-query", get(handlers::index::all_query))
        .route("/validate", get(handlers::index::validate_name))
        .route("/json_or_form", post(handlers::index::json_or_form))
        .with_state(state);

    // router group like /api/user/xxx this way
    // /api/foo/xxx
    // /api/hello
    let api_routes = Router::new()
        .nest("/user", user_router())
        .nest("/foo", foo_router())
        .route("/hello", get(handlers::index::root))
        .route("/either/{id}", get(handlers::index::either_handler))
        .fallback(api_not_found); // set api group and not found handler for api/xxx

    let router = Router::new()
        .merge(router)
        .nest("/api", api_routes)
        .fallback(not_found_handler) // global router not found
        .layer(middleware::from_fn(ware::access_log))
        .layer(middleware::from_fn(ware::no_cache_header));
    router
}

/// router group like these way
/// /api/user/query?id=1&username=daheige
/// /api/user/1
fn user_router() -> Router {
    let router = Router::new()
        .route("/{id}", get(handlers::index::user_info))
        .route("/query", get(handlers::index::query_user));
    router
}

/// router group like /foo/xxx
fn foo_router() -> Router {
    let router = Router::new()
        .route("/empty-array", get(handlers::index::empty_array))
        .route("/empty-object", get(handlers::index::empty_object));

    router
}

// handler not found for global router not found
async fn not_found_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "this page not found")
}

// handler not found
async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(handlers::Reply {
            code: 404,
            message: "api not found".to_string(),
            data: Some(handlers::EmptyObject {}),
        }),
    )
}
