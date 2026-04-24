use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TlsBackend {
    Rustls,
    NativeTls,
}

impl Default for TlsBackend {
    fn default() -> Self {
        Self::Rustls
    }
}

/// KNA application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_region")]
    pub region: String,

    /// Auth Region (for token refresh), falls back to region if not configured
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_region: Option<String>,

    /// API Region (for API requests), falls back to region if not configured
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_region: Option<String>,

    #[serde(default = "default_kiro_version")]
    pub kiro_version: String,

    #[serde(default)]
    pub machine_id: Option<String>,

    #[serde(default)]
    pub api_key: Option<String>,

    #[serde(default = "default_system_version")]
    pub system_version: String,

    #[serde(default = "default_node_version")]
    pub node_version: String,

    #[serde(default = "default_tls_backend")]
    pub tls_backend: TlsBackend,

    /// External count_tokens API address (optional)
    #[serde(default)]
    pub count_tokens_api_url: Option<String>,

    /// count_tokens API key (optional)
    #[serde(default)]
    pub count_tokens_api_key: Option<String>,

    /// count_tokens API authentication type (optional, "x-api-key" or "bearer", default "x-api-key")
    #[serde(default = "default_count_tokens_auth_type")]
    pub count_tokens_auth_type: String,

    /// HTTP proxy address (optional)
    /// Supported formats: http://host:port, https://host:port, socks5://host:port
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// Proxy authentication username (optional)
    #[serde(default)]
    pub proxy_username: Option<String>,

    /// Proxy authentication password (optional)
    #[serde(default)]
    pub proxy_password: Option<String>,

    /// Admin API key (optional, enables Admin API functionality)
    #[serde(default)]
    pub admin_api_key: Option<String>,

    /// Load balancing mode ("priority" or "balanced")
    #[serde(default = "default_load_balancing_mode")]
    pub load_balancing_mode: String,

    /// Whether to enable thinking block extraction for non-streaming responses (default true)
    ///
    /// When enabled, `<thinking>...</thinking>` tags in non-streaming responses will be parsed as
    /// independent `{"type": "thinking", ...}` content blocks, consistent with streaming response behavior.
    #[serde(default = "default_extract_thinking")]
    pub extract_thinking: bool,

    /// Default endpoint name (used when credentials do not explicitly specify an endpoint, default "ide")
    #[serde(default = "default_endpoint")]
    pub default_endpoint: String,

    /// Endpoint-specific configuration
    ///
    /// Keys are endpoint names (e.g., "ide" / "cli"), values are freely defined parameter objects for that endpoint.
    /// Endpoints not listed here use the implementation's built-in default values.
    #[serde(default)]
    pub endpoints: HashMap<String, serde_json::Value>,

    /// Configuration file path (runtime metadata, not written to JSON)
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_region() -> String {
    "us-east-1".to_string()
}

fn default_kiro_version() -> String {
    "0.11.107".to_string()
}

fn default_system_version() -> String {
    const SYSTEM_VERSIONS: &[&str] = &["darwin#24.6.0", "win32#10.0.22631"];
    SYSTEM_VERSIONS[fastrand::usize(..SYSTEM_VERSIONS.len())].to_string()
}

fn default_node_version() -> String {
    "22.22.0".to_string()
}

fn default_count_tokens_auth_type() -> String {
    "x-api-key".to_string()
}

fn default_tls_backend() -> TlsBackend {
    TlsBackend::Rustls
}

fn default_load_balancing_mode() -> String {
    "priority".to_string()
}

fn default_extract_thinking() -> bool {
    true
}

fn default_endpoint() -> String {
    crate::kiro::endpoint::ide::IDE_ENDPOINT_NAME.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            region: default_region(),
            auth_region: None,
            api_region: None,
            kiro_version: default_kiro_version(),
            machine_id: None,
            api_key: None,
            system_version: default_system_version(),
            node_version: default_node_version(),
            tls_backend: default_tls_backend(),
            count_tokens_api_url: None,
            count_tokens_api_key: None,
            count_tokens_auth_type: default_count_tokens_auth_type(),
            proxy_url: None,
            proxy_username: None,
            proxy_password: None,
            admin_api_key: None,
            load_balancing_mode: default_load_balancing_mode(),
            extract_thinking: default_extract_thinking(),
            default_endpoint: default_endpoint(),
            endpoints: HashMap::new(),
            config_path: None,
        }
    }
}

impl Config {
    /// Get the default configuration file path
    pub fn default_config_path() -> &'static str {
        "config.json"
    }

    /// Get the effective Auth Region (for token refresh)
    /// Prefers auth_region, falls back to region if not configured
    pub fn effective_auth_region(&self) -> &str {
        self.auth_region.as_deref().unwrap_or(&self.region)
    }

    /// Get the effective API Region (for API requests)
    /// Prefers api_region, falls back to region if not configured
    pub fn effective_api_region(&self) -> &str {
        self.api_region.as_deref().unwrap_or(&self.region)
    }

    /// Load configuration from file
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            // Configuration file does not exist, return default configuration
            let mut config = Self::default();
            config.config_path = Some(path.to_path_buf());
            return Ok(config);
        }

        let content = fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&content)?;
        config.config_path = Some(path.to_path_buf());
        Ok(config)
    }

    /// Get the configuration file path (if any)
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Write the current configuration back to the original file
    pub fn save(&self) -> anyhow::Result<()> {
        let path = self
            .config_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Configuration file path unknown, cannot save configuration"))?;

        let content = serde_json::to_string_pretty(self).context("Failed to serialize configuration")?;
        fs::write(path, content).with_context(|| format!("Failed to write configuration file: {}", path.display()))?;
        Ok(())
    }
}
