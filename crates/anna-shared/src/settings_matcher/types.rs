// v0.0.687: Settings Matcher Types (Phase 263)
// Match types, targets, config, and rules

use serde::{Deserialize, Serialize};

/// Match type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MatchType {
    /// Exact match
    #[default]
    Exact,
    /// Prefix match
    Prefix,
    /// Suffix match
    Suffix,
    /// Contains match
    Contains,
}

impl std::fmt::Display for MatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "exact"),
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
            Self::Contains => write!(f, "contains"),
        }
    }
}

/// Match target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MatchTarget {
    /// Match key
    #[default]
    Key,
    /// Match value
    Value,
    /// Match both
    Both,
    /// Match either
    Either,
}

impl std::fmt::Display for MatchTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key => write!(f, "key"),
            Self::Value => write!(f, "value"),
            Self::Both => write!(f, "both"),
            Self::Either => write!(f, "either"),
        }
    }
}

/// Matcher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatcherConfig {
    /// Match type
    pub match_type: MatchType,
    /// Match target
    pub target: MatchTarget,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Invert match
    pub invert: bool,
}

impl MatcherConfig {
    /// Create new config
    pub fn new(match_type: MatchType) -> Self {
        Self {
            match_type,
            target: MatchTarget::Key,
            case_insensitive: true,
            invert: false,
        }
    }

    /// Set target
    pub fn target(mut self, target: MatchTarget) -> Self {
        self.target = target;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set invert
    pub fn invert(mut self, inv: bool) -> Self {
        self.invert = inv;
        self
    }
}

impl Default for MatcherConfig {
    fn default() -> Self {
        Self::new(MatchType::Contains)
    }
}

/// Match rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    /// Rule ID
    pub id: String,
    /// Pattern
    pub pattern: String,
    /// Match type
    pub match_type: MatchType,
    /// Target
    pub target: MatchTarget,
}

impl MatchRule {
    /// Create new rule
    pub fn new(id: impl Into<String>, pattern: impl Into<String>, match_type: MatchType) -> Self {
        Self {
            id: id.into(),
            pattern: pattern.into(),
            match_type,
            target: MatchTarget::Key,
        }
    }

    /// Set target
    pub fn target(mut self, target: MatchTarget) -> Self {
        self.target = target;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_type_display() {
        assert_eq!(format!("{}", MatchType::Exact), "exact");
        assert_eq!(format!("{}", MatchType::Contains), "contains");
    }

    #[test]
    fn test_match_target_display() {
        assert_eq!(format!("{}", MatchTarget::Key), "key");
        assert_eq!(format!("{}", MatchTarget::Both), "both");
    }

    #[test]
    fn test_config_new() {
        let c = MatcherConfig::new(MatchType::Prefix);
        assert_eq!(c.match_type, MatchType::Prefix);
    }

    #[test]
    fn test_config_builder() {
        let c = MatcherConfig::new(MatchType::Exact)
            .target(MatchTarget::Value)
            .invert(true);
        assert_eq!(c.target, MatchTarget::Value);
        assert!(c.invert);
    }

    #[test]
    fn test_rule_new() {
        let r = MatchRule::new("r1", "app.*", MatchType::Prefix);
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_rule_target() {
        let r = MatchRule::new("r1", "test", MatchType::Contains).target(MatchTarget::Value);
        assert_eq!(r.target, MatchTarget::Value);
    }
}
