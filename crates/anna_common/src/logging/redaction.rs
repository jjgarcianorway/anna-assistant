//! Redaction and Truncation Rules v0.8.0
//!
//! Protects sensitive information in logs:
//! - Redacts passwords, API tokens, private keys
//! - Truncates large outputs
//! - Sanitizes user queries

use super::config::TRUNCATION_LIMIT;
use regex::Regex;
use std::sync::LazyLock;

/// Marker for redacted content
pub const REDACTED: &str = "[REDACTED]";

/// Marker for truncated content
pub const TRUNCATED: &str = "[TRUNCATED]";

/// Patterns for secrets that should be redacted
static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Passwords in various formats
        Regex::new(r"(?i)(password|passwd|pwd)\s*[:=]\s*\S+").unwrap(),
        // API keys and tokens
        Regex::new(r"(?i)(api[_-]?key|api[_-]?token|auth[_-]?token|bearer)\s*[:=]\s*\S+").unwrap(),
        // Generic secrets
        Regex::new(r"(?i)(secret|private[_-]?key)\s*[:=]\s*\S+").unwrap(),
        // SSH private key content
        Regex::new(r"-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----")
            .unwrap(),
        // AWS credentials
        Regex::new(
            r"(?i)(aws[_-]?access[_-]?key[_-]?id|aws[_-]?secret[_-]?access[_-]?key)\s*[:=]\s*\S+",
        )
        .unwrap(),
        // Generic token patterns (long hex strings that look like tokens)
        Regex::new(r"(?i)(token|key)\s*[:=]\s*[a-f0-9]{32,}").unwrap(),
        // Database connection strings
        Regex::new(r"(?i)(mysql|postgres|mongodb|redis)://[^@]+@").unwrap(),
    ]
});

/// Environment variables known to contain secrets
const SECRET_ENV_VARS: &[&str] = &[
    "PASSWORD",
    "PASSWD",
    "SECRET",
    "API_KEY",
    "API_TOKEN",
    "AUTH_TOKEN",
    "BEARER_TOKEN",
    "AWS_SECRET_ACCESS_KEY",
    "AWS_ACCESS_KEY_ID",
    "DATABASE_URL",
    "DB_PASSWORD",
    "GITHUB_TOKEN",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
];

/// File paths that should never have contents logged
const SENSITIVE_PATHS: &[&str] = &[
    ".ssh/",
    ".gnupg/",
    ".aws/credentials",
    ".netrc",
    ".git-credentials",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
];

/// Redact secrets from a string
pub fn redact_secrets(input: &str) -> String {
    let mut result = input.to_string();

    for pattern in SECRET_PATTERNS.iter() {
        result = pattern.replace_all(&result, REDACTED).to_string();
    }

    result
}

/// Check if a path is sensitive and should not have contents logged
pub fn is_sensitive_path(path: &str) -> bool {
    SENSITIVE_PATHS.iter().any(|p| path.contains(p))
}

/// Check if an environment variable name is known to contain secrets
pub fn is_secret_env_var(name: &str) -> bool {
    let upper = name.to_uppercase();
    SECRET_ENV_VARS.iter().any(|s| upper.contains(s))
}

/// Truncate a string if it exceeds the limit
pub fn truncate(input: &str, limit: usize) -> String {
    if input.len() <= limit {
        return input.to_string();
    }

    let truncated_len = limit.saturating_sub(TRUNCATED.len() + 10);
    let preview = &input[..truncated_len.min(input.len())];

    format!(
        "{} {} (original: {} bytes)",
        preview,
        TRUNCATED,
        input.len()
    )
}

/// Truncate with default limit
pub fn truncate_default(input: &str) -> String {
    truncate(input, TRUNCATION_LIMIT)
}

/// Sanitize a user query for logging
/// Removes obvious secrets but preserves the query structure
pub fn sanitize_query(query: &str) -> String {
    let redacted = redact_secrets(query);

    // Also truncate very long queries
    truncate(&redacted, 1024)
}

/// Summarize large data for logging
/// Returns a compact representation with key metrics
pub fn summarize_large_data(data: &str, _data_type: &str) -> String {
    let line_count = data.lines().count();
    let byte_count = data.len();

    if byte_count <= TRUNCATION_LIMIT {
        return redact_secrets(data);
    }

    // Take first few lines as preview
    let preview_lines: Vec<&str> = data.lines().take(5).collect();
    let preview = preview_lines.join("\n");

    format!(
        "{}\n... {} ({} lines, {} bytes total)",
        redact_secrets(&preview),
        TRUNCATED,
        line_count,
        byte_count
    )
}

/// Create a hash of content for debugging large payloads
pub fn content_hash(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_password() {
        let input = "database config: password=mysecret123";
        let result = redact_secrets(input);
        assert!(!result.contains("mysecret123"));
        assert!(result.contains(REDACTED));
    }

    #[test]
    fn test_redact_api_key() {
        let input = "api_key=abcdef1234567890";
        let result = redact_secrets(input);
        assert!(!result.contains("abcdef1234567890"));
        assert!(result.contains(REDACTED));
    }

    #[test]
    fn test_redact_ssh_key() {
        let input = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA...
-----END RSA PRIVATE KEY-----"#;
        let result = redact_secrets(input);
        assert!(!result.contains("MIIEowIBAAKCAQEA"));
        assert!(result.contains(REDACTED));
    }

    #[test]
    fn test_no_redaction_needed() {
        let input = "How many CPU cores do I have?";
        let result = redact_secrets(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_truncate_short() {
        let input = "short text";
        let result = truncate(input, 100);
        assert_eq!(result, input);
    }

    #[test]
    fn test_truncate_long() {
        let input = "a".repeat(5000);
        let result = truncate(&input, 100);
        assert!(result.contains(TRUNCATED));
        assert!(result.len() < input.len());
    }

    #[test]
    fn test_is_sensitive_path() {
        assert!(is_sensitive_path("/home/user/.ssh/id_rsa"));
        assert!(is_sensitive_path("/root/.gnupg/secring.gpg"));
        assert!(!is_sensitive_path("/etc/passwd"));
        assert!(!is_sensitive_path("/proc/cpuinfo"));
    }

    #[test]
    fn test_is_secret_env_var() {
        assert!(is_secret_env_var("API_KEY"));
        assert!(is_secret_env_var("GITHUB_TOKEN"));
        assert!(is_secret_env_var("db_password"));
        assert!(!is_secret_env_var("HOME"));
        assert!(!is_secret_env_var("PATH"));
    }

    #[test]
    fn test_sanitize_query() {
        let query = "connect to mysql://user:password@localhost";
        let result = sanitize_query(query);
        assert!(!result.contains("password"));
    }

    #[test]
    fn test_summarize_large_data() {
        let large = "line\n".repeat(1000);
        let summary = summarize_large_data(&large, "text");
        assert!(summary.contains(TRUNCATED));
        assert!(summary.contains("lines"));
        assert!(summary.contains("bytes"));
    }

    #[test]
    fn test_content_hash() {
        let data = b"test data";
        let hash = content_hash(data);
        assert_eq!(hash.len(), 16);
        // Same input should produce same hash
        assert_eq!(hash, content_hash(data));
    }
}
