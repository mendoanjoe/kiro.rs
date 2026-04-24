//! Token calculation module
//!
//! Provides text token count calculation functionality.
//!
//! # Calculation rules
//! - Non-western characters: counted as 4.5 character units each
//! - Western characters: counted as 1 character unit each
//! - 4 character units = 1 token (rounded)

use crate::anthropic::types::{
    CountTokensRequest, CountTokensResponse, Message, SystemMessage, Tool,
};
use crate::http_client::{ProxyConfig, build_client};
use crate::model::config::TlsBackend;
use std::sync::OnceLock;

/// Count Tokens API configuration
#[derive(Clone, Default)]
pub struct CountTokensConfig {
    /// External count_tokens API address
    pub api_url: Option<String>,
    /// count_tokens API key
    pub api_key: Option<String>,
    /// count_tokens API authentication type ("x-api-key" or "bearer")
    pub auth_type: String,
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,

    pub tls_backend: TlsBackend,
}

/// Global configuration storage
static COUNT_TOKENS_CONFIG: OnceLock<CountTokensConfig> = OnceLock::new();

/// Initialize count_tokens configuration
///
/// Should be called once at application startup
pub fn init_config(config: CountTokensConfig) {
    let _ = COUNT_TOKENS_CONFIG.set(config);
}

/// Get configuration
fn get_config() -> Option<&'static CountTokensConfig> {
    COUNT_TOKENS_CONFIG.get()
}

/// Determine if a character is non-western
///
/// Western characters include:
/// - ASCII characters (U+0000..U+007F)
/// - Latin Extended (U+0080..U+024F)
/// - Latin Extended Additional (U+1E00..U+1EFF)
///
/// Returns true if the character is non-western (e.g., Chinese, Japanese, Korean, Arabic, etc.)
fn is_non_western_char(c: char) -> bool {
    !matches!(c,
        // Basic ASCII
        '\u{0000}'..='\u{007F}' |
        // Latin Extended-A
        '\u{0080}'..='\u{00FF}' |
        // Latin Extended-B
        '\u{0100}'..='\u{024F}' |
        // Latin Extended Additional
        '\u{1E00}'..='\u{1EFF}' |
        // Latin Extended-C/D/E
        '\u{2C60}'..='\u{2C7F}' |
        '\u{A720}'..='\u{A7FF}' |
        '\u{AB30}'..='\u{AB6F}'
    )
}

/// Calculate the token count for text
///
/// # Calculation rules
/// - Non-western characters: counted as 4.5 character units each
/// - Western characters: counted as 1 character unit each
/// - 4 character units = 1 token (rounded)
/// ```
pub fn count_tokens(text: &str) -> u64 {
    // println!("text: {}", text);

    let char_units: f64 = text
        .chars()
        .map(|c| if is_non_western_char(c) { 4.0 } else { 1.0 })
        .sum();

    let tokens = char_units / 4.0;

    let acc_token = if tokens < 100.0 {
        tokens * 1.5
    } else if tokens < 200.0 {
        tokens * 1.3
    } else if tokens < 300.0 {
        tokens * 1.25
    } else if tokens < 800.0 {
        tokens * 1.2
    } else {
        tokens * 1.0
    } as u64;

    // println!("tokens: {}, acc_tokens: {}", tokens, acc_token);
    acc_token
}

/// Estimate input tokens for a request
///
/// Prefers calling the remote API, falls back to local calculation on failure
pub(crate) fn count_all_tokens(
    model: String,
    system: Option<Vec<SystemMessage>>,
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
) -> u64 {
    // Check if remote API is configured
    if let Some(config) = get_config() {
        if let Some(api_url) = &config.api_url {
            // Try calling remote API
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(call_remote_count_tokens(
                    api_url, config, model, &system, &messages, &tools,
                ))
            });

            match result {
                Ok(tokens) => {
                    tracing::debug!("Remote count_tokens API returned: {}", tokens);
                    return tokens;
                }
                Err(e) => {
                    tracing::warn!("Remote count_tokens API call failed, falling back to local calculation: {}", e);
                }
            }
        }
    }

    // Local calculation
    count_all_tokens_local(system, messages, tools)
}

/// Call the remote count_tokens API
async fn call_remote_count_tokens(
    api_url: &str,
    config: &CountTokensConfig,
    model: String,
    system: &Option<Vec<SystemMessage>>,
    messages: &Vec<Message>,
    tools: &Option<Vec<Tool>>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let client = build_client(config.proxy.as_ref(), 300, config.tls_backend)?;

    // Build request body
    let request = CountTokensRequest {
        model: model, // Model name for token calculation
        messages: messages.clone(),
        system: system.clone(),
        tools: tools.clone(),
    };

    // Build request
    let mut req_builder = client.post(api_url);

    // Set authentication headers
    if let Some(api_key) = &config.api_key {
        if config.auth_type == "bearer" {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        } else {
            req_builder = req_builder.header("x-api-key", api_key);
        }
    }

    // Send request
    let response = req_builder
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API returned error status: {}", response.status()).into());
    }

    let result: CountTokensResponse = response.json().await?;
    Ok(result.input_tokens as u64)
}

/// Calculate input tokens locally for a request
fn count_all_tokens_local(
    system: Option<Vec<SystemMessage>>,
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
) -> u64 {
    let mut total = 0;

    // System messages
    if let Some(ref system) = system {
        for msg in system {
            total += count_tokens(&msg.text);
        }
    }

    // User messages
    for msg in &messages {
        if let serde_json::Value::String(s) = &msg.content {
            total += count_tokens(s);
        } else if let serde_json::Value::Array(arr) = &msg.content {
            for item in arr {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    total += count_tokens(text);
                }
            }
        }
    }

    // Tool definitions
    if let Some(ref tools) = tools {
        for tool in tools {
            total += count_tokens(&tool.name);
            total += count_tokens(&tool.description);
            let input_schema_json = serde_json::to_string(&tool.input_schema).unwrap_or_default();
            total += count_tokens(&input_schema_json);
        }
    }

    total.max(1)
}

/// Estimate output tokens
pub(crate) fn estimate_output_tokens(content: &[serde_json::Value]) -> i32 {
    let mut total = 0;

    for block in content {
        if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
            total += count_tokens(text) as i32;
        }
        if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
            // Tool call overhead
            if let Some(input) = block.get("input") {
                let input_str = serde_json::to_string(input).unwrap_or_default();
                total += count_tokens(&input_str) as i32;
            }
        }
    }

    total.max(1)
}
