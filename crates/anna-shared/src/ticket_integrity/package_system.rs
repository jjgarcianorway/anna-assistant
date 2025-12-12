//! Clean Package vs System Logic (Part 3) - v0.0.442.
//!
//! Stop confusing "package presence" with "system feature".
//!
//! WRONG: "do I have swap?" → "**swap** package is not installed"
//! RIGHT: "do I have swap?" → check /proc/swaps, report swap status
//!
//! Clear, separate intents:
//! - `system.swap_configured` - System swap memory
//! - `packages.check_installed` - Package installation
//! - `packages.search_by_name` - Search for packages

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

impl SystemIntent {
    /// Get probes for this intent.
    pub fn probes(&self) -> Vec<&'static str> {
        match self {
            Self::SwapConfigured | Self::SwapSize => vec!["cat /proc/swaps", "free -h"],
            Self::TrimEnabled => vec!["systemctl status fstrim.timer"],
            Self::FirewallEnabled => vec!["systemctl status firewalld", "systemctl status ufw"],
        }
    }

    /// Parse from question.
    pub fn from_question(question: &str) -> Option<Self> {
        let lower = question.to_lowercase();

        // Swap questions
        if (lower.contains("swap") || lower.contains("swapfile"))
            && (lower.contains("have") || lower.contains("enabled") || lower.contains("configured"))
        {
            return Some(Self::SwapConfigured);
        }
        if lower.contains("swap") && (lower.contains("how much") || lower.contains("size")) {
            return Some(Self::SwapSize);
        }

        // Trim questions
        if lower.contains("trim") && lower.contains("enabled") {
            return Some(Self::TrimEnabled);
        }

        // Firewall questions
        if lower.contains("firewall") && lower.contains("enabled") {
            return Some(Self::FirewallEnabled);
        }

        None
    }
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

impl PackageIntent {
    /// Get probe command for this intent.
    pub fn probe_command(&self) -> String {
        match self {
            Self::CheckInstalled { package } => {
                format!("pacman -Q {} 2>/dev/null || echo 'NOT_INSTALLED'", package)
            }
            Self::Install { package } => {
                format!("pacman -Si {} 2>/dev/null || echo 'NOT_FOUND'", package)
            }
            Self::SearchByName { query } => {
                format!("pacman -Ss {} 2>/dev/null | head -20", query)
            }
        }
    }

    /// Parse from question with entity extraction.
    pub fn from_question(question: &str, entity: Option<&str>) -> Option<Self> {
        let lower = question.to_lowercase();

        // "is X installed?"
        if lower.contains("installed") {
            if let Some(pkg) = entity.or_else(|| extract_package_name(&lower)) {
                return Some(Self::CheckInstalled {
                    package: pkg.to_string(),
                });
            }
        }

        // "can you install X?"
        if lower.contains("install") && !lower.contains("installed") {
            if let Some(pkg) = entity.or_else(|| extract_package_name(&lower)) {
                return Some(Self::Install {
                    package: pkg.to_string(),
                });
            }
        }

        // "do I have X?" with package context
        if lower.contains("do i have") || lower.contains("have i got") {
            if let Some(pkg) = entity.or_else(|| extract_package_name(&lower)) {
                // Check if this looks like a system feature vs package
                if !is_system_feature(&lower) && !is_vague_package_name(pkg) {
                    return Some(Self::CheckInstalled {
                        package: pkg.to_string(),
                    });
                }
            }
        }

        None
    }
}

/// Check if a package name is too vague (not a real package name).
fn is_vague_package_name(name: &str) -> bool {
    let vague_terms = [
        "games",
        "game",
        "apps",
        "app",
        "software",
        "programs",
        "program",
        "tools",
        "tool",
        "stuff",
        "things",
        "applications",
        "utilities",
        "anything",
        "something",
    ];
    vague_terms.contains(&name.to_lowercase().as_str())
}

/// Extract package name from question.
fn extract_package_name(question: &str) -> Option<&str> {
    // Common patterns: "is nano installed", "install vim", "have steam"
    let words: Vec<&str> = question.split_whitespace().collect();

    // Look for word after "is" and before "installed"
    if let Some(is_pos) = words.iter().position(|&w| w == "is") {
        if let Some(installed_pos) = words.iter().position(|&w| w == "installed") {
            if is_pos + 1 < installed_pos {
                return Some(words[is_pos + 1]);
            }
        }
    }

    // Look for word after "install"
    if let Some(install_pos) = words.iter().position(|&w| w == "install") {
        if install_pos + 1 < words.len() {
            return Some(words[install_pos + 1]);
        }
    }

    // Look for word after "have"
    if let Some(have_pos) = words.iter().position(|&w| w == "have") {
        if have_pos + 1 < words.len() {
            let candidate = words[have_pos + 1];
            // Filter out common non-package words
            if !["a", "an", "the", "any", "some", "i", "you"].contains(&candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

/// Check if a question is about a system feature (not a package).
fn is_system_feature(question: &str) -> bool {
    let system_features = [
        "swap",
        "swapfile",
        "trim",
        "firewall",
        "bluetooth",
        "wifi",
        "network",
        "audio",
        "sound",
        "graphics",
        "memory",
        "ram",
        "cpu",
        "disk",
        "space",
    ];

    system_features.iter().any(|f| question.contains(f))
}

/// Swap status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStatus {
    /// Does system have swap?
    pub has_swap: bool,
    /// Total swap in GiB.
    pub total_swap_gib: f64,
    /// Kind of swap.
    pub kind: SwapKind,
    /// Evidence (probe outputs).
    pub evidence: Vec<String>,
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

impl SwapStatus {
    /// Parse from /proc/swaps output.
    pub fn from_proc_swaps(output: &str) -> Self {
        let lines: Vec<&str> = output.lines().collect();

        // Skip header, check for swap entries
        if lines.len() <= 1 || output.trim().is_empty() {
            return Self {
                has_swap: false,
                total_swap_gib: 0.0,
                kind: SwapKind::None,
                evidence: vec!["/proc/swaps is empty".to_string()],
            };
        }

        let mut total_kb: u64 = 0;
        let mut kind = SwapKind::None;

        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0];
                let swap_type = parts[1];

                // Parse size (in KB)
                if let Ok(size) = parts[2].parse::<u64>() {
                    total_kb += size;
                }

                // Determine kind
                kind = if name.contains("zram") {
                    SwapKind::Zram
                } else if swap_type == "file" {
                    SwapKind::File
                } else {
                    SwapKind::Partition
                };
            }
        }

        let total_gib = total_kb as f64 / 1024.0 / 1024.0;

        Self {
            has_swap: total_kb > 0,
            total_swap_gib: total_gib,
            kind,
            evidence: vec![format!("From /proc/swaps: {} KB total", total_kb)],
        }
    }

    /// Format for user display.
    pub fn display(&self) -> String {
        if !self.has_swap {
            return "You have NO swap configured on this system.\n• evidence: /proc/swaps is empty".to_string();
        }

        let kind_str = match self.kind {
            SwapKind::File => "swapfile",
            SwapKind::Partition => "partition",
            SwapKind::Zram => "zram",
            SwapKind::None => "unknown",
        };

        format!(
            "Yes, you have swap: {:.1} GiB ({}).",
            self.total_swap_gib, kind_str
        )
    }
}

/// Package status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStatus {
    /// Package name.
    pub package: String,
    /// Is installed?
    pub installed: bool,
    /// Version if installed.
    pub version: Option<String>,
    /// Evidence (probe output).
    pub evidence: Vec<String>,
}

impl PackageStatus {
    /// Parse from pacman -Q output.
    pub fn from_pacman_output(package: &str, output: &str) -> Self {
        let trimmed = output.trim();

        // Check for NOT_INSTALLED marker or error
        if trimmed.contains("NOT_INSTALLED") || trimmed.contains("was not found") || trimmed.is_empty() {
            return Self {
                package: package.to_string(),
                installed: false,
                version: None,
                evidence: vec![format!("pacman -Q {} → not found", package)],
            };
        }

        // Parse "package version" format
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == package {
            return Self {
                package: package.to_string(),
                installed: true,
                version: Some(parts[1].to_string()),
                evidence: vec![format!("pacman -Q {} → {}", package, trimmed)],
            };
        }

        // Unexpected format
        Self {
            package: package.to_string(),
            installed: false,
            version: None,
            evidence: vec![format!("Unexpected pacman output: {}", trimmed)],
        }
    }

    /// Format for user display.
    pub fn display(&self) -> String {
        if !self.installed {
            return format!(
                "{} is NOT installed.\n• evidence: {}",
                self.package,
                self.evidence.join(", ")
            );
        }

        let version = self.version.as_deref().unwrap_or("unknown");
        format!(
            "{} is installed, version {}.\n• evidence: {}",
            self.package,
            version,
            self.evidence.join(", ")
        )
    }
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

/// Classify a user question.
pub fn classify_question(question: &str, entity: Option<&str>) -> QuestionClassification {
    let lower = question.to_lowercase();

    // Check system intents first (higher priority for swap, trim, etc.)
    if let Some(system_intent) = SystemIntent::from_question(question) {
        return QuestionClassification::System(system_intent);
    }

    // Check package intents
    if let Some(package_intent) = PackageIntent::from_question(question, entity) {
        return QuestionClassification::Package(package_intent);
    }

    // Ambiguous cases
    if lower.contains("do i have") && !is_system_feature(&lower) {
        return QuestionClassification::Ambiguous {
            question: question.to_string(),
        };
    }

    QuestionClassification::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_question_classification() {
        let result = classify_question("do I have swap?", None);
        assert!(matches!(result, QuestionClassification::System(SystemIntent::SwapConfigured)));

        let result = classify_question("is swap enabled?", None);
        assert!(matches!(result, QuestionClassification::System(SystemIntent::SwapConfigured)));
    }

    #[test]
    fn test_package_question_classification() {
        let result = classify_question("is nano installed?", Some("nano"));
        assert!(matches!(
            result,
            QuestionClassification::Package(PackageIntent::CheckInstalled { package }) if package == "nano"
        ));
    }

    #[test]
    fn test_swap_status_parsing() {
        let empty = "";
        let status = SwapStatus::from_proc_swaps(empty);
        assert!(!status.has_swap);
        assert_eq!(status.kind, SwapKind::None);

        let with_swap = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n/swapfile\tfile\t\t8388604\t\t0\t\t-2";
        let status = SwapStatus::from_proc_swaps(with_swap);
        assert!(status.has_swap);
        assert!(status.total_swap_gib > 7.0);
        assert_eq!(status.kind, SwapKind::File);
    }

    #[test]
    fn test_package_status_parsing() {
        let installed = "nano 7.2-1";
        let status = PackageStatus::from_pacman_output("nano", installed);
        assert!(status.installed);
        assert_eq!(status.version, Some("7.2-1".to_string()));

        let not_installed = "NOT_INSTALLED";
        let status = PackageStatus::from_pacman_output("nano", not_installed);
        assert!(!status.installed);
    }

    #[test]
    fn test_swap_display() {
        let no_swap = SwapStatus {
            has_swap: false,
            total_swap_gib: 0.0,
            kind: SwapKind::None,
            evidence: vec![],
        };
        assert!(no_swap.display().contains("NO swap"));

        let with_swap = SwapStatus {
            has_swap: true,
            total_swap_gib: 8.0,
            kind: SwapKind::File,
            evidence: vec![],
        };
        assert!(with_swap.display().contains("8.0 GiB"));
    }

    #[test]
    fn test_extract_package_name() {
        assert_eq!(extract_package_name("is nano installed"), Some("nano"));
        assert_eq!(extract_package_name("install vim"), Some("vim"));
        assert_eq!(extract_package_name("do i have steam"), Some("steam"));
    }

    #[test]
    fn test_is_system_feature() {
        assert!(is_system_feature("do i have swap"));
        assert!(is_system_feature("memory status"));
        assert!(!is_system_feature("do i have nano"));
    }
}
