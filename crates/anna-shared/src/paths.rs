//! System-wide paths for Anna.
//!
//! ARCHITECTURAL INVARIANT: Anna is a system-wide tool with ZERO state in user home directories.
//! All paths are under /etc/anna, /var/lib/anna, /run/anna, or /var/log/anna.
//!
//! This module is the SINGLE SOURCE OF TRUTH for all file paths in Anna.
//! No other code should construct paths using dirs::home_dir(), $HOME, ~/.anna, etc.

use std::path::PathBuf;

/// System-wide paths for Anna.
///
/// # Invariants
/// - Config: /etc/anna/
/// - State/Data: /var/lib/anna/
/// - Runtime: /run/anna/
/// - Logs: journald (or /var/log/anna/)
/// - Backups: /var/lib/anna/backups/
///
/// # Permissions
/// - anna:anna ownership on /var/lib/anna (mode 0750)
/// - Socket accessible to anna group members
#[derive(Debug, Clone)]
pub struct Paths {
    /// /etc/anna - configuration
    pub config_dir: PathBuf,
    /// /var/lib/anna - persistent state
    pub data_dir: PathBuf,
    /// /run/anna - runtime state
    pub run_dir: PathBuf,
    /// /var/log/anna - logs (if not using journald)
    pub log_dir: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Self::system()
    }
}

impl Paths {
    /// System-wide paths (default for production)
    pub fn system() -> Self {
        Self {
            config_dir: PathBuf::from("/etc/anna"),
            data_dir: PathBuf::from("/var/lib/anna"),
            run_dir: PathBuf::from("/run/anna"),
            log_dir: PathBuf::from("/var/log/anna"),
        }
    }

    /// Development/test paths (only used with ANNA_DEV_MODE=1)
    /// This is for running tests without root; NOT for production.
    #[cfg(any(test, feature = "dev-mode"))]
    pub fn dev() -> Self {
        let base = std::env::temp_dir().join("anna-dev");
        Self {
            config_dir: base.join("etc"),
            data_dir: base.join("var"),
            run_dir: base.join("run"),
            log_dir: base.join("log"),
        }
    }

    /// Get the appropriate paths based on environment
    pub fn get() -> Self {
        // Check for dev mode (testing only)
        #[cfg(any(test, feature = "dev-mode"))]
        if std::env::var("ANNA_DEV_MODE").is_ok() {
            return Self::dev();
        }

        Self::system()
    }

    // =========================================================================
    // Configuration paths
    // =========================================================================

    /// Main config file: /etc/anna/config.toml
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    // =========================================================================
    // Data paths (persistent state)
    // =========================================================================

    /// Stats file: /var/lib/anna/stats.json
    pub fn stats_file(&self) -> PathBuf {
        self.data_dir.join("stats.json")
    }

    /// Tickets file: /var/lib/anna/tickets.json
    pub fn tickets_file(&self) -> PathBuf {
        self.data_dir.join("tickets.json")
    }

    /// Memory file: /var/lib/anna/memory.json
    pub fn memory_file(&self) -> PathBuf {
        self.data_dir.join("memory.json")
    }

    /// Update ledger: /var/lib/anna/update_ledger.json
    pub fn update_ledger_file(&self) -> PathBuf {
        self.data_dir.join("update_ledger.json")
    }

    /// XP file: /var/lib/anna/xp.json
    pub fn xp_file(&self) -> PathBuf {
        self.data_dir.join("xp.json")
    }

    /// Fix history: /var/lib/anna/fix_history.json
    pub fn fix_history_file(&self) -> PathBuf {
        self.data_dir.join("fix_history.json")
    }

    /// Installed deps tracking: /var/lib/anna/installed_deps.txt
    pub fn installed_deps_file(&self) -> PathBuf {
        self.data_dir.join("installed_deps.txt")
    }

    /// Negative memory: /var/lib/anna/negative_memory.json
    pub fn negative_memory_file(&self) -> PathBuf {
        self.data_dir.join("negative_memory.json")
    }

    /// Skill promotions: /var/lib/anna/promotions.json
    pub fn promotions_file(&self) -> PathBuf {
        self.data_dir.join("promotions.json")
    }

    /// Changes tracking: /var/lib/anna/changes.json
    pub fn changes_file(&self) -> PathBuf {
        self.data_dir.join("changes.json")
    }

    /// System baseline: /var/lib/anna/baseline.json
    pub fn baseline_file(&self) -> PathBuf {
        self.data_dir.join("baseline.json")
    }

    /// Learning data: /var/lib/anna/learning.json
    pub fn learning_file(&self) -> PathBuf {
        self.data_dir.join("learning.json")
    }

    // =========================================================================
    // Subdirectories
    // =========================================================================

    /// Backups directory: /var/lib/anna/backups/
    pub fn backups_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }

    /// Recipes directory: /var/lib/anna/recipes/
    pub fn recipes_dir(&self) -> PathBuf {
        self.data_dir.join("recipes")
    }

    /// Experiments directory: /var/lib/anna/experiments/
    pub fn experiments_dir(&self) -> PathBuf {
        self.data_dir.join("experiments")
    }

    /// Wiki cache directory: /var/lib/anna/wiki/
    pub fn wiki_dir(&self) -> PathBuf {
        self.data_dir.join("wiki")
    }

    /// Wiki articles: /var/lib/anna/wiki/articles/
    pub fn wiki_articles_dir(&self) -> PathBuf {
        self.wiki_dir().join("articles")
    }

    /// Man page cache: /var/lib/anna/docs/man/
    pub fn man_cache_dir(&self) -> PathBuf {
        self.data_dir.join("docs").join("man")
    }

    /// Help output cache: /var/lib/anna/docs/help/
    pub fn help_cache_dir(&self) -> PathBuf {
        self.data_dir.join("docs").join("help")
    }

    /// Monitor issues: /var/lib/anna/monitor/
    pub fn monitor_dir(&self) -> PathBuf {
        self.data_dir.join("monitor")
    }

    /// Issues store: /var/lib/anna/monitor/issues.json
    pub fn issues_file(&self) -> PathBuf {
        self.monitor_dir().join("issues.json")
    }

    // =========================================================================
    // Runtime paths
    // =========================================================================

    /// Socket path: /run/anna/anna.sock
    pub fn socket_file(&self) -> PathBuf {
        self.run_dir.join("anna.sock")
    }

    /// PID file: /run/anna/annad.pid
    pub fn pid_file(&self) -> PathBuf {
        self.run_dir.join("annad.pid")
    }

    // =========================================================================
    // Migration support
    // =========================================================================

    /// Migration tombstone: /var/lib/anna/.migrated
    pub fn migration_tombstone(&self) -> PathBuf {
        self.data_dir.join(".migrated")
    }

    /// Check if migration has been completed
    pub fn is_migrated(&self) -> bool {
        self.migration_tombstone().exists()
    }

    // =========================================================================
    // Directory creation
    // =========================================================================

    /// Create all required directories with proper permissions
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        use std::fs;

        // Create main directories
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        fs::create_dir_all(&self.run_dir)?;
        fs::create_dir_all(&self.log_dir)?;

        // Create subdirectories
        fs::create_dir_all(self.backups_dir())?;
        fs::create_dir_all(self.recipes_dir())?;
        fs::create_dir_all(self.experiments_dir())?;
        fs::create_dir_all(self.wiki_articles_dir())?;
        fs::create_dir_all(self.man_cache_dir())?;
        fs::create_dir_all(self.help_cache_dir())?;
        fs::create_dir_all(self.monitor_dir())?;

        Ok(())
    }
}

// =============================================================================
// Global accessor (convenience)
// =============================================================================

use std::sync::OnceLock;

static PATHS: OnceLock<Paths> = OnceLock::new();

/// Get the global Paths instance
pub fn paths() -> &'static Paths {
    PATHS.get_or_init(Paths::get)
}

// =============================================================================
// Legacy path detection for migration
// =============================================================================

/// Detect legacy user-home paths that need migration
pub fn detect_legacy_paths() -> Vec<LegacyPath> {
    let mut legacy = Vec::new();

    // Check all users' home directories (requires root)
    if let Ok(entries) = std::fs::read_dir("/home") {
        for entry in entries.flatten() {
            let home = entry.path();

            // Check ~/.anna
            let anna_dir = home.join(".anna");
            if anna_dir.exists() {
                legacy.push(LegacyPath {
                    path: anna_dir,
                    kind: LegacyPathKind::DotAnna,
                    user: entry.file_name().to_string_lossy().to_string(),
                });
            }

            // Check ~/.local/share/anna
            let local_share = home.join(".local/share/anna");
            if local_share.exists() {
                legacy.push(LegacyPath {
                    path: local_share,
                    kind: LegacyPathKind::LocalShare,
                    user: entry.file_name().to_string_lossy().to_string(),
                });
            }
        }
    }

    // Also check /root
    let root_anna = PathBuf::from("/root/.anna");
    if root_anna.exists() {
        legacy.push(LegacyPath {
            path: root_anna,
            kind: LegacyPathKind::DotAnna,
            user: "root".to_string(),
        });
    }

    let root_local = PathBuf::from("/root/.local/share/anna");
    if root_local.exists() {
        legacy.push(LegacyPath {
            path: root_local,
            kind: LegacyPathKind::LocalShare,
            user: "root".to_string(),
        });
    }

    legacy
}

#[derive(Debug, Clone)]
pub struct LegacyPath {
    pub path: PathBuf,
    pub kind: LegacyPathKind,
    pub user: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LegacyPathKind {
    DotAnna,      // ~/.anna
    LocalShare,   // ~/.local/share/anna
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_paths_are_absolute() {
        let paths = Paths::system();
        assert!(paths.config_dir.is_absolute());
        assert!(paths.data_dir.is_absolute());
        assert!(paths.run_dir.is_absolute());
        assert!(paths.log_dir.is_absolute());
    }

    #[test]
    fn test_no_home_in_system_paths() {
        let paths = Paths::system();
        let home_indicators = ["~", "/home/", "/root/", ".local", ".anna"];

        let all_paths = [
            paths.config_dir.to_string_lossy().to_string(),
            paths.data_dir.to_string_lossy().to_string(),
            paths.run_dir.to_string_lossy().to_string(),
            paths.log_dir.to_string_lossy().to_string(),
            paths.stats_file().to_string_lossy().to_string(),
            paths.tickets_file().to_string_lossy().to_string(),
            paths.memory_file().to_string_lossy().to_string(),
            paths.update_ledger_file().to_string_lossy().to_string(),
            paths.socket_file().to_string_lossy().to_string(),
            paths.backups_dir().to_string_lossy().to_string(),
        ];

        for path in &all_paths {
            for indicator in &home_indicators {
                assert!(
                    !path.contains(indicator),
                    "Path '{}' contains home indicator '{}'",
                    path,
                    indicator
                );
            }
        }
    }

    #[test]
    fn test_paths_use_correct_base_dirs() {
        let paths = Paths::system();

        assert_eq!(paths.config_dir, PathBuf::from("/etc/anna"));
        assert_eq!(paths.data_dir, PathBuf::from("/var/lib/anna"));
        assert_eq!(paths.run_dir, PathBuf::from("/run/anna"));
        assert_eq!(paths.log_dir, PathBuf::from("/var/log/anna"));
    }

    #[test]
    fn test_config_file_under_etc() {
        let paths = Paths::system();
        assert!(paths.config_file().starts_with("/etc/anna"));
    }

    #[test]
    fn test_data_files_under_var_lib() {
        let paths = Paths::system();
        assert!(paths.stats_file().starts_with("/var/lib/anna"));
        assert!(paths.tickets_file().starts_with("/var/lib/anna"));
        assert!(paths.memory_file().starts_with("/var/lib/anna"));
        assert!(paths.update_ledger_file().starts_with("/var/lib/anna"));
    }

    #[test]
    fn test_socket_under_run() {
        let paths = Paths::system();
        assert!(paths.socket_file().starts_with("/run/anna"));
    }

    #[test]
    fn test_backups_under_var_lib() {
        let paths = Paths::system();
        assert!(paths.backups_dir().starts_with("/var/lib/anna"));
    }
}
