use crate::configuration::Settings;
use crate::{app, shutdown_signal};

pub async fn run(configuration: Settings) {
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], configuration.application_port));

    axum::Server::bind(&address)
        .serve(app().into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
