//! Fact requirements for recipe preconditions (v0.0.420).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Operator for fact comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FactOp {
    /// Value must equal
    #[default]
    Eq,
    /// Value must not equal
    Ne,
    /// Value must be in list (comma-separated)
    In,
    /// Value must not be in list
    NotIn,
    /// Key must exist (any value)
    Exists,
}

/// A fact requirement that must be satisfied for a recipe to match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactRequirement {
    /// Fact key, e.g., "editor", "os", "package_manager", "has_swap"
    pub key: String,
    /// Comparison operator
    #[serde(default)]
    pub operator: FactOp,
    /// Expected value (for Eq/Ne) or comma-separated list (for In/NotIn)
    pub value: String,
}

impl FactRequirement {
    /// Create an equality requirement
    pub fn eq(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            operator: FactOp::Eq,
            value: value.to_string(),
        }
    }

    /// Create a not-equal requirement
    pub fn ne(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            operator: FactOp::Ne,
            value: value.to_string(),
        }
    }

    /// Create an "in list" requirement
    pub fn in_list(key: &str, values: &[&str]) -> Self {
        Self {
            key: key.to_string(),
            operator: FactOp::In,
            value: values.join(","),
        }
    }

    /// Create an "exists" requirement
    pub fn exists(key: &str) -> Self {
        Self {
            key: key.to_string(),
            operator: FactOp::Exists,
            value: String::new(),
        }
    }

    /// Check if the requirement is satisfied given a set of facts
    pub fn is_satisfied(&self, facts: &HashMap<String, String>) -> bool {
        match self.operator {
            FactOp::Exists => facts.contains_key(&self.key),
            FactOp::Eq => facts.get(&self.key).map(|v| v == &self.value).unwrap_or(false),
            FactOp::Ne => facts.get(&self.key).map(|v| v != &self.value).unwrap_or(true),
            FactOp::In => {
                let allowed: Vec<&str> = self.value.split(',').map(|s| s.trim()).collect();
                facts
                    .get(&self.key)
                    .map(|v| allowed.contains(&v.as_str()))
                    .unwrap_or(false)
            }
            FactOp::NotIn => {
                let disallowed: Vec<&str> = self.value.split(',').map(|s| s.trim()).collect();
                facts
                    .get(&self.key)
                    .map(|v| !disallowed.contains(&v.as_str()))
                    .unwrap_or(true)
            }
        }
    }

    /// Get the fact key that needs to be checked
    pub fn required_key(&self) -> &str {
        &self.key
    }
}

/// Check if all requirements are satisfied
pub fn check_requirements(requirements: &[FactRequirement], facts: &HashMap<String, String>) -> bool {
    requirements.iter().all(|r| r.is_satisfied(facts))
}

/// Get the list of missing fact keys
pub fn missing_facts(requirements: &[FactRequirement], facts: &HashMap<String, String>) -> Vec<String> {
    requirements
        .iter()
        .filter(|r| !r.is_satisfied(facts))
        .map(|r| r.key.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_requirement() {
        let req = FactRequirement::eq("editor", "vim");
        let mut facts = HashMap::new();

        // Not satisfied when key missing
        assert!(!req.is_satisfied(&facts));

        // Not satisfied when wrong value
        facts.insert("editor".to_string(), "emacs".to_string());
        assert!(!req.is_satisfied(&facts));

        // Satisfied when correct value
        facts.insert("editor".to_string(), "vim".to_string());
        assert!(req.is_satisfied(&facts));
    }

    #[test]
    fn test_exists_requirement() {
        let req = FactRequirement::exists("has_swap");
        let mut facts = HashMap::new();

        assert!(!req.is_satisfied(&facts));

        facts.insert("has_swap".to_string(), "true".to_string());
        assert!(req.is_satisfied(&facts));
    }

    #[test]
    fn test_in_requirement() {
        let req = FactRequirement::in_list("os", &["arch", "manjaro"]);
        let mut facts = HashMap::new();

        facts.insert("os".to_string(), "ubuntu".to_string());
        assert!(!req.is_satisfied(&facts));

        facts.insert("os".to_string(), "arch".to_string());
        assert!(req.is_satisfied(&facts));
    }
}
