//! Policy Engine - Rule-based decision making for autonomous actions
//!
//! Sprint 3: Intelligence, Policies & Event Reactions
//!
//! The policy engine evaluates conditions before allowing autonomous actions.
//! Rules are defined in YAML files in /etc/anna/policies.d/*.{yaml,yml}
//!
//! Rule syntax:
//! ```yaml
//! - when: telemetry.error_rate > 5%
//!   then: disable autonomy
//! - when: uptime > 48h
//!   then: run doctor
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Policy evaluation errors
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Failed to load policy file: {0}")]
    LoadError(String),

    #[error("Failed to parse policy YAML: {0}")]
    ParseError(String),

    #[error("Policy evaluation failed: {0}")]
    EvaluationError(String),

    #[error("Policy directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Policy action to execute when condition is met
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    DisableAutonomy,
    EnableAutonomy,
    RunDoctor,
    RestartService,
    SendAlert,
    Custom(String),
}

/// Condition operator for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">=")]
    GreaterOrEqual,
    #[serde(rename = "<=")]
    LessOrEqual,
    #[serde(rename = "==")]
    Equal,
    #[serde(rename = "!=")]
    NotEqual,
}

/// Policy condition to evaluate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: Operator,
    pub value: ConditionValue,
}

/// Value type for condition comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConditionValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Percentage(String), // e.g., "5%"
}

/// Policy rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    #[serde(rename = "when")]
    pub condition: String, // Raw condition string (e.g., "telemetry.error_rate > 5%")

    #[serde(rename = "then")]
    pub action: PolicyAction,

    #[serde(default)]
    pub enabled: bool,

    #[serde(skip)]
    pub parsed_condition: Option<Condition>,
}

impl PolicyRule {
    /// Parse the condition string into structured condition
    pub fn parse_condition(&mut self) -> Result<(), PolicyError> {
        let parts: Vec<&str> = self.condition.split_whitespace().collect();

        if parts.len() < 3 {
            return Err(PolicyError::ParseError(
                format!("Invalid condition format: {}", self.condition)
            ));
        }

        let field = parts[0].to_string();
        let op_str = parts[1];
        let value_str = parts[2..].join(" ");

        let operator = match op_str {
            ">" => Operator::GreaterThan,
            "<" => Operator::LessThan,
            ">=" => Operator::GreaterOrEqual,
            "<=" => Operator::LessOrEqual,
            "==" => Operator::Equal,
            "!=" => Operator::NotEqual,
            _ => return Err(PolicyError::ParseError(
                format!("Unknown operator: {}", op_str)
            )),
        };

        let value = if value_str.ends_with('%') {
            ConditionValue::Percentage(value_str)
        } else if let Ok(num) = value_str.parse::<f64>() {
            ConditionValue::Number(num)
        } else if value_str == "true" || value_str == "false" {
            ConditionValue::Boolean(value_str == "true")
        } else {
            ConditionValue::String(value_str)
        };

        self.parsed_condition = Some(Condition {
            field,
            operator,
            value,
        });

        Ok(())
    }
}

/// Policy evaluation context containing current system state
#[derive(Debug, Clone, Default)]
pub struct PolicyContext {
    pub metrics: HashMap<String, f64>,
    pub flags: HashMap<String, bool>,
    pub strings: HashMap<String, String>,
}

impl PolicyContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a numeric metric
    pub fn set_metric(&mut self, key: &str, value: f64) {
        self.metrics.insert(key.to_string(), value);
    }

    /// Set a boolean flag
    pub fn set_flag(&mut self, key: &str, value: bool) {
        self.flags.insert(key.to_string(), value);
    }

    /// Set a string value
    pub fn set_string(&mut self, key: &str, value: String) {
        self.strings.insert(key.to_string(), value);
    }

    /// Get a value by field path (e.g., "telemetry.error_rate")
    fn get_value(&self, field: &str) -> Option<ConditionValue> {
        if let Some(val) = self.metrics.get(field) {
            return Some(ConditionValue::Number(*val));
        }
        if let Some(val) = self.flags.get(field) {
            return Some(ConditionValue::Boolean(*val));
        }
        if let Some(val) = self.strings.get(field) {
            return Some(ConditionValue::String(val.clone()));
        }
        None
    }
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub matched: bool,
    pub actions: Vec<PolicyAction>,
    #[allow(dead_code)]
    pub triggered_rules: Vec<String>,
}

/// Policy Engine - manages and evaluates policy rules
pub struct PolicyEngine {
    rules: Arc<RwLock<Vec<PolicyRule>>>,
    policy_dir: PathBuf,
}

impl PolicyEngine {
    /// Create a new policy engine with specified policy directory
    pub fn new(policy_dir: impl AsRef<Path>) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            policy_dir: policy_dir.as_ref().to_path_buf(),
        }
    }

    /// Load all policies from the policy directory
    pub fn load_policies(&self) -> Result<usize, PolicyError> {
        if !self.policy_dir.exists() {
            return Err(PolicyError::DirectoryNotFound(
                self.policy_dir.display().to_string()
            ));
        }

        let mut loaded_rules = Vec::new();

        // Read all .yaml and .yml files in the policy directory
        for entry in fs::read_dir(&self.policy_dir)? {
            let entry = entry?;
            let path = entry.path();

            let ext = path.extension().and_then(|s| s.to_str());
            if ext == Some("yaml") || ext == Some("yml") {
                match self.load_policy_file(&path) {
                    Ok(mut rules) => {
                        // Parse conditions for each rule
                        for rule in &mut rules {
                            rule.parse_condition().map_err(|e| {
                                PolicyError::ParseError(
                                    format!("In file {}: {}", path.display(), e)
                                )
                            })?;
                        }
                        loaded_rules.extend(rules);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load policy file {}: {}", path.display(), e);
                    }
                }
            }
        }

        let count = loaded_rules.len();
        let mut rules = self.rules.write().unwrap();
        *rules = loaded_rules;

        Ok(count)
    }

    /// Load a single policy file
    fn load_policy_file(&self, path: &Path) -> Result<Vec<PolicyRule>, PolicyError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| PolicyError::LoadError(format!("{}: {}", path.display(), e)))?;

        let rules: Vec<PolicyRule> = serde_yaml::from_str(&contents)
            .map_err(|e| PolicyError::ParseError(format!("{}: {}", path.display(), e)))?;

        Ok(rules)
    }

    /// Reload all policies from disk
    pub fn reload(&self) -> Result<usize, PolicyError> {
        self.load_policies()
    }

    /// Evaluate all policies against the current context
    pub fn evaluate(&self, context: &PolicyContext) -> Result<EvaluationResult, PolicyError> {
        let rules = self.rules.read().unwrap();

        let mut matched = false;
        let mut actions = Vec::new();
        let mut triggered_rules = Vec::new();

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if let Some(condition) = &rule.parsed_condition {
                if self.evaluate_condition(condition, context)? {
                    matched = true;
                    actions.push(rule.action.clone());
                    triggered_rules.push(rule.condition.clone());
                }
            }
        }

        Ok(EvaluationResult {
            matched,
            actions,
            triggered_rules,
        })
    }

    /// Evaluate a single condition against the context
    fn evaluate_condition(
        &self,
        condition: &Condition,
        context: &PolicyContext,
    ) -> Result<bool, PolicyError> {
        let context_value = context.get_value(&condition.field)
            .ok_or_else(|| PolicyError::EvaluationError(
                format!("Field not found in context: {}", condition.field)
            ))?;

        // Convert percentage strings to numbers for comparison
        let (left_val, right_val) = match (&context_value, &condition.value) {
            (ConditionValue::Number(n), ConditionValue::Percentage(p)) => {
                let pct = p.trim_end_matches('%')
                    .parse::<f64>()
                    .map_err(|e| PolicyError::EvaluationError(
                        format!("Invalid percentage: {}", e)
                    ))? / 100.0;
                (*n, pct)
            }
            (ConditionValue::Number(n1), ConditionValue::Number(n2)) => (*n1, *n2),
            _ => {
                // For non-numeric comparisons, use direct comparison
                return Ok(self.compare_values(&context_value, &condition.value, &condition.operator));
            }
        };

        Ok(match condition.operator {
            Operator::GreaterThan => left_val > right_val,
            Operator::LessThan => left_val < right_val,
            Operator::GreaterOrEqual => left_val >= right_val,
            Operator::LessOrEqual => left_val <= right_val,
            Operator::Equal => (left_val - right_val).abs() < f64::EPSILON,
            Operator::NotEqual => (left_val - right_val).abs() >= f64::EPSILON,
        })
    }

    /// Compare non-numeric values
    fn compare_values(&self, left: &ConditionValue, right: &ConditionValue, op: &Operator) -> bool {
        match op {
            Operator::Equal => left == right,
            Operator::NotEqual => left != right,
            _ => false, // Other operators not supported for non-numeric values
        }
    }

    /// Get list of all loaded rules
    pub fn list_rules(&self) -> Vec<PolicyRule> {
        self.rules.read().unwrap().clone()
    }

    /// Get count of loaded rules
    pub fn rule_count(&self) -> usize {
        self.rules.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_condition_parsing() {
        let mut rule = PolicyRule {
            condition: "telemetry.error_rate > 5%".to_string(),
            action: PolicyAction::DisableAutonomy,
            enabled: true,
            parsed_condition: None,
        };

        rule.parse_condition().unwrap();
        assert!(rule.parsed_condition.is_some());

        let cond = rule.parsed_condition.unwrap();
        assert_eq!(cond.field, "telemetry.error_rate");
        assert_eq!(cond.operator, Operator::GreaterThan);
        assert_eq!(cond.value, ConditionValue::Percentage("5%".to_string()));
    }

    #[test]
    fn test_policy_evaluation() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("test.yaml");

        let yaml = r#"
- when: error_rate > 0.05
  then: disable_autonomy
  enabled: true
- when: uptime > 172800
  then: run_doctor
  enabled: true
"#;
        fs::write(&policy_file, yaml).unwrap();

        let engine = PolicyEngine::new(temp_dir.path());
        let loaded = engine.load_policies().unwrap();
        assert_eq!(loaded, 2);

        let mut context = PolicyContext::new();
        context.set_metric("error_rate", 0.06);
        context.set_metric("uptime", 100000.0);

        let result = engine.evaluate(&context).unwrap();
        assert!(result.matched);
        assert_eq!(result.actions.len(), 1);
        assert_eq!(result.actions[0], PolicyAction::DisableAutonomy);
    }

    #[test]
    fn test_percentage_comparison() {
        let mut rule = PolicyRule {
            condition: "error_rate > 5%".to_string(),
            action: PolicyAction::DisableAutonomy,
            enabled: true,
            parsed_condition: None,
        };
        rule.parse_condition().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let engine = PolicyEngine::new(temp_dir.path());

        let mut context = PolicyContext::new();
        context.set_metric("error_rate", 0.06); // 6%

        let result = engine.evaluate_condition(
            rule.parsed_condition.as_ref().unwrap(),
            &context,
        ).unwrap();

        assert!(result);
    }
}
