mod admin;
mod admin_ui;
mod anthropic;
mod common;
mod http_client;
mod kiro;
mod model;
pub mod token;

use std::collections::HashMap;
use std::sync::Arc;

use clap::Parser;
use kiro::endpoint::{IdeEndpoint, KiroEndpoint};
use kiro::model::credentials::{CredentialsConfig, KiroCredentials};
use kiro::provider::KiroProvider;
use kiro::token_manager::MultiTokenManager;
use model::arg::Args;
use model::config::Config;

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config_path = args
        .config
        .unwrap_or_else(|| Config::default_config_path().to_string());
    let config = Config::load(&config_path).unwrap_or_else(|e| {
        tracing::error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    // Load credentials (supports single object or array format)
    let credentials_path = args
        .credentials
        .unwrap_or_else(|| KiroCredentials::default_credentials_path().to_string());
    let credentials_config = CredentialsConfig::load(&credentials_path).unwrap_or_else(|e| {
        tracing::error!("Failed to load credentials: {}", e);
        std::process::exit(1);
    });

    // Check if multiple credentials format (for writing back after refresh)
    let is_multiple_format = credentials_config.is_multiple();

    // Convert to credentials list sorted by priority
    let mut credentials_list = credentials_config.into_sorted_credentials();

    // Check KIRO_API_KEY environment variable, auto-create API Key credential
    if let Ok(kiro_api_key) = std::env::var("KIRO_API_KEY") {
        if kiro_api_key.is_empty() {
            tracing::warn!("KIRO_API_KEY environment variable is set but empty, treating as not configured");
        } else {
            tracing::info!("KIRO_API_KEY environment variable detected, adding API Key credential (highest priority)");
            let api_key_cred = KiroCredentials {
                kiro_api_key: Some(kiro_api_key),
                auth_method: Some("api_key".to_string()),
                priority: 0,
                ..Default::default()
            };
            credentials_list.insert(0, api_key_cred);
        }
    }

    tracing::info!("Loaded {} credential configurations", credentials_list.len());

    // Get the first credential for logging
    let first_credentials = credentials_list.first().cloned().unwrap_or_default();
    tracing::debug!("Primary credential: {:?}", first_credentials);

    // Get API Key
    let api_key = config.api_key.clone().unwrap_or_else(|| {
        tracing::error!("apiKey not set in configuration file");
        std::process::exit(1);
    });

    // Build proxy configuration
    let proxy_config = config.proxy_url.as_ref().map(|url| {
        let mut proxy = http_client::ProxyConfig::new(url);
        if let (Some(username), Some(password)) = (&config.proxy_username, &config.proxy_password) {
            proxy = proxy.with_auth(username, password);
        }
        proxy
    });

    if proxy_config.is_some() {
        tracing::info!("HTTP proxy configured: {}", config.proxy_url.as_ref().unwrap());
    }

    // Build endpoint registry
    let mut endpoints: HashMap<String, Arc<dyn KiroEndpoint>> = HashMap::new();
    {
        let ide = IdeEndpoint::new();
        endpoints.insert(ide.name().to_string(), Arc::new(ide));
    }

    // Validate that the default endpoint exists
    if !endpoints.contains_key(&config.default_endpoint) {
        tracing::error!("Default endpoint \"{}\" is not registered", config.default_endpoint);
        std::process::exit(1);
    }

    // Validate that all endpoints declared by credentials are registered
    for cred in &credentials_list {
        let name = cred
            .endpoint
            .as_deref()
            .unwrap_or(&config.default_endpoint);
        if !endpoints.contains_key(name) {
            tracing::error!(
                "Credential id={:?} specified unknown endpoint \"{}\" (registered: {:?})",
                cred.id,
                name,
                endpoints.keys().collect::<Vec<_>>()
            );
            std::process::exit(1);
        }
    }

    let endpoint_names: Vec<String> = endpoints.keys().cloned().collect();

    // Create MultiTokenManager and KiroProvider
    let token_manager = MultiTokenManager::new(
        config.clone(),
        credentials_list,
        proxy_config.clone(),
        Some(credentials_path.into()),
        is_multiple_format,
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to create Token manager: {}", e);
        std::process::exit(1);
    });
    let token_manager = Arc::new(token_manager);
    let kiro_provider = KiroProvider::with_proxy(
        token_manager.clone(),
        proxy_config.clone(),
        endpoints,
        config.default_endpoint.clone(),
    );

    // Initialize count_tokens configuration
    token::init_config(token::CountTokensConfig {
        api_url: config.count_tokens_api_url.clone(),
        api_key: config.count_tokens_api_key.clone(),
        auth_type: config.count_tokens_auth_type.clone(),
        proxy: proxy_config,
        tls_backend: config.tls_backend,
    });

    // Build Anthropic API router (profile_arn is dynamically injected by the provider layer based on actual credentials)
    let anthropic_app = anthropic::create_router_with_provider(
        &api_key,
        Some(kiro_provider),
        config.extract_thinking,
    );

    // Build Admin API router (if a non-empty admin_api_key is configured)
    // Security check: empty string is treated as not configured, preventing empty key from bypassing authentication
    let admin_key_valid = config
        .admin_api_key
        .as_ref()
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);

    let app = if let Some(admin_key) = &config.admin_api_key {
        if admin_key.trim().is_empty() {
            tracing::warn!("admin_api_key is empty, Admin API is not enabled");
            anthropic_app
        } else {
            let admin_service =
                admin::AdminService::new(token_manager.clone(), endpoint_names.clone());
            let admin_state = admin::AdminState::new(admin_key, admin_service);
            let admin_app = admin::create_admin_router(admin_state);

            // Create Admin UI router
            let admin_ui_app = admin_ui::create_admin_ui_router();

            tracing::info!("Admin API enabled");
            tracing::info!("Admin UI enabled: /admin");
            anthropic_app
                .nest("/api/admin", admin_app)
                .nest("/admin", admin_ui_app)
        }
    } else {
        anthropic_app
    };

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Starting Anthropic API endpoint: {}", addr);
    tracing::info!("API Key: {}***", &api_key[..(api_key.len() / 2)]);
    tracing::info!("Available APIs:");
    tracing::info!("  GET  /v1/models");
    tracing::info!("  POST /v1/messages");
    tracing::info!("  POST /v1/messages/count_tokens");
    if admin_key_valid {
        tracing::info!("Admin API:");
        tracing::info!("  GET  /api/admin/credentials");
        tracing::info!("  POST /api/admin/credentials/:index/disabled");
        tracing::info!("  POST /api/admin/credentials/:index/priority");
        tracing::info!("  POST /api/admin/credentials/:index/reset");
        tracing::info!("  GET  /api/admin/credentials/:index/balance");
        tracing::info!("Admin UI:");
        tracing::info!("  GET  /admin");
    }

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
