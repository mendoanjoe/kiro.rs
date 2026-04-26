//! Anthropic API compatible service module
//!
//! Provides HTTP service endpoints compatible with the Anthropic Claude API.
//!
//! # Supported Endpoints
//!
//! ## Standard Endpoints (/v1)
//! - `GET /v1/models` - List available models
//! - `POST /v1/messages` - Create a message (chat)
//! - `POST /v1/messages/count_tokens` - Count tokens
//!
//! ## Claude Code Compatible Endpoints (/cc/v1)
//! - `POST /cc/v1/messages` - Create a message (streaming waits for contextUsageEvent before sending message_start, ensuring accurate input_tokens)
//! - `POST /cc/v1/messages/count_tokens` - Count tokens (same as /v1)
//!
//! # Usage示例
//! ```rust,ignore
//! use kiro_rs::anthropic;
//!
//! let app = anthropic::create_router("your-api-key");
//! let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
//! axum::serve(listener, app).await?;
//! ```

mod converter;
mod handlers;
mod middleware;
mod router;
mod stream;
pub mod types;
mod websearch;

pub use router::create_router_with_provider;
