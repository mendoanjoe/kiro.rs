//! Anthropic API middleware

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};

use crate::common::auth;
use crate::kiro::provider::KiroProvider;

use super::types::ErrorResponse;

/// Application shared state
#[derive(Clone)]
pub struct AppState {
    /// API key
    pub api_key: String,
    /// Kiro Provider (optional, used for actual API calls)
    /// Internally uses MultiTokenManager, which supports thread-safe multi-credential management
    pub kiro_provider: Option<Arc<KiroProvider>>,
    /// Whether to enable thinking block extraction for non-streaming responses
    pub extract_thinking: bool,
}

impl AppState {
    /// Create new application state
    pub fn new(api_key: impl Into<String>, extract_thinking: bool) -> Self {
        Self {
            api_key: api_key.into(),
            kiro_provider: None,
            extract_thinking,
        }
    }

    /// Set the KiroProvider
    pub fn with_kiro_provider(mut self, provider: KiroProvider) -> Self {
        self.kiro_provider = Some(Arc::new(provider));
        self
    }
}

/// API Key authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    match auth::extract_api_key(&request) {
        Some(key) if auth::constant_time_eq(&key, &state.api_key) => next.run(request).await,
        _ => {
            let error = ErrorResponse::authentication_error();
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        }
    }
}

/// CORS middleware layer
///
/// **Security Note**: The current configuration allows all origins (Any) to support public API services.
/// For stricter security, configure specific allowed origins, methods, and headers as needed.
///
/// # Configuration Notes
/// - `allow_origin(Any)`: allows requests from any origin
/// - `allow_methods(Any)`: allows any HTTP method
/// - `allow_headers(Any)`: allows any request header
pub fn cors_layer() -> tower_http::cors::CorsLayer {
    use tower_http::cors::{Any, CorsLayer};

    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
