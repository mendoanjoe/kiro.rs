//! Anthropic API route configuration

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};

use crate::kiro::provider::KiroProvider;

use super::{
    handlers::{count_tokens, get_models, post_messages, post_messages_cc},
    middleware::{AppState, auth_middleware, cors_layer},
};

/// Maximum request body size limit (50MB)
const MAX_BODY_SIZE: usize = 50 * 1024 * 1024;

/// Create Anthropic API routes
///
/// # Endpoints
/// - `GET /v1/models` - List available models
/// - `POST /v1/messages` - Create a message (chat)
/// - `POST /v1/messages/count_tokens` - Count tokens
///
/// # Authentication
/// All `/v1` paths require API Key authentication, supports:
/// - `x-api-key` header
/// - `Authorization: Bearer <token>` header
///
/// # Parameters
/// - `api_key`: API key for authenticating client requests
/// - `kiro_provider`: optional KiroProvider for calling the upstream API

/// Create Anthropic API routes with a KiroProvider
pub fn create_router_with_provider(
    api_key: impl Into<String>,
    kiro_provider: Option<KiroProvider>,
    extract_thinking: bool,
) -> Router {
    let mut state = AppState::new(api_key, extract_thinking);
    if let Some(provider) = kiro_provider {
        state = state.with_kiro_provider(provider);
    }

    // Routes requiring authentication for /v1
    let v1_routes = Router::new()
        .route("/models", get(get_models))
        .route("/messages", post(post_messages))
        .route("/messages/count_tokens", post(count_tokens))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Routes requiring authentication for /cc/v1 (Claude Code compatible endpoint)
    // Difference from /v1: streaming waits for contextUsageEvent before sending message_start
    let cc_v1_routes = Router::new()
        .route("/messages", post(post_messages_cc))
        .route("/messages/count_tokens", post(count_tokens))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .nest("/v1", v1_routes)
        .nest("/cc/v1", cc_v1_routes)
        .layer(cors_layer())
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .with_state(state)
}
