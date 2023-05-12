use axum::{
    http::StatusCode,
    // response::IntoResponse,
    routing::{get, post},
    Json,
    Router,
    // ServiceExt,
};
use serde::{Deserialize, Serialize};

pub fn api_router() -> Router {
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user));
    app
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}