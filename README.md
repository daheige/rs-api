# rs-api

`rs-api` 是一个基于 Rust 与 [axum](https://github.com/tokio-rs/axum) 0.8.x 构建的轻量级 Web API 工程模板。它演示了如何在生产级场景中组织代码结构、读取 YAML 配置、管理 MySQL/Redis 连接池、实现参数校验、编写访问日志/无缓存等中间件、进行路由分组与统一响应封装，并提供优雅启停与独立任务入口。

本项目适合：

- 希望快速搭建 Rust Web 后端服务的开发者。
- 希望了解 axum + sqlx + redis 工程化实践的初学者。
- 希望作为团队内部 Rust API 服务脚手架的参考。

---

## 目录

- [核心特性](#核心特性)
- [技术栈选型](#技术栈选型)
- [架构设计与分层](#架构设计与分层)
- [目录结构](#目录结构)
- [快速开始](#快速开始)
  - [环境要求](#环境要求)
  - [本地开发](#本地开发)
  - [环境变量](#环境变量)
  - [运行测试](#运行测试)
  - [Docker 部署](#docker-部署)
- [核心组件](#核心组件)
- [配置文件说明与读取](#配置文件说明与读取)
- [错误处理](#错误处理)
- [中间件设计](#中间件设计)
- [路由规则](#路由规则)
- [Handlers 处理器](#handlers-处理器)
- [JSON 与 Form 参数校验](#json-与-form-参数校验)
- [数据库处理](#数据库处理)
- [Redis 缓存使用](#redis-缓存使用)
- [服务可观测性](#服务可观测性)
- [代码格式化与 rustfmt](#代码格式化与-rustfmt)
- [接口请求示例](#接口请求示例)
- [版本说明](#版本说明)
- [相关生态](#相关生态)
- [License](#license)

---

## 核心特性

- **axum 0.8.x 驱动**：采用最新 axum 的路由、提取器（Extractor）、中间件与状态管理，类型安全且异步友好。
- **清晰分层架构**：handlers / services / entity / infras / config / middleware / routes 职责分离，易于扩展为 DDD 或微服务。
- **统一响应封装**：所有接口统一返回 `{code, message, data}`，前端与客户端对接一致。
- **启动时配置加载**：基于标准库 `std::sync::LazyLock`（代码中别名 `Lazy`）全局加载 `app.yaml`，支持 `CONFIG_DIR` 环境变量指定配置路径。
- **MySQL 连接池**：基于 `sqlx` 实现异步 MySQL 连接池，配合 `sqlx::FromRow` 自动映射查询结果。
- **Redis 连接池**：基于 `redis` + `r2d2` 实现单机/集群 Redis 连接池，支持缓存读写与缓存穿透处理。
- **参数校验**：基于 `validator` 实现自定义 `ValidatedForm` / `JsonOrForm` 提取器，统一处理 Form/JSON 参数校验与错误响应。
- **可观测中间件**：`access_log` 自动记录请求方法、URI、Query、User-Agent、Request-Id 与执行耗时；通过 `OUTPUT_BODY` 常量控制请求/响应 body 打印；`no_cache_header` 自动禁用客户端缓存。
- **路由分组与 Fallback**：支持根路由、`/api` 前缀分组、子路由嵌套以及全局/分组 404 处理。
- **Cookie 操作示例**：演示如何设置与读取 Cookie，便于实现登录态等场景。
- **优雅关闭**：监听 `Ctrl+C` / `SIGTERM`，等待 `graceful_wait_time` 后平滑退出。
- **代码格式化**：提供 `rustfmt.toml`（stable）与 `rustfmt-nightly.toml`（nightly），统一团队代码风格。
- **独立任务入口**：`job` 二进制入口演示非 Web 场景下复用配置、数据库与缓存。

---

## 技术栈选型

| 组件           | 选型                                                                   | 说明                              |
|--------------|----------------------------------------------------------------------|---------------------------------|
| Web 框架       | [axum](https://github.com/tokio-rs/axum)                             | 基于 tokio/tower 的模块化、类型安全 Web 框架 |
| 异步运行时        | [tokio](https://github.com/tokio-rs/tokio)                           | Rust 生态主流异步运行时                  |
| 序列化          | [serde](https://github.com/serde-rs/serde) / serde_json / serde_yaml | 结构体与 JSON/YAML 互转               |
| 数据库          | [sqlx](https://github.com/launchbadge/sqlx)                          | 异步、编译期检查 SQL 的 Rust SQL 工具包     |
| Redis        | [redis](https://github.com/redis-rs/redis-rs) + r2d2                 | Redis 客户端与连接池管理                 |
| 校验           | [validator](https://github.com/Keats/validator)                      | 基于宏的结构体验证                       |
| 错误处理         | [thiserror](https://github.com/dtolnay/thiserror) + anyhow           | 结构化错误与可传播错误                     |
| UUID         | [uuid](https://github.com/uuid-rs/uuid)                              | 生成请求追踪 ID                       |
| HTTP body 工具 | [http-body-util](https://github.com/hyperium/http-body)              | 中间件中读取 body 数据                  |
| 工具库          | async-trait                                                          | 异步 trait                       |
| 异步锁/懒加载     | std::sync::LazyLock                                                   | Rust 1.80+ 标准库懒加载全局静态配置        |

---

## 架构设计与分层

项目采用经典的分层架构，自上向下职责依次为：HTTP 接入、业务编排、领域实体、基础设施与配置。各层通过明确的依赖方向协作，便于后续扩展为 DDD、微服务或多仓库结构。

### 分层架构图

```text
          HTTP Request
               │
               ▼
┌───────────────────────────────────────┐
│              routes                   │  ← 路由匹配、分组、中间件挂载
├───────────────────────────────────────┤
│            middleware                 │  ← access_log / no_cache_header / 认证
├───────────────────────────────────────┤
│              handlers                 │  ← 参数提取、响应封装、HTTP 状态码
├───────────────────────────────────────┤
│              services                 │  ← 业务逻辑编排、事务/缓存策略
├───────────────────────────────────────┤
│               entity                  │  ← DTO / 数据库模型 / 校验结构体
├───────────────────────────────────────┤
│   config  /  infras                   │  ← 配置读取、MySQL/Redis 连接池、工具函数
└───────────────────────────────────────┘
```

### 请求生命周期

以 `GET /user/1` 为例，整个请求在 rs-api 中的流转过程如下：

```mermaid
flowchart TD
    A["客户端发起请求<br/>GET /user/1"] --> B["access_log 中间件<br/>记录请求开始/注入 x-request-id"]
    B --> C{"路由匹配<br/>routes/router.rs"}
    C -->|"命中 /user/{id}"| D["handler: handlers::index::user_info<br/>提取 Path/State 参数"]
    C -->|"未命中"| E["fallback 404 处理器"]
    D --> F["service: userService::get_user_cache<br/>查询 Redis 缓存"]
    F -->|"缓存命中"| G["返回 User 数据"]
    F -->|"缓存未命中"| H["service: userService::get_user_by_id<br/>查询 MySQL 数据库"]
    H --> I["infras 层<br/>r2d2 Redis 连接池 / sqlx MySQL 连接池"]
    I --> J["数据库返回用户记录"]
    J --> K["service: set_user_cache<br/>写入 Redis 缓存"]
    K --> G
    G --> L["handler 封装响应<br/>Reply&lt;User&gt;"]
    L --> M["no_cache_header 中间件<br/>添加禁用缓存头"]
    M --> N["access_log 中间件<br/>输出执行耗时/注入响应 x-request-id"]
    N --> O["客户端收到 JSON 响应"]
    E --> O
```

#### 详细说明

1. **routes**：`routes/router.rs` 将请求匹配到 `handlers::index::user_info`。
2. **middleware**：`access_log` 记录请求开始、注入 `x-request-id`；`no_cache_header` 在响应阶段添加禁用缓存头。
3. **handlers**：`user_info` 提取路径参数 `id`，并通过 `State` 获取 `AppState`。
4. **services**：调用 `userService::get_user_cache`，未命中则查询 `userService::get_user_by_id`。
5. **infras**：通过 `r2d2` / `sqlx` 连接池访问 Redis / MySQL。
6. **services**：数据库命中后写入 Redis 缓存。
7. **handlers**：将结果封装为 `Reply<User>` 返回。
8. **middleware**：`access_log` 输出执行耗时；若 `OUTPUT_BODY` 为 `true`，还会输出响应 body。

### 模块职责

| 模块 | 职责 | 不做什么 |
|------|------|----------|
| `handlers` | 接收 HTTP 请求、提取参数、调用 service、封装响应 | 不直接读写数据库/缓存 |
| `services` | 编排业务逻辑、组合数据库/缓存操作、处理业务错误 | 不处理 HTTP 协议细节 |
| `entity` | 定义请求 DTO、响应 DTO、数据库模型、校验结构体 | 不包含业务逻辑 |
| `config` | 定义配置结构体、加载 `app.yaml`、初始化连接池 | 不处理业务配置以外的逻辑 |
| `infras` | 配置文件读取、MySQL/Redis 连接池封装、通用工具函数 | 不调用 service/handler |
| `middleware` | 访问日志、无缓存头、认证等横切逻辑 | 不执行业务逻辑 |
| `routes` | 集中管理路由映射、分组、中间件挂载、404 fallback | 不处理具体请求逻辑 |

---

## 目录结构

```
.
├── app.yaml                  # 应用配置文件
├── rustfmt.toml              # 代码格式化配置
├── Cargo.toml                # Rust 项目依赖与二进制配置
├── src
│   ├── main.rs               # Web 服务入口：加载配置、初始化连接池、启动 axum
│   ├── job.rs                # 独立任务入口：演示非 Web 场景使用 service
│   ├── config                # 配置读取与初始化
│   │   ├── mod.rs
│   │   ├── config.rs         # AppConfig 全局静态配置加载
│   │   ├── app.rs            # AppState 状态结构体
│   │   ├── mysql.rs          # MysqlConfig 与连接池初始化
│   │   └── xredis.rs         # RedisConfig 与连接池初始化
│   ├── entity                # 实体/DTO 定义
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── handlers              # HTTP 处理器函数
│   │   ├── mod.rs            # Reply/EmptyObject/EmptyArray 统一响应结构
│   │   ├── index.rs          # 核心 handler 集合
│   │   ├── validate_form.rs  # 自定义 ValidatedForm 提取器与错误处理
│   │   └── json_or_form.rs   # 同时支持 JSON/Form 的提取器
│   ├── infras                # 基础设施封装
│   │   ├── mod.rs
│   │   ├── config.rs         # YAML 文件读取 trait 与实现
│   │   ├── xmysql.rs         # MySQL 连接池 builder 封装
│   │   ├── xredis.rs         # Redis 单机/集群连接池 builder 封装
│   │   └── utils
│   │       ├── mod.rs
│   │       └── header.rs     # Header 读取工具函数
│   ├── middleware            # 中间件
│   │   ├── mod.rs
│   │   ├── request.rs        # access_log 访问日志中间件
│   │   └── header.rs         # no_cache_header 无缓存头中间件
│   ├── routes                # 路由配置
│   │   ├── mod.rs
│   │   └── router.rs         # API 路由组装
│   └── services              # 业务逻辑层
│       ├── mod.rs
│       └── user.rs           # 用户相关数据库/缓存操作
```

---

## 快速开始

### 环境要求

- Rust >= 1.75（当前使用 1.97.1，edition 2024）
- axum >= 0.8.0（当前项目使用 0.8.9）
- MySQL 与 Redis 服务（配置见 `app.yaml`）

### 本地开发

#### 1. 克隆项目并准备数据库

```shell
git clone <repo-url>
cd rs-api
```

在 MySQL 中创建 `users` 表：

```sql
CREATE TABLE users
(
    id       BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL
);
```

#### 2. 修改配置

编辑 `app.yaml` 中的 `mysql_conf` 与 `redis_conf` 为你本地或开发环境的地址。完整配置说明见 [配置文件说明与读取](#配置文件说明与读取)。

#### 3. 运行 Web 服务

```shell
cargo run --bin rs-api
```

服务启动后会输出当前 PID、监听地址与配置摘要，例如：

```text
Hello, world!
app_debug:true
current process pid:12345
app run on:0.0.0.0:1338
```

启动后访问首页：

```shell
curl http://localhost:1338/
```

响应：

```html
Hello, World!
```

#### 4. 运行独立任务

项目同时定义了 `job` 二进制入口，用于演示在非 Web 场景下复用配置、数据库与 Redis 连接：

```shell
cargo run --bin job
```

#### 5. 代码格式化

```shell
cargo fmt
```

### 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `CONFIG_DIR` | `./` | `app.yaml` 所在目录 |
| `RUST_LOG` | 无 | 可配合 `env_logger` / `tracing` 设置日志级别 |

### 运行测试

项目未编写完整单元测试，但你可以通过以下命令检查编译与格式化：

```shell
# 编译检查
cargo check

# 格式化检查
cargo fmt -- --check

# 运行所有测试（如有）
cargo test
```

### Docker 部署

项目暂未提供 `Dockerfile`，以下是一个可直接使用的最小示例：

```dockerfile
FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin rs-api

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/rs-api /app/rs-api
COPY --from=builder /app/app.yaml /app/app.yaml
EXPOSE 1338
CMD ["./rs-api"]
```

构建并运行：

```shell
docker build -t rs-api:latest .
docker run -p 1338:1338 --env CONFIG_DIR=/app rs-api:latest
```

生产部署建议将 `app.yaml` 通过 ConfigMap / 环境变量挂载，避免镜像中硬编码数据库密码。

---

## 核心组件

### AppState

`src/config/app.rs` 定义了全局共享状态，包含 MySQL 与 Redis 连接池，通过 `Arc<AppState>` 注入到路由与 handler 中：

```rust
use r2d2::Pool;

#[derive(Clone)]
pub struct AppState {
    pub mysql_pool: sqlx::MySqlPool,
    pub redis_pool: Pool<redis::Client>,
}
```

在 `main.rs` 中初始化并传入路由：

```rust
let app_state = Arc::new(config::app::AppState {
redis_pool,
mysql_pool,
});
let router = routes::router::api_router(app_state);
```

### 统一响应结构

`src/handlers/mod.rs` 定义了统一返回结构：

```rust
#[derive(Deserialize, Serialize, Debug)]
pub struct Reply<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EmptyObject {}

type EmptyArray = Vec<EmptyObject>;
```

所有接口都返回 `Reply<T>`，业务成功时 `code = 0`，失败时返回对应错误码与提示。

---

## 配置文件说明与读取

### app.yaml

```yaml
# 应用基础配置
app_name: rs-api
app_debug: true
app_port: 1338
monitor_port: 8090      # metrics 指标监控端口
graceful_wait_time: 5
log_level: info         # 日志级别：error > warn > info > debug > trace

# Redis 配置
redis_conf:
  dsn: "redis://:@127.0.0.1:6379/0"
  max_size: 200
  min_idle: 10
  max_lifetime: 1800   # 单位 s
  idle_timeout: 300    # 单位 s
  connection_timeout: 10

# MySQL 配置
mysql_conf:
  dsn: "mysql://root:root123456@127.0.0.1/test"
  max_connections: 100
  min_connections: 10
  max_lifetime: 1800
  idle_timeout: 300
  connect_timeout: 10
```

### 配置读取流程

1. `src/infras/config.rs` 提供 `ConfigTrait` 与 `Config` 实现，负责从文件路径读取 YAML 文本。
2. `src/config/config.rs` 使用标准库 `std::sync::LazyLock`（代码中别名 `Lazy`）在首次访问 `APP_CONFIG` 时加载 `app.yaml`：
    - 支持通过 `CONFIG_DIR` 环境变量指定配置文件所在目录。
    - 反序列化为 `AppConfig` 结构体。
3. `src/config/mysql.rs` 与 `src/config/xredis.rs` 分别基于 `AppConfig` 初始化 MySQL/Redis 连接池。

```rust
use std::sync::LazyLock as Lazy;

pub static APP_CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let config_dir = std::env::var("CONFIG_DIR").unwrap_or("./".to_string());
    let filename = Path::new(config_dir.as_str()).join("app.yaml");
    let c = Config::load(filename);
    let conf: AppConfig = serde_yaml::from_str(c.content()).unwrap();
    conf
});
```

### 自定义配置目录

```shell
CONFIG_DIR=/path/to/config cargo run --bin rs-api
```

---

## 错误处理

项目结合 `thiserror`、`anyhow` 与自定义提取器实现分层错误处理：

- **基础设施层**：MySQL/Redis 初始化失败直接 `panic` 或返回底层错误，因为连接池失败属于不可恢复错误。
- **Service 层**：使用 `anyhow::Result<T>` 统一包裹数据库、缓存、序列化错误，便于上层传播。
- **Handler 层**：通过 `thiserror` 定义 `ServerError`，将 `validator` 校验错误与 axum 提取错误统一转换为 HTTP 响应：

`src/handlers/validate_form.rs`：

```rust
#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(err) => {
                let msg = format!("input validation error: [{}]", err).replace('\n', ", ");
                (StatusCode::OK, Json(Reply { code: 1001, message: msg, data: Some(EmptyObject {}) }))
            }
            ServerError::AxumFormRejection(_) => {
                (StatusCode::BAD_REQUEST, Json(Reply { code: 500, message: format!("param error:{}", self), data: Some(EmptyObject {}) }))
            }
        }.into_response()
    }
}
```

业务错误（如用户已存在）通过自定义 `code/message` 返回，HTTP 状态码保持 `200 OK`，由 `code` 字段表达业务状态。

---

## 中间件设计

中间件位于 `src/middleware/`，使用 axum 的 `middleware::from_fn` 注册到路由。

### access_log 访问日志

`src/middleware/request.rs`：

- 记录请求方法、URI、Path、Query、User-Agent。
- 若请求头不存在 `x-request-id`，则自动生成 UUID 作为追踪 ID。
- 通过 `src/middleware/request.rs` 中的 `OUTPUT_BODY` 常量控制是否打印请求 body 与响应 body（默认关闭，避免不必要的 body 收集开销）。
- 统计并输出请求执行耗时。
- 将 `x-request-id` 注入响应头，便于全链路追踪。

示例输出（`OUTPUT_BODY = true` 时）：

```text
exec begin,method:GET uri:/empty-object?id=1 path:/empty-object query:Some("id=1") ua:curl/8.0 request_id:f1d720c8-2eab-408a-bd0a-41c924512d7f
request_id:f1d720c8-2eab-408a-bd0a-41c924512d7f request body = ""
exec end,request_id:f1d720c8-2eab-408a-bd0a-41c924512d7f,exec_time:0ms
request_id:f1d720c8-2eab-408a-bd0a-41c924512d7f response body = "{\"code\":0,\"message\":\"ok\",\"data\":{}}"
```

### no_cache_header 无缓存头

`src/middleware/header.rs`：

为所有响应自动添加：

```text
Cache-Control: no-cache,must-revalidate,no-store
Pragma: no-cache
Expires: -1
```

### 中间件注册顺序

`src/routes/router.rs`：

```rust
let router = Router::new()
.merge(router)
.nest("/api", api_routes)
.fallback(not_found_handler)
.layer(middleware::from_fn(ware::access_log))
.layer(middleware::from_fn(ware::no_cache_header));
```

注意：`layer` 的调用顺序与执行顺序相反，`access_log` 会先执行；越后调用的 `layer` 越靠近请求入口。

---

## 路由规则

### 基本路由

`src/routes/router.rs` 集中管理所有路由：

```rust
let router = Router::new()
.route("/", get(handlers::index::root))
.route("/users", post(handlers::index::create_user))
.route("/user/{id}", get(handlers::index::user_info))
.route("/repo/{repo}/{name}", get(handlers::index::repo_info))
.route("/form", post(handlers::index::accept_form))
.route("/validate", get(handlers::index::validate_name))
.route("/json_or_form", post(handlers::index::json_or_form))
.with_state(state1);
```

### 路由分组

使用 `nest` 实现 `/api` 前缀分组：

```rust
let api_routes = Router::new()
.nest("/user", user_router(state))
.nest("/foo", foo_router())
.route("/hello", get(handlers::index::root))
.route("/either/{id}", get(handlers::index::either_handler))
.fallback(api_not_found);

let router = Router::new()
.merge(router)
.nest("/api", api_routes)
.fallback(not_found_handler);
```

对应接口：

- `/api/user/{id}`
- `/api/user/query?id=1&username=daheige`
- `/api/foo/empty-array`
- `/api/hello`
- `/api/either/{id}`

### 404 处理

- 全局未匹配路由返回 `(StatusCode::NOT_FOUND, "this page not found")`。
- `/api/*` 分组内未匹配路由返回 JSON：`{code: 404, message: "api not found", data: {}}`。

---

## Handlers 处理器

`src/handlers/index.rs` 展示了 axum 中常见的 handler 写法：

### 返回纯文本

```rust
pub async fn root() -> &'static str {
    "Hello, World!"
}
```

### 注入状态并访问数据库

```rust
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<user::CreateUser>,
) -> Response {
    let user = userService::get_user(state.mysql_pool.clone(), &payload.username).await;
    if user.is_ok() {
        return (StatusCode::OK, Json(Reply { code: 500, message: "user already exists".to_string(), data: Some(EmptyObject {}) })).into_response();
    }
    let res = userService::create_user(state.mysql_pool.clone(), &payload.username).await;
    // ...
}
```

### 读取请求头

```rust
pub async fn accept_form(
    headers: HeaderMap,
    Form(input): Form<user::UserForm>,
) -> impl IntoResponse {
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap();
    // ...
}
```

### 路径参数与查询参数

```rust
// /user/123
pub async fn user_info(Path(id): Path<i64>, State(state): State<Arc<AppState>>) -> Response { ... }

// /repo/user/daheige
pub async fn repo_info(Path((repo, name)): Path<(String, String)>) -> String { ... }

// /query_user?id=1&username=daheige
pub async fn query_user(Query(args): Query<user::User>) -> String { ... }

// 所有查询参数以 HashMap 接收
pub async fn all_query(Query(args): Query<HashMap<String, String>>) -> String { ... }
```

### Cookie 操作

```rust
// 设置 Cookie
pub async fn set_user_cookie() -> impl IntoResponse { ... }

// 读取 Cookie
pub async fn get_user_cookie(headers: HeaderMap) -> impl IntoResponse { ... }
```

### 返回不同类型

```rust
pub async fn either_handler(Path(id): Path<i64>) -> Response {
    if id == 1 {
        return format!("user id:{}", id).into_response();
    }
    (StatusCode::OK, Json(Reply { code: 0, message: "ok".to_string(), data: Some(format!("hello,id:{}", id)) })).into_response()
}
```

---

## JSON 与 Form 参数校验

项目基于 `validator` 实现了两个自定义提取器，统一处理参数校验与错误响应。

### ValidatedForm

`src/handlers/validate_form.rs` 为 `Form<T>` 增加 `Validate` 能力：

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedForm<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Form<T>: FromRequest<S, Rejection=FormRejection>,
{
    type Rejection = ServerError;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}
```

使用示例：

```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NameInput {
    #[validate(length(min = 1, message = "can not be empty"))]
    pub name: String,
}

pub async fn validate_name(ValidatedForm(input): ValidatedForm<NameInput>) -> impl IntoResponse {
    (StatusCode::OK, Json(Reply { code: 0, message: "ok".to_string(), data: Some(format!("hello,{}!", input.name)) }))
}
```

### JsonOrForm

`src/handlers/json_or_form.rs` 根据 `Content-Type` 自动选择 JSON 或 Form 提取，并进行校验：

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonOrForm<T>(pub T);

impl<S, T> FromRequest<S> for JsonOrForm<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
    Json<T>: FromRequest<S, Rejection=JsonRejection>,
    Form<T>: FromRequest<S, Rejection=FormRejection>,
{
    type Rejection = Response;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 读取 Content-Type，分别调用 Json/Form 提取器并执行 validate()
    }
}
```

### 校验错误响应

- 校验失败：`{code: 1001, message: "input validation error: [...]", data: {}}`
- 参数缺失/反序列化失败：`{code: 500, message: "param error:...", data: {}}`

---

## 数据库处理

### MySQL 连接池封装

`src/infras/xmysql.rs` 使用 Builder 模式封装 `sqlx::MySqlPoolOptions`：

```rust
let pool = MysqlService::builder("mysql://root:root123456@127.0.0.1/test")
.with_max_connections(100)
.with_min_connections(10)
.with_max_lifetime(Duration::from_secs(1800))
.with_idle_timeout(Duration::from_secs(300))
.with_connect_timeout(Duration::from_secs(10))
.pool()
.await?;
```

### SQL 查询

`src/services/user.rs`：

```rust
// 查询单条并映射到 User 结构体
pub async fn get_user_by_id(db: sqlx::MySqlPool, id: i64) -> anyhow::Result<User> {
    let sql = "select * from users where id = ?";
    let user: User = sqlx::query_as(sql).bind(id).fetch_one(&db).await?;
    Ok(user)
}

// 插入数据并返回自增 ID
pub async fn create_user(db: sqlx::MySqlPool, username: &str) -> anyhow::Result<i64> {
    let sql = "insert into users (username) values (?)";
    let res = sqlx::query(sql).bind(username).execute(&db).await?;
    Ok(res.last_insert_id() as i64)
}

// 聚合查询
pub async fn query_user_count(db: sqlx::MySqlPool) -> anyhow::Result<u64> {
    let sql = "select count(*) as cnt from users";
    let result: (i64,) = sqlx::query_as(sql).fetch_one(&db).await?;
    Ok(result.0 as u64)
}
```

### 模型定义

`src/entity/user.rs`：

```rust
#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
}
```

通过 `sqlx::FromRow` 自动将查询结果映射到结构体。

---

## Redis 缓存使用

### Redis 连接池封装

`src/infras/xredis.rs` 支持单机与集群模式：

```rust
let pool = RedisService::builder()
.with_dsn("redis://:@127.0.0.1:6379/0")
.with_max_size(200)
.with_min_idle(10)
.with_max_lifetime(Duration::from_secs(1800))
.with_idle_timeout(Duration::from_secs(300))
.with_connect_timeout(Duration::from_secs(10))
.pool();
```

### 缓存读写示例

`src/services/user.rs`：

```rust
pub async fn set_user_cache(redis: Pool<redis::Client>, user: &User) -> anyhow::Result<()> {
    let mut conn = redis.get()?;
    let key = format!("user:{}", user.id);
    let value = serde_json::to_string(&user)?;
    conn.set(key, value)?;
    Ok(())
}

pub async fn get_user_cache(redis: Pool<redis::Client>, id: i64) -> anyhow::Result<User> {
    let mut conn = redis.get()?;
    let key = format!("user:{}", id);
    let res: String = conn.get(key)?;
    let user: User = serde_json::from_str(&res)?;
    Ok(user)
}
```

### 缓存穿透处理

在 `handlers::index::user_info` 中：

1. 先查 Redis 缓存。
2. 缓存命中直接返回。
3. 缓存未命中查 MySQL。
4. 数据库命中后写入 Redis，供下次访问使用。

---

## 服务可观测性

服务监控和告警，可基于logger/metrics/trace方式实现。

### metrics 指标采集

- metrics访问地址：localhost:8090/metrics 运行效果如下：
  ![metrics.png](metrics.png)

- 对于metrics指标采集，以及日志上报，推荐使用openobserve平台采集和查看。
- openobserve搭建和使用参考：https://github.com/daheige/rs-openobserve

### 日志输出

在 main 函数中初始化 Logger

```rust
use log::info;
use logger::Logger;

// JSON 格式，携带 caller 行号
Logger::new()
.with_caller_line()
.with_json()
.init();

info!(a = 1, b = "xxx"; "hello,world");
```

输出：

```json
{
  "ts": "2026-07-18T14:25:16Z",
  "level": "INFO",
  "module": "my_app::main",
  "caller": 7,
  "msg": "hello,world",
  "a": 1,
  "b": "xxx"
}
```

自定义时间格式：

```rust
Logger::new()
.with_json()
.with_time_format("%Y-%m-%d %H:%M:%S")
.init();
```

普通文本格式（带行号）：

```rust
Logger::new().with_caller_line().init();
info!("hello,world");
// [2026-07-18T14:25:16Z INFO my_app::main:7] hello,world
```

配合环境变量使用：

```bash
RUST_LOG=info cargo run --bin rs-api
```

---

## 代码格式化与 rustfmt

项目提供两份 rustfmt 配置，分别适配 stable 与 nightly 工具链：

### 默认 stable 配置

`rustfmt.toml` 仅使用 stable 选项，可在默认 stable 工具链下直接使用：

```toml
edition = "2024"
max_width = 100
merge_derives = true
reorder_imports = true
reorder_modules = true
```

### nightly 扩展配置

`rustfmt-nightly.toml` 包含更多格式化选项（注释换行、import 分组、注释规范化等），需要 nightly 工具链：

```toml
comment_width = 80
max_width = 100
edition = "2024"
format_code_in_doc_comments = true
format_strings = true
group_imports = "StdExternalCrate"
imports_granularity = "Crate"
normalize_comments = true
normalize_doc_attributes = true
wrap_comments = true
blank_lines_lower_bound = 0
blank_lines_upper_bound = 1
brace_style = "SameLineWhere"
merge_derives = true
```

### 配置项可用性说明

`rustfmt-nightly.toml` 中各项在 stable 工具链下的可用性如下：

| 配置项                           | 可用性       | 说明             |
|-------------------------------|-----------|----------------|
| `max_width`                   | ✅ stable  | 行最大宽度          |
| `edition`                     | ✅ stable  | Rust edition   |
| `merge_derives`               | ✅ stable  | 合并连续 derive 属性 |
| `comment_width`               | ❌ nightly | 注释最大宽度         |
| `format_code_in_doc_comments` | ❌ nightly | 格式化文档注释中的代码    |
| `format_strings`              | ❌ nightly | 自动换行长字符串       |
| `group_imports`               | ❌ nightly | import 分组策略    |
| `imports_granularity`         | ❌ nightly | import 合并粒度    |
| `normalize_comments`          | ❌ nightly | 规范化注释格式        |
| `normalize_doc_attributes`    | ❌ nightly | 规范化文档属性        |
| `wrap_comments`               | ❌ nightly | 自动换行注释         |
| `blank_lines_lower_bound`     | ❌ nightly | 最小空行数          |
| `blank_lines_upper_bound`     | ❌ nightly | 最大空行数          |
| `brace_style`                 | ❌ nightly | 大括号风格          |

因此，`rustfmt.toml` 中只保留了 `max_width`、`edition`、`merge_derives` 等 stable 选项，避免在 stable 工具链下产生警告。

### 格式化命令

```shell
# 默认使用 rustfmt.toml（stable 配置）
cargo fmt

# 检查格式
cargo fmt -- --check

# 使用 nightly 配置（需要安装 nightly 工具链）
rustup component add rustfmt --toolchain nightly
cargo +nightly fmt -- --config-path rustfmt-nightly.toml
```

### CI 建议

建议将 `cargo fmt -- --check` 加入 CI 流水线，确保提交代码风格一致。如果团队统一使用 nightly 工具链，可改用
`cargo +nightly fmt -- --check --config-path rustfmt-nightly.toml`。

---

## 接口请求示例

以下示例均在服务启动后执行，默认端口 `1338`。

### 首页

```shell
curl http://localhost:1338/
```

响应：

```html
Hello, World!
```

### Form 提交

```shell
curl --location --request POST 'http://localhost:1338/form' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'name=daheige' \
--data-urlencode 'age=30'
```

响应：

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

### 创建用户

```shell
curl --location --request POST 'http://localhost:1338/users' \
--header 'Content-Type: application/json' \
--data-raw '{"username":"daheige"}'
```

响应：

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

### 查询用户（带缓存）

```shell
curl 'http://localhost:1338/user/1'
```

### 空数组与空对象

```shell
curl http://localhost:1338/empty-array
curl http://localhost:1338/empty-object
```

### HTML 响应

```shell
curl http://localhost:1338/html
```

响应：

```html
<h1>hello,rs-api</h1>
```

### 参数校验

```shell
curl 'http://localhost:1338/validate?name='
```

响应：

```json
{
  "code": 1001,
  "message": "input validation error: [name: can not be empty]",
  "data": {}
}
```

### JSON/Form 自适应

JSON 请求：

```shell
curl --location --request POST 'http://localhost:1338/json_or_form' \
--header 'Content-Type: application/json' \
--data-raw '{"foo":"abc"}'
```

Form 请求：

```shell
curl --location --request POST 'http://localhost:1338/json_or_form' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'foo=abc'
```

响应均为：

```json
{
  "code": 0,
  "message": "ok",
  "data": "hello,abc!"
}
```

### Cookie 设置与读取

```shell
# 设置 cookie
curl -c cookies.txt http://localhost:1338/set-user-cookie

# 读取 cookie
curl -b cookies.txt http://localhost:1338/get-user-cookie
```

---

## 版本说明

- **当前 main 分支**：axum >= 0.8.0
- **axum < 0.7**：请使用 `v1` 分支
- **0.7.x <= axum < 0.8**：请使用 `v0.7.x` 分支

---

## 相关生态

- [axum](https://crates.io/crates/axum) / [GitHub](https://github.com/tokio-rs/axum)
- [tokio](https://crates.io/crates/tokio) / [GitHub](https://github.com/tokio-rs/tokio)
- [serde](https://crates.io/crates/serde) / [GitHub](https://github.com/serde-rs/serde)
- [redis](https://crates.io/crates/redis) / [GitHub](https://github.com/redis-rs/redis-rs)
- [sqlx](https://crates.io/crates/sqlx)
- Rust 定时任务：[rcron](https://github.com/rs-god/rcron)
- Rust gRPC 项目：[rs-rpc](https://github.com/daheige/rs-rpc)

---

## License

MIT
