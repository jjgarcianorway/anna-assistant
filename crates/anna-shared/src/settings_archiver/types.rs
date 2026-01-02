// v0.0.658: Settings Archiver Types (Phase 234)
// Core types and enums for settings archiver

use serde::{Deserialize, Serialize};

/// Archive format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArchiveFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// Binary format
    Binary,
    /// Compressed format
    Compressed,
}

impl std::fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Binary => write!(f, "binary"),
            Self::Compressed => write!(f, "compressed"),
        }
    }
}

/// Archive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArchiveType {
    /// Full backup
    #[default]
    Full,
    /// Incremental backup
    Incremental,
    /// Differential backup
    Differential,
    /// Snapshot
    Snapshot,
}

impl std::fmt::Display for ArchiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Incremental => write!(f, "incremental"),
            Self::Differential => write!(f, "differential"),
            Self::Snapshot => write!(f, "snapshot"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format_display() {
        assert_eq!(format!("{}", ArchiveFormat::Json), "json");
        assert_eq!(format!("{}", ArchiveFormat::Toml), "toml");
    }

    #[test]
    fn test_archive_type_display() {
        assert_eq!(format!("{}", ArchiveType::Full), "full");
        assert_eq!(format!("{}", ArchiveType::Incremental), "incremental");
    }
}
