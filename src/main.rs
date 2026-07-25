use crate::config::{mysql, xredis};
use log::info;
use logger::Logger;
use monitor::metrics::prometheus_init;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, process};
use tokio::net::TcpListener;
use tokio::signal;

// define module
mod config;
mod entity;
mod handlers;
mod infras;
mod middleware;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    // 初始化日志 logger，日志级别通过环境变量 RUST_LOG 控制
    // 优先级：error > warn > info > debug > trace
    if !config::APP_CONFIG.log_level.is_empty() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }

    // JSON 格式，携带 caller 行号
    Logger::new().with_caller_line().with_json().init();

    info!("app_debug:{:?}", config::APP_CONFIG.app_debug);
    info!("current process pid:{}", process::id());

    let address: SocketAddr = format!("0.0.0.0:{}", config::APP_CONFIG.app_port)
        .parse()
        .unwrap();
    info!("app run on:{}", address.to_string());

    // create mysql pool
    let mysql_pool = mysql::pool(&config::APP_CONFIG.mysql_conf)
        .await
        .expect("mysql pool init failed");

    // create redis pool
    let redis_pool = xredis::pool(&config::APP_CONFIG.redis_conf);
    let app_state = Arc::new(config::app::AppState {
        redis_pool,
        mysql_pool,
    });

    // create axum router
    let router = routes::router::api_router(app_state);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(address).await.unwrap();

    // http handler
    let http_handler = tokio::spawn(async move {
        // Run the server with graceful shutdown
        axum::serve(listener, router)
            .with_graceful_shutdown(graceful_shutdown())
            .await
            .expect("failed to start http service");
    });

    // metrics
    let metrics_server = prometheus_init(config::APP_CONFIG.monitor_port);
    let metrics_handler = tokio::spawn(metrics_server);

    // start http and metrics service
    let _ = tokio::try_join!(http_handler, metrics_handler)
        .expect("failed to start http service and metrics service");
}

// graceful shutdown
async fn graceful_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c =>{
            info!("received ctrl_c signal,server will exist...");
            tokio::time::sleep(Duration::from_secs(config::APP_CONFIG.graceful_wait_time)).await;
        },
        _ = terminate => {
             info!("received terminate signal,server will exist...");
            tokio::time::sleep(Duration::from_secs(config::APP_CONFIG.graceful_wait_time)).await;
        },
    }

    info!("signal received,starting graceful shutdown");
}
