use crate::entity::user;
use crate::services::user as userService;
use axum::http::{header, HeaderMap};
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
// pub async fn accept_form(Form(input): Form<user::UserForm>) -> impl IntoResponse {
pub async fn accept_form(
    headers: HeaderMap,
    Form(input): Form<user::UserForm>,
) -> impl IntoResponse {
    println!("headers: {:?}", headers);
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
