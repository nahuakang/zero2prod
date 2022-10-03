use axum::body::Body;
use axum::routing::{get, post};
use axum::Router;
use sqlx::{
    pool::Pool,
    postgres::{PgPoolOptions, Postgres},
};
use std::time::Duration;
use tokio::signal;

use crate::configuration::get_configuration;
use crate::routes;

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

pub async fn connection_pool(connection_string: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(connection_string)
        .await
        .expect("can connect to database")
}

pub fn app(pool: Pool<Postgres>) -> Router<Pool<Postgres>, Body> {
    axum::Router::with_state(pool)
        .fallback(routes::fallback)
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
}

pub async fn run() {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = std::net::SocketAddr::from(([127, 0, 0, 1], configuration.application_port));

    // setup connection pool
    let pool = connection_pool(&configuration.database.connection_string()).await;

    axum::Server::bind(&address)
        .serve(app(pool).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
