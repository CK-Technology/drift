use axum::{response::Response, http::StatusCode};
use std::time::Instant;
use tracing::info;

pub struct Metrics {
    pub requests_total: u64,
    pub requests_duration_seconds: f64,
    pub registry_uploads_total: u64,
    pub registry_downloads_total: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            requests_total: 0,
            requests_duration_seconds: 0.0,
            registry_uploads_total: 0,
            registry_downloads_total: 0,
        }
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_requests(&mut self) {
        self.requests_total += 1;
    }

    pub fn record_request_duration(&mut self, duration: f64) {
        self.requests_duration_seconds = duration;
    }

    pub fn increment_uploads(&mut self) {
        self.registry_uploads_total += 1;
    }

    pub fn increment_downloads(&mut self) {
        self.registry_downloads_total += 1;
    }

    pub fn export_prometheus(&self) -> String {
        format!(
            "# HELP requests_total Total number of HTTP requests\n\
             # TYPE requests_total counter\n\
             requests_total {}\n\
             # HELP requests_duration_seconds Duration of HTTP requests in seconds\n\
             # TYPE requests_duration_seconds gauge\n\
             requests_duration_seconds {}\n\
             # HELP registry_uploads_total Total number of registry uploads\n\
             # TYPE registry_uploads_total counter\n\
             registry_uploads_total {}\n\
             # HELP registry_downloads_total Total number of registry downloads\n\
             # TYPE registry_downloads_total counter\n\
             registry_downloads_total {}\n",
            self.requests_total,
            self.requests_duration_seconds,
            self.registry_uploads_total,
            self.registry_downloads_total
        )
    }
}

pub async fn metrics_handler() -> Response {
    let metrics = Metrics::new(); // In a real implementation, this would be shared state
    let prometheus_output = metrics.export_prometheus();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(prometheus_output.into())
        .unwrap()
}