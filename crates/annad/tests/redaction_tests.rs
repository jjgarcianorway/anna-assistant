//! Tests for evidence redaction rules.
//!
//! Verifies sensitive data patterns are properly redacted.

fn redact(text: &str) -> String {
    let patterns = [
        // Private keys
        (r"-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----", "[REDACTED: private key]"),
        // AWS access keys (AKIA...)
        (r"AKIA[0-9A-Z]{16}", "[REDACTED: AWS key]"),
        // Password patterns
        (r"(?i)password\s*[=:]\s*\S+", "[REDACTED: password]"),
        // /etc/shadow paths
        (r"/etc/shadow", "/etc/[REDACTED]"),
    ];

    let mut result = text.to_string();
    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }
    result
}

fn contains_sensitive(text: &str) -> bool {
    text != redact(text)
}

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
fn test_redact_aws_key() {
    let text = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
    let redacted = redact(text);
    assert!(redacted.contains("[REDACTED: AWS key]"));
    assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn test_redact_password() {
    let text = "config: password=super_secret_123";
    let redacted = redact(text);
    assert!(redacted.contains("[REDACTED: password]"));
    assert!(!redacted.contains("super_secret_123"));
}

#[test]
fn test_redact_shadow_path() {
    let text = "cat /etc/shadow";
    let redacted = redact(text);
    assert!(redacted.contains("[REDACTED]"));
    assert!(!redacted.contains("/etc/shadow"));
}

#[test]
fn test_normal_text_unchanged() {
    let text = "CPU: Intel Core i7 (8 cores)";
    let redacted = redact(text);
    assert_eq!(text, redacted);
}

#[test]
fn test_contains_sensitive_detection() {
    assert!(contains_sensitive("password=secret"));
    assert!(contains_sensitive("AKIAIOSFODNN7EXAMPLE"));
    assert!(!contains_sensitive("hello world"));
    assert!(!contains_sensitive("CPU usage: 50%"));
}

#[test]
fn test_multiple_sensitive_patterns() {
    let text = "config:\n  password=secret\n  aws_key=AKIAIOSFODNN7EXAMPLE";
    let redacted = redact(text);
    assert!(redacted.contains("[REDACTED: password]"));
    assert!(redacted.contains("[REDACTED: AWS key]"));
    assert!(!redacted.contains("secret"));
    assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE"));
}
