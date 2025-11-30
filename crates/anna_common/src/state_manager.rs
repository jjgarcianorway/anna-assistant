//! Anna State Manager v3.13.3
//!
//! Unified state management for Anna with atomic operations and proper permissions.
//!
//! ## Problem Solved
//!
//! Previously, state was fragmented across multiple files with different:
//! - Write mechanisms (some atomic, some not)
//! - Permission handling (some failed silently, some crashed)
//! - Reset behaviors (some reset, some didn't)
//!
//! ## Solution
//!
//! Single StateManager that:
//! 1. Owns all state file paths
//! 2. Provides atomic write operations (temp file + rename)
//! 3. Handles permissions consistently
//! 4. Implements reset_soft() and reset_hard() that actually work
//! 5. Verifies resets completed correctly

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Path Constants
// ============================================================================

/// Base data directory
pub const DATA_DIR: &str = "/var/lib/anna";

/// Base log directory
pub const LOG_DIR: &str = "/var/log/anna";

/// XP store file
pub const XP_STORE_FILE: &str = "/var/lib/anna/xp/xp_store.json";

/// Telemetry file
pub const TELEMETRY_FILE: &str = "/var/log/anna/telemetry.jsonl";

/// XP events log
pub const XP_EVENTS_FILE: &str = "/var/lib/anna/knowledge/stats/xp_events.jsonl";

/// Knowledge directory
pub const KNOWLEDGE_DIR: &str = "/var/lib/anna/knowledge";

/// Stats directory
pub const STATS_DIR: &str = "/var/lib/anna/knowledge/stats";

/// LLM state directory (autoprovision)
pub const LLM_STATE_DIR: &str = "/var/lib/anna/llm";

/// Benchmarks directory
pub const BENCHMARKS_DIR: &str = "/var/lib/anna/benchmarks";

/// Model registry file
pub const MODEL_REGISTRY_FILE: &str = "/var/lib/anna/model_registry.json";

// ============================================================================
// Atomic File Operations
// ============================================================================

/// Write data to a file atomically using temp file + rename
/// This ensures the file is never in a partial state
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        // Try to make directory writable
        let _ = set_permissions_rwx(parent);
    }

    // Create temp file in same directory (for atomic rename)
    let temp_path = path.with_extension("tmp");

    // Write to temp file
    let mut file = File::create(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?; // Ensure data is on disk

    // Atomic rename
    fs::rename(&temp_path, path)?;

    // Set permissions on new file
    let _ = set_permissions_rw(path);

    Ok(())
}

/// Write string data atomically
pub fn atomic_write_str(path: &Path, data: &str) -> io::Result<()> {
    atomic_write(path, data.as_bytes())
}

/// Safely truncate a file to zero length
pub fn safe_truncate(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // Open with truncate flag
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;

    file.sync_all()?;
    Ok(())
}

/// Safely delete a file (no error if doesn't exist)
pub fn safe_delete(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Safely delete a directory recursively
pub fn safe_delete_dir(path: &Path) -> io::Result<()> {
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Clear directory contents but keep the directory
pub fn clear_directory(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            fs::remove_dir_all(&entry_path)?;
        } else {
            fs::remove_file(&entry_path)?;
        }
    }
    Ok(())
}

/// Set file permissions to rw-rw-rw- (666)
fn set_permissions_rw(path: &Path) -> io::Result<()> {
    let perms = fs::Permissions::from_mode(0o666);
    fs::set_permissions(path, perms)
}

/// Set directory permissions to rwxrwxrwx (777)
fn set_permissions_rwx(path: &Path) -> io::Result<()> {
    let perms = fs::Permissions::from_mode(0o777);
    fs::set_permissions(path, perms)
}

// ============================================================================
// State Verification
// ============================================================================

/// Result of state verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVerification {
    pub xp_reset: bool,
    pub telemetry_reset: bool,
    pub xp_events_reset: bool,
    pub stats_reset: bool,
    pub knowledge_reset: bool,
    pub llm_state_reset: bool,
    pub benchmarks_reset: bool,
    pub all_verified: bool,
    pub errors: Vec<String>,
}

impl StateVerification {
    pub fn new() -> Self {
        Self {
            xp_reset: false,
            telemetry_reset: false,
            xp_events_reset: false,
            stats_reset: false,
            knowledge_reset: false,
            llm_state_reset: false,
            benchmarks_reset: false,
            all_verified: false,
            errors: vec![],
        }
    }

    pub fn verify_soft_reset(&mut self) {
        self.all_verified = self.xp_reset
            && self.telemetry_reset
            && self.xp_events_reset
            && self.stats_reset;
    }

    pub fn verify_hard_reset(&mut self) {
        self.all_verified = self.xp_reset
            && self.telemetry_reset
            && self.xp_events_reset
            && self.stats_reset
            && self.knowledge_reset
            && self.llm_state_reset
            && self.benchmarks_reset;
    }
}

impl Default for StateVerification {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Reset Result
// ============================================================================

/// Result of a reset operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetResult {
    pub success: bool,
    pub reset_type: String,
    pub components_reset: Vec<String>,
    pub components_clean: Vec<String>,
    pub errors: Vec<String>,
    pub verification: StateVerification,
}

impl ResetResult {
    pub fn new(reset_type: &str) -> Self {
        Self {
            success: true,
            reset_type: reset_type.to_string(),
            components_reset: vec![],
            components_clean: vec![],
            errors: vec![],
            verification: StateVerification::new(),
        }
    }

    pub fn add_reset(&mut self, component: &str) {
        self.components_reset.push(component.to_string());
    }

    pub fn add_clean(&mut self, component: &str) {
        self.components_clean.push(component.to_string());
    }

    pub fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
        self.success = false;
    }
}

// ============================================================================
// Baseline XP Store
// ============================================================================

/// Generate baseline XP store JSON (fresh state)
pub fn baseline_xp_store() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    format!(r#"{{
  "anna": {{
    "name": "Anna",
    "level": 1,
    "xp": 0,
    "xp_to_next": 100,
    "streak_good": 0,
    "streak_bad": 0,
    "trust": 0.5,
    "total_good": 0,
    "total_bad": 0,
    "last_update": {}
  }},
  "junior": {{
    "name": "Junior",
    "level": 1,
    "xp": 0,
    "xp_to_next": 100,
    "streak_good": 0,
    "streak_bad": 0,
    "trust": 0.5,
    "total_good": 0,
    "total_bad": 0,
    "last_update": {}
  }},
  "senior": {{
    "name": "Senior",
    "level": 1,
    "xp": 0,
    "xp_to_next": 100,
    "streak_good": 0,
    "streak_bad": 0,
    "trust": 0.5,
    "total_good": 0,
    "total_bad": 0,
    "last_update": {}
  }},
  "anna_stats": {{
    "self_solves": 0,
    "brain_assists": 0,
    "llm_answers": 0,
    "refusals": 0,
    "timeouts": 0,
    "total_questions": 0,
    "avg_reliability": 0.0,
    "avg_latency_ms": 0
  }},
  "junior_stats": {{
    "good_plans": 0,
    "bad_plans": 0,
    "timeouts": 0,
    "needs_fix": 0,
    "overcomplicated": 0
  }},
  "senior_stats": {{
    "approvals": 0,
    "fix_and_accept": 0,
    "rubber_stamps_blocked": 0,
    "refusals": 0,
    "timeouts": 0
  }}
}}"#, now, now, now)
}

// ============================================================================
// State Manager
// ============================================================================

/// Unified state manager for Anna
///
/// Handles all state files with atomic operations and proper permissions.
pub struct AnnaStateManager {
    pub xp_store_path: PathBuf,
    pub telemetry_path: PathBuf,
    pub xp_events_path: PathBuf,
    pub stats_dir: PathBuf,
    pub knowledge_dir: PathBuf,
    pub llm_state_dir: PathBuf,
    pub benchmarks_dir: PathBuf,
}

impl Default for AnnaStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AnnaStateManager {
    /// Create state manager with default paths
    pub fn new() -> Self {
        Self {
            xp_store_path: PathBuf::from(XP_STORE_FILE),
            telemetry_path: PathBuf::from(TELEMETRY_FILE),
            xp_events_path: PathBuf::from(XP_EVENTS_FILE),
            stats_dir: PathBuf::from(STATS_DIR),
            knowledge_dir: PathBuf::from(KNOWLEDGE_DIR),
            llm_state_dir: PathBuf::from(LLM_STATE_DIR),
            benchmarks_dir: PathBuf::from(BENCHMARKS_DIR),
        }
    }

    /// Create state manager with custom root (for testing)
    pub fn with_root(root: &Path) -> Self {
        Self {
            xp_store_path: root.join("var/lib/anna/xp/xp_store.json"),
            telemetry_path: root.join("var/log/anna/telemetry.jsonl"),
            xp_events_path: root.join("var/lib/anna/knowledge/stats/xp_events.jsonl"),
            stats_dir: root.join("var/lib/anna/knowledge/stats"),
            knowledge_dir: root.join("var/lib/anna/knowledge"),
            llm_state_dir: root.join("var/lib/anna/llm"),
            benchmarks_dir: root.join("var/lib/anna/benchmarks"),
        }
    }

    // ========================================================================
    // Directory Validation
    // ========================================================================

    /// Ensure all state directories exist with proper permissions
    pub fn ensure_directories(&self) -> io::Result<()> {
        let dirs = [
            self.xp_store_path.parent().unwrap_or(Path::new("/")),
            self.telemetry_path.parent().unwrap_or(Path::new("/")),
            &self.stats_dir,
            &self.knowledge_dir,
            &self.llm_state_dir,
            &self.benchmarks_dir,
        ];

        for dir in dirs {
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }
            let _ = set_permissions_rwx(dir);
        }

        Ok(())
    }

    /// Validate that we can write to all state locations
    pub fn validate_permissions(&self) -> Vec<String> {
        let mut errors = vec![];

        // Test XP store directory
        if let Some(parent) = self.xp_store_path.parent() {
            let test_file = parent.join(".permission_test");
            if fs::write(&test_file, "test").is_err() {
                errors.push(format!("Cannot write to XP directory: {}", parent.display()));
            } else {
                let _ = fs::remove_file(&test_file);
            }
        }

        // Test telemetry directory
        if let Some(parent) = self.telemetry_path.parent() {
            let test_file = parent.join(".permission_test");
            if fs::write(&test_file, "test").is_err() {
                errors.push(format!("Cannot write to telemetry directory: {}", parent.display()));
            } else {
                let _ = fs::remove_file(&test_file);
            }
        }

        errors
    }

    // ========================================================================
    // XP Store Operations
    // ========================================================================

    /// Write XP store atomically
    pub fn write_xp_store(&self, data: &str) -> io::Result<()> {
        atomic_write_str(&self.xp_store_path, data)
    }

    /// Read XP store
    pub fn read_xp_store(&self) -> io::Result<String> {
        fs::read_to_string(&self.xp_store_path)
    }

    /// Reset XP store to baseline
    pub fn reset_xp_store(&self) -> io::Result<()> {
        atomic_write_str(&self.xp_store_path, &baseline_xp_store())
    }

    /// Verify XP store is at baseline
    pub fn verify_xp_baseline(&self) -> bool {
        if let Ok(content) = self.read_xp_store() {
            // Check for baseline indicators
            content.contains("\"level\": 1")
                && content.contains("\"xp\": 0")
                && content.contains("\"total_questions\": 0")
                && content.contains("\"trust\": 0.5")
        } else {
            // No file = baseline (will be created on first write)
            true
        }
    }

    // ========================================================================
    // Telemetry Operations
    // ========================================================================

    /// Append to telemetry file
    pub fn append_telemetry(&self, line: &str) -> io::Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.telemetry_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.telemetry_path)?;

        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Reset telemetry (truncate to empty)
    pub fn reset_telemetry(&self) -> io::Result<()> {
        if self.telemetry_path.exists() {
            // Truncate by creating empty file
            File::create(&self.telemetry_path)?;
        }
        Ok(())
    }

    /// Get telemetry line count
    pub fn telemetry_line_count(&self) -> usize {
        if let Ok(content) = fs::read_to_string(&self.telemetry_path) {
            content.lines().count()
        } else {
            0
        }
    }

    /// Verify telemetry is empty
    pub fn verify_telemetry_empty(&self) -> bool {
        if self.telemetry_path.exists() {
            if let Ok(meta) = fs::metadata(&self.telemetry_path) {
                return meta.len() == 0;
            }
        }
        true // No file = empty
    }

    // ========================================================================
    // XP Events Log Operations
    // ========================================================================

    /// Append to XP events log
    pub fn append_xp_event(&self, line: &str) -> io::Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.xp_events_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.xp_events_path)?;

        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Reset XP events log
    pub fn reset_xp_events(&self) -> io::Result<()> {
        if self.xp_events_path.exists() {
            File::create(&self.xp_events_path)?;
        }
        Ok(())
    }

    /// Verify XP events log is empty
    pub fn verify_xp_events_empty(&self) -> bool {
        if self.xp_events_path.exists() {
            if let Ok(meta) = fs::metadata(&self.xp_events_path) {
                return meta.len() == 0;
            }
        }
        true
    }

    // ========================================================================
    // Stats Directory Operations
    // ========================================================================

    /// Reset stats directory (clear contents)
    pub fn reset_stats(&self) -> io::Result<()> {
        clear_directory(&self.stats_dir)
    }

    /// Verify stats directory is empty
    pub fn verify_stats_empty(&self) -> bool {
        if self.stats_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.stats_dir) {
                return entries.count() == 0;
            }
        }
        true
    }

    // ========================================================================
    // Knowledge Directory Operations
    // ========================================================================

    /// Reset knowledge directory (for hard reset)
    pub fn reset_knowledge(&self) -> io::Result<()> {
        clear_directory(&self.knowledge_dir)?;
        // Recreate stats subdirectory
        fs::create_dir_all(&self.stats_dir)?;
        Ok(())
    }

    /// Count knowledge files
    pub fn knowledge_file_count(&self) -> usize {
        if let Ok(entries) = fs::read_dir(&self.knowledge_dir) {
            entries.filter(|e| e.is_ok()).count()
        } else {
            0
        }
    }

    /// Verify knowledge is empty (except stats dir)
    pub fn verify_knowledge_empty(&self) -> bool {
        if self.knowledge_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.knowledge_dir) {
                // Allow stats subdirectory but nothing else
                let count = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path() != self.stats_dir)
                    .count();
                return count == 0;
            }
        }
        true
    }

    // ========================================================================
    // LLM State Operations (Autoprovision)
    // ========================================================================

    /// Reset LLM state directory
    pub fn reset_llm_state(&self) -> io::Result<()> {
        clear_directory(&self.llm_state_dir)
    }

    /// Verify LLM state is empty
    pub fn verify_llm_state_empty(&self) -> bool {
        if self.llm_state_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.llm_state_dir) {
                return entries.count() == 0;
            }
        }
        true
    }

    // ========================================================================
    // Benchmarks Operations
    // ========================================================================

    /// Reset benchmarks directory
    pub fn reset_benchmarks(&self) -> io::Result<()> {
        clear_directory(&self.benchmarks_dir)
    }

    /// Verify benchmarks are empty
    pub fn verify_benchmarks_empty(&self) -> bool {
        if self.benchmarks_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.benchmarks_dir) {
                return entries.count() == 0;
            }
        }
        true
    }

    // ========================================================================
    // Unified Reset Operations
    // ========================================================================

    /// Soft reset: XP to baseline, clear telemetry/stats, preserve knowledge
    pub fn reset_soft(&self) -> ResetResult {
        let mut result = ResetResult::new("soft");

        // Ensure directories exist
        if let Err(e) = self.ensure_directories() {
            result.add_error(&format!("Failed to ensure directories: {}", e));
        }

        // 1. Reset XP store to baseline
        match self.reset_xp_store() {
            Ok(_) => result.add_reset("XP store"),
            Err(e) => result.add_error(&format!("Failed to reset XP store: {}", e)),
        }

        // 2. Reset telemetry
        match self.reset_telemetry() {
            Ok(_) => result.add_reset("Telemetry"),
            Err(e) => result.add_error(&format!("Failed to reset telemetry: {}", e)),
        }

        // 3. Reset XP events log
        match self.reset_xp_events() {
            Ok(_) => result.add_reset("XP events"),
            Err(e) => result.add_error(&format!("Failed to reset XP events: {}", e)),
        }

        // 4. Reset stats directory
        match self.reset_stats() {
            Ok(_) => result.add_reset("Stats"),
            Err(e) => result.add_error(&format!("Failed to reset stats: {}", e)),
        }

        // Verify reset
        result.verification = self.verify_soft_reset();
        if !result.verification.all_verified {
            result.add_error("Verification failed: some components not reset");
        }

        result
    }

    /// Hard reset: Everything including knowledge, LLM state, benchmarks
    pub fn reset_hard(&self) -> ResetResult {
        let mut result = ResetResult::new("hard");

        // Ensure directories exist
        if let Err(e) = self.ensure_directories() {
            result.add_error(&format!("Failed to ensure directories: {}", e));
        }

        // 1. Reset XP store to baseline
        match self.reset_xp_store() {
            Ok(_) => result.add_reset("XP store"),
            Err(e) => result.add_error(&format!("Failed to reset XP store: {}", e)),
        }

        // 2. Reset telemetry
        match self.reset_telemetry() {
            Ok(_) => result.add_reset("Telemetry"),
            Err(e) => result.add_error(&format!("Failed to reset telemetry: {}", e)),
        }

        // 3. Reset XP events log
        match self.reset_xp_events() {
            Ok(_) => result.add_reset("XP events"),
            Err(e) => result.add_error(&format!("Failed to reset XP events: {}", e)),
        }

        // 4. Reset stats directory
        match self.reset_stats() {
            Ok(_) => result.add_reset("Stats"),
            Err(e) => result.add_error(&format!("Failed to reset stats: {}", e)),
        }

        // 5. Reset knowledge (hard reset only)
        match self.reset_knowledge() {
            Ok(_) => result.add_reset("Knowledge"),
            Err(e) => result.add_error(&format!("Failed to reset knowledge: {}", e)),
        }

        // 6. Reset LLM state / autoprovision
        match self.reset_llm_state() {
            Ok(_) => result.add_reset("LLM state"),
            Err(e) => result.add_error(&format!("Failed to reset LLM state: {}", e)),
        }

        // 7. Reset benchmarks
        match self.reset_benchmarks() {
            Ok(_) => result.add_reset("Benchmarks"),
            Err(e) => result.add_error(&format!("Failed to reset benchmarks: {}", e)),
        }

        // Verify reset
        result.verification = self.verify_hard_reset();
        if !result.verification.all_verified {
            result.add_error("Verification failed: some components not reset");
        }

        result
    }

    /// Verify soft reset completed
    pub fn verify_soft_reset(&self) -> StateVerification {
        let mut v = StateVerification::new();

        v.xp_reset = self.verify_xp_baseline();
        v.telemetry_reset = self.verify_telemetry_empty();
        v.xp_events_reset = self.verify_xp_events_empty();
        v.stats_reset = self.verify_stats_empty();

        if !v.xp_reset { v.errors.push("XP not at baseline".to_string()); }
        if !v.telemetry_reset { v.errors.push("Telemetry not empty".to_string()); }
        if !v.xp_events_reset { v.errors.push("XP events not empty".to_string()); }
        if !v.stats_reset { v.errors.push("Stats not empty".to_string()); }

        v.verify_soft_reset();
        v
    }

    /// Verify hard reset completed
    pub fn verify_hard_reset(&self) -> StateVerification {
        let mut v = StateVerification::new();

        v.xp_reset = self.verify_xp_baseline();
        v.telemetry_reset = self.verify_telemetry_empty();
        v.xp_events_reset = self.verify_xp_events_empty();
        v.stats_reset = self.verify_stats_empty();
        v.knowledge_reset = self.verify_knowledge_empty();
        v.llm_state_reset = self.verify_llm_state_empty();
        v.benchmarks_reset = self.verify_benchmarks_empty();

        if !v.xp_reset { v.errors.push("XP not at baseline".to_string()); }
        if !v.telemetry_reset { v.errors.push("Telemetry not empty".to_string()); }
        if !v.xp_events_reset { v.errors.push("XP events not empty".to_string()); }
        if !v.stats_reset { v.errors.push("Stats not empty".to_string()); }
        if !v.knowledge_reset { v.errors.push("Knowledge not empty".to_string()); }
        if !v.llm_state_reset { v.errors.push("LLM state not empty".to_string()); }
        if !v.benchmarks_reset { v.errors.push("Benchmarks not empty".to_string()); }

        v.verify_hard_reset();
        v
    }
}

// ============================================================================
// Global State Manager Instance
// ============================================================================

/// Get the global state manager instance
/// Uses lazy initialization with default paths
pub fn state_manager() -> AnnaStateManager {
    AnnaStateManager::new()
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Reset Anna to soft baseline (preserves knowledge)
pub fn reset_soft() -> ResetResult {
    state_manager().reset_soft()
}

/// Reset Anna to hard baseline (clears everything)
pub fn reset_hard() -> ResetResult {
    state_manager().reset_hard()
}

/// Verify permissions are correct
pub fn verify_permissions() -> Vec<String> {
    state_manager().validate_permissions()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_baseline_xp_store() {
        let json = baseline_xp_store();
        assert!(json.contains("\"level\": 1"));
        assert!(json.contains("\"xp\": 0"));
        assert!(json.contains("\"trust\": 0.5"));
        assert!(json.contains("\"total_questions\": 0"));
    }

    #[test]
    fn test_state_manager_with_root() {
        let temp = TempDir::new().unwrap();
        let mgr = AnnaStateManager::with_root(temp.path());

        // Paths should be under temp dir
        assert!(mgr.xp_store_path.starts_with(temp.path()));
        assert!(mgr.telemetry_path.starts_with(temp.path()));
    }

    #[test]
    fn test_soft_reset() {
        let temp = TempDir::new().unwrap();
        let mgr = AnnaStateManager::with_root(temp.path());

        // Ensure directories
        mgr.ensure_directories().unwrap();

        // Write some state
        atomic_write_str(&mgr.xp_store_path, r#"{"anna":{"level":5}}"#).unwrap();
        mgr.append_telemetry(r#"{"event":"test"}"#).unwrap();

        // Reset
        let result = mgr.reset_soft();

        // Verify
        assert!(result.success, "Reset should succeed: {:?}", result.errors);
        assert!(mgr.verify_xp_baseline(), "XP should be at baseline");
        assert!(mgr.verify_telemetry_empty(), "Telemetry should be empty");
    }

    #[test]
    fn test_hard_reset() {
        let temp = TempDir::new().unwrap();
        let mgr = AnnaStateManager::with_root(temp.path());

        // Ensure directories
        mgr.ensure_directories().unwrap();

        // Write some state including knowledge
        atomic_write_str(&mgr.xp_store_path, r#"{"anna":{"level":10}}"#).unwrap();
        fs::write(mgr.knowledge_dir.join("fact.json"), "{}").unwrap();
        fs::write(mgr.llm_state_dir.join("selection.json"), "{}").unwrap();

        // Reset
        let result = mgr.reset_hard();

        // Verify
        assert!(result.success, "Reset should succeed: {:?}", result.errors);
        assert!(result.verification.all_verified, "All components should be verified");
        assert!(mgr.verify_xp_baseline());
        assert!(mgr.verify_knowledge_empty());
        assert!(mgr.verify_llm_state_empty());
    }

    #[test]
    fn test_atomic_write() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.json");

        atomic_write_str(&file, "test data").unwrap();

        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "test data");
    }

    #[test]
    fn test_safe_truncate() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");

        // Write some data
        fs::write(&file, "some data").unwrap();
        assert!(fs::metadata(&file).unwrap().len() > 0);

        // Truncate
        safe_truncate(&file).unwrap();
        assert_eq!(fs::metadata(&file).unwrap().len(), 0);
    }

    #[test]
    fn test_v3133_reset_hard_full_state() {
        let temp = TempDir::new().unwrap();
        let mgr = AnnaStateManager::with_root(temp.path());

        // Setup: Ensure directories and create realistic state
        mgr.ensure_directories().unwrap();

        // Create XP with high level
        let xp_data = r#"{"anna":{"level":10,"xp":1000,"trust":0.9}}"#;
        atomic_write_str(&mgr.xp_store_path, xp_data).unwrap();

        // Create telemetry events
        for i in 0..100 {
            mgr.append_telemetry(&format!(r#"{{"event":{}}}"#, i)).unwrap();
        }

        // Create knowledge files
        fs::write(mgr.knowledge_dir.join("learned.json"), "{}").unwrap();

        // Create LLM selection
        fs::write(mgr.llm_state_dir.join("selection.json"), r#"{"junior":"qwen3:4b"}"#).unwrap();

        // Create benchmark results
        fs::write(mgr.benchmarks_dir.join("last.json"), "{}").unwrap();

        // Verify state exists
        assert!(!mgr.verify_xp_baseline());
        assert!(!mgr.verify_telemetry_empty());
        assert!(!mgr.verify_knowledge_empty());
        assert!(!mgr.verify_llm_state_empty());
        assert!(!mgr.verify_benchmarks_empty());

        // Perform hard reset
        let result = mgr.reset_hard();

        // Verify complete reset
        assert!(result.success, "Hard reset should succeed");
        assert!(result.verification.all_verified, "All should be verified: {:?}", result.verification.errors);

        // Individual checks
        assert!(mgr.verify_xp_baseline(), "XP should be baseline");
        assert!(mgr.verify_telemetry_empty(), "Telemetry should be empty");
        assert!(mgr.verify_xp_events_empty(), "XP events should be empty");
        assert!(mgr.verify_stats_empty(), "Stats should be empty");
        assert!(mgr.verify_knowledge_empty(), "Knowledge should be empty");
        assert!(mgr.verify_llm_state_empty(), "LLM state should be empty");
        assert!(mgr.verify_benchmarks_empty(), "Benchmarks should be empty");

        // Verify XP store content
        let xp_content = mgr.read_xp_store().unwrap();
        assert!(xp_content.contains("\"level\": 1"));
        assert!(xp_content.contains("\"xp\": 0"));
        assert!(xp_content.contains("\"trust\": 0.5"));
    }
}
