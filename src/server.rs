use crate::{api, auth::AuthService, config::Config, storage::StorageBackend, ui};
use anyhow::Result;
use axum::{
    extract::Extension,
    http::{header, Method},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub storage: Arc<dyn StorageBackend>,
    pub auth: Arc<AuthService>,
}

pub struct Server {
    config: Config,
    api_addr: String,
    ui_addr: String,
}

impl Server {
    pub async fn new(config: Config, api_addr: &str, ui_addr: &str) -> Result<Self> {
        Ok(Self {
            config,
            api_addr: api_addr.to_string(),
            ui_addr: ui_addr.to_string(),
        })
    }

    pub async fn run(self) -> Result<()> {
        // Initialize storage backend
        let storage = crate::storage::create_storage_backend(&self.config.storage).await?;

        // Initialize auth service
        let auth = Arc::new(AuthService::new(&self.config.auth)?);

        // Create shared app state
        let state = AppState {
            config: self.config.clone(),
            storage,
            auth,
        };

        // Create registry API router
        let api_router = self.create_api_router(state.clone());

        // Create UI router
        let ui_router = self.create_ui_router(state.clone());

        // Start both servers concurrently
        let api_listener = TcpListener::bind(&self.api_addr).await?;
        let ui_listener = TcpListener::bind(&self.ui_addr).await?;

        info!("🚀 Registry API listening on {}", self.api_addr);
        info!("🖥️  Web UI listening on {}", self.ui_addr);

        tokio::try_join!(
            axum::serve(api_listener, api_router),
            axum::serve(ui_listener, ui_router),
        )?;

        Ok(())
    }

    fn create_api_router(&self, state: AppState) -> Router {
        Router::new()
            .nest("/v2", api::registry::router())
            .nest("/v1", api::bolt::router())
            .route("/health", axum::routing::get(health_check))
            .route("/readyz", axum::routing::get(readiness_check))
            .route("/metrics", axum::routing::get(metrics_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CompressionLayer::new())
                    .layer(
                        CorsLayer::new()
                            .allow_origin(Any)
                            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
                    )
                    .layer(Extension(state)),
            )
    }

    fn create_ui_router(&self, state: AppState) -> Router {
        Router::new()
            .nest("/", ui::router())
            .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CompressionLayer::new())
                    .layer(Extension(state)),
            )
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> &'static str {
    // TODO: Check storage and auth service health
    "Ready"
}

async fn metrics_handler() -> &'static str {
    // TODO: Implement Prometheus metrics
    "# TYPE drift_info counter\ndrift_info{version=\"0.1.0\"} 1\n"
}