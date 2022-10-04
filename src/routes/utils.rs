use crate::{
    configuration::{get_configuration, DatabaseSettings},
    startup::app,
};
use axum::http::StatusCode;
use axum::routing::Router;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

/// Utility struct for testing that contains an app router and a DB pool.
pub struct TestApp {
    pub app: Router<PgPool>,
    pub db_pool: PgPool,
}

/// Utility function to configure a DB for testing.
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres Pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database");

    pool
}

/// Utility function to spawn an app for testing.
pub async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configuration.database).await;
    let app = app(db_pool.clone());

    TestApp { app, db_pool }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
