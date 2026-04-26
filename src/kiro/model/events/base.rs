//! Base event definitions
//!
//! Defines the event type enum, trait, and unified event structure

use crate::kiro::parser::error::{ParseError, ParseResult};
use crate::kiro::parser::frame::Frame;

/// Event type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Assistant response event
    AssistantResponse,
    /// Tool use event
    ToolUse,
    /// Metering event
    Metering,
    /// Context usage event
    ContextUsage,
    /// Unknown event type
    Unknown,
}

impl EventType {
    /// Parse from an event type string
    pub fn from_str(s: &str) -> Self {
        match s {
            "assistantResponseEvent" => Self::AssistantResponse,
            "toolUseEvent" => Self::ToolUse,
            "meteringEvent" => Self::Metering,
            "contextUsageEvent" => Self::ContextUsage,
            _ => Self::Unknown,
        }
    }

    /// Convert to an event type string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AssistantResponse => "assistantResponseEvent",
            Self::ToolUse => "toolUseEvent",
            Self::Metering => "meteringEvent",
            Self::ContextUsage => "contextUsageEvent",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Event payload trait
///
/// All concrete event types must implement this trait
pub trait EventPayload: Sized {
    /// Parse the event payload from a frame
    fn from_frame(frame: &Frame) -> ParseResult<Self>;
}

/// Unified event enum
///
/// Encapsulates all possible event types
#[derive(Debug, Clone)]
pub enum Event {
    /// Assistant response
    AssistantResponse(super::AssistantResponseEvent),
    /// Tool use
    ToolUse(super::ToolUseEvent),
    /// Metering
    Metering(()),
    /// Context usage
    ContextUsage(super::ContextUsageEvent),
    /// Unknown event (preserves the raw frame data)
    Unknown {},
    /// Server error
    Error {
        /// Error code
        error_code: String,
        /// Error message
        error_message: String,
    },
    /// Server exception
    Exception {
        /// Exception type
        exception_type: String,
        /// Exception message
        message: String,
    },
}

impl Event {
    /// Parse an event from a frame
    pub fn from_frame(frame: Frame) -> ParseResult<Self> {
        let message_type = frame.message_type().unwrap_or("event");

        match message_type {
            "event" => Self::parse_event(frame),
            "error" => Self::parse_error(frame),
            "exception" => Self::parse_exception(frame),
            other => Err(ParseError::InvalidMessageType(other.to_string())),
        }
    }

    /// Parse an event-type message
    fn parse_event(frame: Frame) -> ParseResult<Self> {
        let event_type_str = frame.event_type().unwrap_or("unknown");
        let event_type = EventType::from_str(event_type_str);

        match event_type {
            EventType::AssistantResponse => {
                let payload = super::AssistantResponseEvent::from_frame(&frame)?;
                Ok(Self::AssistantResponse(payload))
            }
            EventType::ToolUse => {
                let payload = super::ToolUseEvent::from_frame(&frame)?;
                Ok(Self::ToolUse(payload))
            }
            EventType::Metering => Ok(Self::Metering(())),
            EventType::ContextUsage => {
                let payload = super::ContextUsageEvent::from_frame(&frame)?;
                Ok(Self::ContextUsage(payload))
            }
            EventType::Unknown => Ok(Self::Unknown {}),
        }
    }

    /// Parse an error-type message
    fn parse_error(frame: Frame) -> ParseResult<Self> {
        let error_code = frame
            .headers
            .error_code()
            .unwrap_or("UnknownError")
            .to_string();
        let error_message = frame.payload_as_str();

        Ok(Self::Error {
            error_code,
            error_message,
        })
    }

    /// Parse an exception-type message
    fn parse_exception(frame: Frame) -> ParseResult<Self> {
        let exception_type = frame
            .headers
            .exception_type()
            .unwrap_or("UnknownException")
            .to_string();
        let message = frame.payload_as_str();

        Ok(Self::Exception {
            exception_type,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(
            EventType::from_str("assistantResponseEvent"),
            EventType::AssistantResponse
        );
        assert_eq!(EventType::from_str("toolUseEvent"), EventType::ToolUse);
        assert_eq!(EventType::from_str("meteringEvent"), EventType::Metering);
        assert_eq!(
            EventType::from_str("contextUsageEvent"),
            EventType::ContextUsage
        );
        assert_eq!(EventType::from_str("unknown_type"), EventType::Unknown);
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(
            EventType::AssistantResponse.as_str(),
            "assistantResponseEvent"
        );
        assert_eq!(EventType::ToolUse.as_str(), "toolUseEvent");
    }
}
