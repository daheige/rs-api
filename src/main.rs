use crate::config::{mysql, xredis};
use std::net::SocketAddr;
use std::process;
use std::sync::Arc;
use std::time::Duration;
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
    println!("Hello, world!");
    println!("app_debug:{:?}", config::APP_CONFIG.app_debug);
    println!("current process pid:{}", process::id());

    let address: SocketAddr = format!("0.0.0.0:{}", config::APP_CONFIG.app_port)
        .parse()
        .unwrap();
    println!("app run on:{}", address.to_string());

    // create mysql pool
    let mysql_pool = mysql::pool(&config::APP_CONFIG.mysql_conf)
        .await
        .expect("mysql pool init failed");

    // create redis pool
    let redis_pool = xredis::pool(&config::APP_CONFIG.redis_conf);
    let app_state = Arc::new(config::app::AppState {
        redis_pool: redis_pool,
        mysql_pool: mysql_pool,
    });

    // create axum router
    let router = routes::router::api_router(app_state);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(address).await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(listener, router)
        .with_graceful_shutdown(graceful_shutdown())
        .await
        .unwrap();
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
            println!("received ctrl_c signal,server will exist...");
            tokio::time::sleep(Duration::from_secs(config::APP_CONFIG.graceful_wait_time)).await;
        },
        _ = terminate => {
            println!("received terminate signal,server will exist...");
            tokio::time::sleep(Duration::from_secs(config::APP_CONFIG.graceful_wait_time)).await;
        },
    }

    println!("signal received,starting graceful shutdown");
}
