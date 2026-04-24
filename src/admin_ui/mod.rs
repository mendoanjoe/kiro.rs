//! Admin UI static file serving module
//!
//! Embeds frontend build artifacts using rust-embed

mod router;

pub use router::create_admin_ui_router;
