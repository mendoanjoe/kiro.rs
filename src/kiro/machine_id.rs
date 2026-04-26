//! Device fingerprint generator
//!

use std::collections::HashMap;
use std::sync::OnceLock;

use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::kiro::model::credentials::KiroCredentials;
use crate::model::config::Config;

/// Fallback machineId cache (bucketed by credential id, stable within process lifetime)
///
/// Key is `credentials.id`; credentials without an id share the same fallback value (should not occur in normal flow).
static FALLBACK_MACHINE_IDS: OnceLock<Mutex<HashMap<Option<u64>, String>>> = OnceLock::new();

/// Normalize the machineId format
///
/// Supports the following formats:
/// - 64-character hex string (returned as-is)
/// - UUID format (e.g. "2582956e-cc88-4669-b546-07adbffcb894"; stripped of hyphens and padded to 64 characters)
fn normalize_machine_id(machine_id: &str) -> Option<String> {
    let trimmed = machine_id.trim();

    // If already 64 characters, return as-is
    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(trimmed.to_string());
    }

    // Try to parse UUID format (remove hyphens)
    let without_dashes: String = trimmed.chars().filter(|c| *c != '-').collect();

    // UUID without hyphens is 32 characters
    if without_dashes.len() == 32 && without_dashes.chars().all(|c| c.is_ascii_hexdigit()) {
        // Pad to 64 characters (repeat once)
        return Some(format!("{}{}", without_dashes, without_dashes));
    }

    // Unrecognized format
    None
}

/// Generate a unique Machine ID based on credential information
///
/// Priority：
/// 1. Credential-level `machineId` (if configured and format is valid)
/// 2. Global `config.machineId` (if configured and format is valid)
/// 3. Derived from the credential type (mutually exclusive, branched by [`KiroCredentials::is_api_key_credential`]):
///    - API Key credential: derived from `kiroApiKey`
///    - OAuth credential: derived from `refreshToken`
/// 4. Fallback: derived from a random seed, cached in-process by `credentials.id` (logs a warning on first use)
pub fn generate_from_credentials(credentials: &KiroCredentials, config: &Config) -> String {
    // If a credential-level machineId is configured, use it first
    if let Some(ref machine_id) = credentials.machine_id {
        if let Some(normalized) = normalize_machine_id(machine_id) {
            return normalized;
        }
    }

    // If a global machineId is configured, use it as the default
    if let Some(ref machine_id) = config.machine_id {
        if let Some(normalized) = normalize_machine_id(machine_id) {
            return normalized;
        }
    }

    // Derive by credential type (API Key and refreshToken paths are mutually exclusive; no fallback)
    if credentials.is_api_key_credential() {
        // API Key credential: derive from kiroApiKey
        if let Some(ref api_key) = credentials.kiro_api_key {
            if !api_key.is_empty() {
                return sha256_hex(&format!("KiroAPIKey/{}", api_key));
            }
        }
    } else if let Some(ref refresh_token) = credentials.refresh_token {
        // OAuth credential: derive from refreshToken
        if !refresh_token.is_empty() {
            return sha256_hex(&format!("KotlinNativeAPI/{}", refresh_token));
        }
    }

    // Fallback: derive a random machineId, stable in-process per credential id
    fallback_machine_id(credentials)
}

/// Generate a fallback machineId for credentials that lack derivation material
///
/// - Still derived via `sha256("KiroFallback/<uuid>")`, output format consistent with normal path (64-char hex)
/// - Cached in-process by `credentials.id`; multiple calls for the same credential return the same value
/// - Re-randomized on process restart; not persisted
/// - Logs a warning once the first time it is generated for each credential
fn fallback_machine_id(credentials: &KiroCredentials) -> String {
    let cache = FALLBACK_MACHINE_IDS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = cache.lock();
    if let Some(existing) = map.get(&credentials.id) {
        return existing.clone();
    }

    let seed = Uuid::new_v4();
    let derived = sha256_hex(&format!("KiroFallback/{}", seed));
    tracing::warn!(
        credential_id = ?credentials.id,
        "Credential lacks derivation material (kiroApiKey/refreshToken both unavailable); using random fallback machineId (stable within process)"
    );
    map.insert(credentials.id, derived.clone());
    derived
}

/// SHA-256 hash implementation (returns a hex string)
fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let result = sha256_hex("test");
        assert_eq!(result.len(), 64);
        assert_eq!(
            result,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_generate_with_custom_machine_id() {
        let credentials = KiroCredentials::default();
        let mut config = Config::default();
        config.machine_id = Some("a".repeat(64));

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result, "a".repeat(64));
    }

    #[test]
    fn test_generate_with_credential_machine_id_overrides_config() {
        let mut credentials = KiroCredentials::default();
        credentials.machine_id = Some("b".repeat(64));

        let mut config = Config::default();
        config.machine_id = Some("a".repeat(64));

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result, "b".repeat(64));
    }

    #[test]
    fn test_generate_with_refresh_token() {
        let mut credentials = KiroCredentials::default();
        credentials.refresh_token = Some("test_refresh_token".to_string());
        let config = Config::default();

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_generate_without_credentials_uses_fallback() {
        // 完全空凭据会走兜底分支，返回派生后的随机 machineId
        let credentials = KiroCredentials::default();
        let config = Config::default();

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_with_api_key() {
        let mut credentials = KiroCredentials::default();
        credentials.kiro_api_key = Some("ksk_test_api_key".to_string());
        let config = Config::default();

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result.len(), 64);
        // 应与 KiroAPIKey/<api_key> 的哈希一致
        assert_eq!(result, sha256_hex("KiroAPIKey/ksk_test_api_key"));
    }

    #[test]
    fn test_api_key_and_refresh_token_are_mutually_exclusive() {
        // 同时存在 kiroApiKey 和 refreshToken 时，应走 API Key 分支
        let mut credentials = KiroCredentials::default();
        credentials.kiro_api_key = Some("ksk_test".to_string());
        credentials.refresh_token = Some("should_not_be_used".to_string());
        let config = Config::default();

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result, sha256_hex("KiroAPIKey/ksk_test"));
    }

    #[test]
    fn test_api_key_auth_method_empty_uses_fallback_not_refresh_token() {
        // auth_method=api_key 但 kiro_api_key 为空：不回落到 refreshToken，走兜底分支
        let mut credentials = KiroCredentials::default();
        credentials.id = Some(u64::MAX - 1);
        credentials.auth_method = Some("api_key".to_string());
        credentials.refresh_token = Some("should_not_be_used".to_string());
        let config = Config::default();

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result.len(), 64);
        // 必须不是基于 refresh_token 派生的值（互斥性验证）
        assert_ne!(result, sha256_hex("KotlinNativeAPI/should_not_be_used"));
    }

    #[test]
    fn test_fallback_is_stable_per_credential() {
        // 同一凭据（按 id 区分）多次调用兜底应返回同一值
        let mut credentials = KiroCredentials::default();
        credentials.id = Some(u64::MAX - 10);
        let config = Config::default();

        let first = generate_from_credentials(&credentials, &config);
        let second = generate_from_credentials(&credentials, &config);
        assert_eq!(first, second);
    }

    #[test]
    fn test_fallback_differs_across_credentials() {
        // 不同凭据（不同 id）的兜底值应互不相同
        let mut cred_a = KiroCredentials::default();
        cred_a.id = Some(u64::MAX - 20);
        let mut cred_b = KiroCredentials::default();
        cred_b.id = Some(u64::MAX - 21);
        let config = Config::default();

        let id_a = generate_from_credentials(&cred_a, &config);
        let id_b = generate_from_credentials(&cred_b, &config);
        assert_ne!(id_a, id_b);
    }

    #[test]
    fn test_normalize_uuid_format() {
        // UUID 格式应该被转换为 64 字符
        let uuid = "2582956e-cc88-4669-b546-07adbffcb894";
        let result = normalize_machine_id(uuid);
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert_eq!(normalized.len(), 64);
        // UUID 去掉连字符后重复一次
        assert_eq!(
            normalized,
            "2582956ecc884669b54607adbffcb8942582956ecc884669b54607adbffcb894"
        );
    }

    #[test]
    fn test_normalize_64_char_hex() {
        // 64 字符十六进制应该直接返回
        let hex64 = "a".repeat(64);
        let result = normalize_machine_id(&hex64);
        assert_eq!(result, Some(hex64));
    }

    #[test]
    fn test_normalize_invalid_format() {
        // 无效格式应该返回 None
        assert!(normalize_machine_id("invalid").is_none());
        assert!(normalize_machine_id("too-short").is_none());
        assert!(normalize_machine_id(&"g".repeat(64)).is_none()); // 非十六进制
    }

    #[test]
    fn test_generate_with_uuid_machine_id() {
        let mut credentials = KiroCredentials::default();
        credentials.machine_id = Some("2582956e-cc88-4669-b546-07adbffcb894".to_string());

        let config = Config::default();

        let result = generate_from_credentials(&credentials, &config);
        assert_eq!(result.len(), 64);
    }
}
