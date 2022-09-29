use zero2prod::{app, shutdown_signal};

#[tokio::main]
async fn main() {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8000));

    axum::Server::bind(&addr)
        .serve(app().into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
