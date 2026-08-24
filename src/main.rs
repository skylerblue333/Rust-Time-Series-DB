use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Deserialize)]
struct InsertRequest {
    key: String,
    value: f64,
    timestamp: u64,
}

#[derive(Deserialize)]
struct RangeQuery {
    start: u64,
    end: u64,
}

async fn insert(
    State(store): State<app::Store>,
    Json(req): Json<InsertRequest>,
) -> impl IntoResponse {
    match store.insert(&req.key, req.value, req.timestamp) {
        Ok(()) => match store.count() {
            Ok(total) => (
                StatusCode::CREATED,
                Json(json!({"status": "inserted", "total": total})),
            ),
            Err(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": error})),
            ),
        },
        Err(error) => (StatusCode::BAD_REQUEST, Json(json!({"error": error}))),
    }
}

async fn range(
    State(store): State<app::Store>,
    Path(key): Path<String>,
    Query(query): Query<RangeQuery>,
) -> impl IntoResponse {
    match store.range(&key, query.start, query.end) {
        Ok(records) => {
            let values: Vec<_> = records
                .into_iter()
                .map(|record| {
                    json!({
                        "key": record.key,
                        "value": record.value,
                        "timestamp": record.timestamp
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!({"records": values})))
        }
        Err(error) => (StatusCode::BAD_REQUEST, Json(json!({"error": error}))),
    }
}

async fn health() -> impl IntoResponse {
    Json(json!({"status": "healthy", "service": "sky-timeseries"}))
}

async fn ready(State(store): State<app::Store>) -> impl IntoResponse {
    match store.count() {
        Ok(total) => (
            StatusCode::OK,
            Json(json!({"status": "ready", "service": "sky-timeseries", "points": total})),
        ),
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": error})),
        ),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let bind = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    let app = Router::new()
        .route("/api/v1/points", post(insert))
        .route("/api/v1/series/{key}", get(range))
        .route("/healthz", get(health))
        .route("/readyz", get(ready))
        .with_state(app::Store::new());
    axum::serve(listener, app).await
}
