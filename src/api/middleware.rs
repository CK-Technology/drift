use crate::auth::User;
use crate::server::AppState;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use base64::{engine::general_purpose, Engine as _};
use tracing::{debug, warn};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health checks and public endpoints
    let path = request.uri().path();
    if path.starts_with("/health") || path.starts_with("/readyz") || path.starts_with("/metrics") {
        return Ok(next.run(request).await);
    }

    // Skip auth for registry version endpoint
    if path == "/v2/" {
        return Ok(next.run(request).await);
    }

    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let user = if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // JWT token authentication
            match state.auth.validate_token(token) {
                Ok(Some(user)) => Some(user),
                Ok(None) => {
                    warn!("Invalid or expired token");
                    return Err(StatusCode::UNAUTHORIZED);
                }
                Err(e) => {
                    warn!("Token validation error: {}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        } else if let Some(basic) = auth_header.strip_prefix("Basic ") {
            // Basic authentication
            match general_purpose::STANDARD.decode(basic) {
                Ok(decoded) => {
                    if let Ok(credentials) = String::from_utf8(decoded) {
                        if let Some((username, password)) = credentials.split_once(':') {
                            match state.auth.authenticate(username, password).await {
                                Ok(Some(user)) => Some(user),
                                Ok(None) => {
                                    warn!("Invalid credentials for user: {}", username);
                                    return Err(StatusCode::UNAUTHORIZED);
                                }
                                Err(e) => {
                                    warn!("Authentication error: {}", e);
                                    return Err(StatusCode::UNAUTHORIZED);
                                }
                            }
                        } else {
                            warn!("Invalid basic auth format");
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    } else {
                        warn!("Invalid basic auth encoding");
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
                Err(_) => {
                    warn!("Failed to decode basic auth");
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        } else {
            warn!("Unsupported authorization scheme");
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        // No authorization header
        warn!("Missing authorization header for path: {}", path);
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(user) = user {
        // Check scope authorization for specific operations
        let required_scope = determine_required_scope(path, request.method());
        if !state.auth.check_scope(&user, &required_scope) {
            warn!("User {} lacks required scope: {}", user.username, required_scope);
            return Err(StatusCode::FORBIDDEN);
        }

        debug!("Authenticated user: {} for path: {}", user.username, path);
        request.extensions_mut().insert(user);
    }

    Ok(next.run(request).await)
}

fn determine_required_scope(path: &str, method: &axum::http::Method) -> String {
    use axum::http::Method;

    // Parse OCI registry paths
    if let Some(captures) = regex::Regex::new(r"^/v2/([^/]+)/(manifests|blobs)/")
        .unwrap()
        .captures(path)
    {
        let repo = captures.get(1).unwrap().as_str();
        match method {
            &Method::GET | &Method::HEAD => format!("repository:{}:pull", repo),
            &Method::PUT | &Method::POST | &Method::PATCH => format!("repository:{}:push", repo),
            &Method::DELETE => format!("repository:{}:delete", repo),
            _ => format!("repository:{}:pull", repo),
        }
    } else if path.starts_with("/v2/") && path.contains("/blobs/uploads/") {
        // Upload endpoints
        if let Some(repo) = path.split('/').nth(2) {
            format!("repository:{}:push", repo)
        } else {
            "repository:*:push".to_string()
        }
    } else if path == "/v2/_catalog" {
        "registry:catalog:*".to_string()
    } else if path.starts_with("/v1/") {
        // Bolt API endpoints
        match method {
            &Method::GET => "bolt:read".to_string(),
            &Method::POST | &Method::PUT => "bolt:write".to_string(),
            &Method::DELETE => "bolt:delete".to_string(),
            _ => "bolt:read".to_string(),
        }
    } else {
        "registry:*".to_string()
    }
}

pub async fn cors_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        "*".parse().unwrap(),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH".parse().unwrap(),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "authorization, content-type, docker-content-digest, docker-upload-uuid"
            .parse()
            .unwrap(),
    );

    response
}

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    debug!(
        "{} {} - {} in {:?}",
        method, uri, status, duration
    );

    response
}