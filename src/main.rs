use axum::handler::Handler;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::signal;

pub async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route {}", uri))
}

pub async fn greet_name(axum::extract::Path(name): axum::extract::Path<String>) -> String {
    format!("Hello {}!", name)
}

pub async fn greet() -> String {
    "Hello, World!".into()
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Health check passed!".to_string())
}

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

#[tokio::main]
pub async fn main() {
    let app = axum::Router::new()
        .fallback(fallback.into_service())
        .route("/health_check", get(health_check))
        .route("/:name", get(greet_name))
        .route("/", get(greet));

    axum::Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
