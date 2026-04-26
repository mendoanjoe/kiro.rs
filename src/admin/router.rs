//! Admin API route configuration

use axum::{
    Router, middleware,
    routing::{delete, get, post},
};

use super::{
    handlers::{
        add_credential, delete_credential, force_refresh_token, get_all_credentials,
        get_credential_balance, get_load_balancing_mode, reset_failure_count,
        set_credential_disabled, set_credential_priority, set_load_balancing_mode,
    },
    middleware::{AdminState, admin_auth_middleware},
};

/// Create Admin API routes
///
/// # Endpoints
/// - `GET /credentials` - Get all credential statuses
/// - `POST /credentials` - Add a new credential
/// - `DELETE /credentials/:id` - Delete a credential
/// - `POST /credentials/:id/disabled` - Set credential disabled state
/// - `POST /credentials/:id/priority` - Set credential priority
/// - `POST /credentials/:id/reset` - Reset failure count
/// - `POST /credentials/:id/refresh` - Force-refresh token
/// - `GET /credentials/:id/balance` - Get credential balance
/// - `GET /config/load-balancing` - Get load balancing mode
/// - `PUT /config/load-balancing` - Set load balancing mode
///
/// # Authentication
/// Requires Admin API Key authentication, supports:
/// - `x-api-key` header
/// - `Authorization: Bearer <token>` header
pub fn create_admin_router(state: AdminState) -> Router {
    Router::new()
        .route(
            "/credentials",
            get(get_all_credentials).post(add_credential),
        )
        .route("/credentials/{id}", delete(delete_credential))
        .route("/credentials/{id}/disabled", post(set_credential_disabled))
        .route("/credentials/{id}/priority", post(set_credential_priority))
        .route("/credentials/{id}/reset", post(reset_failure_count))
        .route("/credentials/{id}/refresh", post(force_refresh_token))
        .route("/credentials/{id}/balance", get(get_credential_balance))
        .route(
            "/config/load-balancing",
            get(get_load_balancing_mode).put(set_load_balancing_mode),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ))
        .with_state(state)
}
