//! Experience Reset Module v1.3.0
//!
//! Provides two reset modes for Anna's learned state:
//!
//! ## Experience Reset (Soft Reset)
//!
//! Resets XP, trust, streaks, and counters to baseline values (level 1, trust 0.5).
//! Clears telemetry and stats but preserves knowledge.
//!
//! **What Gets Reset to Baseline:**
//! - XP Store: `/var/lib/anna/xp/xp_store.json` → fresh XpStore (level 1, trust 0.5)
//! - Telemetry: `/var/log/anna/telemetry.jsonl` → truncated to empty
//! - Stats/Learning: `/var/lib/anna/knowledge/stats/*` → directory cleared
//!
//! **What Is Preserved:**
//! - Config: `/etc/anna/*`
//! - Knowledge base: `/var/lib/anna/knowledge/*` (except stats)
//! - Probes, binaries, services
//!
//! ## Factory Reset (Hard Reset)
//!
//! Does everything Experience Reset does PLUS deletes the knowledge base.
//! Requires explicit confirmation string to execute.
//!
//! **What Gets Deleted:**
//! - Everything from Experience Reset
//! - Knowledge directory: `/var/lib/anna/knowledge/*`
//!
//! **What Is Preserved:**
//! - Config: `/etc/anna/*`
//! - Probes, binaries, services
//! - System packages

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Path Configuration
// ============================================================================

/// Paths that can be reset (configurable for testing)
#[derive(Debug, Clone)]
pub struct ExperiencePaths {
    /// XP store directory (contains xp_store.json)
    pub xp_dir: PathBuf,
    /// Telemetry file
    pub telemetry_file: PathBuf,
    /// Stats/learning directory
    pub stats_dir: PathBuf,
    /// Knowledge directory (for factory reset)
    pub knowledge_dir: PathBuf,
}

impl Default for ExperiencePaths {
    fn default() -> Self {
        Self {
            xp_dir: PathBuf::from("/var/lib/anna/xp"),
            telemetry_file: PathBuf::from("/var/log/anna/telemetry.jsonl"),
            stats_dir: PathBuf::from("/var/lib/anna/knowledge/stats"),
            knowledge_dir: PathBuf::from("/var/lib/anna/knowledge"),
        }
    }
}

impl ExperiencePaths {
    /// Create paths rooted at a custom directory (for testing)
    pub fn with_root(root: &Path) -> Self {
        Self {
            xp_dir: root.join("var/lib/anna/xp"),
            telemetry_file: root.join("var/log/anna/telemetry.jsonl"),
            stats_dir: root.join("var/lib/anna/knowledge/stats"),
            knowledge_dir: root.join("var/lib/anna/knowledge"),
        }
    }

    /// XP store file path
    pub fn xp_store_file(&self) -> PathBuf {
        self.xp_dir.join("xp_store.json")
    }
}

// ============================================================================
// Reset Type
// ============================================================================

/// Type of reset to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetType {
    /// Soft reset: XP to baseline, clear telemetry/stats, keep knowledge
    Experience,
    /// Hard reset: Everything from Experience + delete knowledge
    Factory,
}

impl ResetType {
    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            ResetType::Experience => "Experience Reset",
            ResetType::Factory => "Factory Reset",
        }
    }

    /// Confirmation message for this reset type
    pub fn confirmation_prompt(&self) -> &'static str {
        match self {
            ResetType::Experience => {
                "This will reset XP, trust, and counters to baseline (level 1, trust 0.5).\n\
                 Telemetry and stats will be cleared. Knowledge is preserved.\n\
                 Type 'yes' to confirm."
            }
            ResetType::Factory => {
                "⚠️  FACTORY RESET WARNING ⚠️\n\n\
                 This will delete ALL learned data including:\n\
                 - XP, levels, trust, streaks (reset to baseline)\n\
                 - Telemetry history\n\
                 - Stats and learning artifacts\n\
                 - Knowledge base and learned facts\n\n\
                 This is IRREVERSIBLE. To confirm, type exactly:\n\
                 I UNDERSTAND AND CONFIRM FACTORY RESET"
            }
        }
    }

    /// Check if confirmation string matches
    pub fn is_confirmed(&self, input: &str) -> bool {
        let trimmed = input.trim();
        match self {
            ResetType::Experience => {
                trimmed.eq_ignore_ascii_case("yes")
            }
            ResetType::Factory => {
                trimmed == "I UNDERSTAND AND CONFIRM FACTORY RESET"
            }
        }
    }
}

// ============================================================================
// Reset Result
// ============================================================================

/// Result of a reset operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceResetResult {
    /// Type of reset performed
    pub reset_type: ResetType,
    /// Whether the reset was successful overall
    pub success: bool,
    /// Components that were reset
    pub components_reset: Vec<String>,
    /// Components that were already clean
    pub components_clean: Vec<String>,
    /// Errors encountered (non-fatal)
    pub errors: Vec<String>,
    /// Human-readable summary
    pub summary: String,
}

impl ExperienceResetResult {
    fn new(reset_type: ResetType) -> Self {
        Self {
            reset_type,
            success: true,
            components_reset: vec![],
            components_clean: vec![],
            errors: vec![],
            summary: String::new(),
        }
    }

    fn add_reset(&mut self, component: &str) {
        self.components_reset.push(component.to_string());
    }

    fn add_clean(&mut self, component: &str) {
        self.components_clean.push(component.to_string());
    }

    fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
        self.success = false;
    }

    fn finalize(&mut self) {
        let reset_count = self.components_reset.len();
        let clean_count = self.components_clean.len();
        let error_count = self.errors.len();
        let type_label = self.reset_type.label();

        if error_count > 0 {
            self.summary = format!(
                "{} completed with errors: {} reset, {} already clean, {} errors",
                type_label, reset_count, clean_count, error_count
            );
        } else if reset_count == 0 && clean_count > 0 {
            self.summary = format!(
                "{}: Already clean, {} components had nothing to reset",
                type_label, clean_count
            );
        } else {
            self.summary = format!(
                "{} complete: {} components reset, {} already clean",
                type_label, reset_count, clean_count
            );
        }
    }
}

// ============================================================================
// Baseline XP Store (fresh state)
// ============================================================================

/// Generate a fresh XP store JSON with baseline values
/// This matches XpStore::new() from xp_track.rs
fn baseline_xp_store_json() -> String {
    // XpTrack::new() baseline: level 1, xp 0, trust 0.5, all counters 0
    r#"{
  "anna": {
    "name": "Anna",
    "level": 1,
    "xp": 0,
    "xp_to_next": 100,
    "streak_good": 0,
    "streak_bad": 0,
    "trust": 0.5,
    "total_good": 0,
    "total_bad": 0,
    "last_update": 0
  },
  "junior": {
    "name": "Junior",
    "level": 1,
    "xp": 0,
    "xp_to_next": 100,
    "streak_good": 0,
    "streak_bad": 0,
    "trust": 0.5,
    "total_good": 0,
    "total_bad": 0,
    "last_update": 0
  },
  "senior": {
    "name": "Senior",
    "level": 1,
    "xp": 0,
    "xp_to_next": 100,
    "streak_good": 0,
    "streak_bad": 0,
    "trust": 0.5,
    "total_good": 0,
    "total_bad": 0,
    "last_update": 0
  },
  "anna_stats": {
    "self_solves": 0,
    "brain_assists": 0,
    "llm_answers": 0,
    "refusals": 0,
    "timeouts": 0,
    "total_questions": 0,
    "avg_reliability": 0.0,
    "avg_latency_ms": 0
  },
  "junior_stats": {
    "good_plans": 0,
    "bad_plans": 0,
    "timeouts": 0,
    "needs_fix": 0,
    "overcomplicated": 0
  },
  "senior_stats": {
    "approvals": 0,
    "fix_and_accept": 0,
    "rubber_stamps_blocked": 0,
    "refusals": 0,
    "timeouts": 0
  }
}"#.to_string()
}

// ============================================================================
// Experience Reset Implementation (Soft Reset)
// ============================================================================

/// Reset Anna's experience (XP, telemetry, stats) to baseline
///
/// This resets:
/// - XP store to baseline (level 1, trust 0.5, all counters 0)
/// - Telemetry (cleared)
/// - Stats/learning artifacts (cleared)
///
/// This preserves:
/// - Config files
/// - Knowledge base
/// - Probes and binaries
pub fn reset_experience(paths: &ExperiencePaths) -> ExperienceResetResult {
    let mut result = ExperienceResetResult::new(ResetType::Experience);

    // 1. Reset XP store to baseline
    reset_xp_store_to_baseline(paths, &mut result);

    // 2. Reset telemetry
    reset_telemetry(paths, &mut result);

    // 3. Reset stats directory
    reset_stats_dir(paths, &mut result);

    result.finalize();
    result
}

/// Reset XP store to baseline values (not delete)
fn reset_xp_store_to_baseline(paths: &ExperiencePaths, result: &mut ExperienceResetResult) {
    let xp_file = paths.xp_store_file();

    // Check if already at baseline (file doesn't exist or is baseline)
    if !xp_file.exists() {
        // No file = already clean, write baseline
        if let Err(e) = fs::create_dir_all(&paths.xp_dir) {
            result.add_error(&format!("Failed to create XP directory: {}", e));
            return;
        }
        match fs::write(&xp_file, baseline_xp_store_json()) {
            Ok(_) => result.add_clean("XP store (initialized to baseline)"),
            Err(e) => result.add_error(&format!("Failed to write baseline XP store: {}", e)),
        }
        return;
    }

    // File exists - check if it's already at baseline
    if let Ok(content) = fs::read_to_string(&xp_file) {
        // Quick check: baseline has level 1, xp 0
        if content.contains("\"level\": 1") && content.contains("\"xp\": 0")
            && content.contains("\"total_questions\": 0")
        {
            result.add_clean("XP store");
            return;
        }
    }

    // Write baseline
    match fs::write(&xp_file, baseline_xp_store_json()) {
        Ok(_) => result.add_reset("XP store (reset to level 1, trust 0.5)"),
        Err(e) => result.add_error(&format!("Failed to reset XP store: {}", e)),
    }
}

/// Reset telemetry file (truncate to empty)
fn reset_telemetry(paths: &ExperiencePaths, result: &mut ExperienceResetResult) {
    let telemetry_file = &paths.telemetry_file;

    if !telemetry_file.exists() {
        result.add_clean("Telemetry");
        return;
    }

    // Check if already empty
    match fs::metadata(telemetry_file) {
        Ok(meta) if meta.len() == 0 => {
            result.add_clean("Telemetry");
            return;
        }
        Err(e) => {
            result.add_error(&format!("Failed to read telemetry metadata: {}", e));
            return;
        }
        _ => {}
    }

    // Truncate to empty (preserves file, clears content)
    match fs::write(telemetry_file, "") {
        Ok(_) => result.add_reset("Telemetry"),
        Err(e) => result.add_error(&format!("Failed to truncate telemetry: {}", e)),
    }
}

/// Reset stats directory (remove all files, keep directory)
fn reset_stats_dir(paths: &ExperiencePaths, result: &mut ExperienceResetResult) {
    let stats_dir = &paths.stats_dir;

    if !stats_dir.exists() {
        result.add_clean("Stats");
        return;
    }

    // Check if directory is empty
    match fs::read_dir(stats_dir) {
        Ok(entries) => {
            let files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            if files.is_empty() {
                result.add_clean("Stats");
                return;
            }

            // Remove all files in the directory
            let mut removed = 0;
            let mut errors = 0;
            for entry in files {
                let path = entry.path();
                if path.is_file() {
                    if fs::remove_file(&path).is_ok() {
                        removed += 1;
                    } else {
                        errors += 1;
                    }
                } else if path.is_dir() {
                    if fs::remove_dir_all(&path).is_ok() {
                        removed += 1;
                    } else {
                        errors += 1;
                    }
                }
            }

            if errors > 0 {
                result.add_error(&format!(
                    "Failed to remove {} files in stats directory",
                    errors
                ));
            }
            if removed > 0 {
                result.add_reset(&format!("Stats ({} files)", removed));
            }
        }
        Err(e) => {
            result.add_error(&format!("Failed to read stats directory: {}", e));
        }
    }
}

// ============================================================================
// Factory Reset Implementation (Hard Reset)
// ============================================================================

/// Factory reset: Experience reset + delete knowledge
///
/// This resets EVERYTHING:
/// - XP store to baseline
/// - Telemetry (cleared)
/// - Stats (cleared)
/// - Knowledge directory (deleted)
///
/// This preserves:
/// - Config files
/// - Probes and binaries
pub fn reset_factory(paths: &ExperiencePaths) -> ExperienceResetResult {
    let mut result = ExperienceResetResult::new(ResetType::Factory);

    // 1. Reset XP store to baseline
    reset_xp_store_to_baseline(paths, &mut result);

    // 2. Reset telemetry
    reset_telemetry(paths, &mut result);

    // 3. Delete knowledge directory (not just stats)
    reset_knowledge_dir(paths, &mut result);

    result.finalize();
    result
}

/// Delete knowledge directory contents (factory reset only)
fn reset_knowledge_dir(paths: &ExperiencePaths, result: &mut ExperienceResetResult) {
    let knowledge_dir = &paths.knowledge_dir;

    if !knowledge_dir.exists() {
        result.add_clean("Knowledge");
        return;
    }

    // Count files before deletion
    let file_count = match fs::read_dir(knowledge_dir) {
        Ok(entries) => entries.count(),
        Err(_) => 0,
    };

    if file_count == 0 {
        result.add_clean("Knowledge");
        return;
    }

    // Remove all contents (but keep the directory)
    match fs::read_dir(knowledge_dir) {
        Ok(entries) => {
            let mut removed = 0;
            let mut errors = 0;

            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if fs::remove_file(&path).is_ok() {
                        removed += 1;
                    } else {
                        errors += 1;
                    }
                } else if path.is_dir() {
                    if fs::remove_dir_all(&path).is_ok() {
                        removed += 1;
                    } else {
                        errors += 1;
                    }
                }
            }

            if errors > 0 {
                result.add_error(&format!(
                    "Failed to remove {} items in knowledge directory",
                    errors
                ));
            }
            if removed > 0 {
                result.add_reset(&format!("Knowledge ({} items)", removed));
            }
        }
        Err(e) => {
            result.add_error(&format!("Failed to read knowledge directory: {}", e));
        }
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Reset experience using default system paths
pub fn reset_experience_default() -> ExperienceResetResult {
    reset_experience(&ExperiencePaths::default())
}

/// Factory reset using default system paths
pub fn reset_factory_default() -> ExperienceResetResult {
    reset_factory(&ExperiencePaths::default())
}

/// Check if experience data exists (for status display)
pub fn has_experience_data(paths: &ExperiencePaths) -> bool {
    // Check XP store - if it exists and has non-baseline data
    if paths.xp_store_file().exists() {
        if let Ok(content) = fs::read_to_string(paths.xp_store_file()) {
            // Has experience if total_questions > 0 or xp > 0
            if content.contains("\"total_questions\": 0") && content.contains("\"xp\": 0") {
                // At baseline, no real experience
            } else {
                return true;
            }
        }
    }

    // Check telemetry
    if paths.telemetry_file.exists() {
        if let Ok(meta) = fs::metadata(&paths.telemetry_file) {
            if meta.len() > 0 {
                return true;
            }
        }
    }

    // Check stats directory
    if paths.stats_dir.exists() {
        if let Ok(entries) = fs::read_dir(&paths.stats_dir) {
            if entries.count() > 0 {
                return true;
            }
        }
    }

    false
}

/// Check if knowledge data exists (for factory reset warning)
/// Only counts files and non-stats directories to avoid false positives from
/// the stats subdirectory which is always created.
pub fn has_knowledge_data(paths: &ExperiencePaths) -> bool {
    if !paths.knowledge_dir.exists() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(&paths.knowledge_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            // Skip the stats subdirectory (it's managed separately)
            if path == paths.stats_dir {
                continue;
            }
            // Any other file or directory counts as knowledge data
            return true;
        }
    }
    false
}

/// Get a summary of current experience data (for pre-reset snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceSnapshot {
    pub xp_store_exists: bool,
    pub xp_store_size_bytes: u64,
    pub anna_level: u8,
    pub anna_xp: u64,
    pub total_questions: u64,
    pub telemetry_exists: bool,
    pub telemetry_size_bytes: u64,
    pub telemetry_line_count: u64,
    pub stats_file_count: u64,
    pub knowledge_file_count: u64,
}

impl ExperienceSnapshot {
    pub fn capture(paths: &ExperiencePaths) -> Self {
        let xp_file = paths.xp_store_file();
        let xp_store_exists = xp_file.exists();
        let xp_store_size_bytes = fs::metadata(&xp_file).map(|m| m.len()).unwrap_or(0);

        // Parse XP values
        let (anna_level, anna_xp, total_questions) = if xp_store_exists {
            parse_xp_summary(&xp_file)
        } else {
            (1, 0, 0)
        };

        let telemetry_exists = paths.telemetry_file.exists();
        let telemetry_size_bytes = fs::metadata(&paths.telemetry_file)
            .map(|m| m.len())
            .unwrap_or(0);
        let telemetry_line_count = if telemetry_exists {
            fs::read_to_string(&paths.telemetry_file)
                .map(|s| s.lines().count() as u64)
                .unwrap_or(0)
        } else {
            0
        };

        let stats_file_count = if paths.stats_dir.exists() {
            fs::read_dir(&paths.stats_dir)
                .map(|entries| entries.count() as u64)
                .unwrap_or(0)
        } else {
            0
        };

        // Count knowledge files, excluding the stats subdirectory
        let knowledge_file_count = if paths.knowledge_dir.exists() {
            fs::read_dir(&paths.knowledge_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path() != paths.stats_dir)
                        .count() as u64
                })
                .unwrap_or(0)
        } else {
            0
        };

        Self {
            xp_store_exists,
            xp_store_size_bytes,
            anna_level,
            anna_xp,
            total_questions,
            telemetry_exists,
            telemetry_size_bytes,
            telemetry_line_count,
            stats_file_count,
            knowledge_file_count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anna_level == 1
            && self.anna_xp == 0
            && self.total_questions == 0
            && self.telemetry_line_count == 0
            && self.stats_file_count == 0
    }

    pub fn is_factory_clean(&self) -> bool {
        self.is_empty() && self.knowledge_file_count == 0
    }
}

/// Parse XP summary from store file
fn parse_xp_summary(path: &Path) -> (u8, u64, u64) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (1, 0, 0),
    };

    // Simple JSON parsing without full deserialize
    let level = extract_json_u8(&content, "anna", "level").unwrap_or(1);
    let xp = extract_json_u64(&content, "anna", "xp").unwrap_or(0);
    let total = extract_json_u64(&content, "anna_stats", "total_questions").unwrap_or(0);

    (level, xp, total)
}

/// Extract a u8 value from nested JSON
fn extract_json_u8(content: &str, section: &str, key: &str) -> Option<u8> {
    // Find section
    let section_start = content.find(&format!("\"{}\"", section))?;
    let section_content = &content[section_start..];

    // Find key within reasonable range - try with and without space after colon
    let key_pattern_space = format!("\"{}\": ", key);
    let key_pattern_no_space = format!("\"{}\":", key);

    let (key_start, pattern_len) = if let Some(pos) = section_content.find(&key_pattern_space) {
        (pos, key_pattern_space.len())
    } else if let Some(pos) = section_content.find(&key_pattern_no_space) {
        (pos, key_pattern_no_space.len())
    } else {
        return None;
    };

    let value_start = key_start + pattern_len;

    // Extract number
    let value_content = &section_content[value_start..];
    let end = value_content.find(|c: char| !c.is_ascii_digit())?;
    if end == 0 {
        return None;
    }
    value_content[..end].parse().ok()
}

/// Extract a u64 value from nested JSON
fn extract_json_u64(content: &str, section: &str, key: &str) -> Option<u64> {
    let section_start = content.find(&format!("\"{}\"", section))?;
    let section_content = &content[section_start..];

    // Try with and without space after colon
    let key_pattern_space = format!("\"{}\": ", key);
    let key_pattern_no_space = format!("\"{}\":", key);

    let (key_start, pattern_len) = if let Some(pos) = section_content.find(&key_pattern_space) {
        (pos, key_pattern_space.len())
    } else if let Some(pos) = section_content.find(&key_pattern_no_space) {
        (pos, key_pattern_no_space.len())
    } else {
        return None;
    };

    let value_start = key_start + pattern_len;

    let value_content = &section_content[value_start..];
    let end = value_content.find(|c: char| !c.is_ascii_digit())?;
    if end == 0 {
        return None;
    }
    value_content[..end].parse().ok()
}

// ============================================================================
// Test Helper (v1.2.0)
// ============================================================================

/// Reset experience for tests using a custom root directory
pub fn reset_experience_for_tests(root: &Path) -> ExperienceResetResult {
    let paths = ExperiencePaths::with_root(root);

    // Create directories if they don't exist (for test setup)
    let _ = fs::create_dir_all(&paths.xp_dir);
    let _ = fs::create_dir_all(paths.telemetry_file.parent().unwrap_or(Path::new("/")));
    let _ = fs::create_dir_all(&paths.stats_dir);
    let _ = fs::create_dir_all(&paths.knowledge_dir);

    reset_experience(&paths)
}

/// Factory reset for tests using a custom root directory
pub fn reset_factory_for_tests(root: &Path) -> ExperienceResetResult {
    let paths = ExperiencePaths::with_root(root);

    let _ = fs::create_dir_all(&paths.xp_dir);
    let _ = fs::create_dir_all(paths.telemetry_file.parent().unwrap_or(Path::new("/")));
    let _ = fs::create_dir_all(&paths.stats_dir);
    let _ = fs::create_dir_all(&paths.knowledge_dir);

    reset_factory(&paths)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_paths() -> (TempDir, ExperiencePaths) {
        let temp = TempDir::new().unwrap();
        let paths = ExperiencePaths::with_root(temp.path());

        // Create directories
        fs::create_dir_all(&paths.xp_dir).unwrap();
        fs::create_dir_all(paths.telemetry_file.parent().unwrap()).unwrap();
        fs::create_dir_all(&paths.stats_dir).unwrap();
        fs::create_dir_all(&paths.knowledge_dir).unwrap();

        (temp, paths)
    }

    #[test]
    fn test_reset_type_confirmation() {
        // Experience reset accepts "yes"
        assert!(ResetType::Experience.is_confirmed("yes"));
        assert!(ResetType::Experience.is_confirmed("YES"));
        assert!(ResetType::Experience.is_confirmed("  yes  "));
        assert!(!ResetType::Experience.is_confirmed("no"));

        // Factory reset requires exact phrase
        let exact = "I UNDERSTAND AND CONFIRM FACTORY RESET";
        assert!(ResetType::Factory.is_confirmed(exact));
        assert!(!ResetType::Factory.is_confirmed("yes"));
        assert!(!ResetType::Factory.is_confirmed("i understand and confirm factory reset"));
    }

    #[test]
    fn test_experience_reset_creates_baseline() {
        let (_temp, paths) = setup_test_paths();

        // No XP store exists initially
        assert!(!paths.xp_store_file().exists());

        // Reset creates baseline
        let result = reset_experience(&paths);
        assert!(result.success);

        // XP store now exists with baseline values
        assert!(paths.xp_store_file().exists());
        let content = fs::read_to_string(paths.xp_store_file()).unwrap();
        assert!(content.contains("\"level\": 1"));
        assert!(content.contains("\"trust\": 0.5"));
        assert!(content.contains("\"xp\": 0"));
    }

    #[test]
    fn test_experience_reset_resets_to_baseline() {
        let (_temp, paths) = setup_test_paths();

        // Create XP store with non-baseline values
        let advanced_xp = r#"{"anna":{"name":"Anna","level":10,"xp":5000,"trust":0.9}}"#;
        fs::write(paths.xp_store_file(), advanced_xp).unwrap();

        // Reset should overwrite with baseline
        let result = reset_experience(&paths);
        assert!(result.success);
        assert!(result.components_reset.iter().any(|c| c.contains("XP")));

        let content = fs::read_to_string(paths.xp_store_file()).unwrap();
        assert!(content.contains("\"level\": 1"));
        assert!(content.contains("\"trust\": 0.5"));
    }

    #[test]
    fn test_experience_reset_preserves_knowledge() {
        let (_temp, paths) = setup_test_paths();

        // Create knowledge file
        let knowledge_file = paths.knowledge_dir.join("learned.db");
        fs::write(&knowledge_file, "learned data").unwrap();

        // Experience reset preserves knowledge
        let result = reset_experience(&paths);
        assert!(result.success);
        assert!(knowledge_file.exists());
        assert_eq!(fs::read_to_string(&knowledge_file).unwrap(), "learned data");
    }

    #[test]
    fn test_factory_reset_deletes_knowledge() {
        let (_temp, paths) = setup_test_paths();

        // Create knowledge file
        let knowledge_file = paths.knowledge_dir.join("learned.db");
        fs::write(&knowledge_file, "learned data").unwrap();

        // Factory reset deletes knowledge
        let result = reset_factory(&paths);
        assert!(result.success);
        assert!(result.reset_type == ResetType::Factory);
        assert!(result.components_reset.iter().any(|c| c.contains("Knowledge")));
        assert!(!knowledge_file.exists());
    }

    #[test]
    fn test_factory_reset_creates_baseline_xp() {
        let (_temp, paths) = setup_test_paths();

        // Create XP store with advanced values
        let advanced_xp = r#"{"anna":{"name":"Anna","level":50,"xp":50000}}"#;
        fs::write(paths.xp_store_file(), advanced_xp).unwrap();

        // Factory reset should reset to baseline
        let result = reset_factory(&paths);
        assert!(result.success);

        let content = fs::read_to_string(paths.xp_store_file()).unwrap();
        assert!(content.contains("\"level\": 1"));
    }

    #[test]
    fn test_reset_telemetry() {
        let (_temp, paths) = setup_test_paths();

        // Create telemetry with content
        let telemetry = r#"{"timestamp":"2024-01-01","outcome":"success"}"#;
        fs::write(&paths.telemetry_file, telemetry).unwrap();

        let result = reset_experience(&paths);
        assert!(result.success);
        assert!(result.components_reset.iter().any(|c| c.contains("Telemetry")));

        // File should exist but be empty
        assert!(paths.telemetry_file.exists());
        assert_eq!(fs::read_to_string(&paths.telemetry_file).unwrap(), "");
    }

    #[test]
    fn test_reset_stats_directory() {
        let (_temp, paths) = setup_test_paths();

        // Create stats files
        fs::write(paths.stats_dir.join("metrics.json"), "{}").unwrap();
        fs::write(paths.stats_dir.join("events.jsonl"), "event1").unwrap();

        let result = reset_experience(&paths);
        assert!(result.success);
        assert!(result.components_reset.iter().any(|c| c.contains("Stats")));

        // Directory should exist but be empty
        assert!(paths.stats_dir.exists());
        assert_eq!(fs::read_dir(&paths.stats_dir).unwrap().count(), 0);
    }

    #[test]
    fn test_reset_idempotent() {
        let (_temp, paths) = setup_test_paths();

        // Create content
        fs::write(paths.xp_store_file(), r#"{"anna":{"level":5}}"#).unwrap();
        fs::write(&paths.telemetry_file, "line1\nline2").unwrap();

        // First reset
        let result1 = reset_experience(&paths);
        assert!(result1.success);
        assert!(!result1.components_reset.is_empty());

        // Second reset - should show components as clean
        let result2 = reset_experience(&paths);
        assert!(result2.success);
        // XP should be clean (baseline already written)
        // Telemetry should be clean (already empty)
    }

    #[test]
    fn test_experience_snapshot() {
        let (_temp, paths) = setup_test_paths();

        // Empty state
        let snapshot = ExperienceSnapshot::capture(&paths);
        assert!(snapshot.is_empty());

        // Add content
        let xp_data = r#"{"anna":{"name":"Anna","level":5,"xp":500},"anna_stats":{"total_questions":10}}"#;
        fs::write(paths.xp_store_file(), xp_data).unwrap();
        fs::write(&paths.telemetry_file, "line1\nline2\nline3").unwrap();
        fs::write(paths.stats_dir.join("file1.json"), "{}").unwrap();
        fs::write(paths.knowledge_dir.join("learned.db"), "data").unwrap();

        let snapshot = ExperienceSnapshot::capture(&paths);
        assert!(!snapshot.is_empty());
        assert!(snapshot.xp_store_exists);
        assert_eq!(snapshot.anna_level, 5);
        assert_eq!(snapshot.anna_xp, 500);
        assert_eq!(snapshot.total_questions, 10);
        assert_eq!(snapshot.telemetry_line_count, 3);
        assert_eq!(snapshot.stats_file_count, 1);
        assert_eq!(snapshot.knowledge_file_count, 1);
    }

    #[test]
    fn test_has_experience_data() {
        let (_temp, paths) = setup_test_paths();

        // Empty state - write baseline
        fs::write(paths.xp_store_file(), baseline_xp_store_json()).unwrap();
        assert!(!has_experience_data(&paths)); // Baseline = no experience

        // Add non-baseline XP
        let xp_data = r#"{"anna":{"level":2,"xp":100},"anna_stats":{"total_questions":5}}"#;
        fs::write(paths.xp_store_file(), xp_data).unwrap();
        assert!(has_experience_data(&paths));
    }

    #[test]
    fn test_has_knowledge_data() {
        let (_temp, paths) = setup_test_paths();

        assert!(!has_knowledge_data(&paths));

        fs::write(paths.knowledge_dir.join("data.db"), "data").unwrap();
        assert!(has_knowledge_data(&paths));
    }

    #[test]
    fn test_baseline_xp_store_json_valid() {
        let json = baseline_xp_store_json();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Check structure
        assert!(parsed.get("anna").is_some());
        assert!(parsed.get("junior").is_some());
        assert!(parsed.get("senior").is_some());
        assert!(parsed.get("anna_stats").is_some());

        // Check baseline values
        assert_eq!(parsed["anna"]["level"], 1);
        assert_eq!(parsed["anna"]["xp"], 0);
        assert_eq!(parsed["anna"]["trust"], 0.5);
    }
}
