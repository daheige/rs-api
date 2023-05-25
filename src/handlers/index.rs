use super::json_or_form::JsonOrForm;
use super::validate_form::ValidatedForm;
use crate::entity::user;
use crate::services::user as userService;
use crate::utils::get_header;
use axum::http::{header, HeaderMap};
use axum::response::Response;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form, Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// validate error
use validator::Validate;

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

pub async fn empty_object() -> impl IntoResponse {
    let empty_object = super::EmptyObject {};
    (
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(empty_object),
        }),
    )
}

// returns html entity
pub async fn html_foo() -> Html<&'static str> {
    Html("<h1>hello,rs-api</h1>")
}

// get params from form request
// Content-Type: application/x-www-form-urlencoded
// pub async fn accept_form(Form(input): Form<user::UserForm>) -> impl IntoResponse {
pub async fn accept_form(
    headers: HeaderMap,
    Form(input): Form<user::UserForm>,
) -> impl IntoResponse {
    println!("headers: {:?}", headers);
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap();
    println!("user-agent:{}", ua);

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

pub async fn set_user_cookie() -> impl IntoResponse {
    let user = user::User {
        id: 1,
        username: "daheige".to_string(),
    };

    let mut headers = HeaderMap::new();
    let cookie = format!("{}={}", "user_name", user.username);
    headers.insert(header::SET_COOKIE, cookie.as_str().parse().unwrap());

    // redirect to / with cookie
    // headers.insert(header::LOCATION, "/".parse().unwrap());
    // (StatusCode::FOUND, headers, ())

    // returns json and set cookie
    (
        StatusCode::OK,
        headers,
        Json(super::Reply {
            code: 0,
            message: "login success".to_string(),
            data: Some(super::EmptyObject {}),
        }),
    )
}

pub async fn get_user_cookie(headers: HeaderMap) -> impl IntoResponse {
    let cookies = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or("".to_string()); // get all cookie
    if cookies.is_empty() {
        return Err("cookies is empty"); // no Cookie
    }

    let mut username: Option<String> = None;
    let cookies: Vec<&str> = cookies.split(';').collect(); // split with ;
    for cookie in cookies {
        let cookie_pair: Vec<&str> = cookie.split('=').collect(); // splice with =
        let cookie_name = cookie_pair[0].trim();
        let cookie_value = cookie_pair[1].trim();
        // cookie no empty
        if cookie_name == "user_name" && !cookie_value.is_empty() {
            username = Some(String::from(cookie_value));
            break;
        }
    }

    if username.is_none() {
        return Err("username cookie is empty");
    }

    Ok((
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(username),
        }),
    ))
}

// extract::{Path, Query} for query params and path params

/// get path params
/// /user/:id
/// eg: /user/123
pub async fn user_info(Path(id): Path<i64>) -> String {
    format!("user id:{}", id)
}

/// /repo/:repo/:id
/// eg: /repo/user/daheige
pub async fn repo_info(Path((repo, name)): Path<(String, String)>) -> String {
    format!("repo:{},name:{}", repo, name)
}

// query_user?id=1&username=daheige
pub async fn query_user(Query(args): Query<user::User>) -> String {
    format!("user id:{},username:{}", args.id, args.username)
}

/// bind params to option struct
/// eg:query_user_opt?id=1&username=daheige
pub async fn query_user_opt(args: Option<Query<user::User>>) -> String {
    if let Some(args) = args {
        let user = args.0;
        return format!("user id:{},username:{}", user.id, user.username);
    }

    "query user params invalid".to_string()
}

// option params default value
// eg: /query_user_opt_done?id=1&username=daheige
pub async fn query_user_opt_done(Query(args): Query<user::UserOpt>) -> String {
    let id = args.id.unwrap_or(0);
    let username = args.username.unwrap_or("".to_string());
    format!("user id:{},username:{}", id, username)
}

/// get all query params
/// eg: /all-query?id=1&username=daheige
pub async fn all_query(headers: HeaderMap, Query(args): Query<HashMap<String, String>>) -> String {
    // get ua
    let ua = get_header(&headers, "user-agent");
    println!("ua:{}", ua);

    format!("all query:{:?}", args)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NameInput {
    #[validate(length(min = 1, message = "can not be empty"))]
    pub name: String,
}

/// validate input name
/// /validate
/// /validate?name=
/// /validate?name=daheige
pub async fn validate_name(ValidatedForm(input): ValidatedForm<NameInput>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(format!("hello,{}!", input.name)),
        }),
    )
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Payload {
    #[validate(length(min = 1, message = "can not be empty"))]
    foo: String,
}

pub async fn json_or_form(JsonOrForm(payload): JsonOrForm<Payload>) -> impl IntoResponse {
    println!("{:?}", payload);
    (
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(format!("hello,{}!", &payload.foo)),
        }),
    )
}

/// Returning different response types
/// http://localhost:1338/api/either/1
/// http://localhost:1338/api/either/2
pub async fn either_handler(Path(id): Path<i64>) -> Response {
    if id == 1 {
        return format!("user id:{}", id).into_response();
    }

    (
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(format!("hello,id:{}", id)),
        }),
    )
        .into_response()
}
