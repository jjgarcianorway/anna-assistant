//! Version information types.
//! v0.3.21: Single source of truth from VERSION file, verified at build time.

use serde::{Deserialize, Serialize};

/// Build information embedded at compile time
pub struct BuildInfo {
    /// Version from VERSION file (verified to match Cargo.toml)
    pub version: &'static str,
    /// Git commit SHA (short)
    pub git_sha: &'static str,
    /// Whether there were uncommitted changes at build time
    pub git_dirty: bool,
    /// Build timestamp (RFC3339)
    pub build_time: &'static str,
}

impl BuildInfo {
    /// Get build info (embedded at compile time)
    pub fn get() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_sha: option_env!("ANNA_GIT_SHA").unwrap_or("unknown"),
            git_dirty: option_env!("ANNA_GIT_DIRTY") == Some("true"),
            build_time: option_env!("ANNA_BUILD_TIME").unwrap_or("unknown"),
        }
    }

    /// Format as display string
    pub fn display(&self) -> String {
        let dirty = if self.git_dirty { "-dirty" } else { "" };
        format!("{} ({}{}) built {}", self.version, self.git_sha, dirty, self.build_time)
    }

    /// Short version string
    pub fn short(&self) -> String {
        let dirty = if self.git_dirty { "*" } else { "" };
        format!("{}+{}{}", self.version, self.git_sha, dirty)
    }
}

/// Version information (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub git_sha: String,
    pub git_dirty: bool,
    pub build_time: String,
}

impl VersionInfo {
    /// Get current build's version info
    pub fn current() -> Self {
        let build = BuildInfo::get();
        Self {
            version: build.version.to_string(),
            git_sha: build.git_sha.to_string(),
            git_dirty: build.git_dirty,
            build_time: build.build_time.to_string(),
        }
    }

    /// Check if two versions match (same version string)
    pub fn matches(&self, other: &VersionInfo) -> bool {
        self.version == other.version
    }

    /// Format as display string
    pub fn display(&self) -> String {
        let dirty = if self.git_dirty { "-dirty" } else { "" };
        format!("{} ({}{}) built {}", self.version, self.git_sha, dirty, self.build_time)
    }
}

/// Compare versions, returns true if remote is newer
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let current_parts = parse(current);
    let remote_parts = parse(remote);

    if current_parts.is_empty() || remote_parts.is_empty() {
        return false;
    }

    for i in 0..3 {
        let c = current_parts.get(i).unwrap_or(&0);
        let r = remote_parts.get(i).unwrap_or(&0);
        if r > c {
            return true;
        }
        if r < c {
            return false;
        }
    }
    false
}

/// Read VERSION file at runtime (for verification)
pub fn read_version_file() -> Option<String> {
    // Try relative to binary first, then system locations
    // v0.3.31: Use system paths only
    let paths = [
        std::path::PathBuf::from("VERSION"),
        std::path::PathBuf::from("/usr/local/share/anna/VERSION"),
        crate::paths::paths().data_dir.join("VERSION"),
    ];

    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Some(content.trim().to_string());
        }
    }
    None
}

/// Verify runtime VERSION file matches compiled version
pub fn verify_version_integrity() -> Result<(), String> {
    let build = BuildInfo::get();
    if let Some(file_version) = read_version_file() {
        if file_version != build.version {
            return Err(format!(
                "Version mismatch: binary={}, VERSION file={}",
                build.version, file_version
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.0.1", "0.0.2"));
        assert!(is_newer_version("0.0.9", "0.1.0"));
        assert!(!is_newer_version("0.0.2", "0.0.1"));
        assert!(!is_newer_version("0.0.1", "0.0.1"));
        // Test with v prefix
        assert!(is_newer_version("v0.0.1", "v0.0.2"));
        assert!(is_newer_version("0.0.1", "v0.0.2"));
    }

    #[test]
    fn test_build_info() {
        let info = BuildInfo::get();
        assert!(!info.version.is_empty());
        assert!(!info.display().is_empty());
    }

    #[test]
    fn test_version_info() {
        let info = VersionInfo::current();
        assert!(!info.version.is_empty());
        assert!(info.matches(&info));
    }
}
