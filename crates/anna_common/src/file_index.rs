//! File-Level Indexing - Track every file on the system
//!
//! Beta.84: Comprehensive file awareness system
//!
//! Privacy-aware design:
//! - Default: Index system directories only (/var, /etc, /usr, /opt)
//! - Opt-in: Home directory indexing (/home)
//! - Configurable: User can specify additional paths to index
//!
//! Features:
//! - Track file path, size, mtime, owner, permissions
//! - Detect large files, rapid growth, permission changes
//! - Fast search using SQLite FTS5
//! - Incremental updates (only scan changed files)

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// File index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub mtime: DateTime<Utc>,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub permissions: u32,
    pub file_type: FileType,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    Socket,
    BlockDevice,
    CharDevice,
    Fifo,
    Unknown,
}

impl FileType {
    pub fn from_metadata(metadata: &fs::Metadata) -> Self {
        use std::os::unix::fs::FileTypeExt;
        let ft = metadata.file_type();

        if ft.is_file() {
            FileType::Regular
        } else if ft.is_dir() {
            FileType::Directory
        } else if ft.is_symlink() {
            FileType::Symlink
        } else if ft.is_block_device() {
            FileType::BlockDevice
        } else if ft.is_char_device() {
            FileType::CharDevice
        } else if ft.is_fifo() {
            FileType::Fifo
        } else if ft.is_socket() {
            FileType::Socket
        } else {
            FileType::Unknown
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            FileType::Regular => "file",
            FileType::Directory => "dir",
            FileType::Symlink => "link",
            FileType::Socket => "socket",
            FileType::BlockDevice => "block",
            FileType::CharDevice => "char",
            FileType::Fifo => "fifo",
            FileType::Unknown => "unknown",
        }
    }
}

/// File indexing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndexConfig {
    /// Enable file indexing
    pub enabled: bool,

    /// Index home directories (opt-in for privacy)
    pub index_home: bool,

    /// System directories to index (always indexed if enabled)
    pub system_paths: Vec<PathBuf>,

    /// Additional custom paths to index
    pub custom_paths: Vec<PathBuf>,

    /// Paths to exclude from indexing
    pub exclude_paths: Vec<PathBuf>,

    /// Minimum file size to track (bytes) - ignore tiny files
    pub min_file_size: u64,

    /// Maximum file size to track (bytes) - ignore huge files like ISOs
    pub max_file_size: u64,
}

impl Default for FileIndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            index_home: false, // Privacy: opt-in only
            system_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/var"),
                PathBuf::from("/usr/local"),
                PathBuf::from("/opt"),
            ],
            custom_paths: Vec::new(),
            exclude_paths: vec![
                PathBuf::from("/proc"),
                PathBuf::from("/sys"),
                PathBuf::from("/dev"),
                PathBuf::from("/run"),
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
                PathBuf::from("/var/cache"),
                PathBuf::from("/var/lib/docker"), // Huge, changes frequently
                PathBuf::from("/var/lib/systemd"),
            ],
            min_file_size: 0,              // Track all files by default
            max_file_size: 10_000_000_000, // 10GB max
        }
    }
}

impl FileIndexConfig {
    /// Load configuration from file
    pub fn load() -> Self {
        // Try to load from ~/.config/anna/file_index.toml
        if let Some(path) = Self::user_config_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::user_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn user_config_path() -> Option<PathBuf> {
        let config_dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else {
            let home = std::env::var("HOME").ok()?;
            PathBuf::from(home).join(".config")
        };

        Some(config_dir.join("anna").join("file_index.toml"))
    }

    /// Get all paths to index
    pub fn get_paths_to_index(&self) -> Vec<PathBuf> {
        let mut paths = self.system_paths.clone();

        if self.index_home {
            if let Ok(home) = std::env::var("HOME") {
                paths.push(PathBuf::from(home));
            }
        }

        paths.extend(self.custom_paths.clone());
        paths
    }

    /// Check if a path should be indexed
    pub fn should_index(&self, path: &Path) -> bool {
        // Check if path is excluded
        for exclude in &self.exclude_paths {
            if path.starts_with(exclude) {
                return false;
            }
        }

        // Check if path is in indexed paths
        let paths_to_index = self.get_paths_to_index();
        for index_path in &paths_to_index {
            if path.starts_with(index_path) {
                return true;
            }
        }

        false
    }
}

/// File indexer
pub struct FileIndexer {
    config: FileIndexConfig,
}

impl FileIndexer {
    pub fn new(config: FileIndexConfig) -> Self {
        Self { config }
    }

    /// Scan a directory and return file entries
    pub fn scan_directory(&self, path: &Path) -> Result<Vec<FileEntry>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let now = Utc::now();

        info!("Scanning directory for file index: {}", path.display());

        for entry in WalkDir::new(path)
            .follow_links(false) // Don't follow symlinks to avoid loops
            .into_iter()
            .filter_entry(|e| {
                // Filter out excluded paths early
                !self.is_excluded(e.path())
            })
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip if not in indexed paths
                    if !self.config.should_index(path) {
                        continue;
                    }

                    match self.index_file(path, now) {
                        Ok(Some(file_entry)) => entries.push(file_entry),
                        Ok(None) => {} // Skipped (e.g., too small/large)
                        Err(e) => {
                            debug!("Failed to index {}: {}", path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    debug!("Error walking directory: {}", e);
                }
            }
        }

        info!("Scanned {} files from {}", entries.len(), path.display());
        Ok(entries)
    }

    /// Index a single file
    fn index_file(&self, path: &Path, indexed_at: DateTime<Utc>) -> Result<Option<FileEntry>> {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                debug!("Cannot read metadata for {}: {}", path.display(), e);
                return Ok(None);
            }
        };

        let size_bytes = metadata.len();

        // Skip files outside size range
        if size_bytes < self.config.min_file_size || size_bytes > self.config.max_file_size {
            return Ok(None);
        }

        let mtime = metadata.modified()?;
        let mtime_chrono: DateTime<Utc> = mtime.into();

        // Get owner UID/GID
        #[cfg(unix)]
        let (owner_uid, owner_gid) = {
            use std::os::unix::fs::MetadataExt;
            (metadata.uid(), metadata.gid())
        };

        #[cfg(not(unix))]
        let (owner_uid, owner_gid) = (0, 0);

        let permissions = metadata.permissions().mode();
        let file_type = FileType::from_metadata(&metadata);

        Ok(Some(FileEntry {
            path: path.to_string_lossy().to_string(),
            size_bytes,
            mtime: mtime_chrono,
            owner_uid,
            owner_gid,
            permissions,
            file_type,
            indexed_at,
        }))
    }

    /// Check if path should be excluded
    fn is_excluded(&self, path: &Path) -> bool {
        for exclude in &self.config.exclude_paths {
            if path.starts_with(exclude) {
                return true;
            }
        }
        false
    }

    /// Scan all configured paths
    pub fn scan_all(&self) -> Result<Vec<FileEntry>> {
        let mut all_entries = Vec::new();

        for path in self.config.get_paths_to_index() {
            if !path.exists() {
                warn!("Skipping non-existent path: {}", path.display());
                continue;
            }

            match self.scan_directory(&path) {
                Ok(entries) => {
                    all_entries.extend(entries);
                }
                Err(e) => {
                    warn!("Failed to scan {}: {}", path.display(), e);
                }
            }
        }

        Ok(all_entries)
    }
}

/// File change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
    pub old_mtime: Option<DateTime<Utc>>,
    pub new_mtime: Option<DateTime<Utc>>,
    pub old_permissions: Option<u32>,
    pub new_permissions: Option<u32>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    PermissionsChanged,
    OwnerChanged,
}

impl ChangeType {
    pub fn as_str(&self) -> &str {
        match self {
            ChangeType::Created => "created",
            ChangeType::Modified => "modified",
            ChangeType::Deleted => "deleted",
            ChangeType::PermissionsChanged => "permissions",
            ChangeType::OwnerChanged => "owner",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileIndexConfig::default();
        assert!(config.enabled);
        assert!(!config.index_home); // Privacy: off by default
        assert!(!config.system_paths.is_empty());
        assert!(config.exclude_paths.contains(&PathBuf::from("/proc")));
    }

    #[test]
    fn test_should_index() {
        let config = FileIndexConfig::default();

        // Should index system paths
        assert!(config.should_index(Path::new("/etc/fstab")));
        assert!(config.should_index(Path::new("/var/log/messages")));

        // Should not index excluded paths
        assert!(!config.should_index(Path::new("/proc/cpuinfo")));
        assert!(!config.should_index(Path::new("/sys/class/net")));

        // Should not index home by default
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home).join("test.txt");
            assert!(!config.should_index(&home_path));
        }
    }

    #[test]
    fn test_file_type_detection() {
        let temp_file = std::env::temp_dir().join("test_file_index.txt");
        std::fs::write(&temp_file, "test").unwrap();

        let metadata = std::fs::metadata(&temp_file).unwrap();
        let file_type = FileType::from_metadata(&metadata);

        assert_eq!(file_type, FileType::Regular);

        std::fs::remove_file(&temp_file).ok();
    }
}
