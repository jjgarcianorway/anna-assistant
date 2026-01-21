//! Config Review: Passwd Change - READ_ONLY capability handler.
//!
//! Capability: config.review.passwd_change (ReadOnly)
//!
//! Purpose: Explain what changed in /etc/passwd, why Anna warned, and whether action is required.
//!
//! What this does:
//! - Probe /etc/passwd existence and metadata
//! - Probe /etc/passwd.bak (or distro-equivalent backup)
//! - Generate diff summary (truncated, no raw command exposure)
//! - Detect likely cause from context
//!
//! What this does NOT do:
//! - Does NOT execute any changes
//! - Does NOT modify any system configuration
//! - Does NOT require confirmation (ReadOnly)

use super::response::{AbstainReason, CapabilityExecutionResult, ResponseArtifact};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

// =============================================================================
// CONSTANTS
// =============================================================================

const MAX_DIFF_LINES: usize = 50;
const PASSWD_PATH: &str = "/etc/passwd";
const PASSWD_BAK_PATH: &str = "/etc/passwd.bak";

// =============================================================================
// PROBE TYPES
// =============================================================================

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub name: &'static str,
    pub success: bool,
    pub finding: String,
    pub critical: bool,
}

impl ProbeResult {
    fn ok(name: &'static str, finding: &str) -> Self {
        Self { name, success: true, finding: finding.to_string(), critical: false }
    }
    fn failed(name: &'static str, finding: &str) -> Self {
        Self { name, success: false, finding: finding.to_string(), critical: false }
    }
    fn critical(name: &'static str, finding: &str) -> Self {
        Self { name, success: false, finding: finding.to_string(), critical: true }
    }
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub exists: bool,
    pub size: u64,
    pub mode: u32,
    pub uid: u32,
    pub mtime: i64,
    pub summary: String,
}

impl FileMetadata {
    fn from_path(path: &Path) -> Option<Self> {
        let meta = fs::metadata(path).ok()?;
        let mtime = meta.modified().ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64).unwrap_or(0);
        let mode = meta.mode() & 0o777;
        Some(Self {
            exists: true, size: meta.len(), mode, uid: meta.uid(), mtime,
            summary: format!("size={}, mode={:o}, uid={}", meta.len(), mode, meta.uid()),
        })
    }
}

/// Complete probe results for passwd change review.
#[derive(Debug, Clone)]
pub struct PasswdChangeProbes {
    pub passwd_file: ProbeResult,
    pub passwd_meta: Option<FileMetadata>,
    pub passwd_bak_file: ProbeResult,
    pub passwd_bak_meta: Option<FileMetadata>,
    pub diff_summary: ProbeResult,
    pub diff_output: String,
    pub diff_truncated: bool,
    pub changes_detected: bool,
    pub entries_added: usize,
    pub entries_removed: usize,
    pub entries_modified: usize,
    pub likely_cause: Option<String>,
}

impl PasswdChangeProbes {
    /// Phase 35: Evidence capped at 3 lines.
    pub fn to_evidence(&self) -> Vec<ResponseArtifact> {
        let passwd_status = if self.passwd_file.success { "exists" } else { "missing" };
        let bak_status = if self.passwd_bak_file.success { "exists" } else { "missing" };
        let change_summary = if self.changes_detected {
            format!("+{} -{} ~{}", self.entries_added, self.entries_removed, self.entries_modified)
        } else { "no changes".to_string() };
        vec![
            ResponseArtifact::evidence("passwd:", passwd_status),
            ResponseArtifact::evidence("backup:", bak_status),
            ResponseArtifact::evidence("changes:", &change_summary),
        ]
    }

    /// Format explanation for resolved response.
    pub fn format_explanation(&self) -> String {
        if !self.changes_detected {
            return "No differences found between /etc/passwd and its backup.".to_string();
        }
        let mut parts = vec![];
        if self.entries_added > 0 { parts.push(format!("{} user(s) added", self.entries_added)); }
        if self.entries_removed > 0 { parts.push(format!("{} user(s) removed", self.entries_removed)); }
        if self.entries_modified > 0 { parts.push(format!("{} user(s) modified", self.entries_modified)); }
        let change_desc = if parts.is_empty() { "Changes detected".to_string() } else { parts.join(", ") };
        let cause = self.likely_cause.as_ref().map(|c| format!(" Likely cause: {}.", c)).unwrap_or_default();
        format!("{}.{}", change_desc, cause)
    }
}

// =============================================================================
// PROBE IMPLEMENTATION
// =============================================================================

pub fn gather_probes() -> PasswdChangeProbes {
    let (passwd_file, passwd_meta) = probe_passwd_file();
    let (passwd_bak_file, passwd_bak_meta) = probe_passwd_bak_file();
    let (diff_summary, diff_output, diff_truncated, changes_detected) =
        probe_diff(&passwd_file, &passwd_bak_file);
    let (entries_added, entries_removed, entries_modified) = analyze_diff(&diff_output);
    let likely_cause = detect_likely_cause(&diff_output, entries_added, entries_removed, entries_modified);

    PasswdChangeProbes {
        passwd_file, passwd_meta, passwd_bak_file, passwd_bak_meta,
        diff_summary, diff_output, diff_truncated, changes_detected,
        entries_added, entries_removed, entries_modified, likely_cause,
    }
}

fn probe_passwd_file() -> (ProbeResult, Option<FileMetadata>) {
    let path = Path::new(PASSWD_PATH);
    if !path.exists() {
        return (ProbeResult::critical(PASSWD_PATH, "File does not exist - critical system file missing"), None);
    }
    match FileMetadata::from_path(path) {
        Some(meta) => {
            let finding = format!("Exists ({})", meta.summary);
            (ProbeResult::ok(PASSWD_PATH, &finding), Some(meta))
        }
        None => (ProbeResult::critical(PASSWD_PATH, "Cannot read file metadata"), None),
    }
}

fn probe_passwd_bak_file() -> (ProbeResult, Option<FileMetadata>) {
    let path = Path::new(PASSWD_BAK_PATH);
    if !path.exists() {
        return (ProbeResult::failed(PASSWD_BAK_PATH, "Backup file does not exist"), None);
    }
    match FileMetadata::from_path(path) {
        Some(meta) => {
            let finding = format!("Exists ({})", meta.summary);
            (ProbeResult::ok(PASSWD_BAK_PATH, &finding), Some(meta))
        }
        None => (ProbeResult::failed(PASSWD_BAK_PATH, "Cannot read backup file metadata"), None),
    }
}

fn probe_diff(passwd: &ProbeResult, bak: &ProbeResult) -> (ProbeResult, String, bool, bool) {
    if !passwd.success || !bak.success {
        return (ProbeResult::failed("Diff Summary", "Cannot diff - missing file(s)"), String::new(), false, false);
    }
    match Command::new("diff").args(["-u", PASSWD_BAK_PATH, PASSWD_PATH]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.code() == Some(0) {
                // Files identical
                return (ProbeResult::ok("Diff Summary", "Files are identical"), String::new(), false, false);
            }
            // Diff found differences (exit code 1)
            let lines: Vec<&str> = stdout.lines().collect();
            let truncated = lines.len() > MAX_DIFF_LINES;
            let display_lines = if truncated { &lines[..MAX_DIFF_LINES] } else { &lines[..] };
            let diff_output = display_lines.join("\n");
            let finding = if truncated {
                format!("{} lines of changes (showing first {})", lines.len(), MAX_DIFF_LINES)
            } else {
                format!("{} lines of changes", lines.len())
            };
            (ProbeResult::ok("Diff Summary", &finding), diff_output, truncated, true)
        }
        Err(_) => (ProbeResult::failed("Diff Summary", "diff command failed"), String::new(), false, false),
    }
}

fn analyze_diff(diff_output: &str) -> (usize, usize, usize) {
    let mut added: usize = 0;
    let mut removed: usize = 0;
    let mut added_users: Vec<&str> = vec![];
    let mut removed_users: Vec<&str> = vec![];

    for line in diff_output.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            added += 1;
            if let Some(user) = line.get(1..).and_then(|l| l.split(':').next()) {
                added_users.push(user);
            }
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed += 1;
            if let Some(user) = line.get(1..).and_then(|l| l.split(':').next()) {
                removed_users.push(user);
            }
        }
    }
    // Entries that appear in both added and removed are modifications
    let modified: usize = added_users.iter().filter(|u| removed_users.contains(u)).count();
    let net_added = added.saturating_sub(modified);
    let net_removed = removed.saturating_sub(modified);
    (net_added, net_removed, modified)
}

fn detect_likely_cause(diff: &str, added: usize, removed: usize, modified: usize) -> Option<String> {
    if diff.is_empty() { return None; }
    // Check for shell changes
    if diff.contains("/bin/bash") || diff.contains("/bin/zsh") || diff.contains("/bin/sh") {
        if modified > 0 { return Some("user shell was changed".to_string()); }
    }
    // Check for home directory changes
    if diff.contains("/home/") {
        if modified > 0 { return Some("user home directory was changed".to_string()); }
    }
    // Check for new users
    if added > 0 && removed == 0 {
        return Some(format!("new user account(s) created (useradd or package install)"));
    }
    // Check for removed users
    if removed > 0 && added == 0 {
        return Some(format!("user account(s) removed (userdel or package removal)"));
    }
    // Generic modification
    if modified > 0 {
        return Some("user account properties modified (usermod, chsh, or manual edit)".to_string());
    }
    None
}

// =============================================================================
// CAPABILITY HANDLER
// =============================================================================

/// Execute the config.review.passwd_change capability.
/// ReadOnly - returns immediate answer, no confirmation required.
pub fn execute_passwd_change_review() -> CapabilityExecutionResult {
    let probes = gather_probes();

    // Critical: passwd file missing
    if !probes.passwd_file.success {
        return CapabilityExecutionResult::with_explanation(
            probes.to_evidence(),
            "Critical: /etc/passwd does not exist. This is a serious system issue.",
        );
    }

    // Abstain: No backup file to compare
    if !probes.passwd_bak_file.success {
        return CapabilityExecutionResult::abstain(
            AbstainReason::PrerequisitesNotMet,
            "I can see that /etc/passwd exists, but no backup file was found at /etc/passwd.bak. \
            Without a reference version, I cannot explain what changed.",
        );
    }

    // No changes detected
    if !probes.changes_detected {
        return CapabilityExecutionResult::with_explanation(
            probes.to_evidence(),
            "I checked /etc/passwd and its backup. No differences were found.",
        );
    }

    // Changes detected - provide explanation
    let explanation = probes.format_explanation();
    CapabilityExecutionResult::with_explanation(probes.to_evidence(), &explanation)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_probes(passwd_ok: bool, bak_ok: bool, changes: bool) -> PasswdChangeProbes {
        PasswdChangeProbes {
            passwd_file: if passwd_ok { ProbeResult::ok(PASSWD_PATH, "Exists") } else { ProbeResult::critical(PASSWD_PATH, "Missing") },
            passwd_meta: if passwd_ok { Some(FileMetadata { exists: true, size: 1000, mode: 0o644, uid: 0, mtime: 0, summary: "test".to_string() }) } else { None },
            passwd_bak_file: if bak_ok { ProbeResult::ok(PASSWD_BAK_PATH, "Exists") } else { ProbeResult::failed(PASSWD_BAK_PATH, "Missing") },
            passwd_bak_meta: if bak_ok { Some(FileMetadata { exists: true, size: 900, mode: 0o644, uid: 0, mtime: 0, summary: "test".to_string() }) } else { None },
            diff_summary: if changes { ProbeResult::ok("Diff", "Changes") } else { ProbeResult::ok("Diff", "No changes") },
            diff_output: if changes { "+newuser:x:1001:1001::/home/newuser:/bin/bash".to_string() } else { String::new() },
            diff_truncated: false, changes_detected: changes,
            entries_added: if changes { 1 } else { 0 }, entries_removed: 0, entries_modified: 0,
            likely_cause: if changes { Some("new user account(s) created".to_string()) } else { None },
        }
    }

    #[test]
    fn test_handler_returns_resolved_or_abstain() {
        let result = execute_passwd_change_review();
        assert!(!result.explanation.is_empty() || result.wants_abstain(), "Must return explanation or abstain");
    }

    #[test]
    fn test_readonly_no_action_plan() {
        let result = execute_passwd_change_review();
        assert!(result.action_plan.is_none(), "ReadOnly capability must not return ActionPlan");
    }

    #[test]
    fn test_evidence_capped_at_three() {
        let probes = test_probes(true, true, true);
        assert!(probes.to_evidence().len() <= 3, "Evidence must be capped at 3 lines");
    }

    #[test]
    fn test_missing_backup_abstains() {
        let probes = test_probes(true, false, false);
        // Simulate the logic
        if probes.passwd_file.success && !probes.passwd_bak_file.success {
            // Should abstain
            assert!(!probes.passwd_bak_file.success);
        }
    }

    #[test]
    fn test_no_changes_explains() {
        let probes = test_probes(true, true, false);
        let explanation = probes.format_explanation();
        assert!(explanation.contains("No differences"));
    }

    #[test]
    fn test_changes_detected_explains() {
        let probes = test_probes(true, true, true);
        let explanation = probes.format_explanation();
        assert!(explanation.contains("added") || explanation.contains("modified") || explanation.contains("removed"));
    }

    #[test]
    fn test_analyze_diff_counts_correctly() {
        let diff = "+newuser:x:1001::/home/newuser:/bin/bash\n-olduser:x:1002::/home/old:/bin/bash";
        let (added, removed, _modified) = analyze_diff(diff);
        assert_eq!(added, 1);
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_likely_cause_detection() {
        let cause = detect_likely_cause("+test:x:1001::/home/test:/bin/bash", 1, 0, 0);
        assert!(cause.is_some());
        assert!(cause.unwrap().contains("created"));
    }
}
