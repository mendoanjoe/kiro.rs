//! Admin API module
//!
//! HTTP API for credential management and monitoring
//!
//! # Features
//! - Query all credential statuses
//! - Enable/disable credentials
//! - Modify credential priority
//! - Reset failure counts
//! - Query credential balance
//!
//! # Usage
//! ```ignore
//! let admin_service = AdminService::new(token_manager.clone(), endpoint_names);
//! let admin_state = AdminState::new(admin_api_key, admin_service);
//! let admin_router = create_admin_router(admin_state);
//! ```

mod error;
mod handlers;
mod middleware;
mod router;
mod service;
pub mod types;

pub use middleware::AdminState;
pub use router::create_admin_router;
pub use service::AdminService;
