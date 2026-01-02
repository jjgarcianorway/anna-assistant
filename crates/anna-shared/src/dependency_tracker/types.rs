//! Dependency types and records

use serde::{Deserialize, Serialize};

/// Dependency type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DependencyType {
    #[default]
    Runtime,
    Build,
    Optional,
    Recommended,
    Suggested,
    Conflict,
}

impl DependencyType {
    pub fn name(&self) -> &'static str {
        match self {
            DependencyType::Runtime => "Runtime",
            DependencyType::Build => "Build",
            DependencyType::Optional => "Optional",
            DependencyType::Recommended => "Recommended",
            DependencyType::Suggested => "Suggested",
            DependencyType::Conflict => "Conflict",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            DependencyType::Runtime => "→",
            DependencyType::Build => "⚙",
            DependencyType::Optional => "?",
            DependencyType::Recommended => "+",
            DependencyType::Suggested => "~",
            DependencyType::Conflict => "!",
        }
    }
}

/// Dependency status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DependencyStatus {
    #[default]
    Installed,
    Missing,
    Outdated,
    Orphaned,
    Unknown,
}

impl DependencyStatus {
    pub fn name(&self) -> &'static str {
        match self {
            DependencyStatus::Installed => "Installed",
            DependencyStatus::Missing => "Missing",
            DependencyStatus::Outdated => "Outdated",
            DependencyStatus::Orphaned => "Orphaned",
            DependencyStatus::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            DependencyStatus::Installed => "✓",
            DependencyStatus::Missing => "✗",
            DependencyStatus::Outdated => "↑",
            DependencyStatus::Orphaned => "○",
            DependencyStatus::Unknown => "?",
        }
    }
}

/// A dependency record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRecord {
    /// Package name
    pub package: String,
    /// Dependency name
    pub dependency: String,
    /// Type of dependency
    pub dep_type: DependencyType,
    /// Current status
    pub status: DependencyStatus,
    /// Required version (if any)
    pub version_req: Option<String>,
    /// Installed version (if any)
    pub installed_version: Option<String>,
    /// When last checked
    pub last_check: u64,
}
