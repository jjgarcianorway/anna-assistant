//! Redaction Functions - v0.0.443.
//!
//! Functions for redacting sensitive data from trace logs.

/// Redact sensitive fields from JSON.
pub fn redact_sensitive(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                if is_sensitive_key(key) {
                    new_map.insert(
                        key.clone(),
                        serde_json::Value::String("[REDACTED]".to_string()),
                    );
                } else {
                    new_map.insert(key.clone(), redact_sensitive(val));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(redact_sensitive).collect())
        }
        serde_json::Value::String(s) if looks_like_secret(s) => {
            serde_json::Value::String("[REDACTED]".to_string())
        }
        other => other.clone(),
    }
}

/// Check if a key name is sensitive.
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("token")
        || lower.contains("key")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("auth")
        || lower.contains("credential")
}

/// Check if a string value looks like a secret.
pub fn looks_like_secret(s: &str) -> bool {
    // SSH private keys
    if s.contains("-----BEGIN") && (s.contains("PRIVATE KEY") || s.contains("RSA")) {
        return true;
    }

    // API keys (long alphanumeric strings)
    if s.len() > 30
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return true;
    }

    // Bearer tokens
    if s.starts_with("Bearer ") || s.starts_with("sk-") || s.starts_with("ghp_") {
        return true;
    }

    false
}

/// Format JSON with indentation.
pub fn format_json_indented(value: &serde_json::Value, indent: usize) -> String {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_default();
    let prefix = " ".repeat(indent);
    pretty
        .lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction() {
        let value = serde_json::json!({
            "query": "test",
            "api_key": "sk-12345",
            "data": {
                "token": "secret123"
            }
        });

        let redacted = redact_sensitive(&value);
        assert_eq!(redacted["api_key"], "[REDACTED]");
        assert_eq!(redacted["data"]["token"], "[REDACTED]");
        assert_eq!(redacted["query"], "test");
    }

    #[test]
    fn test_sensitive_detection() {
        assert!(is_sensitive_key("api_key"));
        assert!(is_sensitive_key("AUTH_TOKEN"));
        assert!(!is_sensitive_key("query"));

        assert!(looks_like_secret("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(looks_like_secret("sk-proj-abc123"));
        assert!(!looks_like_secret("hello world"));
    }
}
