//! Evidence redaction rules (v0.0.324).
//!
//! Removes sensitive data patterns from probe outputs before display.
//! Applied even in debug mode for security.
//!
//! v0.0.290: Added LLM reasoning tag stripping (<think>, etc.) to prevent
//!           internal model dialog from leaking into user-facing answers.
//! v0.0.324: Added quality self-assessment extraction for learning.

use regex::Regex;
use std::sync::LazyLock;

/// Patterns that should be redacted
static REDACTION_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Private keys
        (
            Regex::new(
                r"-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----",
            )
            .unwrap(),
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
            Regex::new(r"(?i)(aws_secret_access_key|secret_key)\s*[=:]\s*[a-zA-Z0-9/+=]{40}")
                .unwrap(),
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
        (Regex::new(r"/etc/shadow").unwrap(), "/etc/[REDACTED]"),
    ]
});

/// v0.0.290: Patterns for stripping LLM reasoning/thinking tags
/// These are used by models like DeepSeek-R1 that expose internal reasoning
static LLM_REASONING_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // DeepSeek-R1 style <think>...</think> tags
        Regex::new(r"(?s)<think>.*?</think>").unwrap(),
        // Alternative thinking tags
        Regex::new(r"(?s)<thinking>.*?</thinking>").unwrap(),
        // Internal reasoning blocks
        Regex::new(r"(?s)<reasoning>.*?</reasoning>").unwrap(),
        // Scratchpad blocks
        Regex::new(r"(?s)<scratchpad>.*?</scratchpad>").unwrap(),
        // Internal monologue
        Regex::new(r"(?s)<internal>.*?</internal>").unwrap(),
        // Reflection blocks
        Regex::new(r"(?s)<reflection>.*?</reflection>").unwrap(),
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

/// v0.0.290: Strip LLM reasoning/thinking tags from responses
/// This prevents internal model dialog from leaking into user-facing answers
pub fn strip_reasoning_tags(text: &str) -> String {
    let mut result = text.to_string();

    for pattern in LLM_REASONING_PATTERNS.iter() {
        result = pattern.replace_all(&result, "").to_string();
    }

    // Clean up any extra whitespace left behind
    result = result.trim().to_string();
    // Collapse multiple newlines
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    result
}

/// v0.0.290: Combined redaction and reasoning tag stripping
/// Use this for LLM responses to ensure both security and clean output
pub fn clean_llm_response(text: &str) -> String {
    let stripped = strip_reasoning_tags(text);
    // v0.0.324: Strip quality assessment from user-visible output
    let without_quality = strip_quality_assessment(&stripped);
    redact(&without_quality)
}

/// v0.0.324: Extract quality self-assessment from LLM response
/// Returns (quality_score, cleaned_text) where score is 1-5 or None
pub fn extract_quality_assessment(text: &str) -> (Option<u8>, String) {
    // Pattern: [QUALITY: X/5] at end of response
    static QUALITY_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[QUALITY:\s*([1-5])/5\]").unwrap());

    if let Some(caps) = QUALITY_PATTERN.captures(text) {
        if let Some(score_match) = caps.get(1) {
            if let Ok(score) = score_match.as_str().parse::<u8>() {
                // Remove the quality tag from the response
                let cleaned = QUALITY_PATTERN.replace_all(text, "").trim().to_string();
                return (Some(score), cleaned);
            }
        }
    }

    (None, text.to_string())
}

/// v0.0.324: Strip quality assessment from text (for clean display)
fn strip_quality_assessment(text: &str) -> String {
    static QUALITY_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[QUALITY:\s*[1-5]/5\]").unwrap());

    QUALITY_PATTERN.replace_all(text, "").trim().to_string()
}

/// Redact sensitive patterns from probe output
pub fn redact_probe_output(stdout: &str, stderr: &str) -> (String, String) {
    (redact(stdout), redact(stderr))
}

/// Check if text contains sensitive patterns
#[allow(dead_code)]
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

    // v0.0.290: Tests for LLM reasoning tag stripping

    #[test]
    fn test_strip_think_tags() {
        let text = "<think>Let me analyze this query...</think>Your disk is 50% full.";
        let stripped = strip_reasoning_tags(text);
        assert_eq!(stripped, "Your disk is 50% full.");
        assert!(!stripped.contains("<think>"));
    }

    #[test]
    fn test_strip_thinking_tags() {
        let text = "<thinking>I should check the memory...</thinking>You have 8GB RAM.";
        let stripped = strip_reasoning_tags(text);
        assert_eq!(stripped, "You have 8GB RAM.");
    }

    #[test]
    fn test_strip_multiline_thinking() {
        let text = r#"<think>
First, I need to understand what the user wants.
They're asking about disk space.
Let me check the df output.
</think>

Your /home partition is 75% full with 50GB available."#;
        let stripped = strip_reasoning_tags(text);
        assert!(stripped.contains("Your /home partition"));
        assert!(!stripped.contains("understand what the user wants"));
    }

    #[test]
    fn test_strip_multiple_thinking_blocks() {
        let text = "<think>First thought</think>Answer part 1. <think>Second thought</think>Answer part 2.";
        let stripped = strip_reasoning_tags(text);
        assert_eq!(stripped, "Answer part 1. Answer part 2.");
    }

    #[test]
    fn test_clean_llm_response() {
        let text = "<think>Check if there are secrets...</think>password=secret123456 is unsafe";
        let cleaned = clean_llm_response(text);
        assert!(!cleaned.contains("<think>"));
        assert!(cleaned.contains("[REDACTED: password]"));
    }

    #[test]
    fn test_no_reasoning_tags_unchanged() {
        let text = "Your system has 16GB RAM and 500GB SSD.";
        let stripped = strip_reasoning_tags(text);
        assert_eq!(text, stripped);
    }

    // v0.0.324: Tests for quality self-assessment extraction

    #[test]
    fn test_extract_quality_score() {
        let text = "Your disk is 50% full.\n\n[QUALITY: 5/5]";
        let (score, cleaned) = extract_quality_assessment(text);
        assert_eq!(score, Some(5));
        assert_eq!(cleaned, "Your disk is 50% full.");
    }

    #[test]
    fn test_extract_quality_score_partial() {
        let text = "I couldn't find that information.\n[QUALITY: 2/5]";
        let (score, cleaned) = extract_quality_assessment(text);
        assert_eq!(score, Some(2));
        assert!(!cleaned.contains("[QUALITY"));
    }

    #[test]
    fn test_no_quality_score() {
        let text = "Your system has 16GB RAM.";
        let (score, cleaned) = extract_quality_assessment(text);
        assert_eq!(score, None);
        assert_eq!(cleaned, text);
    }

    #[test]
    fn test_quality_stripped_from_clean_response() {
        let text = "Answer here.\n\n[QUALITY: 4/5]";
        let cleaned = clean_llm_response(text);
        assert!(!cleaned.contains("[QUALITY"));
        assert!(cleaned.contains("Answer here"));
    }
}
