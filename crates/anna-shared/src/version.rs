//! Version information types.

use serde::{Deserialize, Serialize};

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub git_sha: String,
    pub build_date: String,
}

impl VersionInfo {
    pub fn current() -> Self {
        Self {
            version: crate::VERSION.to_string(),
            git_sha: env!("CARGO_PKG_VERSION").to_string(),
            build_date: "unknown".to_string(),
        }
    }

    /// Check if two versions match (same version string)
    pub fn matches(&self, other: &VersionInfo) -> bool {
        self.version == other.version
    }
}

/// Compare versions, returns true if remote is newer
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.0.1", "0.0.2"));
        assert!(is_newer_version("0.0.9", "0.1.0"));
        assert!(!is_newer_version("0.0.2", "0.0.1"));
        assert!(!is_newer_version("0.0.1", "0.0.1"));
    }
}
