//! Evidence redaction rules.
//!
//! Removes sensitive data patterns from probe outputs before display.
//! Applied even in debug mode for security.

use regex::Regex;
use std::sync::LazyLock;

/// Patterns that should be redacted
static REDACTION_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Private keys
        (
            Regex::new(r"-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----").unwrap(),
            "[REDACTED: private key]",
        ),
        // SSH private key file paths
        (
            Regex::new(r"(/[\w/.-]*id_rsa|/[\w/.-]*id_ed25519|/[\w/.-]*id_ecdsa)(?:\s|$)").unwrap(),
            "[REDACTED: ssh key path] ",
        ),
        // /etc/shadow content patterns
        (
            Regex::new(r"^\w+:\$[0-9a-zA-Z$./]+:[0-9:]+$").unwrap(),
            "[REDACTED: shadow entry]",
        ),
        // Password hashes ($6$, $5$, $y$, etc.)
        (
            Regex::new(r"\$[0-9a-z]+\$[a-zA-Z0-9./]+\$[a-zA-Z0-9./]+").unwrap(),
            "[REDACTED: password hash]",
        ),
        // AWS access keys
        (
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            "[REDACTED: AWS access key]",
        ),
        // AWS secret keys (40 char base64)
        (
            Regex::new(r"(?i)(aws_secret_access_key|secret_key)\s*[=:]\s*[a-zA-Z0-9/+=]{40}").unwrap(),
            "[REDACTED: AWS secret]",
        ),
        // Generic API keys
        (
            Regex::new(r"(?i)(api_key|apikey|api-key)\s*[=:]\s*[a-zA-Z0-9_-]{20,}").unwrap(),
            "[REDACTED: API key]",
        ),
        // Bearer tokens
        (
            Regex::new(r"(?i)bearer\s+[a-zA-Z0-9._-]{20,}").unwrap(),
            "[REDACTED: bearer token]",
        ),
        // Database connection strings with passwords
        (
            Regex::new(r"(?i)(mysql|postgres|mongodb)://[^:]+:[^@]+@").unwrap(),
            "[REDACTED: db connection] ",
        ),
        // Generic password in config
        (
            Regex::new(r#"(?i)(password|passwd|pwd)\s*[=:]\s*["']?[^\s"']{8,}["']?"#).unwrap(),
            "[REDACTED: password]",
        ),
        // /etc/shadow file path mentions
        (
            Regex::new(r"/etc/shadow").unwrap(),
            "/etc/[REDACTED]",
        ),
    ]
});

/// Redact sensitive patterns from text
pub fn redact(text: &str) -> String {
    let mut result = text.to_string();

    for (pattern, replacement) in REDACTION_PATTERNS.iter() {
        result = pattern.replace_all(&result, *replacement).to_string();
    }

    result
}

/// Redact sensitive patterns from probe output
pub fn redact_probe_output(stdout: &str, stderr: &str) -> (String, String) {
    (redact(stdout), redact(stderr))
}

/// Check if text contains sensitive patterns
pub fn contains_sensitive(text: &str) -> bool {
    REDACTION_PATTERNS
        .iter()
        .any(|(pattern, _)| pattern.is_match(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_private_key() {
        let text = r#"Here is a key:
-----BEGIN RSA PRIVATE KEY-----
MIIEpQIBAAKCAQEA0Z3VS...
-----END RSA PRIVATE KEY-----
Done."#;
        let redacted = redact(text);
        assert!(redacted.contains("[REDACTED: private key]"));
        assert!(!redacted.contains("MIIEpQIBAAKCAQEA0Z3VS"));
    }

    #[test]
    fn test_redact_password_hash() {
        let text = "user:$6$rounds=5000$salt$hashedpassword:19000:0:99999:7:::";
        let redacted = redact(text);
        assert!(redacted.contains("[REDACTED"));
    }

    #[test]
    fn test_redact_aws_key() {
        let text = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let redacted = redact(text);
        assert!(redacted.contains("[REDACTED: AWS access key]"));
    }

    #[test]
    fn test_redact_api_key() {
        let text = "api_key=test_token_xyz_abc_def_ghij_klm_nop";
        let redacted = redact(text);
        assert!(redacted.contains("[REDACTED: API key]"));
    }

    #[test]
    fn test_redact_shadow_path() {
        let text = "cat /etc/shadow";
        let redacted = redact(text);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("/etc/shadow"));
    }

    #[test]
    fn test_redact_db_connection() {
        let text = "DATABASE_URL=postgres://user:secretpass@localhost/db";
        let redacted = redact(text);
        assert!(redacted.contains("[REDACTED: db connection]"));
    }

    #[test]
    fn test_normal_text_unchanged() {
        let text = "CPU: Intel Core i7-9700K @ 3.60GHz (8 cores)";
        let redacted = redact(text);
        assert_eq!(text, redacted);
    }

    #[test]
    fn test_contains_sensitive() {
        assert!(contains_sensitive("password=secret123456"));
        assert!(!contains_sensitive("hello world"));
    }
}
