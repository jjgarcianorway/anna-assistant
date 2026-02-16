//! System-wide paths for Anna. SINGLE SOURCE OF TRUTH for all file paths.
//! ARCHITECTURAL INVARIANT: Zero state in user home directories.
//! All paths are under /etc/anna, /var/lib/anna, /run/anna, or /var/log/anna.

use std::path::PathBuf;

/// System-wide paths for Anna.
/// Config: /etc/anna/, State: /var/lib/anna/, Runtime: /run/anna/, Logs: /var/log/anna/
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

    // Configuration paths
    /// Main config file: /etc/anna/config.toml
    pub fn config_file(&self) -> PathBuf { self.config_dir.join("config.toml") }

    // Data paths (persistent state)
    pub fn stats_file(&self) -> PathBuf { self.data_dir.join("stats.json") }
    pub fn tickets_file(&self) -> PathBuf { self.data_dir.join("tickets.json") }
    pub fn memory_file(&self) -> PathBuf { self.data_dir.join("memory.json") }
    pub fn update_ledger_file(&self) -> PathBuf { self.data_dir.join("update_ledger.json") }
    pub fn xp_file(&self) -> PathBuf { self.data_dir.join("xp.json") }
    pub fn fix_history_file(&self) -> PathBuf { self.data_dir.join("fix_history.json") }
    pub fn installed_deps_file(&self) -> PathBuf { self.data_dir.join("installed_deps.txt") }
    pub fn negative_memory_file(&self) -> PathBuf { self.data_dir.join("negative_memory.json") }
    pub fn promotions_file(&self) -> PathBuf { self.data_dir.join("promotions.json") }
    pub fn changes_file(&self) -> PathBuf { self.data_dir.join("changes.json") }
    pub fn baseline_file(&self) -> PathBuf { self.data_dir.join("baseline.json") }
    pub fn learning_file(&self) -> PathBuf { self.data_dir.join("learning.json") }
    /// v0.3.56: Phase 23 outcomes ledger (append-only JSONL)
    pub fn outcomes_ledger_file(&self) -> PathBuf { self.data_dir.join("outcomes.jsonl") }

    // Subdirectories
    pub fn backups_dir(&self) -> PathBuf { self.data_dir.join("backups") }
    pub fn recipes_dir(&self) -> PathBuf { self.data_dir.join("recipes") }
    pub fn experiments_dir(&self) -> PathBuf { self.data_dir.join("experiments") }
    pub fn wiki_dir(&self) -> PathBuf { self.data_dir.join("wiki") }
    pub fn wiki_articles_dir(&self) -> PathBuf { self.wiki_dir().join("articles") }
    pub fn man_cache_dir(&self) -> PathBuf { self.data_dir.join("docs").join("man") }
    pub fn help_cache_dir(&self) -> PathBuf { self.data_dir.join("docs").join("help") }
    pub fn monitor_dir(&self) -> PathBuf { self.data_dir.join("monitor") }
    pub fn issues_file(&self) -> PathBuf { self.monitor_dir().join("issues.json") }

    // Runtime paths
    pub fn socket_file(&self) -> PathBuf { self.run_dir.join("anna.sock") }
    /// Executor socket: privileged RPC channel between annad and anna-executor.
    /// Owner: root:anna, mode: 0660 — only the anna service user can connect.
    pub fn executor_socket_file(&self) -> PathBuf { self.run_dir.join("anna-executor.sock") }
    pub fn pid_file(&self) -> PathBuf { self.run_dir.join("annad.pid") }

    // Migration support
    pub fn migration_tombstone(&self) -> PathBuf { self.data_dir.join(".migrated") }
    pub fn is_migrated(&self) -> bool { self.migration_tombstone().exists() }

    /// Create all required directories
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

use std::sync::OnceLock;

static PATHS: OnceLock<Paths> = OnceLock::new();

/// Get the global Paths instance
pub fn paths() -> &'static Paths { PATHS.get_or_init(Paths::get) }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_paths() {
        let p = Paths::system();
        // All paths are absolute
        assert!(p.config_dir.is_absolute() && p.data_dir.is_absolute());
        assert!(p.run_dir.is_absolute() && p.log_dir.is_absolute());
        // Correct base directories
        assert_eq!(p.config_dir, PathBuf::from("/etc/anna"));
        assert_eq!(p.data_dir, PathBuf::from("/var/lib/anna"));
        assert_eq!(p.run_dir, PathBuf::from("/run/anna"));
        assert_eq!(p.log_dir, PathBuf::from("/var/log/anna"));
        // Files under correct directories
        assert!(p.config_file().starts_with("/etc/anna"));
        assert!(p.stats_file().starts_with("/var/lib/anna"));
        assert!(p.socket_file().starts_with("/run/anna"));
        assert!(p.backups_dir().starts_with("/var/lib/anna"));
    }

    #[test]
    fn test_no_home_paths() {
        let p = Paths::system();
        let indicators = ["~", "/home/", "/root/", ".local", ".anna"];
        let stats = p.stats_file();
        let socket = p.socket_file();
        let paths = [
            p.config_dir.to_string_lossy(), p.data_dir.to_string_lossy(),
            stats.to_string_lossy(), socket.to_string_lossy(),
        ];
        for path in &paths {
            for ind in &indicators {
                assert!(!path.contains(ind), "Path '{}' contains '{}'", path, ind);
            }
        }
    }
}
