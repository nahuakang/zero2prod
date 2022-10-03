pub mod configuration;
pub mod routes;
pub mod startup;

use axum::routing::{get, post};
use axum::Router;
use tokio::signal;

pub async fn shutdown_signal() {
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

pub fn app() -> Router {
    axum::Router::new()
        .fallback(routes::fallback)
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
}
