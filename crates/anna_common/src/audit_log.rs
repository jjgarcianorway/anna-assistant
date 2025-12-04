//! Audit Log System v0.0.14
//!
//! Structured audit logging for all tool calls (read-only and mutation).
//! Provides security audit trail and traceability.
//!
//! Storage: /var/lib/anna/audit/

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Audit log directory
pub const AUDIT_DIR: &str = "/var/lib/anna/audit";

/// Current audit log file
pub const AUDIT_LOG_FILE: &str = "/var/lib/anna/audit/audit.jsonl";

/// Archived audit logs directory
pub const AUDIT_ARCHIVE_DIR: &str = "/var/lib/anna/audit/archive";

/// Maximum audit log size before rotation (10 MB)
pub const MAX_AUDIT_LOG_SIZE: u64 = 10_485_760;

/// Audit entry types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEntryType {
    /// Read-only tool execution
    ReadOnlyTool,
    /// Mutation tool execution
    MutationTool,
    /// Policy check
    PolicyCheck,
    /// Confirmation received
    Confirmation,
    /// Action blocked
    ActionBlocked,
    /// Rollback performed
    Rollback,
    /// Session start
    SessionStart,
    /// Session end
    SessionEnd,
    /// Policy reload
    PolicyReload,
    /// Security event
    SecurityEvent,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Entry type
    pub entry_type: AuditEntryType,

    /// Session ID (if applicable)
    pub session_id: Option<String>,

    /// Tool name (if applicable)
    pub tool_name: Option<String>,

    /// Evidence ID (if applicable)
    pub evidence_id: Option<String>,

    /// Request summary (sanitized)
    pub request_summary: Option<String>,

    /// Result (success/failure/blocked)
    pub result: AuditResult,

    /// Additional details
    pub details: Option<serde_json::Value>,

    /// User (from environment)
    pub user: Option<String>,

    /// Policy rule involved (if applicable)
    pub policy_rule: Option<String>,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
    Blocked,
    Pending,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(entry_type: AuditEntryType, result: AuditResult) -> Self {
        Self {
            timestamp: Utc::now(),
            entry_type,
            session_id: None,
            tool_name: None,
            evidence_id: None,
            request_summary: None,
            result,
            details: None,
            user: std::env::var("USER").ok(),
            policy_rule: None,
        }
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Set tool name
    pub fn with_tool(mut self, tool_name: &str) -> Self {
        self.tool_name = Some(tool_name.to_string());
        self
    }

    /// Set evidence ID
    pub fn with_evidence(mut self, evidence_id: &str) -> Self {
        self.evidence_id = Some(evidence_id.to_string());
        self
    }

    /// Set request summary (sanitized)
    pub fn with_request(mut self, request: &str) -> Self {
        self.request_summary = Some(sanitize_for_audit(request));
        self
    }

    /// Set details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Set policy rule
    pub fn with_policy_rule(mut self, rule: &str) -> Self {
        self.policy_rule = Some(rule.to_string());
        self
    }
}

/// Audit logger
pub struct AuditLogger;

impl AuditLogger {
    /// Ensure audit directory exists with proper permissions
    pub fn ensure_dirs() -> std::io::Result<()> {
        fs::create_dir_all(AUDIT_DIR)?;
        fs::create_dir_all(AUDIT_ARCHIVE_DIR)?;

        // Set restrictive permissions (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            fs::set_permissions(AUDIT_DIR, perms.clone())?;
            fs::set_permissions(AUDIT_ARCHIVE_DIR, perms)?;
        }

        Ok(())
    }

    /// Log an audit entry
    pub fn log(entry: &AuditEntry) -> std::io::Result<()> {
        Self::ensure_dirs()?;

        // Check for rotation
        Self::rotate_if_needed()?;

        // Append entry
        let json = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(AUDIT_LOG_FILE)?;

        writeln!(file, "{}", json)?;
        file.sync_all()?;

        Ok(())
    }

    /// Log a read-only tool execution
    pub fn log_read_only_tool(
        tool_name: &str,
        evidence_id: &str,
        session_id: Option<&str>,
        success: bool,
    ) -> std::io::Result<()> {
        let mut entry = AuditEntry::new(
            AuditEntryType::ReadOnlyTool,
            if success {
                AuditResult::Success
            } else {
                AuditResult::Failure
            },
        )
        .with_tool(tool_name)
        .with_evidence(evidence_id);

        if let Some(sid) = session_id {
            entry = entry.with_session(sid);
        }

        Self::log(&entry)
    }

    /// Log a mutation tool execution
    pub fn log_mutation_tool(
        tool_name: &str,
        evidence_id: &str,
        session_id: Option<&str>,
        result: AuditResult,
        details: Option<serde_json::Value>,
    ) -> std::io::Result<()> {
        let mut entry = AuditEntry::new(AuditEntryType::MutationTool, result)
            .with_tool(tool_name)
            .with_evidence(evidence_id);

        if let Some(sid) = session_id {
            entry = entry.with_session(sid);
        }

        if let Some(d) = details {
            entry = entry.with_details(d);
        }

        Self::log(&entry)
    }

    /// Log a policy check
    pub fn log_policy_check(
        check_type: &str,
        target: &str,
        evidence_id: &str,
        allowed: bool,
        policy_rule: Option<&str>,
    ) -> std::io::Result<()> {
        let mut entry = AuditEntry::new(
            AuditEntryType::PolicyCheck,
            if allowed {
                AuditResult::Success
            } else {
                AuditResult::Blocked
            },
        )
        .with_evidence(evidence_id)
        .with_details(serde_json::json!({
            "check_type": check_type,
            "target": target,
        }));

        if let Some(rule) = policy_rule {
            entry = entry.with_policy_rule(rule);
        }

        Self::log(&entry)
    }

    /// Log an action blocked by policy
    pub fn log_blocked(
        action: &str,
        reason: &str,
        evidence_id: &str,
        policy_rule: &str,
    ) -> std::io::Result<()> {
        let entry = AuditEntry::new(AuditEntryType::ActionBlocked, AuditResult::Blocked)
            .with_evidence(evidence_id)
            .with_policy_rule(policy_rule)
            .with_details(serde_json::json!({
                "action": action,
                "reason": reason,
            }));

        Self::log(&entry)
    }

    /// Log a security event
    pub fn log_security_event(event: &str, details: serde_json::Value) -> std::io::Result<()> {
        let entry = AuditEntry::new(AuditEntryType::SecurityEvent, AuditResult::Success)
            .with_details(serde_json::json!({
                "event": event,
                "details": details,
            }));

        Self::log(&entry)
    }

    /// Rotate log if needed
    fn rotate_if_needed() -> std::io::Result<()> {
        let path = Path::new(AUDIT_LOG_FILE);
        if !path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() < MAX_AUDIT_LOG_SIZE {
            return Ok(());
        }

        // Rotate
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let archive_path = format!("{}/audit_{}.jsonl", AUDIT_ARCHIVE_DIR, timestamp);
        fs::rename(AUDIT_LOG_FILE, archive_path)?;

        Ok(())
    }

    /// Get recent audit entries
    pub fn get_recent(limit: usize) -> Vec<AuditEntry> {
        let path = Path::new(AUDIT_LOG_FILE);
        if !path.exists() {
            return Vec::new();
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let entries: Vec<AuditEntry> = content
            .lines()
            .rev()
            .take(limit)
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        entries
    }

    /// Get audit entries for a session
    pub fn get_for_session(session_id: &str) -> Vec<AuditEntry> {
        let path = Path::new(AUDIT_LOG_FILE);
        if !path.exists() {
            return Vec::new();
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        content
            .lines()
            .filter_map(|line| serde_json::from_str::<AuditEntry>(line).ok())
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .collect()
    }
}

/// Sanitize text for audit logging (remove potential secrets)
pub fn sanitize_for_audit(text: &str) -> String {
    let mut sanitized = text.to_string();

    // Patterns that might indicate secrets
    let secret_patterns = [
        (
            r"(?i)(password|passwd|pwd)\s*[=:]\s*\S+",
            "[REDACTED_PASSWORD]",
        ),
        (
            r"(?i)(api[_-]?key|apikey)\s*[=:]\s*\S+",
            "[REDACTED_API_KEY]",
        ),
        (r"(?i)(secret|token)\s*[=:]\s*\S+", "[REDACTED_SECRET]"),
        (r"(?i)(bearer)\s+\S+", "Bearer [REDACTED]"),
        (r"(?i)Authorization:\s*\S+", "Authorization: [REDACTED]"),
    ];

    for (pattern, replacement) in &secret_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, *replacement).to_string();
        }
    }

    // Truncate if too long
    if sanitized.len() > 1000 {
        sanitized = format!("{}... [truncated]", &sanitized[..1000]);
    }

    sanitized
}

/// Redact secrets from environment for logging
pub fn redact_env_secrets(env_vars: &[(String, String)]) -> Vec<(String, String)> {
    let secret_keys = [
        "PASSWORD",
        "PASSWD",
        "PWD",
        "SECRET",
        "TOKEN",
        "API_KEY",
        "APIKEY",
        "AUTH",
        "CREDENTIAL",
        "PRIVATE_KEY",
        "AWS_",
        "AZURE_",
        "GCP_",
    ];

    env_vars
        .iter()
        .map(|(k, v)| {
            let is_secret = secret_keys.iter().any(|s| k.to_uppercase().contains(s));
            if is_secret {
                (k.clone(), "[REDACTED]".to_string())
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(AuditEntryType::ReadOnlyTool, AuditResult::Success)
            .with_tool("disk_usage")
            .with_evidence("E12345");

        assert_eq!(entry.entry_type, AuditEntryType::ReadOnlyTool);
        assert_eq!(entry.result, AuditResult::Success);
        assert_eq!(entry.tool_name, Some("disk_usage".to_string()));
        assert_eq!(entry.evidence_id, Some("E12345".to_string()));
    }

    #[test]
    fn test_sanitize_password() {
        let text = "config password=secret123 done";
        let sanitized = sanitize_for_audit(text);
        assert!(sanitized.contains("[REDACTED_PASSWORD]"));
        assert!(!sanitized.contains("secret123"));
    }

    #[test]
    fn test_sanitize_api_key() {
        let text = "API_KEY=abc123xyz";
        let sanitized = sanitize_for_audit(text);
        assert!(sanitized.contains("[REDACTED_API_KEY]"));
        assert!(!sanitized.contains("abc123xyz"));
    }

    #[test]
    fn test_sanitize_bearer() {
        let text = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let sanitized = sanitize_for_audit(text);
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_truncation() {
        let long_text = "a".repeat(2000);
        let sanitized = sanitize_for_audit(&long_text);
        assert!(sanitized.len() < 1100);
        assert!(sanitized.contains("[truncated]"));
    }

    #[test]
    fn test_redact_env_secrets() {
        let env_vars = vec![
            ("HOME".to_string(), "/home/user".to_string()),
            ("API_KEY".to_string(), "secret123".to_string()),
            (
                "AWS_SECRET_ACCESS_KEY".to_string(),
                "aws_secret".to_string(),
            ),
        ];

        let redacted = redact_env_secrets(&env_vars);

        assert_eq!(redacted[0].1, "/home/user");
        assert_eq!(redacted[1].1, "[REDACTED]");
        assert_eq!(redacted[2].1, "[REDACTED]");
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry::new(AuditEntryType::MutationTool, AuditResult::Success)
            .with_tool("file_edit")
            .with_evidence("MUT12345")
            .with_details(serde_json::json!({"path": "/etc/test.conf"}));

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("file_edit"));
        assert!(json.contains("MUT12345"));
    }
}
