//! Common authentication utility functions

use axum::{
    body::Body,
    http::{Request, header},
};
use subtle::ConstantTimeEq;

/// Extract API Key from request
///
/// Supports two authentication methods:
/// - `x-api-key` header
/// - `Authorization: Bearer <token>` header
pub fn extract_api_key(request: &Request<Body>) -> Option<String> {
    // Check x-api-key first
    if let Some(key) = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
    {
        return Some(key.to_string());
    }

    // Then check Authorization: Bearer
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Constant-time string comparison to prevent timing attacks
///
/// Regardless of string content, the time required for comparison is constant,
/// which prevents attackers from guessing the API Key by measuring response time.
///
/// Implemented using the security-audited `subtle` crate
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}
