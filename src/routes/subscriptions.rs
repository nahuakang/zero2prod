use axum::extract::Form;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

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

#[cfg(test)]
mod test {
    use crate::{app, configuration::get_configuration};
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use sqlx::{Connection, PgConnection};
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn test_subscribe_returns_200_for_valid_form_data() {
        let app = app();

        let configuration = get_configuration().expect("Failed to read configuration");
        let connection_string = configuration.database.connection_string();
        let mut connection = PgConnection::connect(&connection_string)
            .await
            .expect("Failed to connect to Postgres.");

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
            .fetch_one(&mut connection)
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
