//! Version information for Anna components.
//! v0.0.73: Single source of truth for version, git SHA, and build date.
//!
//! This module provides compile-time version information that is consistent
//! across all Anna binaries (annactl, annad).
//!
//! # Build-time injection
//! The build.rs in anna-shared sets these environment variables:
//! - `ANNA_VERSION`: From Cargo package version
//! - `ANNA_GIT_SHA`: From `git rev-parse --short HEAD`
//! - `ANNA_BUILD_DATE`: UTC ISO timestamp
//!
//! # Usage
//! ```ignore
//! use anna_shared::version::{VERSION, GIT_SHA, BUILD_DATE, full_version_string};
//! println!("Anna v{} ({})", VERSION, GIT_SHA);
//! ```

use serde::{Deserialize, Serialize};

/// Anna version from Cargo.toml (single source of truth)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Git commit SHA (short form, set at build time)
/// Falls back to "unknown" if not available (e.g., not in git repo)
pub const GIT_SHA: &str = match option_env!("ANNA_GIT_SHA") {
    Some(sha) => sha,
    None => "unknown",
};

/// Build date in UTC ISO format (set at build time)
/// Falls back to "unknown" if not available
pub const BUILD_DATE: &str = match option_env!("ANNA_BUILD_DATE") {
    Some(date) => date,
    None => "unknown",
};

/// RPC protocol version for client/daemon compatibility
/// Increment this when RPC interface changes in incompatible ways
pub const PROTOCOL_VERSION: u32 = 2;

/// Full version string for display (e.g., "0.0.73 (abc1234)")
pub fn full_version_string() -> String {
    if GIT_SHA == "unknown" {
        VERSION.to_string()
    } else {
        format!("{} ({})", VERSION, GIT_SHA)
    }
}

/// Version info struct for RPC responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionInfo {
    /// Semantic version (X.Y.Z)
    pub version: String,
    /// Git commit SHA (short form)
    pub git_sha: String,
    /// Build date in UTC ISO format
    pub build_date: String,
    /// RPC protocol version
    pub protocol_version: u32,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::current()
    }
}

impl VersionInfo {
    /// Get current build's version info
    pub fn current() -> Self {
        Self {
            version: VERSION.to_string(),
            git_sha: GIT_SHA.to_string(),
            build_date: BUILD_DATE.to_string(),
            protocol_version: PROTOCOL_VERSION,
        }
    }

    /// Check if this version is compatible with another (same protocol version)
    pub fn is_compatible_with(&self, other: &VersionInfo) -> bool {
        self.protocol_version == other.protocol_version
    }

    /// Check if versions match exactly
    pub fn matches(&self, other: &VersionInfo) -> bool {
        self.version == other.version
    }

    /// Format for display in status output
    pub fn display_string(&self) -> String {
        if self.git_sha == "unknown" {
            self.version.clone()
        } else {
            format!("{} ({})", self.version, self.git_sha)
        }
    }
}

/// Compare two version strings semantically
/// Returns true if `remote` is newer than `current`
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let current_parts = parse(current);
    let remote_parts = parse(remote);

    // If either version is empty/invalid, don't report as newer
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
    fn test_version_is_semver() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "VERSION must be X.Y.Z format");
        for part in parts {
            assert!(part.parse::<u32>().is_ok(), "Each part must be a number");
        }
    }

    #[test]
    fn test_version_info_current() {
        let info = VersionInfo::current();
        assert_eq!(info.version, VERSION);
        assert_eq!(info.git_sha, GIT_SHA);
        assert_eq!(info.build_date, BUILD_DATE);
        assert_eq!(info.protocol_version, PROTOCOL_VERSION);
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = VersionInfo {
            version: "0.0.73".to_string(),
            git_sha: "abc1234".to_string(),
            build_date: "2025-12-06".to_string(),
            protocol_version: 2,
        };
        let v2 = VersionInfo {
            version: "0.0.74".to_string(),
            git_sha: "def5678".to_string(),
            build_date: "2025-12-07".to_string(),
            protocol_version: 2,
        };
        let v3 = VersionInfo {
            version: "0.1.0".to_string(),
            git_sha: "ghi9012".to_string(),
            build_date: "2025-12-08".to_string(),
            protocol_version: 3,
        };

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
        assert!(!v1.matches(&v2));
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.0.72", "0.0.73"));
        assert!(!is_newer_version("0.0.73", "0.0.73"));
        assert!(!is_newer_version("0.0.74", "0.0.73"));
        assert!(!is_newer_version("", "0.0.73"));
        assert!(!is_newer_version("0.0.73", ""));
        assert!(is_newer_version("0.0.9", "0.0.10"));
    }

    #[test]
    fn test_full_version_string() {
        let s = full_version_string();
        assert!(s.contains(VERSION));
    }
}
