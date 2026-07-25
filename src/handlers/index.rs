use super::json_or_form::JsonOrForm;
use super::validate_form::ValidatedForm;
use crate::config::app::AppState;
use crate::entity::user;
use crate::infras::utils::get_header;
use crate::services::user as userService;
use axum::extract::State;
use axum::http::{HeaderMap, header};
use axum::response::Response;
use axum::{
    Form, Json,
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
// validate error
use crate::entity::user::User;
use autometrics::autometrics;
use monitor::metrics::API_SLO;
use validator::Validate;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

// create user
#[autometrics(objective = API_SLO)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<user::CreateUser>,
) -> Response {
    // query current user
    let user = userService::get_user(state.mysql_pool.clone(), &payload.username).await;
    if user.is_ok() {
        return (
            StatusCode::OK,
            Json(super::Reply {
                code: 500,
                message: "user already exists".to_string(),
                data: Some(super::EmptyObject {}),
            }),
        )
            .into_response();
    }

    // create user
    let res = userService::create_user(state.mysql_pool.clone(), &payload.username).await;
    if res.is_err() {
        println!("create user error:{}", res.err().unwrap());
        return (
            StatusCode::OK,
            Json(super::Reply {
                code: 500,
                message: "failed to create user".to_string(),
                data: Some(super::EmptyObject {}),
            }),
        )
            .into_response();
    }

    let id = res.unwrap();
    let user = User {
        id: id as u64,
        username: payload.username,
    };

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
        .into_response()
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
#[autometrics(objective = API_SLO)]
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
#[autometrics(objective = API_SLO)]
pub async fn user_info(Path(id): Path<i64>, State(state): State<Arc<AppState>>) -> Response {
    let res = userService::get_user_cache(state.redis_pool.clone(), id).await;
    if res.is_ok() {
        let user = res.unwrap();
        println!("user cache hit");
        println!("user:{:?}", user);
        return (
            StatusCode::OK,
            Json(super::Reply {
                code: 0,
                message: "ok".to_string(),
                data: Some(user),
            }),
        )
            .into_response();
    }

    println!("user cache not hit");
    // query current user
    let res = userService::get_user_by_id(state.mysql_pool.clone(), id).await;
    if res.is_err() {
        // 用户不存在
        return (
            StatusCode::OK,
            Json(super::Reply {
                code: 500,
                message: "user not found".to_string(),
                data: Some(super::EmptyObject {}),
            }),
        )
            .into_response();
    }

    let user = res.unwrap();
    println!("get user:{:?}", user);

    // 设置缓存
    let _ = userService::set_user_cache(state.redis_pool.clone(), &user).await;
    (
        StatusCode::OK,
        Json(super::Reply {
            code: 0,
            message: "ok".to_string(),
            data: Some(user),
        }),
    )
        .into_response()
}

/// /repo/:repo/:id
/// eg: /repo/user/daheige
pub async fn repo_info(Path((repo, name)): Path<(String, String)>) -> String {
    format!("repo:{},name:{}", repo, name)
}

// query_user?id=1&username=daheige
#[autometrics(objective = API_SLO)]
pub async fn query_user(Query(args): Query<user::User>) -> String {
    format!("user id:{},username:{}", args.id, args.username)
}

/// bind params to option struct
/// eg:query_user_opt?id=1&username=daheige
#[autometrics(objective = API_SLO)]
pub async fn query_user_opt(user: Query<user::User>) -> String {
    if user.id.gt(&0) && user.username.ne("") {
        return format!("user id:{},username:{}", user.id, user.username);
    }

    "query user params invalid".to_string()
}

// option params default value
// eg: /query_user_opt_done?id=1&username=daheige
#[autometrics(objective = API_SLO)]
pub async fn query_user_opt_done(Query(args): Query<user::UserOpt>) -> String {
    let id = args.id.unwrap_or(0);
    let username = args.username.unwrap_or("".to_string());
    format!("user id:{},username:{}", id, username)
}

/// get all query params
/// eg: /all-query?id=1&username=daheige
#[autometrics(objective = API_SLO)]
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
#[autometrics(objective = API_SLO)]
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
