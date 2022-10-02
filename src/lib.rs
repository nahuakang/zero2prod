use axum::extract::Form;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use tokio::signal;

#[derive(Deserialize)]
pub struct SubscribeData {
    email: String,
    name: String,
}

pub async fn subscribe(Form(subscribe_data): Form<SubscribeData>) -> impl IntoResponse {
    (
        StatusCode::OK,
        format!(
            "User {} with email {} is subscribed!",
            subscribe_data.name, subscribe_data.email,
        ),
    )
}

pub async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route {}", uri))
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

pub fn app() -> Router {
    axum::Router::new()
        .fallback(fallback)
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn test_health_check() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health_check")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(&body[..], b"Health check passed!");
    }

    #[tokio::test]
    async fn test_subscribe_returns_200_for_valid_form_data() {
        let app = app();
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/subscriptions")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_subscribe_returns_400_when_data_is_missing() {
        let test_cases = vec![
            ("name=le%20guin", "missing email"),
            ("email=ursula_le_guin%40gmail.com", "missing name"),
            ("", "missing email and name"),
        ];

        for (invalid_body, error_message) in test_cases {
            let app = app();
            let response = app
                .oneshot(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri("/subscriptions")
                        .header(
                            http::header::CONTENT_TYPE,
                            mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                        )
                        .body(Body::from(invalid_body))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "The API did not fail with 400 Bad Request when the payload was {}.",
                error_message
            );
        }
    }
}
