//! Sanitization for debug output (v0.0.444).
//!
//! Redacts sensitive information from debug logs.
//! HARD RULE: If sanitization fails, fall back to TRACE level.

use super::config::RedactConfig;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Result of sanitization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizeResult {
    /// Sanitized content
    pub content: String,
    /// Number of redactions made
    pub redaction_count: u32,
    /// Whether sanitization succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl SanitizeResult {
    /// Create successful result.
    pub fn ok(content: String, redaction_count: u32) -> Self {
        Self {
            content,
            redaction_count,
            success: true,
            error: None,
        }
    }

    /// Create failed result.
    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            content: "[SANITIZATION_FAILED]".to_string(),
            redaction_count: 0,
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Sanitizer for redacting sensitive information.
pub struct Sanitizer {
    config: RedactConfig,
    // Compiled regexes
    email_re: Regex,
    ip_private_re: Regex,
    key_value_re: Regex,
    auth_header_re: Regex,
    ssh_key_re: Regex,
    password_re: Regex,
}

impl Sanitizer {
    /// Create sanitizer with config.
    pub fn new(config: RedactConfig) -> Self {
        Self {
            config,
            // Email pattern
            email_re: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
                .expect("valid email regex"),
            // Private IP ranges (10.x, 172.16-31.x, 192.168.x)
            ip_private_re: Regex::new(
                r"(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[0-1])\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3})"
            ).expect("valid IP regex"),
            // KEY=, TOKEN=, SECRET=, API_KEY= patterns (PASSWORD handled separately)
            key_value_re: Regex::new(
                r"(?i)(?:KEY|TOKEN|SECRET|API_KEY|APIKEY|AUTH|CREDENTIAL)\s*[=:]\s*[^\s]+"
            ).expect("valid key-value regex"),
            // Authorization header
            auth_header_re: Regex::new(r"(?i)Authorization:\s*[^\r\n]+")
                .expect("valid auth header regex"),
            // SSH key content (-----BEGIN ... KEY-----)
            ssh_key_re: Regex::new(r"-----BEGIN [A-Z ]+ KEY-----[\s\S]*?-----END [A-Z ]+ KEY-----")
                .expect("valid SSH key regex"),
            // Password-like patterns in configs
            password_re: Regex::new(r#"(?i)["']?password["']?\s*[=:]\s*["']?[^"'\s,}]+["']?"#)
                .expect("valid password regex"),
        }
    }

    /// Create sanitizer with default config.
    pub fn default() -> Self {
        Self::new(RedactConfig::default())
    }

    /// Sanitize text content.
    pub fn sanitize(&self, input: &str) -> SanitizeResult {
        let mut content = input.to_string();
        let mut count = 0u32;

        // Always redact sensitive files (SSH keys, shadow-like content)
        if self.config.redact_sensitive_files {
            let (new_content, n) = self.redact_sensitive_files(&content);
            content = new_content;
            count += n;
        }

        // Always redact secrets (KEY=, TOKEN=, Authorization:)
        if self.config.redact_secrets {
            let (new_content, n) = self.redact_secrets(&content);
            content = new_content;
            count += n;
        }

        // Optional: redact emails
        if self.config.redact_emails {
            let (new_content, n) = self.redact_emails(&content);
            content = new_content;
            count += n;
        }

        // Optional: redact private IPs
        if self.config.redact_private_ips {
            let (new_content, n) = self.redact_private_ips(&content);
            content = new_content;
            count += n;
        }

        SanitizeResult::ok(content, count)
    }

    /// Sanitize and truncate text.
    pub fn sanitize_and_truncate(&self, input: &str, max_chars: usize) -> SanitizeResult {
        let result = self.sanitize(input);
        if !result.success {
            return result;
        }

        if result.content.len() <= max_chars {
            return result;
        }

        let truncated = format!(
            "{}...[TRUNCATED {} chars]",
            &result.content[..max_chars],
            result.content.len() - max_chars
        );

        SanitizeResult::ok(truncated, result.redaction_count)
    }

    /// Sanitize probe output with line limit.
    pub fn sanitize_probe_output(&self, input: &str) -> SanitizeResult {
        let result = self.sanitize(input);
        if !result.success {
            return result;
        }

        let lines: Vec<&str> = result.content.lines().collect();
        if lines.len() <= self.config.max_probe_lines {
            return result;
        }

        let truncated_lines: Vec<&str> = lines[..self.config.max_probe_lines].to_vec();
        let truncated = format!(
            "{}\n...[TRUNCATED {} lines]",
            truncated_lines.join("\n"),
            lines.len() - self.config.max_probe_lines
        );

        SanitizeResult::ok(truncated, result.redaction_count)
    }

    /// Sanitize LLM output with character limit.
    pub fn sanitize_llm_output(&self, input: &str) -> SanitizeResult {
        self.sanitize_and_truncate(input, self.config.max_llm_output_chars)
    }

    // === Internal redaction methods ===

    fn redact_sensitive_files(&self, input: &str) -> (String, u32) {
        let mut content = input.to_string();
        let mut count = 0u32;

        // SSH keys
        if self.ssh_key_re.is_match(&content) {
            content = self
                .ssh_key_re
                .replace_all(&content, "[REDACTED_SSH_KEY]")
                .to_string();
            count += 1;
        }

        // /etc/shadow content (root:$..., user:$...)
        if content.contains("/etc/shadow") || content.contains(":$6$") || content.contains(":$y$") {
            // Redact password hashes
            let shadow_re = Regex::new(r"\$[0-9a-z]+\$[^\s:]+").expect("valid shadow regex");
            if shadow_re.is_match(&content) {
                content = shadow_re
                    .replace_all(&content, "[REDACTED_HASH]")
                    .to_string();
                count += 1;
            }
        }

        // ~/.ssh paths - redact content after them
        if content.contains("/.ssh/") {
            // Keep the path but redact key content
            let ssh_content_re =
                Regex::new(r"(~?/[^\s]*\.ssh/[^\s]+)\s+[^\n]+").expect("valid ssh path regex");
            if ssh_content_re.is_match(&content) {
                content = ssh_content_re
                    .replace_all(&content, "$1 [REDACTED_CONTENT]")
                    .to_string();
                count += 1;
            }
        }

        (content, count)
    }

    fn redact_secrets(&self, input: &str) -> (String, u32) {
        let mut content = input.to_string();
        let mut count = 0u32;

        // KEY=value, TOKEN=value, etc.
        if self.key_value_re.is_match(&content) {
            let matches = self.key_value_re.find_iter(&content).count();
            content = self
                .key_value_re
                .replace_all(&content, "[REDACTED_SECRET]")
                .to_string();
            count += matches as u32;
        }

        // Authorization headers
        if self.auth_header_re.is_match(&content) {
            let matches = self.auth_header_re.find_iter(&content).count();
            content = self
                .auth_header_re
                .replace_all(&content, "Authorization: [REDACTED]")
                .to_string();
            count += matches as u32;
        }

        // Password fields
        if self.password_re.is_match(&content) {
            let matches = self.password_re.find_iter(&content).count();
            content = self
                .password_re
                .replace_all(&content, "password=[REDACTED]")
                .to_string();
            count += matches as u32;
        }

        (content, count)
    }

    fn redact_emails(&self, input: &str) -> (String, u32) {
        if !self.email_re.is_match(input) {
            return (input.to_string(), 0);
        }

        let matches = self.email_re.find_iter(input).count();
        let content = self
            .email_re
            .replace_all(input, "[REDACTED_EMAIL]")
            .to_string();
        (content, matches as u32)
    }

    fn redact_private_ips(&self, input: &str) -> (String, u32) {
        if !self.ip_private_re.is_match(input) {
            return (input.to_string(), 0);
        }

        let matches = self.ip_private_re.find_iter(input).count();
        let content = self
            .ip_private_re
            .replace_all(input, "[REDACTED_IP]")
            .to_string();
        (content, matches as u32)
    }
}

/// Quick sanitization with default config.
pub fn sanitize(input: &str) -> SanitizeResult {
    Sanitizer::default().sanitize(input)
}

/// Quick sanitization of probe output.
pub fn sanitize_probe(input: &str) -> SanitizeResult {
    Sanitizer::default().sanitize_probe_output(input)
}

/// Quick sanitization of LLM output.
pub fn sanitize_llm(input: &str) -> SanitizeResult {
    Sanitizer::default().sanitize_llm_output(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_email() {
        let sanitizer = Sanitizer::default();
        let result = sanitizer.sanitize("Contact: user@example.com for help");
        assert!(result.success);
        assert!(result.content.contains("[REDACTED_EMAIL]"));
        assert!(!result.content.contains("user@example.com"));
        assert!(result.redaction_count > 0);
    }

    #[test]
    fn test_sanitize_private_ip() {
        let sanitizer = Sanitizer::default();
        let result = sanitizer.sanitize("Server at 192.168.1.100 is down");
        assert!(result.success);
        assert!(result.content.contains("[REDACTED_IP]"));
        assert!(!result.content.contains("192.168.1.100"));
    }

    #[test]
    fn test_sanitize_secrets() {
        let sanitizer = Sanitizer::default();

        // API key
        let result = sanitizer.sanitize("API_KEY=sk-abc123xyz");
        assert!(result.content.contains("[REDACTED_SECRET]"));
        assert!(!result.content.contains("sk-abc123xyz"));

        // Authorization header
        let result = sanitizer.sanitize("Authorization: Bearer token123");
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("token123"));
    }

    #[test]
    fn test_sanitize_ssh_key() {
        let sanitizer = Sanitizer::default();
        let input = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA0Z...
-----END RSA PRIVATE KEY-----"#;

        let result = sanitizer.sanitize(input);
        assert!(result.content.contains("[REDACTED_SSH_KEY]"));
        assert!(!result.content.contains("MIIEpAIBAAKCAQEA0Z"));
    }

    #[test]
    fn test_sanitize_truncate() {
        let sanitizer = Sanitizer::default();
        let input = "x".repeat(5000);
        let result = sanitizer.sanitize_and_truncate(&input, 100);
        assert!(result.success);
        assert!(result.content.len() < 200);
        assert!(result.content.contains("[TRUNCATED"));
    }

    #[test]
    fn test_sanitize_probe_lines() {
        let mut config = RedactConfig::default();
        config.max_probe_lines = 5;
        let sanitizer = Sanitizer::new(config);

        let input = (1..=20)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let result = sanitizer.sanitize_probe_output(&input);
        assert!(result.success);
        assert!(result.content.contains("[TRUNCATED"));
        assert!(result.content.contains("Line 1"));
        assert!(!result.content.contains("Line 20"));
    }

    #[test]
    fn test_sanitize_disabled() {
        let mut config = RedactConfig::default();
        config.redact_emails = false;
        config.redact_private_ips = false;
        let sanitizer = Sanitizer::new(config);

        // Emails and IPs should NOT be redacted
        let result = sanitizer.sanitize("Contact user@example.com at 192.168.1.1");
        assert!(result.content.contains("user@example.com"));
        assert!(result.content.contains("192.168.1.1"));

        // But secrets should still be redacted
        let result = sanitizer.sanitize("API_KEY=secret123");
        assert!(result.content.contains("[REDACTED_SECRET]"));
    }

    #[test]
    fn test_no_false_positives() {
        let sanitizer = Sanitizer::default();

        // Normal text should not be redacted
        let result = sanitizer.sanitize("The disk is 80% full. Check /var/log for details.");
        assert_eq!(result.redaction_count, 0);
        assert!(result.content.contains("80% full"));
    }
}
