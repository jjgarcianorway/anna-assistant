//! SSH configuration auditor.
//!
//! Reads /etc/ssh/sshd_config, queries Arch Wiki for hardening best practices,
//! asks LLM to analyze the config and produce severity-ranked findings.
//!
//! No hardcoded rules — LLM extracts relevant checks from wiki for THIS config.
//! Optional: apply suggested fixes via the caller (each requires confirmation).

use anyhow::{anyhow, Result};
use tracing::info;

/// A single finding from the SSH audit.
#[derive(Debug, Clone)]
pub struct SshFinding {
    pub severity: SshSeverity,
    pub setting: String,
    pub current_value: Option<String>,
    pub recommendation: String,
    pub wiki_reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SshSeverity {
    Critical,
    Warning,
    Info,
}

impl std::fmt::Display for SshSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshSeverity::Critical => write!(f, "CRITICAL"),
            SshSeverity::Warning => write!(f, "WARNING"),
            SshSeverity::Info => write!(f, "INFO"),
        }
    }
}

/// Read the sshd_config file.
fn read_sshd_config() -> Result<String> {
    let paths = ["/etc/ssh/sshd_config", "/etc/sshd_config"];
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            info!("Read sshd_config from {}", path);
            return Ok(content);
        }
    }
    Err(anyhow!("sshd_config not found (checked /etc/ssh/sshd_config)"))
}

/// Parse LLM audit output into structured findings.
/// Expected format per finding:
/// SEVERITY: CRITICAL|WARNING|INFO
/// SETTING: <setting name>
/// CURRENT: <current value or "not set">
/// RECOMMENDATION: <what to change>
fn parse_findings(output: &str) -> Vec<SshFinding> {
    let mut findings = Vec::new();
    let mut current: Option<(SshSeverity, String, Option<String>, String)> = None;

    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SEVERITY:") {
            // Save previous finding
            if let Some((sev, setting, current_val, rec)) = current.take() {
                findings.push(SshFinding {
                    severity: sev,
                    setting,
                    current_value: current_val,
                    recommendation: rec,
                    wiki_reference: None,
                });
            }
            let sev = match rest.trim().to_uppercase().as_str() {
                "CRITICAL" => SshSeverity::Critical,
                "WARNING" => SshSeverity::Warning,
                _ => SshSeverity::Info,
            };
            current = Some((sev, String::new(), None, String::new()));
        } else if let Some(rest) = line.strip_prefix("SETTING:") {
            if let Some(ref mut c) = current {
                c.1 = rest.trim().to_string();
            }
        } else if let Some(rest) = line.strip_prefix("CURRENT:") {
            if let Some(ref mut c) = current {
                let val = rest.trim().to_string();
                c.2 = if val == "not set" || val == "default" { None } else { Some(val) };
            }
        } else if let Some(rest) = line.strip_prefix("RECOMMENDATION:") {
            if let Some(ref mut c) = current {
                c.3 = rest.trim().to_string();
            }
        }
    }

    // Save last finding
    if let Some((sev, setting, current_val, rec)) = current {
        if !setting.is_empty() {
            findings.push(SshFinding {
                severity: sev,
                setting,
                current_value: current_val,
                recommendation: rec,
                wiki_reference: None,
            });
        }
    }

    findings
}

/// Run a full SSH config audit.
pub async fn audit_ssh_config(model: &str) -> Result<String> {
    let config_content = read_sshd_config()?;

    // Research: Arch Wiki on SSH security
    let wiki_ssh = anna_shared::wiki::search::keyword_search_text("OpenSSH", 1500)
        .unwrap_or_default();

    let prompt = format!(
        "You are a security auditor analyzing an SSH server configuration on Arch Linux.\n\
        \n\
        Current sshd_config:\n\
        ---\n\
        {config_content}\n\
        ---\n\
        \n\
        Arch Wiki on OpenSSH security:\n\
        {wiki_ssh}\n\
        \n\
        Analyze the sshd_config against current best practices from the Arch Wiki.\n\
        For each issue found, output in EXACTLY this format (repeat for each finding):\n\
        \n\
        SEVERITY: CRITICAL|WARNING|INFO\n\
        SETTING: <setting name>\n\
        CURRENT: <current value or 'not set'>\n\
        RECOMMENDATION: <specific change to make>\n\
        \n\
        Focus on security issues. Check especially:\n\
        - PasswordAuthentication (should be no)\n\
        - PermitRootLogin (should be no or prohibit-password)\n\
        - Protocol version\n\
        - PermitEmptyPasswords (should be no)\n\
        - MaxAuthTries\n\
        - ClientAliveInterval\n\
        - AllowUsers/DenyUsers\n\
        - Ciphers and MACs (weak algorithms)\n\
        - Port (non-standard reduces noise)\n\
        Output ONLY findings in the format above, no other text."
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 45).await
        .map_err(|e| anyhow!("LLM error during SSH audit: {}", e))?;

    let findings = parse_findings(&response);

    if findings.is_empty() {
        return Ok("SSH configuration audit complete. No significant issues found — your sshd_config follows current best practices.".to_string());
    }

    // Sort by severity: Critical first
    let mut sorted = findings;
    sorted.sort_by_key(|f| match f.severity {
        SshSeverity::Critical => 0,
        SshSeverity::Warning => 1,
        SshSeverity::Info => 2,
    });

    let critical_count = sorted.iter().filter(|f| f.severity == SshSeverity::Critical).count();
    let warning_count = sorted.iter().filter(|f| f.severity == SshSeverity::Warning).count();

    let mut out = format!(
        "SSH Configuration Audit — {} finding{}:\n  {} critical, {} warning\n\n",
        sorted.len(),
        if sorted.len() == 1 { "" } else { "s" },
        critical_count,
        warning_count,
    );

    for finding in &sorted {
        out.push_str(&format!("[{}] {}\n", finding.severity, finding.setting));
        if let Some(ref val) = finding.current_value {
            out.push_str(&format!("  Current: {}\n", val));
        }
        out.push_str(&format!("  Fix: {}\n\n", finding.recommendation));
    }

    if critical_count > 0 || warning_count > 0 {
        out.push_str("To apply any of these fixes, ask me: \"fix my SSH config\" and I'll apply the changes with confirmation for each step.");
    }

    // Record audit in registry
    let mut registry = crate::artifact_registry::ArtifactRegistry::load();
    let artifact = crate::artifact_registry::CreatedArtifact::new(
        crate::artifact_registry::ArtifactKind::SshAuditReport,
        "SSH config audit",
        &format!("{} findings ({} critical)", sorted.len(), critical_count),
        vec!["/etc/ssh/sshd_config".into()],
        vec![], // no removal needed for audit reports
    );
    registry.add(artifact);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_findings_empty() {
        let findings = parse_findings("No issues found.");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_findings_basic() {
        let output = "SEVERITY: CRITICAL\nSETTING: PasswordAuthentication\nCURRENT: yes\nRECOMMENDATION: Set to no\nSEVERITY: WARNING\nSETTING: MaxAuthTries\nCURRENT: not set\nRECOMMENDATION: Set to 3";
        let findings = parse_findings(output);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].severity, SshSeverity::Critical);
        assert_eq!(findings[0].setting, "PasswordAuthentication");
        assert_eq!(findings[1].severity, SshSeverity::Warning);
    }

    #[test]
    fn test_read_sshd_config_missing() {
        // Should return error on systems without sshd_config in test
        // (the function is tested indirectly)
        let _ = read_sshd_config(); // just ensure no panic
    }
}
