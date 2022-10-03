use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Health check passed!".to_string())
}

#[cfg(test)]
mod test {
    use crate::app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
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
}
