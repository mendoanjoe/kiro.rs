//! Debug utilities module
//!
//! Provides hex printing and CRC debugging functionality

use crate::kiro::model::events::Event;
use std::io::Write;

/// Print hex data (similar to xxd format)
pub fn print_hex(data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        // Print offset
        print!("{:08x}: ", i * 16);

        // Print hex
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{:02x} ", byte);
        }

        // Pad with spaces
        let padding = 16 - chunk.len();
        for j in 0..padding {
            if chunk.len() + j == 8 {
                print!(" ");
            }
            print!("   ");
        }

        // Print ASCII
        print!(" |");
        for byte in chunk {
            if *byte >= 0x20 && *byte < 0x7f {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
    std::io::stdout().flush().ok();
}

/// Debug CRC calculation - analyze CRC of AWS Event Stream frames
pub fn debug_crc(data: &[u8]) {
    if data.len() < 12 {
        println!("[CRC Debug] Insufficient data, less than 12 bytes");
        return;
    }

    use crc::{Crc, CRC_32_BZIP2, CRC_32_ISO_HDLC, CRC_32_ISCSI, CRC_32_JAMCRC};

    let total_length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let header_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let prelude_crc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

    println!("\n[CRC Debug]");
    println!("  total_length: {} (0x{:08x})", total_length, total_length);
    println!(
        "  header_length: {} (0x{:08x})",
        header_length, header_length
    );
    println!("  prelude_crc (from data): 0x{:08x}", prelude_crc);

    // Test various CRC32 variants
    let crc32c: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
    let crc32_iso: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let crc32_bzip2: Crc<u32> = Crc::<u32>::new(&CRC_32_BZIP2);
    let crc32_jamcrc: Crc<u32> = Crc::<u32>::new(&CRC_32_JAMCRC);

    let prelude = &data[..8];

    println!("  CRC32C (ISCSI):   0x{:08x}", crc32c.checksum(prelude));
    println!(
        "  CRC32 ISO-HDLC:   0x{:08x} {}",
        crc32_iso.checksum(prelude),
        if crc32_iso.checksum(prelude) == prelude_crc {
            "<-- MATCH"
        } else {
            ""
        }
    );
    println!("  CRC32 BZIP2:      0x{:08x}", crc32_bzip2.checksum(prelude));
    println!(
        "  CRC32 JAMCRC:     0x{:08x}",
        crc32_jamcrc.checksum(prelude)
    );

    // Print first 8 bytes
    print!("  First 8 bytes: ");
    for byte in prelude {
        print!("{:02x} ", byte);
    }
    println!();
}

/// Print frame summary
pub fn print_frame_summary(data: &[u8]) {
    if data.len() < 12 {
        println!("[Frame Summary] Insufficient data");
        return;
    }

    let total_length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let header_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;

    println!("\n[Frame Summary]");
    println!("  Total length: {} bytes", total_length);
    println!("  Header length: {} bytes", header_length);
    println!("  Payload length: {} bytes", total_length.saturating_sub(12 + header_length + 4));
    println!("  Data available: {} bytes", data.len());

    if data.len() >= total_length {
        println!("  Status: complete frame");
    } else {
        println!(
            "  Status: incomplete (missing {} bytes)",
            total_length - data.len()
        );
    }
}

/// Print event verbosely (debug format, includes event type and full data)
pub fn print_event_verbose(event: &Event) {
    match event {
        Event::AssistantResponse(e) => {
            println!("\n[Event] AssistantResponse");
            println!("  content: {:?}", e.content());
        }
        Event::ToolUse(e) => {
            println!("\n[Event] ToolUse");
            println!("  name: {:?}", e.name());
            println!("  tool_use_id: {:?}", e.tool_use_id());
            println!("  input: {:?}", e.input());
            println!("  stop: {}", e.is_complete());
        }
        Event::Metering(e) => {
            println!("\n[Event] Metering");
            println!("  unit: {:?}", e.unit);
            println!("  unit_plural: {:?}", e.unit_plural);
            println!("  usage: {}", e.usage);
        }
        Event::ContextUsage(e) => {
            println!("\n[Event] ContextUsage");
            println!("  context_usage_percentage: {}", e.context_usage_percentage);
        }
        Event::Unknown { event_type, payload } => {
            println!("\n[Event] Unknown");
            println!("  event_type: {:?}", event_type);
            println!("  payload ({} bytes):", payload.len());
            print_hex(payload);
        }
        Event::Error {
            error_code,
            error_message,
        } => {
            println!("\n[Event] Error");
            println!("  error_code: {:?}", error_code);
            println!("  error_message: {:?}", error_message);
        }
        Event::Exception {
            exception_type,
            message,
        } => {
            println!("\n[Event] Exception");
            println!("  exception_type: {:?}", exception_type);
            println!("  message: {:?}", message);
        }
    }
}

/// Print event concisely (for normal output)
pub fn print_event(event: &Event) {
    match event {
        Event::AssistantResponse(e) => {
            // Print assistant response in real-time, without newline
            print!("{}", e.content());
            std::io::stdout().flush().ok();
        }
        Event::ToolUse(e) => {
            println!("\n[Tool Call] {} (id: {})", e.name(), e.tool_use_id());
            println!("  Input: {}", e.input());
            if e.is_complete() {
                println!("  [Call ended]");
            }
        }
        Event::Metering(e) => {
            println!("\n[Billing] {}", e);
        }
        Event::ContextUsage(e) => {
            println!("\n[Context Usage] {}", e);
        }
        Event::Unknown { event_type, .. } => {
            println!("\n[Unknown Event] {}", event_type);
        }
        Event::Error {
            error_code,
            error_message,
        } => {
            println!("\n[Error] {}: {}", error_code, error_message);
        }
        Event::Exception {
            exception_type,
            message,
        } => {
            println!("\n[Exception] {}: {}", exception_type, message);
        }
    }
}
