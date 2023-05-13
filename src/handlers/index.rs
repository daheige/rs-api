use crate::entity::user;
use crate::services::user as userService;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Form, Json,
};

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

// create user
pub async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<user::CreateUser>,
) -> impl IntoResponse {
    // insert your application logic here
    let user = user::User {
        id: 1337,
        username: payload.username,
    };

    // set user cache
    let res = userService::set_user(&user);
    if res.is_err() {
        return (
            StatusCode::OK,
            Json(super::Reply {
                code: 500,
                message: format!("{}", res.err().unwrap()),
                data: None,
            }),
        );
    }

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (
        StatusCode::CREATED,
        Json(super::Reply {
            code: 0,
            message: "success".to_string(),
            data: Some(user),
        }),
    )
}

pub async fn empty_array() -> impl IntoResponse {
    let empty_arr: super::EmptyArray = Vec::new();
    (
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(empty_arr),
        }),
    )
}

// returns html entity
pub async fn html_foo() -> Html<&'static str> {
    Html("<h1>hello,rs-api</h1>")
}

// Content-Type: application/x-www-form-urlencoded
pub async fn accept_form(Form(input): Form<user::UserForm>) -> impl IntoResponse {
    println!("current input:{:?}", input);
    (
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(input),
        }),
    )
}
