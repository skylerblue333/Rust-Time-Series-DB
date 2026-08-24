use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
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

async fn insert(store: web::Data<app::Store>, req: web::Json<InsertRequest>) -> impl Responder {
    match store.insert(&req.key, req.value, req.timestamp) {
        Ok(()) => match store.count() {
            Ok(total) => HttpResponse::Created().json(serde_json::json!({
                "status": "inserted",
                "total": total
            })),
            Err(error) => {
                HttpResponse::InternalServerError().json(serde_json::json!({ "error": error }))
            }
        },
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({ "error": error })),
    }
}

async fn range(
    store: web::Data<app::Store>,
    path: web::Path<String>,
    query: web::Query<RangeQuery>,
) -> impl Responder {
    match store.range(&path.into_inner(), query.start, query.end) {
        Ok(records) => {
            let values: Vec<_> = records
                .into_iter()
                .map(|record| {
                    serde_json::json!({
                        "key": record.key,
                        "value": record.value,
                        "timestamp": record.timestamp
                    })
                })
                .collect();
            HttpResponse::Ok().json(serde_json::json!({ "records": values }))
        }
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({ "error": error })),
    }
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "healthy", "service": "sky-timeseries" }))
}

async fn ready(store: web::Data<app::Store>) -> impl Responder {
    match store.count() {
        Ok(total) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": "sky-timeseries",
            "points": total
        })),
        Err(error) => {
            HttpResponse::ServiceUnavailable().json(serde_json::json!({ "error": error }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let store = app::Store::new();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(store.clone()))
            .route("/api/v1/points", web::post().to(insert))
            .route("/api/v1/series/{key}", web::get().to(range))
            .route("/healthz", web::get().to(health))
            .route("/readyz", web::get().to(ready))
    })
    .bind(bind)?
    .run()
    .await
}
