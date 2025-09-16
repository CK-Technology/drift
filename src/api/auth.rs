use crate::auth::User;
use crate::server::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/whoami", get(whoami))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Login attempt for user: {}", request.username);

    match state.auth.authenticate(&request.username, &request.password).await {
        Ok(Some(user)) => {
            let expires_in = 24 * 60 * 60; // 24 hours in seconds
            match state.auth.generate_token(&user, expires_in) {
                Ok(token) => {
                    info!("Successful login for user: {}", user.username);
                    Ok(Json(LoginResponse {
                        token,
                        user,
                        expires_in,
                    }))
                }
                Err(e) => {
                    warn!("Failed to generate token: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => {
            warn!("Invalid credentials for user: {}", request.username);
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(e) => {
            warn!("Authentication error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn register(
    State(_state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Registration attempt for user: {}", request.username);

    // TODO: Implement user registration
    // For now, return not implemented
    warn!("User registration not implemented");
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn refresh_token(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // TODO: Implement token refresh
    warn!("Token refresh not implemented");
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn logout(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // TODO: Implement token invalidation
    Ok(Json(json!({
        "message": "Logged out successfully"
    })))
}

pub async fn whoami(
    user: Option<axum::Extension<User>>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(axum::Extension(user)) = user {
        Ok(Json(user))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}