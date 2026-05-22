use axum::{Router, Json, http::StatusCode, routing::get};
use serde_json::json;
use std::sync::Arc;
use crate::adapters::metrics::health_endpoint::HealthReporter;
use crate::core::ports::health_port::IHealthReporter;

pub async fn start_http_server(
    port: u16,
    health_reporter: Arc<HealthReporter>,
) -> tokio::task::JoinHandle<()> {
    let health_reporter_clone = health_reporter.clone();
    let app = Router::new()
        .route("/health", get(move || health_handler(health_reporter_clone)));

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("HTTP server starting on {}", addr);

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind HTTP server: {}", e);
                return;
            }
        };
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("HTTP server error: {}", e);
        }
    })
}

async fn health_handler(health_reporter: Arc<HealthReporter>) -> (StatusCode, Json<serde_json::Value>) {
    if health_reporter.is_healthy() {
        (
            StatusCode::OK,
            Json(json!({
                "status": health_reporter.get_status(),
                "service": "execution",
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": health_reporter.get_status(),
                "service": "execution",
            })),
        )
    }
}
