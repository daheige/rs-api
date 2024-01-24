# rs-api
rust web(api)/job/rpc application

# axum version
current axum >= 0.7.4+

if you are using axum below version 0.7 please use rs-api v1 branch code

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

# get request header
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
