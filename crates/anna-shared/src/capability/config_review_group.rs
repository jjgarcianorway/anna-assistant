//! Config Review: Group Change - Complete capability handler.
//!
//! Scope: Review changes to /etc/group and provide restore instructions.
//! Method: Diff analysis and backup detection.
//!
//! Non-capabilities (explicit):
//! - Does NOT execute any changes
//! - Does NOT automatically restore files
//! - Does NOT modify any system configuration
//!
//! Probes (all read-only):
//! - /etc/group existence and metadata
//! - /etc/group.bak existence and metadata
//! - Diff summary (capped output)
//! - pacman .pacnew/.pacsave detection

use super::registry::CapabilityId;
use super::response::{AbstainReason, FailedReason, ResponseArtifact, ResponseOutcome};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Maximum diff output lines before truncation.
const MAX_DIFF_LINES: usize = 50;

/// Main group file path.
const GROUP_PATH: &str = "/etc/group";

/// Backup group file path.
const GROUP_BAK_PATH: &str = "/etc/group.bak";

// =============================================================================
// PROBE TYPES
// =============================================================================

/// Result of a single probe.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// Probe name for display.
    pub name: &'static str,
    /// Whether the probe succeeded.
    pub success: bool,
    /// Human-readable finding.
    pub finding: String,
    /// Whether this is a critical prerequisite.
    pub critical: bool,
    /// Optional error detail for Failed responses.
    pub error_detail: Option<String>,
}

impl ProbeResult {
    fn ok(name: &'static str, finding: &str) -> Self {
        Self {
            name,
            success: true,
            finding: finding.to_string(),
            critical: false,
            error_detail: None,
        }
    }

    fn failed(name: &'static str, finding: &str) -> Self {
        Self {
            name,
            success: false,
            finding: finding.to_string(),
            critical: false,
            error_detail: None,
        }
    }

    fn critical(name: &'static str, finding: &str) -> Self {
        Self {
            name,
            success: false,
            finding: finding.to_string(),
            critical: true,
            error_detail: None,
        }
    }

    fn error(name: &'static str, finding: &str, detail: &str) -> Self {
        Self {
            name,
            success: false,
            finding: finding.to_string(),
            critical: true,
            error_detail: Some(detail.to_string()),
        }
    }
}

/// File metadata gathered from stat.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File exists.
    pub exists: bool,
    /// File size in bytes.
    pub size: u64,
    /// File mode (permissions).
    pub mode: u32,
    /// Owner UID.
    pub uid: u32,
    /// Owner GID.
    pub gid: u32,
    /// Modification time (Unix timestamp).
    pub mtime: i64,
    /// Human-readable summary.
    pub summary: String,
}

impl FileMetadata {
    #[allow(dead_code)]
    fn from_path(path: &Path) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mode = metadata.mode();
        let mode_str = format!(
            "{}{}{}",
            if mode & 0o400 != 0 { "r" } else { "-" },
            if mode & 0o200 != 0 { "w" } else { "-" },
            if mode & 0o100 != 0 { "x" } else { "-" }
        );

        let summary = format!(
            "size={}, mode={:o} ({}), uid={}, gid={}, mtime={}",
            metadata.len(),
            mode & 0o777,
            mode_str,
            metadata.uid(),
            metadata.gid(),
            mtime
        );

        Some(Self {
            exists: true,
            size: metadata.len(),
            mode: mode & 0o777,
            uid: metadata.uid(),
            gid: metadata.gid(),
            mtime,
            summary,
        })
    }

    #[allow(dead_code)]
    fn not_found() -> Self {
        Self {
            exists: false,
            size: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            mtime: 0,
            summary: "File does not exist".to_string(),
        }
    }
}

/// All probes gathered for group change review.
#[derive(Debug, Clone)]
pub struct GroupChangeProbes {
    /// /etc/group existence and metadata.
    pub group_file: ProbeResult,
    /// /etc/group metadata (if exists).
    pub group_metadata: Option<FileMetadata>,
    /// /etc/group.bak existence and metadata.
    pub group_bak_file: ProbeResult,
    /// /etc/group.bak metadata (if exists).
    pub group_bak_metadata: Option<FileMetadata>,
    /// Diff summary between the two files.
    pub diff_summary: ProbeResult,
    /// Full diff output (for artifacts).
    pub diff_output: String,
    /// Whether diff was truncated.
    pub diff_truncated: bool,
    /// pacman backup detection (.pacnew/.pacsave).
    pub pacman_backups: ProbeResult,
    /// List of detected pacman backup files.
    pub pacman_backup_files: Vec<String>,
}

impl GroupChangeProbes {
    /// Check if both files exist (required for resolved).
    pub fn both_files_exist(&self) -> bool {
        self.group_file.success && self.group_bak_file.success
    }

    /// Check if only backup is missing (common case for abstain).
    pub fn backup_missing(&self) -> bool {
        self.group_file.success && !self.group_bak_file.success
    }

    /// Check if there was a probe error (permission, command missing).
    pub fn has_probe_error(&self) -> Option<(&'static str, &str)> {
        if let Some(detail) = &self.group_file.error_detail {
            return Some((self.group_file.name, detail));
        }
        if let Some(detail) = &self.diff_summary.error_detail {
            return Some((self.diff_summary.name, detail));
        }
        None
    }

    /// Convert to evidence artifacts.
    pub fn to_evidence(&self) -> Vec<ResponseArtifact> {
        let mut evidence = vec![
            ResponseArtifact::evidence(self.group_file.name, &self.group_file.finding),
            ResponseArtifact::evidence(self.group_bak_file.name, &self.group_bak_file.finding),
        ];

        if let Some(meta) = &self.group_metadata {
            evidence.push(ResponseArtifact::evidence(
                "/etc/group metadata",
                &meta.summary,
            ));
        }

        if let Some(meta) = &self.group_bak_metadata {
            evidence.push(ResponseArtifact::evidence(
                "/etc/group.bak metadata",
                &meta.summary,
            ));
        }

        evidence.push(ResponseArtifact::evidence(
            self.diff_summary.name,
            &self.diff_summary.finding,
        ));

        evidence.push(ResponseArtifact::evidence(
            self.pacman_backups.name,
            &self.pacman_backups.finding,
        ));

        evidence
    }
}

// =============================================================================
// PROBE IMPLEMENTATION
// =============================================================================

/// Run all probes for group change review capability.
pub fn gather_probes() -> GroupChangeProbes {
    let (group_file, group_metadata) = probe_group_file();
    let (group_bak_file, group_bak_metadata) = probe_group_bak_file();
    let (diff_summary, diff_output, diff_truncated) =
        probe_diff(&group_file, &group_bak_file);
    let (pacman_backups, pacman_backup_files) = probe_pacman_backups();

    GroupChangeProbes {
        group_file,
        group_metadata,
        group_bak_file,
        group_bak_metadata,
        diff_summary,
        diff_output,
        diff_truncated,
        pacman_backups,
        pacman_backup_files,
    }
}

/// Probe: Check /etc/group existence and metadata.
fn probe_group_file() -> (ProbeResult, Option<FileMetadata>) {
    let path = Path::new(GROUP_PATH);

    if !path.exists() {
        return (
            ProbeResult::critical("/etc/group", "File does not exist - critical system file missing"),
            None,
        );
    }

    match FileMetadata::from_path(path) {
        Some(meta) => {
            let finding = format!("Exists: {}", meta.summary);
            (ProbeResult::ok("/etc/group", &finding), Some(meta))
        }
        None => (
            ProbeResult::error(
                "/etc/group",
                "Cannot read file metadata",
                "Permission denied or I/O error reading /etc/group metadata",
            ),
            None,
        ),
    }
}

/// Probe: Check /etc/group.bak existence and metadata.
fn probe_group_bak_file() -> (ProbeResult, Option<FileMetadata>) {
    let path = Path::new(GROUP_BAK_PATH);

    if !path.exists() {
        return (
            ProbeResult::failed(
                "/etc/group.bak",
                "Backup file does not exist",
            ),
            None,
        );
    }

    match FileMetadata::from_path(path) {
        Some(meta) => {
            let finding = format!("Exists: {}", meta.summary);
            (ProbeResult::ok("/etc/group.bak", &finding), Some(meta))
        }
        None => (
            ProbeResult::failed("/etc/group.bak", "Cannot read backup file metadata"),
            None,
        ),
    }
}

/// Probe: Generate diff between the two files.
fn probe_diff(group: &ProbeResult, bak: &ProbeResult) -> (ProbeResult, String, bool) {
    // Can only diff if both files exist
    if !group.success || !bak.success {
        return (
            ProbeResult::failed(
                "Diff Summary",
                "Cannot diff: one or both files missing",
            ),
            String::new(),
            false,
        );
    }

    // Run diff -u
    match Command::new("diff")
        .args(["-u", GROUP_BAK_PATH, GROUP_PATH])
        .output()
    {
        Ok(output) => {
            let diff_text = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            // diff returns 0 if same, 1 if different, 2 if error
            if output.status.code() == Some(2) {
                return (
                    ProbeResult::error("Diff Summary", "diff command failed", &stderr),
                    String::new(),
                    false,
                );
            }

            if diff_text.is_empty() {
                return (
                    ProbeResult::ok("Diff Summary", "Files are identical - no changes detected"),
                    String::new(),
                    false,
                );
            }

            // Count lines and truncate if needed
            let lines: Vec<&str> = diff_text.lines().collect();
            let truncated = lines.len() > MAX_DIFF_LINES;
            let display_lines = if truncated {
                &lines[..MAX_DIFF_LINES]
            } else {
                &lines[..]
            };
            let diff_output = display_lines.join("\n");

            let finding = if truncated {
                format!(
                    "{} lines changed (showing first {}, truncated)",
                    lines.len(),
                    MAX_DIFF_LINES
                )
            } else {
                format!("{} lines changed", lines.len())
            };

            (ProbeResult::ok("Diff Summary", &finding), diff_output, truncated)
        }
        Err(e) => (
            ProbeResult::error(
                "Diff Summary",
                "diff command not available",
                &format!("Failed to execute diff: {}", e),
            ),
            String::new(),
            false,
        ),
    }
}

/// Probe: Detect pacman backup files (.pacnew, .pacsave).
fn probe_pacman_backups() -> (ProbeResult, Vec<String>) {
    let mut found = Vec::new();

    let pacman_paths = [
        "/etc/group.pacnew",
        "/etc/group.pacsave",
        "/etc/group-",  // Some systems use this pattern
    ];

    for path_str in &pacman_paths {
        let path = Path::new(path_str);
        if path.exists() {
            found.push(path_str.to_string());
        }
    }

    // Also check /var/cache/pacman for any group-related backups
    // (This is best-effort, may not be accessible)
    if let Ok(entries) = fs::read_dir("/var/cache/pacman/pkg") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("filesystem") && (name.contains(".pkg.tar") || name.contains("backup")) {
                // filesystem package contains /etc/group
                // Note: Not adding to list as this is just metadata
            }
        }
    }

    if found.is_empty() {
        (
            ProbeResult::ok(
                "pacman Backups",
                "No .pacnew/.pacsave files found for /etc/group",
            ),
            found,
        )
    } else {
        let finding = format!("Found: {}", found.join(", "));
        (ProbeResult::ok("pacman Backups", &finding), found)
    }
}

// =============================================================================
// CAPABILITY HANDLER
// =============================================================================

/// Execute the config.review.group_change capability.
///
/// Returns a complete, deterministic response:
/// - Resolved: with operator steps, evidence, rollback, and notes
/// - Abstained: with reason and missing prerequisite list
/// - Failed: with probe error details
/// - Never emits generic fallback
///
/// Note: This handler returns ResponseOutcome directly because it can
/// produce all three outcome types, including Failed for probe errors.
pub fn execute_config_review_group_change() -> ResponseOutcome {
    // Run all probes
    let probes = gather_probes();

    // Check for probe errors first
    if let Some((probe_name, detail)) = probes.has_probe_error() {
        return build_failed_response(probe_name, detail);
    }

    // Check if backup is missing
    if probes.backup_missing() {
        return build_abstain_outcome(&probes);
    }

    // Check if group file itself is missing (critical)
    if !probes.group_file.success {
        return build_critical_abstain_outcome(&probes);
    }

    // Both files exist - provide resolved response
    build_resolved_outcome(&probes)
}

/// Build a failed response for probe errors.
fn build_failed_response(probe_name: &str, detail: &str) -> ResponseOutcome {
    ResponseOutcome::Failed {
        error: FailedReason::ProbeError {
            probe_name: probe_name.to_string(),
        },
        diagnostic: format!(
            "{}: {}\n\n\
             Hints:\n\
             - Check file permissions: ls -la /etc/group*\n\
             - Verify diff command: which diff\n\
             - Try running as root if permission denied",
            probe_name, detail
        ),
    }
}

/// Build an abstain outcome when backup is missing.
fn build_abstain_outcome(probes: &GroupChangeProbes) -> ResponseOutcome {
    // Hints for where backups might be
    let mut hints = vec![
        "Check for pacman backups: ls -la /etc/group.pac*".to_string(),
        "Check filesystem package: pacman -Qql filesystem | grep group".to_string(),
        "Check for btrfs snapshots: btrfs subvolume list /".to_string(),
        "Check /var/backup or /var/backups if exists".to_string(),
    ];

    // Add pacman backup files if any were found
    if !probes.pacman_backup_files.is_empty() {
        hints.push(format!(
            "Found pacman backups: {}",
            probes.pacman_backup_files.join(", ")
        ));
    }

    // Build explanation with evidence summary
    let evidence_summary: Vec<String> = probes
        .to_evidence()
        .iter()
        .map(|e| format!("- {}: {}", e.label, e.content))
        .collect();

    let explanation = format!(
        "Cannot review group changes: backup file /etc/group.bak does not exist.\n\n\
         Evidence gathered:\n{}\n\n\
         To create a baseline backup for future comparisons:\n\
         sudo cp /etc/group /etc/group.bak",
        evidence_summary.join("\n")
    );

    ResponseOutcome::Abstained {
        capability_id: Some(CapabilityId::new("config.review.group_change")),
        reason: AbstainReason::PrerequisitesNotMet,
        explanation,
        hints,
    }
}

/// Build abstain outcome when group file itself is missing (critical).
fn build_critical_abstain_outcome(probes: &GroupChangeProbes) -> ResponseOutcome {
    let evidence_summary: Vec<String> = probes
        .to_evidence()
        .iter()
        .map(|e| format!("- {}: {}", e.label, e.content))
        .collect();

    let explanation = format!(
        "CRITICAL: /etc/group does not exist. This is a required system file.\n\n\
         Evidence gathered:\n{}\n\n\
         Immediate recovery options:\n\
         1. Boot from live USB and copy /etc/group from working system\n\
         2. Check for pacman backup: ls /etc/group.pac*\n\
         3. Reinstall filesystem package: pacman -S filesystem",
        evidence_summary.join("\n")
    );

    ResponseOutcome::Abstained {
        capability_id: Some(CapabilityId::new("config.review.group_change")),
        reason: AbstainReason::PrerequisitesNotMet,
        explanation,
        hints: vec![
            "/etc/group.pacnew".to_string(),
            "/etc/group.pacsave".to_string(),
            "pacman -S filesystem".to_string(),
        ],
    }
}

/// Build the resolved outcome with full operator plan.
fn build_resolved_outcome(probes: &GroupChangeProbes) -> ResponseOutcome {
    let mut artifacts = probes.to_evidence();

    // Build explanation
    let explanation = if probes.diff_output.is_empty() {
        "The /etc/group file and its backup are identical. No changes detected.".to_string()
    } else {
        let truncate_note = if probes.diff_truncated {
            " (output truncated)"
        } else {
            ""
        };
        format!(
            "Changes detected between /etc/group and /etc/group.bak.{}",
            truncate_note
        )
    };

    // Add diff as evidence artifact if there are changes
    if !probes.diff_output.is_empty() {
        artifacts.push(ResponseArtifact::evidence(
            "Diff Output",
            &probes.diff_output,
        ));
    }

    // Operator steps for inspection
    let mut step_num = 1;

    artifacts.push(ResponseArtifact::step(
        step_num,
        "View the full diff:\n  diff -u /etc/group.bak /etc/group",
    ));
    step_num += 1;

    artifacts.push(ResponseArtifact::step(
        step_num,
        "View side-by-side comparison:\n  diff -y /etc/group.bak /etc/group | less",
    ));
    step_num += 1;

    // Only add restore steps if there are actual changes
    if !probes.diff_output.is_empty() {
        artifacts.push(ResponseArtifact::step(
            step_num,
            "To restore from backup (safe method):\n\
             \n  # 1. Copy backup to temp location first\n\
             \n  sudo cp /etc/group.bak /tmp/group.restore\n\
             \n  # 2. Verify the backup looks correct\n\
             \n  cat /tmp/group.restore\n\
             \n  # 3. Install the restored file (preserves SELinux context)\n\
             \n  sudo install -m 644 -o root -g root /tmp/group.restore /etc/group",
        ));
        step_num += 1;

        artifacts.push(ResponseArtifact::step(
            step_num,
            "Validate after restore:\n\
             \n  # Verify file is readable\n\
             \n  getent group\n\
             \n  # Check your user's groups\n\
             \n  id\n\
             \n  # Test a login (optional)\n\
             \n  su - $(whoami)",
        ));

        // Rollback steps
        artifacts.push(ResponseArtifact::rollback(
            1,
            "If restore was wrong, swap back:\n\
             \n  # Backup the restored version first\n\
             \n  sudo cp /etc/group /etc/group.restored\n\
             \n  # Restore the previous state\n\
             \n  sudo cp /tmp/group.restore.before /etc/group\n\
             \n  (Note: Create /tmp/group.restore.before BEFORE restoring)",
        ));

        artifacts.push(ResponseArtifact::rollback(
            2,
            "Full rollback procedure:\n\
             \n  sudo cp /etc/group /etc/group.$(date +%Y%m%d_%H%M%S)\n\
             \n  sudo cp /etc/group.bak /etc/group\n\
             \n  getent group  # Verify",
        ));
    }

    // Notes
    artifacts.push(ResponseArtifact::note(
        "File Purpose",
        "/etc/group defines system groups and group memberships.\n\
         Changes affect user permissions, service access, and security boundaries.",
    ));

    if probes.pacman_backup_files.is_empty() {
        artifacts.push(ResponseArtifact::note(
            "pacman Backups",
            "No pacman .pacnew/.pacsave files found. If you updated the filesystem package,\n\
             pacman may have preserved changes in these files.",
        ));
    } else {
        artifacts.push(ResponseArtifact::note(
            "pacman Backups Available",
            &format!(
                "Found: {}\nThese may contain package-provided defaults.",
                probes.pacman_backup_files.join(", ")
            ),
        ));
    }

    // Add metadata comparison if both exist
    if let (Some(group_meta), Some(bak_meta)) =
        (&probes.group_metadata, &probes.group_bak_metadata)
    {
        let time_diff = group_meta.mtime - bak_meta.mtime;
        let time_info = if time_diff > 0 {
            format!(
                "/etc/group is {} seconds newer than backup",
                time_diff
            )
        } else if time_diff < 0 {
            format!(
                "Backup is {} seconds newer than /etc/group (unusual)",
                -time_diff
            )
        } else {
            "Both files have the same modification time".to_string()
        };

        artifacts.push(ResponseArtifact::note("Timestamp Comparison", &time_info));
    }

    ResponseOutcome::Resolved {
        capability_id: CapabilityId::new("config.review.group_change"),
        explanation,
        artifacts,
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Test: Handler always produces a response (never empty)
    // -------------------------------------------------------------------------

    #[test]
    fn test_handler_always_produces_response() {
        let result = execute_config_review_group_change();

        // Must be one of the three outcome types
        match result {
            ResponseOutcome::Resolved { explanation, artifacts, .. } => {
                assert!(!explanation.is_empty() || !artifacts.is_empty());
            }
            ResponseOutcome::Abstained { explanation, .. } => {
                assert!(!explanation.is_empty());
            }
            ResponseOutcome::Failed { diagnostic, .. } => {
                assert!(!diagnostic.is_empty());
            }
            ResponseOutcome::ConfirmationRequired { .. } => {
                // Phase 31: ConfirmationRequired is a valid outcome for mutating capabilities
            }
        }
    }

    #[test]
    fn test_handler_never_returns_generic_fallback() {
        let result = execute_config_review_group_change();

        // Must never contain generic fallback text
        match &result {
            ResponseOutcome::Resolved { explanation, .. } => {
                assert!(
                    !explanation.contains("could not format"),
                    "Resolved must not contain generic fallback"
                );
            }
            ResponseOutcome::Abstained { explanation, .. } => {
                assert!(
                    !explanation.contains("could not format"),
                    "Abstained must not contain generic fallback"
                );
            }
            ResponseOutcome::Failed { diagnostic, .. } => {
                assert!(
                    !diagnostic.contains("could not format"),
                    "Failed must not contain generic fallback"
                );
            }
            ResponseOutcome::ConfirmationRequired { .. } => {
                // Phase 31: ConfirmationRequired doesn't use generic fallback
            }
        }
    }

    // -------------------------------------------------------------------------
    // Test: Probe structure
    // -------------------------------------------------------------------------

    #[test]
    fn test_probes_have_required_fields() {
        let probes = gather_probes();

        // All probes must have names
        assert!(!probes.group_file.name.is_empty());
        assert!(!probes.group_bak_file.name.is_empty());
        assert!(!probes.diff_summary.name.is_empty());
        assert!(!probes.pacman_backups.name.is_empty());

        // All probes must have findings
        assert!(!probes.group_file.finding.is_empty());
        assert!(!probes.group_bak_file.finding.is_empty());
        assert!(!probes.diff_summary.finding.is_empty());
        assert!(!probes.pacman_backups.finding.is_empty());
    }

    #[test]
    fn test_evidence_artifact_count_is_stable() {
        let probes = gather_probes();
        let evidence = probes.to_evidence();

        // Should have at least 4 evidence artifacts (base probes)
        assert!(evidence.len() >= 4, "Evidence count must be at least 4");
    }

    // -------------------------------------------------------------------------
    // Test: Response outcomes
    // -------------------------------------------------------------------------

    #[test]
    fn test_backup_missing_produces_abstain() {
        // When backup is missing, should abstain with PrerequisitesNotMet
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: Some(FileMetadata {
                exists: true,
                size: 100,
                mode: 0o644,
                uid: 0,
                gid: 0,
                mtime: 1234567890,
                summary: "test".to_string(),
            }),
            group_bak_file: ProbeResult::failed("/etc/group.bak", "Does not exist"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::failed("Diff Summary", "Cannot diff"),
            diff_output: String::new(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "None found"),
            pacman_backup_files: Vec::new(),
        };

        assert!(probes.backup_missing());
        let result = build_abstain_outcome(&probes);

        match result {
            ResponseOutcome::Abstained { reason, explanation, hints, .. } => {
                assert!(matches!(reason, AbstainReason::PrerequisitesNotMet));
                assert!(explanation.contains("backup"));
                assert!(!hints.is_empty(), "Abstain must include hints");
            }
            _ => panic!("Expected Abstained outcome"),
        }
    }

    #[test]
    fn test_backup_missing_includes_hints() {
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: None,
            group_bak_file: ProbeResult::failed("/etc/group.bak", "Does not exist"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::failed("Diff Summary", "Cannot diff"),
            diff_output: String::new(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "None found"),
            pacman_backup_files: Vec::new(),
        };

        let result = build_abstain_outcome(&probes);

        match result {
            ResponseOutcome::Abstained { hints, explanation, .. } => {
                assert!(!hints.is_empty(), "Abstain must include hints");
                // Check hints contain backup-related text
                let hints_text = hints.join(" ");
                assert!(
                    hints_text.contains("pacman") || hints_text.contains("backup"),
                    "Hints must contain backup-related text"
                );
                // Check explanation mentions backup
                assert!(explanation.contains("backup"), "Explanation must mention backup");
            }
            _ => panic!("Expected Abstained outcome"),
        }
    }

    #[test]
    fn test_probe_error_produces_failed() {
        let probes = GroupChangeProbes {
            group_file: ProbeResult::error(
                "/etc/group",
                "Permission denied",
                "Cannot read file",
            ),
            group_metadata: None,
            group_bak_file: ProbeResult::failed("/etc/group.bak", "Not checked"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::failed("Diff Summary", "Not run"),
            diff_output: String::new(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "None found"),
            pacman_backup_files: Vec::new(),
        };

        assert!(probes.has_probe_error().is_some());
        let (probe_name, detail) = probes.has_probe_error().unwrap();
        assert_eq!(probe_name, "/etc/group");
        assert!(detail.contains("Cannot read"));

        // Test build_failed_response
        let result = build_failed_response(probe_name, detail);
        match result {
            ResponseOutcome::Failed { error, diagnostic } => {
                assert!(matches!(error, FailedReason::ProbeError { .. }));
                assert!(diagnostic.contains("/etc/group"));
            }
            _ => panic!("Expected Failed outcome"),
        }
    }

    #[test]
    fn test_both_files_present_produces_resolved() {
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: Some(FileMetadata {
                exists: true,
                size: 100,
                mode: 0o644,
                uid: 0,
                gid: 0,
                mtime: 1234567890,
                summary: "test".to_string(),
            }),
            group_bak_file: ProbeResult::ok("/etc/group.bak", "Exists"),
            group_bak_metadata: Some(FileMetadata {
                exists: true,
                size: 90,
                mode: 0o644,
                uid: 0,
                gid: 0,
                mtime: 1234567800,
                summary: "test".to_string(),
            }),
            diff_summary: ProbeResult::ok("Diff Summary", "5 lines changed"),
            diff_output: "--- /etc/group.bak\n+++ /etc/group\n@@ -1,3 +1,4 @@\n root:x:0:\n+newgroup:x:1001:".to_string(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "None found"),
            pacman_backup_files: Vec::new(),
        };

        assert!(probes.both_files_exist());

        let result = build_resolved_outcome(&probes);

        match result {
            ResponseOutcome::Resolved { explanation, artifacts, .. } => {
                // Must have explanation
                assert!(!explanation.is_empty());

                // Must have evidence (in artifacts)
                let evidence_count = artifacts.iter().filter(|a| a.artifact_type == "evidence").count();
                assert!(evidence_count >= 4, "Should have at least 4 evidence artifacts");

                // Must have steps
                let step_count = artifacts.iter().filter(|a| a.artifact_type == "step").count();
                assert!(step_count >= 2, "Should have at least 2 steps");

                // Must have rollback
                let rollback_count = artifacts.iter().filter(|a| a.artifact_type == "rollback").count();
                assert!(rollback_count >= 1, "Should have at least 1 rollback step");
            }
            _ => panic!("Expected Resolved outcome"),
        }
    }

    #[test]
    fn test_resolved_includes_diff_commands() {
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: None,
            group_bak_file: ProbeResult::ok("/etc/group.bak", "Exists"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::ok("Diff Summary", "Changes detected"),
            diff_output: "some diff output".to_string(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "None"),
            pacman_backup_files: Vec::new(),
        };

        let result = build_resolved_outcome(&probes);

        match result {
            ResponseOutcome::Resolved { artifacts, .. } => {
                let steps: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "step").collect();
                let steps_text: String = steps.iter().map(|s| s.content.clone()).collect();
                assert!(
                    steps_text.contains("diff -u"),
                    "Steps must include diff command"
                );
            }
            _ => panic!("Expected Resolved outcome"),
        }
    }

    #[test]
    fn test_resolved_includes_restore_instructions() {
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: None,
            group_bak_file: ProbeResult::ok("/etc/group.bak", "Exists"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::ok("Diff Summary", "Changes detected"),
            diff_output: "some diff".to_string(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "None"),
            pacman_backup_files: Vec::new(),
        };

        let result = build_resolved_outcome(&probes);

        match result {
            ResponseOutcome::Resolved { artifacts, .. } => {
                let steps: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "step").collect();
                let steps_text: String = steps.iter().map(|s| s.content.clone()).collect();
                assert!(
                    steps_text.contains("restore") || steps_text.contains("install"),
                    "Steps must include restore instructions"
                );
            }
            _ => panic!("Expected Resolved outcome"),
        }
    }

    #[test]
    fn test_resolved_includes_validation_commands() {
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: None,
            group_bak_file: ProbeResult::ok("/etc/group.bak", "Exists"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::ok("Diff Summary", "Changes detected"),
            diff_output: "diff".to_string(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "None"),
            pacman_backup_files: Vec::new(),
        };

        let result = build_resolved_outcome(&probes);

        match result {
            ResponseOutcome::Resolved { artifacts, .. } => {
                let steps: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "step").collect();
                let steps_text: String = steps.iter().map(|s| s.content.clone()).collect();
                assert!(
                    steps_text.contains("getent group") || steps_text.contains("id"),
                    "Steps must include validation commands"
                );
            }
            _ => panic!("Expected Resolved outcome"),
        }
    }

    // -------------------------------------------------------------------------
    // Test: Determinism
    // -------------------------------------------------------------------------

    #[test]
    fn test_handler_is_deterministic() {
        // Handler should produce consistent results
        let result1 = execute_config_review_group_change();
        let result2 = execute_config_review_group_change();

        // Same outcome type
        let is_same_type = matches!(
            (&result1, &result2),
            (ResponseOutcome::Resolved { .. }, ResponseOutcome::Resolved { .. })
                | (ResponseOutcome::Abstained { .. }, ResponseOutcome::Abstained { .. })
                | (ResponseOutcome::Failed { .. }, ResponseOutcome::Failed { .. })
        );
        assert!(is_same_type, "Handler must produce same outcome type on repeated calls");
    }

    #[test]
    fn test_probes_are_read_only() {
        // Running probes multiple times should be safe and produce same structure
        let probes1 = gather_probes();
        let probes2 = gather_probes();

        // Same probe names
        assert_eq!(probes1.group_file.name, probes2.group_file.name);
        assert_eq!(probes1.group_bak_file.name, probes2.group_bak_file.name);
        assert_eq!(probes1.diff_summary.name, probes2.diff_summary.name);
    }

    // -------------------------------------------------------------------------
    // Test: Artifact types
    // -------------------------------------------------------------------------

    #[test]
    fn test_evidence_artifacts_have_correct_type() {
        let probes = gather_probes();
        let evidence = probes.to_evidence();

        for artifact in evidence {
            assert_eq!(artifact.artifact_type, "evidence");
        }
    }

    #[test]
    fn test_step_artifacts_have_correct_type() {
        let step = ResponseArtifact::step(1, "Do something");
        assert_eq!(step.artifact_type, "step");
        assert_eq!(step.label, "Step 1");
    }

    #[test]
    fn test_rollback_artifacts_have_correct_type() {
        let rollback = ResponseArtifact::rollback(1, "Undo something");
        assert_eq!(rollback.artifact_type, "rollback");
        assert_eq!(rollback.label, "Rollback 1");
    }

    // -------------------------------------------------------------------------
    // Test: File metadata
    // -------------------------------------------------------------------------

    #[test]
    fn test_file_metadata_not_found() {
        let meta = FileMetadata::not_found();
        assert!(!meta.exists);
        assert_eq!(meta.size, 0);
    }

    // -------------------------------------------------------------------------
    // Test: Diff truncation
    // -------------------------------------------------------------------------

    #[test]
    fn test_diff_truncation_flag() {
        // When diff is large, should be truncated
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: None,
            group_bak_file: ProbeResult::ok("/etc/group.bak", "Exists"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::ok("Diff Summary", "100 lines (truncated)"),
            diff_output: "truncated output".to_string(),
            diff_truncated: true,
            pacman_backups: ProbeResult::ok("pacman Backups", "None"),
            pacman_backup_files: Vec::new(),
        };

        let result = build_resolved_outcome(&probes);

        match result {
            ResponseOutcome::Resolved { explanation, .. } => {
                assert!(
                    explanation.contains("truncated"),
                    "Should mention truncation"
                );
            }
            _ => panic!("Expected Resolved outcome"),
        }
    }

    // -------------------------------------------------------------------------
    // Test: pacman backup detection
    // -------------------------------------------------------------------------

    #[test]
    fn test_pacman_backups_included_in_hints() {
        let probes = GroupChangeProbes {
            group_file: ProbeResult::ok("/etc/group", "Exists"),
            group_metadata: None,
            group_bak_file: ProbeResult::failed("/etc/group.bak", "Missing"),
            group_bak_metadata: None,
            diff_summary: ProbeResult::failed("Diff Summary", "Cannot diff"),
            diff_output: String::new(),
            diff_truncated: false,
            pacman_backups: ProbeResult::ok("pacman Backups", "Found: /etc/group.pacnew"),
            pacman_backup_files: vec!["/etc/group.pacnew".to_string()],
        };

        let result = build_abstain_outcome(&probes);

        match result {
            ResponseOutcome::Abstained { hints, .. } => {
                let hints_text = hints.join(" ");
                assert!(
                    hints_text.contains("pacnew") || hints_text.contains("pacman"),
                    "Hints should mention pacman backup files"
                );
            }
            _ => panic!("Expected Abstained outcome"),
        }
    }
}
