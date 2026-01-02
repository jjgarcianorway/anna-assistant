//! Recipe enum types (v0.0.423).
//!
//! Core enumeration types for recipes.

use serde::{Deserialize, Serialize};

/// Origin of a recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeOrigin {
    /// Built-in seed recipe
    #[default]
    BuiltIn,
    /// Learned from a successful ticket
    LearnedFromTicket,
    /// Manually authored by user
    UserAuthored,
}

/// Author of a recipe
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeAuthor {
    /// System-generated recipe
    #[default]
    System,
    /// Specialist-generated recipe
    Specialist(String),
    /// User-authored recipe
    User(String),
}

/// Domain/category of a recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeDomain {
    #[default]
    General,
    Service,
    Package,
    Network,
    Disk,
    Memory,
    Process,
    User,
    Config,
    Git,
    Docker,
    Editor,
    Shell,
    Systemd,
    Cron,
}

impl RecipeDomain {
    /// Parse domain from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "service" | "systemd" => Self::Systemd,
            "package" | "pacman" | "yay" | "paru" => Self::Package,
            "network" | "net" | "wifi" | "ethernet" => Self::Network,
            "disk" | "storage" | "filesystem" | "mount" => Self::Disk,
            "memory" | "ram" | "swap" => Self::Memory,
            "process" | "proc" | "ps" | "kill" => Self::Process,
            "user" | "account" | "group" => Self::User,
            "config" | "conf" | "settings" => Self::Config,
            "git" | "github" | "repo" => Self::Git,
            "docker" | "container" | "podman" => Self::Docker,
            "editor" | "vim" | "nvim" | "nano" | "emacs" => Self::Editor,
            "shell" | "bash" | "zsh" | "fish" => Self::Shell,
            "cron" | "timer" | "schedule" => Self::Cron,
            _ => Self::General,
        }
    }
}

/// Risk level of a recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeRiskLevel {
    /// Safe, read-only operations (probes, queries)
    #[default]
    None,
    /// Low risk, easily reversible (config changes with backup)
    Low,
    /// Medium risk, may require manual intervention to reverse
    Medium,
    /// High risk, potentially destructive (rm, format, etc.)
    High,
}

impl RecipeRiskLevel {
    /// Whether this risk level requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Medium | Self::High)
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" | "safe" | "readonly" => Self::None,
            "low" => Self::Low,
            "medium" | "med" => Self::Medium,
            "high" | "dangerous" => Self::High,
            _ => Self::Medium,
        }
    }
}

/// Confirmation policy for recipe execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationPolicy {
    /// Never ask for confirmation (trusted/safe recipes)
    Never,
    /// Ask once before starting execution
    #[default]
    Once,
    /// Ask before each risky step
    PerStep,
    /// Always ask (high-risk recipes)
    Always,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_domain_parse() {
        assert_eq!(RecipeDomain::from_str("systemd"), RecipeDomain::Systemd);
        assert_eq!(RecipeDomain::from_str("pacman"), RecipeDomain::Package);
        assert_eq!(RecipeDomain::from_str("unknown"), RecipeDomain::General);
    }

    #[test]
    fn test_risk_level_confirmation() {
        assert!(!RecipeRiskLevel::None.requires_confirmation());
        assert!(!RecipeRiskLevel::Low.requires_confirmation());
        assert!(RecipeRiskLevel::Medium.requires_confirmation());
        assert!(RecipeRiskLevel::High.requires_confirmation());
    }
}
