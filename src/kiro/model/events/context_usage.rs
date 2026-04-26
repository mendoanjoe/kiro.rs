//! Context usage event
//!
//! Handles contextUsageEvent type events

use serde::Deserialize;

use crate::kiro::parser::error::ParseResult;
use crate::kiro::parser::frame::Frame;

use super::base::EventPayload;

/// Context usage event
///
/// Contains the usage percentage of the current context window
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextUsageEvent {
    /// Context usage percentage (0-100)
    #[serde(default)]
    pub context_usage_percentage: f64,
}

impl EventPayload for ContextUsageEvent {
    fn from_frame(frame: &Frame) -> ParseResult<Self> {
        frame.payload_as_json()
    }
}

impl ContextUsageEvent {
    /// Get the formatted percentage string
    pub fn formatted_percentage(&self) -> String {
        format!("{:.2}%", self.context_usage_percentage)
    }
}

impl std::fmt::Display for ContextUsageEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.formatted_percentage())
    }
}
