//! Intent parsing and classification logic.

use super::helpers::{extract_package_name, is_system_feature, is_vague_package_name};
use super::types::{PackageIntent, QuestionClassification, SystemIntent};

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
        assert!(matches!(
            result,
            QuestionClassification::System(SystemIntent::SwapConfigured)
        ));

        let result = classify_question("is swap enabled?", None);
        assert!(matches!(
            result,
            QuestionClassification::System(SystemIntent::SwapConfigured)
        ));
    }

    #[test]
    fn test_package_question_classification() {
        let result = classify_question("is nano installed?", Some("nano"));
        assert!(matches!(
            result,
            QuestionClassification::Package(PackageIntent::CheckInstalled { package }) if package == "nano"
        ));
    }
}
