# rs-api
rust web(api)/job/rpc application

# related crates
- axum: https://crates.io/crates/axum https://github.com/tokio-rs/axum
- tokio: https://crates.io/crates/tokio https://github.com/tokio-rs/tokio
- serde: https://crates.io/crates/serde https://github.com/serde-rs/serde
- redis: https://crates.io/crates/redis https://github.com/redis-rs/redis-rs
- rs-infras: https://github.com/rs-god/rs-infras
- rust cron job: https://github.com/rs-god/rcron
- rust grpc project: https://github.com/daheige/rs-rpc

# layout
```
.
├── config Configuration file reading and initialization
├── handlers Function handler for complex routing rules
│ └── mod.rs
├── main.rs 
├── middleware The main middleware is rpc/api middleware
│ └── mod.rs
├── routes Routing rule
│ └── mod.rs
└── services Business logic layer
└── mod.rs
```

# run
```shell
cargo run --bin rs-api
```

Visit home page: http://localhost:1338/
```html
Hello, World!
```

# api handler
POST http://localhost:1338/form
```shell
curl --location --request POST 'http://localhost:1338/form' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'name=daheige' \
--data-urlencode 'age=30'
```
response
```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "name": "daheige",
        "email": "",
        "age": 30
    }
}
```

POST http://localhost:1338/users
```shell
curl --location --request POST 'http://localhost:1338/users' \
--header 'Content-Type: application/json' \
--data-raw '{"username":"daheige"}'
```
response
```json
{
    "code": 0,
    "message": "success",
    "data": {
        "id": 1337,
        "username": "daheige"
    }
}
```

GET http://localhost:1338/empty-array
```shell
curl --location --request GET 'localhost:1338/empty-array'
```
response
```json
{
    "code": 0,
    "message": "ok",
    "data": []
}
```

GET http://localhost:1338/empty-object
```shell
curl --location --request GET 'localhost:1338/empty-object'
```
response
```json
{
    "code": 0,
    "message": "ok",
    "data": {}
}
```

GET http://localhost:1338/html
```shell
curl --location --request GET 'localhost:1338/html'
```
response
```html
<h1>hello,rs-api</h1>
```

# get header
from axum::http::HeaderMap
```rust
let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap();
println!("user-agent:{}", ua);

// eg: this code
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
```

# validate middleware for handlers
handlers/validate_form.rs
```rust
use async_trait::async_trait;
use axum::extract::{rejection::FormRejection, Form, FromRequest};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

/// impl FromRequest trait
/// these bounds are required by `async_trait`
#[async_trait]
impl<S, B, T> FromRequest<S, B> for ValidatedForm<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Form<T>: FromRequest<S, B, Rejection = FormRejection>,
{
    type Rejection = ServerError;
    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
}

/// convert the error to response
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(err) => {
                // let message = format!("input validation error: [{}]", self).replace('\n', ", ");
                let msg = format!("input validation error: [{}]", err).replace('\n', ", ");
                (
                    StatusCode::OK,
                    Json(super::Reply {
                        code: 1001,
                        message: msg,
                        data: Some(super::EmptyObject {}),
                    }),
                )
            }
            ServerError::AxumFormRejection(_) => (
                StatusCode::BAD_REQUEST,
                Json(super::Reply {
                    code: 500,
                    message: format!("param error:{}", self.to_string()),
                    data: Some(super::EmptyObject {}),
                }),
            ),
        }
        .into_response()
    }
}
```

```shell
curl --location --request GET 'http://localhost:1338/validate?name='
```
response:
```json
{
    "code": 1001,
    "message": "input validation error: [name: can not be empty]",
    "data": {}
}
```

invalid param
```shell
curl --location --request GET 'http://localhost:1338/validate'
```
response:
```json
{
  "code": 500,
  "message": "param error:Failed to deserialize form",
  "data": {}
}
```

valid param
```shell
curl --location --request GET 'http://localhost:1338/validate?name=daheige'
```
response:
```json
{
  "code": 0,
  "message": "ok",
  "data": "hello,daheige!"
}
```

# json or form handler
handlers/json_or_form.rs
```rust
use axum::{
    async_trait,
    extract::{rejection::FormRejection, rejection::JsonRejection, FromRequest},
    http::request::Request,
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    Form, Json, RequestExt,
};
use serde::de::DeserializeOwned;
use validator::Validate;

// json or form handler
pub struct JsonOrForm<T>(pub T);

#[async_trait]
impl<S, B, T> FromRequest<S, B> for JsonOrForm<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
    Json<T>: FromRequest<S, B, Rejection = JsonRejection>,
    Form<T>: FromRequest<S, B, Rejection = FormRejection>,
{
    type Rejection = Response;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let content_type_header = req.headers().get(CONTENT_TYPE);
        let content_type = content_type_header
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");

        if content_type.starts_with("application/json") {
            let Json(payload) = req
                .extract_with_state(state)
                .await
                .map_err(IntoResponse::into_response)?;
            let res = payload.validate();
            if let Err(err) = res {
                let msg = format!("input validation error: [{}]", err).replace('\n', ", ");
                return Err((
                    StatusCode::OK,
                    Json(super::Reply {
                        code: 1001,
                        message: msg,
                        data: Some(super::EmptyObject {}),
                    }),
                )
                    .into_response());
            }

            return Ok(Self(payload));
        }

        if content_type.starts_with("application/x-www-form-urlencoded") {
            let Form(payload) = req
                .extract_with_state(state)
                .await
                .map_err(IntoResponse::into_response)?;
            let res = payload.validate();
            if let Err(err) = res {
                let msg = format!("input validation error: [{}]", err).replace('\n', ", ");
                return Err((
                    StatusCode::OK,
                    Json(super::Reply {
                        code: 1001,
                        message: msg,
                        data: Some(super::EmptyObject {}),
                    }),
                )
                    .into_response());
            }

            return Ok(Self(payload));
        }

        Err((
            StatusCode::OK,
            Json(super::Reply {
                code: 500,
                message: format!("content-type:{} error", content_type),
                data: Some(super::EmptyObject {}),
            }),
        )
            .into_response())
    }
}
```

json request:
```shell
curl --location --request POST 'http://localhost:1338/json_or_form' \
--header 'Content-Type: application/json' \
--data-raw '{
"foo":"abc"
}'
```
response:
```json
{
    "code": 0,
    "message": "ok",
    "data": "hello,abc!"
}
```

form request:
```shell
curl --location --request POST 'http://localhost:1338/json_or_form' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'foo=abc'
```
response:
```json
{
  "code": 0,
  "message": "ok",
  "data": "hello,abc!"
}
```

invalid request
```shell
curl --location --request POST 'http://localhost:1338/json_or_form' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'foo='
```
response:
```json
{
    "code": 1001,
    "message": "input validation error: [foo: can not be empty]",
    "data": {}
}
```

# handlers usage
please see handlers/index.rs

# access_log middleware
```
exec begin method:GET uri:/empty-object?id=1 path:/empty-object request body:Body(Empty) query:Some("id=1") ua:PostmanRuntime/7.26.8 request_id:f1d720c8-2eab-408a-bd0a-41c924512d7f
exec end,request_id:f1d720c8-2eab-408a-bd0a-41c924512d7f,exec_time:0ms
```

# License
MIT
