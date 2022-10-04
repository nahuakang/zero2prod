use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use serde::Deserialize;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::routes::utils::internal_error;

#[derive(Deserialize)]
pub struct SubscriptionData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(subscription_data): Form<SubscriptionData>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        subscription_data.email,
        subscription_data.name,
        Utc::now(),
    )
    .execute(&pool)
    .await
    .map_err(internal_error)
    {
        Ok(_) => (
            StatusCode::OK,
            format!(
                "User {} with email {} is subscribed!",
                subscription_data.name, subscription_data.email,
            ),
        ),
        Err(e) => e,
    }
}

#[cfg(test)]
mod test {
    use crate::routes::utils::spawn_app;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn test_subscribe_returns_200_for_valid_form_data() {
        let test_app = spawn_app().await;
        let app = test_app.app;
        let pool = test_app.db_pool;

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

        let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch saved subscription.");

        assert_eq!(saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(saved.name, "le guin");
    }

    #[tokio::test]
    async fn test_subscribe_returns_400_when_data_is_missing() {
        let test_cases = vec![
            ("name=le%20guin", "missing email"),
            ("email=ursula_le_guin%40gmail.com", "missing name"),
            ("", "missing email and name"),
        ];

        for (invalid_body, error_message) in test_cases {
            let test_app = spawn_app().await;
            let app = test_app.app;

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
