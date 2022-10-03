use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;

pub async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("No route {}", uri))
}

