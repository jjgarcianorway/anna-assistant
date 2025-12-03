//! Secrets Redaction System v0.0.18
//!
//! Centralized redaction pipeline for sensitive data:
//! - Passwords, tokens, API keys, bearer tokens
//! - Private keys and PEM blocks
//! - SSH keys and known_hosts
//! - Cookies and authorization headers
//! - Cloud credentials (AWS, Azure, GCP)
//! - Git credentials and .netrc entries
//! - Environment variables matching policy patterns
//!
//! Redaction is applied:
//! - Before writing to disk
//! - Before printing to terminal
//! - In evidence excerpts
//! - In audit logs
//! - In memory/recipe storage

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::LazyLock;

// =============================================================================
// Redaction Types
// =============================================================================

/// Evidence ID prefix for redaction events
pub const REDACTION_EVIDENCE_PREFIX: &str = "E-redact-";

/// Types of secrets that can be redacted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretType {
    Password,
    ApiKey,
    BearerToken,
    AuthHeader,
    PrivateKey,
    PemBlock,
    SshKey,
    Cookie,
    AwsCredential,
    AzureCredential,
    GcpCredential,
    GitCredential,
    NetrcEntry,
    EnvSecret,
    JwtToken,
    DatabaseUrl,
    ConnectionString,
    OAuthToken,
    WebhookSecret,
    EncryptionKey,
    Certificate,
    GenericSecret,
}

impl SecretType {
    /// Get the redaction placeholder for this secret type
    pub fn placeholder(&self) -> &'static str {
        match self {
            SecretType::Password => "[REDACTED:PASSWORD]",
            SecretType::ApiKey => "[REDACTED:API_KEY]",
            SecretType::BearerToken => "[REDACTED:BEARER_TOKEN]",
            SecretType::AuthHeader => "[REDACTED:AUTH_HEADER]",
            SecretType::PrivateKey => "[REDACTED:PRIVATE_KEY]",
            SecretType::PemBlock => "[REDACTED:PEM_BLOCK]",
            SecretType::SshKey => "[REDACTED:SSH_KEY]",
            SecretType::Cookie => "[REDACTED:COOKIE]",
            SecretType::AwsCredential => "[REDACTED:AWS_CREDENTIAL]",
            SecretType::AzureCredential => "[REDACTED:AZURE_CREDENTIAL]",
            SecretType::GcpCredential => "[REDACTED:GCP_CREDENTIAL]",
            SecretType::GitCredential => "[REDACTED:GIT_CREDENTIAL]",
            SecretType::NetrcEntry => "[REDACTED:NETRC]",
            SecretType::EnvSecret => "[REDACTED:ENV_SECRET]",
            SecretType::JwtToken => "[REDACTED:JWT]",
            SecretType::DatabaseUrl => "[REDACTED:DATABASE_URL]",
            SecretType::ConnectionString => "[REDACTED:CONNECTION_STRING]",
            SecretType::OAuthToken => "[REDACTED:OAUTH_TOKEN]",
            SecretType::WebhookSecret => "[REDACTED:WEBHOOK_SECRET]",
            SecretType::EncryptionKey => "[REDACTED:ENCRYPTION_KEY]",
            SecretType::Certificate => "[REDACTED:CERTIFICATE]",
            SecretType::GenericSecret => "[REDACTED:SECRET]",
        }
    }
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.placeholder())
    }
}

// =============================================================================
// Redaction Pattern
// =============================================================================

/// A compiled redaction pattern
struct RedactionPattern {
    /// Compiled regex
    regex: Regex,
    /// Type of secret this pattern detects
    secret_type: SecretType,
    /// Whether to replace the entire match or use capture groups
    replace_full: bool,
}

impl RedactionPattern {
    fn new(pattern: &str, secret_type: SecretType, replace_full: bool) -> Option<Self> {
        Regex::new(pattern).ok().map(|regex| Self {
            regex,
            secret_type,
            replace_full,
        })
    }
}

// =============================================================================
// Compiled Patterns (Lazy Static)
// =============================================================================

static REDACTION_PATTERNS: LazyLock<Vec<RedactionPattern>> = LazyLock::new(|| {
    let patterns: Vec<(&str, SecretType, bool)> = vec![
        // JWT tokens (header.payload.signature format) - MUST be first to match before generic patterns
        (r"eyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]+", SecretType::JwtToken, true),

        // Password patterns
        (r#"(?i)(password|passwd|pwd)\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::Password, false),
        (r#"(?i)--password[=\s]+['"]?([^'"\s\n]+)['"]?"#, SecretType::Password, true),

        // API Key patterns
        (r#"(?i)(api[_-]?key|apikey)\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::ApiKey, false),
        (r"(?i)x-api-key:\s*(\S+)", SecretType::ApiKey, true),

        // Bearer tokens
        (r"(?i)bearer\s+([a-zA-Z0-9_\-\.]+)", SecretType::BearerToken, true),
        (r"(?i)authorization:\s*bearer\s+(\S+)", SecretType::BearerToken, true),

        // Authorization headers
        (r"(?i)authorization:\s*basic\s+(\S+)", SecretType::AuthHeader, true),
        (r"(?i)authorization:\s*(\S+)", SecretType::AuthHeader, true),

        // Private keys and PEM blocks
        (r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----[\s\S]*?-----END\s+(RSA\s+)?PRIVATE\s+KEY-----", SecretType::PrivateKey, true),
        (r"-----BEGIN\s+EC\s+PRIVATE\s+KEY-----[\s\S]*?-----END\s+EC\s+PRIVATE\s+KEY-----", SecretType::PrivateKey, true),
        (r"-----BEGIN\s+OPENSSH\s+PRIVATE\s+KEY-----[\s\S]*?-----END\s+OPENSSH\s+PRIVATE\s+KEY-----", SecretType::SshKey, true),
        (r"-----BEGIN\s+PGP\s+PRIVATE\s+KEY\s+BLOCK-----[\s\S]*?-----END\s+PGP\s+PRIVATE\s+KEY\s+BLOCK-----", SecretType::PrivateKey, true),
        (r"-----BEGIN\s+ENCRYPTED\s+PRIVATE\s+KEY-----[\s\S]*?-----END\s+ENCRYPTED\s+PRIVATE\s+KEY-----", SecretType::PrivateKey, true),
        (r"-----BEGIN\s+CERTIFICATE-----[\s\S]*?-----END\s+CERTIFICATE-----", SecretType::Certificate, true),

        // SSH keys (public key format isn't secret, but private key content lines are)
        (r"ssh-(rsa|ed25519|ecdsa)\s+[A-Za-z0-9+/=]+\s+\S+@\S+", SecretType::SshKey, true),

        // AWS credentials
        (r#"(?i)aws_access_key_id\s*[=:]\s*['"]?([A-Z0-9]{20})['"]?"#, SecretType::AwsCredential, true),
        (r#"(?i)aws_secret_access_key\s*[=:]\s*['"]?([A-Za-z0-9+/]{40})['"]?"#, SecretType::AwsCredential, true),
        (r"AKIA[0-9A-Z]{16}", SecretType::AwsCredential, true),

        // Azure credentials
        (r#"(?i)azure[_-]?(client[_-]?secret|storage[_-]?key)\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::AzureCredential, false),

        // GCP credentials
        (r#"(?i)(gcp|google)[_-]?(api[_-]?key|credentials?|service[_-]?account)\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::GcpCredential, false),

        // Git credentials
        (r"https?://[^:]+:[^@]+@[^\s]+\.git", SecretType::GitCredential, true),
        (r"git://[^:]+:[^@]+@[^\s]+", SecretType::GitCredential, true),

        // .netrc entries
        (r"(?i)machine\s+\S+\s+login\s+\S+\s+password\s+(\S+)", SecretType::NetrcEntry, true),

        // Cookies
        (r"(?i)(set-)?cookie:\s*([^;\n]+)", SecretType::Cookie, true),
        (r#"(?i)session[_-]?id\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::Cookie, false),

        // Database URLs (with passwords)
        (r"(?i)(postgres|mysql|mongodb|redis)://[^:]+:([^@]+)@[^\s]+", SecretType::DatabaseUrl, true),
        (r#"(?i)database_url\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::DatabaseUrl, false),

        // Connection strings
        (r"(?i)(server|data\s+source)=[^;]+;.*password\s*=\s*([^;]+)", SecretType::ConnectionString, true),

        // OAuth tokens
        (r#"(?i)oauth[_-]?token\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::OAuthToken, false),
        (r#"(?i)refresh[_-]?token\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::OAuthToken, false),
        (r#"(?i)access[_-]?token\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::OAuthToken, false),

        // Webhook secrets
        (r#"(?i)webhook[_-]?secret\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::WebhookSecret, false),

        // Encryption keys
        (r#"(?i)(encryption|encrypt)[_-]?key\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::EncryptionKey, false),
        (r#"(?i)(aes|des|rsa)[_-]?key\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::EncryptionKey, false),

        // Generic secrets and tokens
        (r#"(?i)(secret|token|credential)\s*[=:]\s*['"]?([^'"\s\n]{8,})['"]?"#, SecretType::GenericSecret, false),
        (r#"(?i)_token\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::GenericSecret, false),
        (r#"(?i)_secret\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::GenericSecret, false),
        (r#"(?i)_key\s*[=:]\s*['"]?([^'"\s\n]+)['"]?"#, SecretType::GenericSecret, false),
    ];

    patterns
        .into_iter()
        .filter_map(|(p, t, f)| RedactionPattern::new(p, t, f))
        .collect()
});

/// Environment variable name patterns that indicate secrets
static SECRET_ENV_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    let patterns = [
        r"(?i).*password.*",
        r"(?i).*passwd.*",
        r"(?i).*secret.*",
        r"(?i).*token.*",
        r"(?i).*api[_-]?key.*",
        r"(?i).*private[_-]?key.*",
        r"(?i).*auth.*",
        r"(?i).*credential.*",
        r"(?i)^aws_.*",
        r"(?i)^azure_.*",
        r"(?i)^gcp_.*",
        r"(?i)^google_.*",
        r"(?i)^github_.*",
        r"(?i)^gitlab_.*",
        r"(?i)^npm_.*token.*",
        r"(?i)^pypi_.*token.*",
        r"(?i).*_key$",
        r"(?i).*_token$",
        r"(?i).*_secret$",
    ];

    patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});

// =============================================================================
// Redaction Result
// =============================================================================

/// Result of a redaction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionResult {
    /// The redacted text
    pub text: String,
    /// Number of redactions applied
    pub redaction_count: usize,
    /// Types of secrets found
    pub secret_types_found: Vec<SecretType>,
    /// Whether any redactions were applied
    pub was_redacted: bool,
}

impl RedactionResult {
    /// Create a result with no redactions
    pub fn unchanged(text: String) -> Self {
        Self {
            text,
            redaction_count: 0,
            secret_types_found: Vec::new(),
            was_redacted: false,
        }
    }

    /// Create a result with redactions
    pub fn redacted(text: String, count: usize, types: Vec<SecretType>) -> Self {
        Self {
            text,
            redaction_count: count,
            secret_types_found: types,
            was_redacted: count > 0,
        }
    }
}

// =============================================================================
// Main Redaction Functions
// =============================================================================

/// Redact all known secret patterns from text
///
/// This is the main entry point for redaction. Use this function
/// before displaying, logging, or storing any text that might
/// contain secrets.
pub fn redact_secrets(text: &str) -> RedactionResult {
    let mut result = text.to_string();
    let mut count = 0;
    let mut types_found: HashSet<SecretType> = HashSet::new();

    for pattern in REDACTION_PATTERNS.iter() {
        let placeholder = pattern.secret_type.placeholder();

        if pattern.replace_full {
            // Replace the entire match
            let new_result = pattern.regex.replace_all(&result, placeholder);
            if new_result != result {
                count += pattern.regex.find_iter(&result).count();
                types_found.insert(pattern.secret_type);
                result = new_result.to_string();
            }
        } else {
            // Replace only the value part (preserve the key for context)
            let new_result = pattern.regex.replace_all(&result, |caps: &regex::Captures| {
                if caps.len() > 2 {
                    // Pattern has key and value groups: key=value -> key=[REDACTED:TYPE]
                    format!(
                        "{}{}",
                        caps.get(1).map(|m| m.as_str()).unwrap_or(""),
                        placeholder
                    )
                } else if caps.len() > 1 {
                    // Pattern has one capture group
                    placeholder.to_string()
                } else {
                    placeholder.to_string()
                }
            });
            if new_result != result {
                count += pattern.regex.find_iter(&result).count();
                types_found.insert(pattern.secret_type);
                result = new_result.to_string();
            }
        }
    }

    if count > 0 {
        RedactionResult::redacted(result, count, types_found.into_iter().collect())
    } else {
        RedactionResult::unchanged(result)
    }
}

/// Redact secrets from text, returning just the redacted string
///
/// Use this for simple cases where you don't need the metadata.
pub fn redact(text: &str) -> String {
    redact_secrets(text).text
}

/// Check if text contains potential secrets (without modifying it)
pub fn contains_secrets(text: &str) -> bool {
    REDACTION_PATTERNS.iter().any(|p| p.regex.is_match(text))
}

/// Get the types of secrets found in text (without modifying it)
pub fn detect_secret_types(text: &str) -> Vec<SecretType> {
    let mut types = HashSet::new();
    for pattern in REDACTION_PATTERNS.iter() {
        if pattern.regex.is_match(text) {
            types.insert(pattern.secret_type);
        }
    }
    types.into_iter().collect()
}

/// Redact environment variable values based on variable names
pub fn redact_env_value<'a>(name: &str, value: &'a str) -> Cow<'a, str> {
    // Check if the variable name matches any secret pattern
    for pattern in SECRET_ENV_PATTERNS.iter() {
        if pattern.is_match(name) {
            return Cow::Owned("[REDACTED:ENV_SECRET]".to_string());
        }
    }

    // Also check if the value itself looks like a secret
    if contains_secrets(value) {
        return Cow::Owned(redact(value));
    }

    Cow::Borrowed(value)
}

/// Redact a map of environment variables
pub fn redact_env_map(vars: &[(String, String)]) -> Vec<(String, String)> {
    vars.iter()
        .map(|(k, v)| (k.clone(), redact_env_value(k, v).to_string()))
        .collect()
}

// =============================================================================
// Restricted Paths (Evidence Policy)
// =============================================================================

/// Paths that should never be excerpted (content is restricted)
pub const RESTRICTED_EVIDENCE_PATHS: &[&str] = &[
    // SSH
    "~/.ssh/",
    "$HOME/.ssh/",
    "/home/*/.ssh/",
    "/root/.ssh/",

    // GPG
    "~/.gnupg/",
    "$HOME/.gnupg/",
    "/home/*/.gnupg/",
    "/root/.gnupg/",

    // System secrets
    "/etc/shadow",
    "/etc/gshadow",
    "/etc/sudoers.d/",

    // Keyrings
    "~/.local/share/keyrings/",
    "$HOME/.local/share/keyrings/",
    "/home/*/.local/share/keyrings/",

    // Browser profiles (credentials)
    "~/.mozilla/**/key*.db",
    "~/.mozilla/**/logins.json",
    "~/.config/chromium/**/Login Data",
    "~/.config/google-chrome/**/Login Data",
    "~/.config/BraveSoftware/**/Login Data",

    // Password stores
    "~/.password-store/",
    "$HOME/.password-store/",
    "~/.config/keepassxc/",

    // Credential files
    "~/.netrc",
    "~/.git-credentials",
    "~/.docker/config.json",
    "~/.kube/config",
    "~/.aws/credentials",
    "~/.azure/",
    "~/.config/gcloud/",

    // Process environment (contains secrets)
    "/proc/*/environ",
];

/// Check if a path is restricted for evidence extraction
pub fn is_path_restricted(path: &str) -> bool {
    let normalized = path.replace('\\', "/");

    for restricted in RESTRICTED_EVIDENCE_PATHS {
        if matches_restricted_pattern(&normalized, restricted) {
            return true;
        }
    }

    false
}

/// Match a path against a restricted pattern
fn matches_restricted_pattern(path: &str, pattern: &str) -> bool {
    // Handle ~ and $HOME separately - they represent any user's home dir
    let patterns_to_check = if pattern.starts_with('~') || pattern.starts_with("$HOME") {
        // Generate patterns for both /home/* and /root
        let without_tilde = if pattern.starts_with('~') {
            &pattern[1..]
        } else {
            &pattern[5..] // $HOME
        };
        vec![
            format!("/home/*{}", without_tilde),
            format!("/root{}", without_tilde),
        ]
    } else {
        vec![pattern.to_string()]
    };

    for expanded_pattern in patterns_to_check {
        if matches_pattern_impl(path, &expanded_pattern) {
            return true;
        }
    }

    false
}

/// Internal pattern matching implementation
fn matches_pattern_impl(path: &str, pattern: &str) -> bool {
    // Handle ** (any depth)
    if pattern.contains("/**/") {
        let parts: Vec<&str> = pattern.split("/**/").collect();
        if parts.len() == 2 {
            return path.starts_with(parts[0]) &&
                   (parts[1].is_empty() || path.contains(parts[1]));
        }
    }

    // Handle * (single segment wildcard)
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();

        // For single wildcard like /home/*/...
        if parts.len() == 2 {
            if parts[0].is_empty() && parts[1].is_empty() {
                return true; // "*" matches everything
            }
            if parts[0].is_empty() {
                return path.ends_with(parts[1]) || path.contains(parts[1]);
            }
            if parts[1].is_empty() {
                return path.starts_with(parts[0]);
            }
            // Check if path starts with prefix and remainder contains suffix
            if path.starts_with(parts[0]) {
                let remainder = &path[parts[0].len()..];
                // Find where the wildcard part ends (next /)
                if let Some(slash_pos) = remainder.find('/') {
                    let after_wildcard = &remainder[slash_pos..];
                    return after_wildcard.starts_with(parts[1]) || after_wildcard == parts[1];
                } else {
                    // No slash, so wildcard is the end
                    return parts[1].is_empty() || remainder.ends_with(parts[1]);
                }
            }
            return false;
        }

        // Multiple wildcards - use glob-style matching
        let mut path_pos = 0;
        let mut pattern_pos = 0;
        while pattern_pos < parts.len() && path_pos <= path.len() {
            let part = parts[pattern_pos];
            if part.is_empty() {
                pattern_pos += 1;
                continue;
            }
            if pattern_pos == 0 {
                if !path.starts_with(part) { return false; }
                path_pos = part.len();
            } else if pattern_pos == parts.len() - 1 {
                if !path.ends_with(part) { return false; }
                return true;
            } else {
                if let Some(found) = path[path_pos..].find(part) {
                    path_pos += found + part.len();
                } else {
                    return false;
                }
            }
            pattern_pos += 1;
        }
        return pattern_pos == parts.len();
    }

    // Exact match or prefix match for directories
    if pattern.ends_with('/') {
        path.starts_with(pattern)
    } else {
        path == pattern || path.starts_with(&format!("{}/", pattern))
    }
}

/// Get the restriction message for a restricted path
pub fn get_restriction_message(path: &str) -> String {
    format!(
        "Evidence exists but content is restricted by policy. Path: {} [{}{}]",
        path,
        REDACTION_EVIDENCE_PREFIX,
        generate_redaction_id()
    )
}

/// Generate a unique redaction evidence ID
pub fn generate_redaction_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u32 % 100000
}

// =============================================================================
// Redaction for Specific Contexts
// =============================================================================

/// Redact secrets from a transcript line
pub fn redact_transcript(text: &str) -> String {
    redact(text)
}

/// Redact secrets from evidence content
pub fn redact_evidence(content: &str, path: Option<&str>) -> Result<String, String> {
    // Check if path is restricted
    if let Some(p) = path {
        if is_path_restricted(p) {
            return Err(get_restriction_message(p));
        }
    }

    // Apply standard redaction
    Ok(redact(content))
}

/// Redact secrets from audit log details
pub fn redact_audit_details(details: &serde_json::Value) -> serde_json::Value {
    match details {
        serde_json::Value::String(s) => {
            serde_json::Value::String(redact(s))
        }
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                // Redact both key-value if key suggests secret
                let is_secret_key = SECRET_ENV_PATTERNS.iter().any(|p| p.is_match(k));
                if is_secret_key {
                    new_map.insert(k.clone(), serde_json::Value::String("[REDACTED:ENV_SECRET]".to_string()));
                } else {
                    new_map.insert(k.clone(), redact_audit_details(v));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(redact_audit_details).collect())
        }
        other => other.clone(),
    }
}

/// Redact secrets from memory/recipe content
pub fn redact_memory_content(content: &str) -> String {
    redact(content)
}

// =============================================================================
// Junior Verification Support
// =============================================================================

/// Check if a response appears to leak secrets (for Junior verification)
pub fn check_for_leaks(text: &str) -> LeakCheckResult {
    let secret_types = detect_secret_types(text);

    if secret_types.is_empty() {
        LeakCheckResult {
            has_leaks: false,
            leak_types: Vec::new(),
            recommendation: None,
            penalty: 0,
        }
    } else {
        let penalty = calculate_leak_penalty(&secret_types);
        LeakCheckResult {
            has_leaks: true,
            leak_types: secret_types,
            recommendation: Some("Response contains potential secrets. Apply redaction and regenerate.".to_string()),
            penalty,
        }
    }
}

/// Result of checking for secret leaks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakCheckResult {
    /// Whether leaks were detected
    pub has_leaks: bool,
    /// Types of secrets found
    pub leak_types: Vec<SecretType>,
    /// Recommendation for handling
    pub recommendation: Option<String>,
    /// Penalty to apply to reliability score
    pub penalty: i32,
}

/// Calculate the penalty for leaking certain secret types
fn calculate_leak_penalty(types: &[SecretType]) -> i32 {
    types.iter().map(|t| match t {
        SecretType::PrivateKey | SecretType::SshKey => -50,
        SecretType::Password | SecretType::AwsCredential => -40,
        SecretType::ApiKey | SecretType::BearerToken => -35,
        SecretType::DatabaseUrl | SecretType::ConnectionString => -35,
        SecretType::JwtToken | SecretType::OAuthToken => -30,
        SecretType::Cookie | SecretType::AuthHeader => -25,
        _ => -20,
    }).sum()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_password() {
        let text = "config password=secret123 done";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.text.contains("[REDACTED:PASSWORD]"));
        assert!(!result.text.contains("secret123"));
    }

    #[test]
    fn test_redact_api_key() {
        let text = "API_KEY=abc123xyz789";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.text.contains("[REDACTED:"));
        assert!(!result.text.contains("abc123xyz789"));
    }

    #[test]
    fn test_redact_bearer_token() {
        let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(!result.text.contains("eyJhbG"));
    }

    #[test]
    fn test_redact_private_key() {
        let text = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA0Z3VS5JJcds3xfn/ygWyF8PbnGy...
-----END RSA PRIVATE KEY-----"#;
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.text.contains("[REDACTED:PRIVATE_KEY]"));
        assert!(!result.text.contains("MIIEpAIBAAKCAQEA"));
    }

    #[test]
    fn test_redact_aws_credentials() {
        let text = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\nAWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.text.contains("[REDACTED:AWS_CREDENTIAL]"));
        assert!(!result.text.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_redact_database_url() {
        let text = "DATABASE_URL=postgres://user:password123@localhost:5432/mydb";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(!result.text.contains("password123"));
    }

    #[test]
    fn test_redact_git_credentials() {
        let text = "remote = https://user:secrettoken@github.com/org/repo.git";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.text.contains("[REDACTED:GIT_CREDENTIAL]"));
        assert!(!result.text.contains("secrettoken"));
    }

    #[test]
    fn test_redact_netrc() {
        let text = "machine github.com login user password ghp_secret123";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.text.contains("[REDACTED:NETRC]"));
        assert!(!result.text.contains("ghp_secret123"));
    }

    #[test]
    fn test_redact_jwt() {
        // JWT token without assignment context (e.g., in Authorization header or standalone)
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = redact_secrets(jwt);
        assert!(result.was_redacted, "JWT should be detected as secret");
        assert!(result.text.contains("[REDACTED:JWT]"), "JWT should be redacted with [REDACTED:JWT], got: {}", result.text);

        // JWT in bearer header
        let bearer_jwt = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.sig";
        let result2 = redact_secrets(bearer_jwt);
        assert!(result2.was_redacted, "Bearer JWT should be detected as secret");

        // When assigned to "token = X", generic secret pattern takes precedence
        // which is acceptable since both redact the sensitive value
        let token_assign = "token = eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.sig";
        let result3 = redact_secrets(token_assign);
        assert!(result3.was_redacted, "Token assignment should be redacted");
        assert!(result3.text.contains("[REDACTED:"), "Value should be redacted: {}", result3.text);
    }

    #[test]
    fn test_no_redaction_for_safe_text() {
        let text = "This is a normal message without any secrets";
        let result = redact_secrets(text);
        assert!(!result.was_redacted);
        assert_eq!(result.text, text);
    }

    #[test]
    fn test_contains_secrets() {
        assert!(contains_secrets("password=secret"));
        assert!(contains_secrets("Bearer abc123"));
        assert!(!contains_secrets("normal text"));
    }

    #[test]
    fn test_detect_secret_types() {
        let text = "password=secret123 and API_KEY=xyz";
        let types = detect_secret_types(text);
        assert!(!types.is_empty());
    }

    #[test]
    fn test_redact_env_value() {
        assert_eq!(redact_env_value("HOME", "/home/user").as_ref(), "/home/user");
        assert_eq!(redact_env_value("API_KEY", "secret123").as_ref(), "[REDACTED:ENV_SECRET]");
        assert_eq!(redact_env_value("AWS_SECRET_KEY", "awssecret").as_ref(), "[REDACTED:ENV_SECRET]");
    }

    #[test]
    fn test_is_path_restricted() {
        assert!(is_path_restricted("/home/user/.ssh/id_rsa"));
        assert!(is_path_restricted("/root/.gnupg/private-keys"));
        assert!(is_path_restricted("/etc/shadow"));
        assert!(is_path_restricted("/proc/1234/environ"));
        assert!(!is_path_restricted("/etc/nginx/nginx.conf"));
        assert!(!is_path_restricted("/home/user/.bashrc"));
    }

    #[test]
    fn test_check_for_leaks() {
        let safe_text = "This is a normal response about system configuration.";
        let leak_result = check_for_leaks(safe_text);
        assert!(!leak_result.has_leaks);

        let leaky_text = "Here is the password: secret123";
        let leak_result = check_for_leaks(leaky_text);
        assert!(leak_result.has_leaks);
        assert!(leak_result.penalty < 0);
    }

    #[test]
    fn test_redact_audit_details() {
        let details = serde_json::json!({
            "path": "/etc/config",
            "password": "secret123",
            "api_key": "key456"
        });
        let redacted = redact_audit_details(&details);
        let obj = redacted.as_object().unwrap();
        assert_eq!(obj.get("path").unwrap(), "/etc/config");
        assert!(obj.get("password").unwrap().as_str().unwrap().contains("REDACTED"));
        assert!(obj.get("api_key").unwrap().as_str().unwrap().contains("REDACTED"));
    }

    #[test]
    fn test_restricted_path_wildcards() {
        // Test home directory expansion
        assert!(is_path_restricted("/home/alice/.ssh/config"));
        assert!(is_path_restricted("/home/bob/.gnupg/pubring.kbx"));

        // Test credential files
        assert!(is_path_restricted("/home/user/.netrc"));
        assert!(is_path_restricted("/home/user/.aws/credentials"));
        assert!(is_path_restricted("/home/user/.git-credentials"));

        // Test non-restricted paths
        assert!(!is_path_restricted("/home/user/.bashrc"));
        assert!(!is_path_restricted("/etc/nginx/nginx.conf"));
    }

    #[test]
    fn test_secret_type_placeholders() {
        assert_eq!(SecretType::Password.placeholder(), "[REDACTED:PASSWORD]");
        assert_eq!(SecretType::ApiKey.placeholder(), "[REDACTED:API_KEY]");
        assert_eq!(SecretType::PrivateKey.placeholder(), "[REDACTED:PRIVATE_KEY]");
    }

    #[test]
    fn test_leak_penalty_calculation() {
        let types = vec![SecretType::PrivateKey, SecretType::Password];
        let penalty = calculate_leak_penalty(&types);
        assert!(penalty <= -90); // -50 + -40
    }

    #[test]
    fn test_redact_cookie() {
        let text = "Set-Cookie: session_id=abc123xyz; Path=/";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.text.contains("[REDACTED:COOKIE]"));
    }

    #[test]
    fn test_multiple_secrets_in_one_text() {
        let text = "config: password=secret123, api_key=abcdef, token=xyz789";
        let result = redact_secrets(text);
        assert!(result.was_redacted);
        assert!(result.redaction_count >= 2);
        assert!(!result.text.contains("secret123"));
        assert!(!result.text.contains("abcdef"));
    }

    #[test]
    fn test_get_restriction_message() {
        let msg = get_restriction_message("/home/user/.ssh/id_rsa");
        assert!(msg.contains("restricted by policy"));
        assert!(msg.contains(".ssh/id_rsa"));
        assert!(msg.contains(REDACTION_EVIDENCE_PREFIX));
    }
}
