# rs-api
rust web api application

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

GET http://localhost:1338/html
```shell
curl --location --request GET 'localhost:1338/html'
```
response
```html
<h1>hello,rs-api</h1>
```
