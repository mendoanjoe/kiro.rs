use futures::StreamExt;

use crate::debug::{print_event, print_event_verbose, debug_crc, print_hex};
use crate::kiro::model::credentials::KiroCredentials;
use crate::kiro::model::events::Event;
use crate::kiro::model::requests::KiroRequest;
use crate::kiro::parser::EventStreamDecoder;
use crate::kiro::provider::KiroProvider;
use crate::kiro::token_manager::TokenManager;
use crate::model::config::Config;


/// Call streaming API and print responses in real-time
pub(crate) async fn call_stream_api() -> anyhow::Result<()> {
    // Read test.json as request body
    let request_body = std::fs::read_to_string("test.json")?;
    println!("Request body loaded, length: {} bytes", request_body.len());

    // Parse request body as KiroRequest object
    let request: KiroRequest = serde_json::from_str(&request_body)?;
    println!("Request object parsed:");
    println!("  Session ID: {}", request.conversation_id());
    println!("  Model ID: {}", request.model_id());
    println!("  Message content length: {} characters", request.current_content().len());
    if let Some(ref task_type) = request.conversation_state.agent_task_type {
        println!("  Task type: {}", task_type);
    }
    if let Some(ref trigger_type) = request.conversation_state.chat_trigger_type {
        println!("  Trigger type: {}", trigger_type);
    }
    println!("  History message count: {}", request.conversation_state.history.len());
    println!("  Tool count: {}", request.conversation_state.current_message.user_input_message.user_input_message_context.tools.len());

    // Load credentials
    let credentials = KiroCredentials::load_default()?;
    println!("Credentials loaded");

    // Load configuration
    let config = Config::load_default()?;
    println!("API region: {}", config.region);

    // Create TokenManager and KiroProvider
    let token_manager = TokenManager::new(config, credentials);
    let mut provider = KiroProvider::new(token_manager);

    println!("\nStarting streaming API call...\n");
    println!("{}", "=".repeat(60));

    // Call streaming API
    let response = provider.call_api_stream(&request_body).await?;

    // Get byte stream
    let mut stream = response.bytes_stream();
    let mut decoder = EventStreamDecoder::new();

    // Process streaming data
    let mut total_bytes = 0usize;
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // Debug mode: print raw hex data
                // println!("\n[Received chunk] {} bytes, offset {}", chunk.len(), total_bytes);
                // print_hex(&chunk);
                // debug_crc(&chunk);

                total_bytes += chunk.len();

                // Feed data to decoder
                if let Err(e) = decoder.feed(&chunk) {
                    eprintln!("[Buffer error] {}", e);
                    continue;
                }

                // Decode all available frames
                for result in decoder.decode_iter() {
                    match result {
                        Ok(frame) => {
                            // Parse event
                            match Event::from_frame(frame) {
                                Ok(event) => {
                                    // Concise output
                                    // print_event(&event);
                                    // Verbose output (for debugging)
                                    print_event_verbose(&event);
                                }
                                Err(e) => eprintln!("[Parse error] {}", e),
                            }
                        }
                        Err(e) => {
                            eprintln!("[Frame parse error] {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Network error] {}", e);
                break;
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Streaming response ended");
    println!("Total received {} bytes, decoded {} frames", total_bytes, decoder.frames_decoded());

    Ok(())
}