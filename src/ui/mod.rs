use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::server::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_repositories: u64,
    pub total_images: u64,
    pub total_users: u64,
    pub storage_used_gb: f64,
    pub active_organizations: u64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard))
        .route("/repositories", get(repositories))
        .route("/users", get(users))
        .route("/organizations", get(organizations))
        .route("/settings", get(settings))
        .route("/api/stats", get(api_stats))
}

async fn dashboard() -> impl IntoResponse {
    Html(include_str!("templates/dashboard.html"))
}

async fn repositories() -> impl IntoResponse {
    Html(include_str!("templates/repositories.html"))
}

async fn users() -> impl IntoResponse {
    Html(include_str!("templates/users.html"))
}

async fn organizations() -> impl IntoResponse {
    Html(include_str!("templates/organizations.html"))
}

async fn settings() -> impl IntoResponse {
    Html(include_str!("templates/settings.html"))
}

async fn api_stats(State(_state): State<AppState>) -> impl IntoResponse {
    let stats = RegistryStats {
        total_repositories: 42,
        total_images: 156,
        total_users: 23,
        storage_used_gb: 12.5,
        active_organizations: 8,
    };

    axum::Json(stats)
}