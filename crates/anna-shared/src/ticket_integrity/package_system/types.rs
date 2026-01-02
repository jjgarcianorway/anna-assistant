//! Core types for package and system intents.

use serde::{Deserialize, Serialize};

/// System-level intents (NOT packages).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemIntent {
    /// "Do I have swap?" - Check /proc/swaps
    SwapConfigured,
    /// "How much swap?" - Swap size
    SwapSize,
    /// "Is trim enabled?" - Filesystem TRIM
    TrimEnabled,
    /// "Is firewall enabled?" - Firewall status
    FirewallEnabled,
}

/// Package-level intents.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageIntent {
    /// "Is nano installed?" - Check specific package
    CheckInstalled { package: String },
    /// "Can you install nano?" - Install package
    Install { package: String },
    /// "Do I have games?" - Search packages (vague)
    SearchByName { query: String },
}

/// Kind of swap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwapKind {
    /// Swap file (/swapfile).
    File,
    /// Swap partition.
    Partition,
    /// Zram.
    Zram,
    /// No swap.
    None,
}

/// Classify a question as system or package intent.
#[derive(Debug, Clone)]
pub enum QuestionClassification {
    /// System-level question.
    System(SystemIntent),
    /// Package-level question.
    Package(PackageIntent),
    /// Could be either, needs clarification.
    Ambiguous { question: String },
    /// Unknown/other.
    Unknown,
}
