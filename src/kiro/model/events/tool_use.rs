//! Tool use event
//!
//! Handles toolUseEvent type events

use serde::Deserialize;

use crate::kiro::parser::error::ParseResult;
use crate::kiro::parser::frame::Frame;

use super::base::EventPayload;

/// Tool use event
///
/// Contains streaming data for a tool call
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUseEvent {
    /// Tool name
    pub name: String,
    /// Tool call ID
    pub tool_use_id: String,
    /// Tool input data (JSON string, may be partial streaming data)
    #[serde(default)]
    pub input: String,
    /// Whether this is the last chunk
    #[serde(default)]
    pub stop: bool,
}

impl EventPayload for ToolUseEvent {
    fn from_frame(frame: &Frame) -> ParseResult<Self> {
        frame.payload_as_json()
    }
}

impl std::fmt::Display for ToolUseEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.stop {
            write!(
                f,
                "ToolUse[{}] (id={}, complete): {}",
                self.name, self.tool_use_id, self.input
            )
        } else {
            write!(
                f,
                "ToolUse[{}] (id={}, partial): {}",
                self.name, self.tool_use_id, self.input
            )
        }
    }
}
