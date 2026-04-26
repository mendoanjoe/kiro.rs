//! AWS Event Stream parser
//!
//! Provides parsing support for the AWS Event Stream protocol,
//! for handling streaming responses from the generateAssistantResponse endpoint

pub mod crc;
pub mod decoder;
pub mod error;
pub mod frame;
pub mod header;
