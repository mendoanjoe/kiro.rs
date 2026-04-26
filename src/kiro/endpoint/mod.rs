//! Kiro endpoint abstraction
//!
//! Different Kiro endpoints (e.g. `ide` / `cli`) differ in URL, request headers, and request body,
//! but share the credential pool, token refresh, retry logic, and AWS event-stream response decoding.
//!
//! [`KiroEndpoint`] abstracts the request-side differences; `KiroProvider` holds an endpoint registry
//! and selects the implementation based on the credential's `endpoint` field.

use reqwest::RequestBuilder;

use crate::kiro::model::credentials::KiroCredentials;
use crate::model::config::Config;

pub mod ide;

pub use ide::IdeEndpoint;

/// Kiro endpoint
///
/// A single `KiroProvider` can hold multiple endpoint implementations, switched via credential-level fields.
pub trait KiroEndpoint: Send + Sync {
    /// Endpoint name (corresponds to the value of credentials.endpoint / config.defaultEndpoint)
    fn name(&self) -> &'static str;

    /// API endpoint URL
    fn api_url(&self, ctx: &RequestContext<'_>) -> String;

    /// MCP endpoint URL
    fn mcp_url(&self, ctx: &RequestContext<'_>) -> String;

    /// Decorate the API request with endpoint-specific headers
    ///
    /// The provider has already set the URL, content-type, Connection, and body;
    /// the implementation is responsible for appending Authorization, host, user-agent, and other endpoint-specific headers.
    fn decorate_api(&self, req: RequestBuilder, ctx: &RequestContext<'_>) -> RequestBuilder;

    /// Decorate the MCP request with endpoint-specific headers
    fn decorate_mcp(&self, req: RequestBuilder, ctx: &RequestContext<'_>) -> RequestBuilder;

    /// Apply endpoint-specific processing to the serialized API request body (e.g. inject profileArn)
    fn transform_api_body(&self, body: &str, ctx: &RequestContext<'_>) -> String;

    /// Apply endpoint-specific processing to the serialized MCP request body (default: unchanged)
    fn transform_mcp_body(&self, body: &str, _ctx: &RequestContext<'_>) -> String {
        body.to_string()
    }

    /// Check whether the response body indicates "monthly quota exhausted" (disables and switches the credential)
    fn is_monthly_request_limit(&self, body: &str) -> bool {
        default_is_monthly_request_limit(body)
    }

    /// Check whether the response body indicates "upstream bearer token invalid" (triggers a force-refresh)
    fn is_bearer_token_invalid(&self, body: &str) -> bool {
        default_is_bearer_token_invalid(body)
    }
}

/// Context available when decorating a request
///
/// Contains all runtime information determined for a single call. Uses references to avoid unnecessary cloning.
pub struct RequestContext<'a> {
    /// Current credential
    pub credentials: &'a KiroCredentials,
    /// Effective access token (for API Key credentials this is kiroApiKey)
    pub token: &'a str,
    /// Current credential对应的 machineId
    pub machine_id: &'a str,
    /// Global configuration
    pub config: &'a Config,
}

/// Default MONTHLY_REQUEST_COUNT detection logic
///
/// Recognizes both the top-level `reason` field and the nested `error.reason` field.
pub fn default_is_monthly_request_limit(body: &str) -> bool {
    if body.contains("MONTHLY_REQUEST_COUNT") {
        return true;
    }

    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };

    if value
        .get("reason")
        .and_then(|v| v.as_str())
        .is_some_and(|v| v == "MONTHLY_REQUEST_COUNT")
    {
        return true;
    }

    value
        .pointer("/error/reason")
        .and_then(|v| v.as_str())
        .is_some_and(|v| v == "MONTHLY_REQUEST_COUNT")
}

/// Default bearer token invalid detection logic
pub fn default_is_bearer_token_invalid(body: &str) -> bool {
    body.contains("The bearer token included in the request is invalid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_monthly_request_limit_detects_reason() {
        let body = r#"{"message":"You have reached the limit.","reason":"MONTHLY_REQUEST_COUNT"}"#;
        assert!(default_is_monthly_request_limit(body));
    }

    #[test]
    fn test_default_monthly_request_limit_nested_reason() {
        let body = r#"{"error":{"reason":"MONTHLY_REQUEST_COUNT"}}"#;
        assert!(default_is_monthly_request_limit(body));
    }

    #[test]
    fn test_default_monthly_request_limit_false() {
        let body = r#"{"message":"nope","reason":"DAILY_REQUEST_COUNT"}"#;
        assert!(!default_is_monthly_request_limit(body));
    }

    #[test]
    fn test_default_bearer_token_invalid() {
        assert!(default_is_bearer_token_invalid(
            "The bearer token included in the request is invalid"
        ));
        assert!(!default_is_bearer_token_invalid("unrelated error"));
    }
}
