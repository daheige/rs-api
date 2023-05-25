use std::net::SocketAddr;
use std::process;
use std::time::Duration;
use tokio::signal;

// define module
mod config;
mod entity;
mod handlers;
mod middleware;
mod routes;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    println!("app_debug:{:?}", config::APP_CONFIG.app_debug);
    println!("current process pid:{}", process::id());

    let address: SocketAddr = format!("0.0.0.0:{}", config::APP_CONFIG.app_port)
        .parse()
        .unwrap();
    println!("app run on:{}", address.to_string());

    // create axum router
    let router = routes::router::api_router();

    // run app
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(graceful_shutdown())
        .await
        .unwrap();
}

// graceful shutdown
async fn graceful_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler");
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
