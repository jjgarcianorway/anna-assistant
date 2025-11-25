//! Policy Engine - User-configurable guardrails for planning and execution
//!
//! v6.52.0: "If Anna can plan, execute, and rollback, she needs rules"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Policy-level risk classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PolicyRiskLevel {
    Safe,
    Moderate,
    High,
}

/// Policy domain classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyDomain {
    Config,
    Packages,
    Services,
    Network,
    Users,
    Filesystem,
    General,
}

/// Action a policy rule can enforce
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireConfirm,
    RequireStrongConfirm, // Type "YES" style
}

/// Source of a policy rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicySource {
    Default,                     // Shipped with Anna
    System,                      // Created by Anna for safety
    NaturalLanguage(String),     // User said: "never touch my ssh config"
}

/// Unique identifier for a policy rule
pub type PolicyRuleId = String;

/// A single policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: PolicyRuleId,
    pub description: String,
    pub domain: PolicyDomain,
    pub max_risk: PolicyRiskLevel,
    pub path_globs: Vec<String>,
    pub tags: Vec<String>,
    pub action: PolicyAction,
    pub created_at: DateTime<Utc>,
    pub source: PolicySource,
}

/// Complete policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySet {
    pub rules: Vec<PolicyRule>,
    pub version: u32,
}

/// Result of evaluating a plan against policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub require_confirm: bool,
    pub require_strong_confirm: bool,
    pub matched_rules: Vec<PolicyRuleId>,
    pub effective_risk_cap: PolicyRiskLevel,
    pub notes: Vec<String>,
}

/// A planned action to be evaluated
#[derive(Debug, Clone)]
pub struct PlannedAction {
    pub domain: PolicyDomain,
    pub risk_level: PolicyRiskLevel,
    pub target_paths: Vec<String>,
    pub target_services: Vec<String>,
    pub target_packages: Vec<String>,
    pub tags: Vec<String>,
}

impl PolicySet {
    /// Create default conservative policy set
    pub fn default_policy() -> Self {
        let mut rules = Vec::new();
        let now = Utc::now();

        // Rule 1: Global risk cap - deny operations that exceed Moderate risk
        // This is a risk-threshold rule: only triggers when risk_level > max_risk
        rules.push(PolicyRule {
            id: "R-001".to_string(),
            description: "Deny High risk operations by default".to_string(),
            domain: PolicyDomain::General,
            max_risk: PolicyRiskLevel::Moderate,
            path_globs: vec![],
            tags: vec![],
            action: PolicyAction::Deny,
            created_at: now,
            source: PolicySource::Default,
        });

        // Rule 2: Require confirmation for Moderate risk in Config domain
        rules.push(PolicyRule {
            id: "R-002".to_string(),
            description: "Require confirmation for Moderate risk config changes".to_string(),
            domain: PolicyDomain::Config,
            max_risk: PolicyRiskLevel::Moderate,
            path_globs: vec![],
            tags: vec![],
            action: PolicyAction::RequireConfirm,
            created_at: now,
            source: PolicySource::Default,
        });

        // Rule 3: Require confirmation for Moderate risk package operations
        rules.push(PolicyRule {
            id: "R-003".to_string(),
            description: "Require confirmation for package install/remove".to_string(),
            domain: PolicyDomain::Packages,
            max_risk: PolicyRiskLevel::Moderate,
            path_globs: vec![],
            tags: vec![],
            action: PolicyAction::RequireConfirm,
            created_at: now,
            source: PolicySource::Default,
        });

        // Rule 4: Require confirmation for Moderate risk service operations
        rules.push(PolicyRule {
            id: "R-004".to_string(),
            description: "Require confirmation for service changes".to_string(),
            domain: PolicyDomain::Services,
            max_risk: PolicyRiskLevel::Moderate,
            path_globs: vec![],
            tags: vec![],
            action: PolicyAction::RequireConfirm,
            created_at: now,
            source: PolicySource::Default,
        });

        // Rule 5: Strong confirmation for /etc/* changes
        rules.push(PolicyRule {
            id: "R-005".to_string(),
            description: "Strong confirmation required for /etc/* changes".to_string(),
            domain: PolicyDomain::Config,
            max_risk: PolicyRiskLevel::Moderate,
            path_globs: vec!["/etc/*".to_string()],
            tags: vec![],
            action: PolicyAction::RequireStrongConfirm,
            created_at: now,
            source: PolicySource::Default,
        });

        // Rule 6: Deny SSH config changes
        rules.push(PolicyRule {
            id: "R-006".to_string(),
            description: "Deny changes to SSH configuration".to_string(),
            domain: PolicyDomain::Config,
            max_risk: PolicyRiskLevel::High,
            path_globs: vec![
                "/etc/ssh/sshd_config".to_string(),
                "/etc/ssh/ssh_config".to_string(),
            ],
            tags: vec!["ssh".to_string(), "network".to_string(), "auth".to_string()],
            action: PolicyAction::Deny,
            created_at: now,
            source: PolicySource::Default,
        });

        // Rule 7: Deny critical system file changes
        rules.push(PolicyRule {
            id: "R-007".to_string(),
            description: "Deny changes to critical system files".to_string(),
            domain: PolicyDomain::Config,
            max_risk: PolicyRiskLevel::High,
            path_globs: vec![
                "/etc/shadow".to_string(),
                "/etc/passwd".to_string(),
                "/etc/sudoers".to_string(),
            ],
            tags: vec!["security".to_string(), "auth".to_string()],
            action: PolicyAction::Deny,
            created_at: now,
            source: PolicySource::Default,
        });

        PolicySet {
            rules,
            version: 1,
        }
    }

    /// Add a new rule to the policy set
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        self.version += 1;
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        let removed = self.rules.len() < initial_len;
        if removed {
            self.version += 1;
        }
        removed
    }

    /// Find rules matching a domain
    pub fn rules_for_domain(&self, domain: PolicyDomain) -> Vec<&PolicyRule> {
        self.rules
            .iter()
            .filter(|r| r.domain == domain || r.domain == PolicyDomain::General)
            .collect()
    }

    /// Find rules matching tags
    pub fn rules_with_tag(&self, tag: &str) -> Vec<&PolicyRule> {
        self.rules
            .iter()
            .filter(|r| r.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }
}

/// Evaluate a planned action against policy
pub fn evaluate_plan_against_policy(
    policy: &PolicySet,
    plan: &PlannedAction,
) -> PolicyDecision {
    let mut decision = PolicyDecision {
        allowed: true,
        require_confirm: false,
        require_strong_confirm: false,
        matched_rules: Vec::new(),
        effective_risk_cap: PolicyRiskLevel::High,
        notes: Vec::new(),
    };

    // Find all potentially matching rules
    let mut matching_rules: Vec<&PolicyRule> = policy.rules
        .iter()
        .filter(|rule| {
            // Match by domain
            if rule.domain != PolicyDomain::General && rule.domain != plan.domain {
                return false;
            }

            // Match by path globs
            if !rule.path_globs.is_empty() {
                let matches_path = plan.target_paths.iter().any(|path| {
                    rule.path_globs.iter().any(|glob| path_matches_glob(path, glob))
                });
                if matches_path {
                    return true;
                }
            }

            // Match by tags
            if !rule.tags.is_empty() {
                let matches_tag = plan.tags.iter().any(|tag| {
                    rule.tags.iter().any(|rule_tag| rule_tag.eq_ignore_ascii_case(tag))
                });
                if matches_tag {
                    return true;
                }
            }

            // Match by domain if no specific paths/tags
            if rule.path_globs.is_empty() && rule.tags.is_empty() {
                return true;
            }

            false
        })
        .collect();

    // Sort by specificity (more specific rules first)
    matching_rules.sort_by(|a, b| {
        let a_score = rule_specificity_score(a);
        let b_score = rule_specificity_score(b);
        b_score.cmp(&a_score)
    });

    // Apply rules in order of specificity
    for rule in matching_rules {
        decision.matched_rules.push(rule.id.clone());

        // Determine rule type and how to apply it
        let has_paths_or_tags = !rule.path_globs.is_empty() || !rule.tags.is_empty();
        let is_general_domain = rule.domain == PolicyDomain::General;

        if has_paths_or_tags {
            // Path/tag-specific rules: Apply action regardless of risk level
            // (e.g., "never touch SSH config" applies at any risk level)
            match rule.action {
                PolicyAction::Deny => {
                    decision.allowed = false;
                    decision.notes.push(format!(
                        "Denied by rule {}: '{}'",
                        rule.id, rule.description
                    ));
                    return decision; // Early return on deny
                }
                PolicyAction::RequireStrongConfirm => {
                    decision.require_strong_confirm = true;
                    decision.notes.push(format!(
                        "Strong confirmation required by rule {}: '{}'",
                        rule.id, rule.description
                    ));
                }
                PolicyAction::RequireConfirm => {
                    decision.require_confirm = true;
                    decision.notes.push(format!(
                        "Confirmation required by rule {}: '{}'",
                        rule.id, rule.description
                    ));
                }
                PolicyAction::Allow => {
                    decision.notes.push(format!("Allowed by rule {}: '{}'", rule.id, rule.description));
                }
            }
        } else if is_general_domain {
            // Global risk cap rules: Only trigger when risk exceeds max_risk
            // (e.g., "deny High risk" triggers when risk > Moderate)
            if plan.risk_level > rule.max_risk {
                match rule.action {
                    PolicyAction::Deny => {
                        decision.allowed = false;
                        decision.notes.push(format!(
                            "Denied by rule {}: '{}' (risk {} exceeds max {})",
                            rule.id,
                            rule.description,
                            format_risk_level(plan.risk_level),
                            format_risk_level(rule.max_risk)
                        ));
                        return decision; // Early return on deny
                    }
                    PolicyAction::RequireStrongConfirm => {
                        decision.require_strong_confirm = true;
                        decision.notes.push(format!(
                            "Strong confirmation required by rule {}: '{}' (risk {} exceeds max {})",
                            rule.id,
                            rule.description,
                            format_risk_level(plan.risk_level),
                            format_risk_level(rule.max_risk)
                        ));
                    }
                    PolicyAction::RequireConfirm => {
                        decision.require_confirm = true;
                        decision.notes.push(format!(
                            "Confirmation required by rule {}: '{}' (risk {} exceeds max {})",
                            rule.id,
                            rule.description,
                            format_risk_level(plan.risk_level),
                            format_risk_level(rule.max_risk)
                        ));
                    }
                    _ => {}
                }
            }
        } else {
            // Domain-specific rules: Apply when risk level is within the max
            // (e.g., "require confirm for Moderate Config" applies to Moderate or lower)
            if plan.risk_level <= rule.max_risk {
                match rule.action {
                    PolicyAction::Deny => {
                        decision.allowed = false;
                        decision.notes.push(format!(
                            "Denied by rule {}: '{}'",
                            rule.id, rule.description
                        ));
                        return decision; // Early return on deny
                    }
                    PolicyAction::RequireStrongConfirm => {
                        decision.require_strong_confirm = true;
                        decision.notes.push(format!(
                            "Strong confirmation required by rule {}: '{}'",
                            rule.id, rule.description
                        ));
                    }
                    PolicyAction::RequireConfirm => {
                        decision.require_confirm = true;
                        decision.notes.push(format!(
                            "Confirmation required by rule {}: '{}'",
                            rule.id, rule.description
                        ));
                    }
                    PolicyAction::Allow => {
                        decision.notes.push(format!("Allowed by rule {}: '{}'", rule.id, rule.description));
                    }
                }
            }
        }

        // Update effective risk cap
        if rule.max_risk < decision.effective_risk_cap {
            decision.effective_risk_cap = rule.max_risk;
        }
    }

    decision
}

/// Calculate specificity score for rule ordering
fn rule_specificity_score(rule: &PolicyRule) -> u32 {
    let mut score = 0u32;

    // Specific paths are more specific than domain-only
    score += rule.path_globs.len() as u32 * 100;

    // Specific tags are moderately specific
    score += rule.tags.len() as u32 * 10;

    // Non-general domains are slightly more specific
    if rule.domain != PolicyDomain::General {
        score += 5;
    }

    // Deny actions are most specific
    if rule.action == PolicyAction::Deny {
        score += 1000;
    }

    score
}

/// Check if a path matches a glob pattern (simple implementation)
fn path_matches_glob(path: &str, glob: &str) -> bool {
    // Handle simple wildcard patterns
    if glob.ends_with("/*") {
        let prefix = glob.trim_end_matches("/*");
        return path.starts_with(prefix);
    }

    // Exact match
    path == glob
}

/// Format risk level for display
fn format_risk_level(level: PolicyRiskLevel) -> &'static str {
    match level {
        PolicyRiskLevel::Safe => "Safe",
        PolicyRiskLevel::Moderate => "Moderate",
        PolicyRiskLevel::High => "High",
    }
}

// ============================================================================
// Policy Persistence
// ============================================================================

/// Load policy set from disk
///
/// Fail-safe: If loading fails for any reason, returns conservative default policy
pub fn load_policy_set<P: AsRef<Path>>(path: P) -> PolicySet {
    match std::fs::read_to_string(path.as_ref()) {
        Ok(contents) => match serde_json::from_str::<PolicySet>(&contents) {
            Ok(policy) => policy,
            Err(e) => {
                eprintln!("Warning: Failed to parse policy file: {}. Using default policy.", e);
                PolicySet::default_policy()
            }
        },
        Err(e) => {
            eprintln!("Warning: Failed to read policy file: {}. Using default policy.", e);
            PolicySet::default_policy()
        }
    }
}

/// Save policy set to disk
pub fn save_policy_set<P: AsRef<Path>>(policy: &PolicySet, path: P) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(policy)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Ensure parent directory exists
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, json)
}

/// Get default policy path
pub fn default_policy_path() -> std::path::PathBuf {
    // Store in /var/lib/anna/ for system-wide policies
    std::path::PathBuf::from("/var/lib/anna/policy.json")
}

/// Load or create default policy
///
/// This is the main entry point for policy loading:
/// 1. Try to load from /var/lib/anna/policy.json
/// 2. If it doesn't exist, create it with default policy
/// 3. If loading fails, fall back to default policy (fail-safe)
pub fn load_or_create_policy() -> PolicySet {
    let path = default_policy_path();

    if path.exists() {
        load_policy_set(&path)
    } else {
        // Create default policy file
        let policy = PolicySet::default_policy();
        if let Err(e) = save_policy_set(&policy, &path) {
            eprintln!("Warning: Failed to save default policy: {}", e);
        }
        policy
    }
}

// ============================================================================
// Natural Language Policy Parser
// ============================================================================

/// Draft policy rule (before adding to PolicySet)
#[derive(Debug, Clone)]
pub struct PolicyRuleDraft {
    pub description: String,
    pub domain: PolicyDomain,
    pub max_risk: PolicyRiskLevel,
    pub path_globs: Vec<String>,
    pub tags: Vec<String>,
    pub action: PolicyAction,
}

/// Parse natural language policy command
///
/// Supports commands like:
/// - "never touch my ssh config"
/// - "you can change my vim config if I confirm"
/// - "don't modify /etc/pacman.conf"
/// - "require confirmation for package changes"
///
/// Returns None if the command is not recognized as a policy statement
pub fn parse_policy_command(input: &str) -> Option<PolicyRuleDraft> {
    let input_lower = input.to_lowercase();

    // Pattern 1: "never touch X" / "don't modify X" / "never change X"
    if input_lower.contains("never") || input_lower.contains("don't") {
        if let Some(target) = extract_target(&input_lower) {
            return Some(PolicyRuleDraft {
                description: format!("Never modify {}", target),
                domain: infer_domain(&target),
                max_risk: PolicyRiskLevel::High,
                path_globs: extract_paths(&target),
                tags: extract_tags(&target),
                action: PolicyAction::Deny,
            });
        }
    }

    // Pattern 2: "require confirmation for X" / "ask before X"
    if input_lower.contains("require confirmation") || input_lower.contains("ask before") || input_lower.contains("confirm") {
        if let Some(target) = extract_target(&input_lower) {
            // Check if "strong" confirmation is requested
            let action = if input_lower.contains("strong") {
                PolicyAction::RequireStrongConfirm
            } else {
                PolicyAction::RequireConfirm
            };

            return Some(PolicyRuleDraft {
                description: format!("Require confirmation for {}", target),
                domain: infer_domain(&target),
                max_risk: PolicyRiskLevel::Moderate,
                path_globs: extract_paths(&target),
                tags: extract_tags(&target),
                action,
            });
        }
    }

    // Pattern 3: "you can X" / "allow X"
    if input_lower.contains("you can") || input_lower.contains("allow") {
        if let Some(target) = extract_target(&input_lower) {
            return Some(PolicyRuleDraft {
                description: format!("Allow {}", target),
                domain: infer_domain(&target),
                max_risk: PolicyRiskLevel::High,
                path_globs: extract_paths(&target),
                tags: extract_tags(&target),
                action: PolicyAction::Allow,
            });
        }
    }

    None
}

/// Query policy rules by natural language
///
/// Supports queries like:
/// - "show your rules" / "list your rules"
/// - "what are your rules about ssh"
/// - "show rules for /etc"
///
/// Returns (should_show_all, filter_keyword)
pub fn parse_policy_query(input: &str) -> Option<(bool, Option<String>)> {
    let input_lower = input.to_lowercase();

    // Pattern: "show/list your rules"
    if (input_lower.contains("show") || input_lower.contains("list"))
        && (input_lower.contains("rule") || input_lower.contains("policy") || input_lower.contains("policies"))
    {
        // Check for filter keyword
        if input_lower.contains(" about ") {
            if let Some(keyword) = input_lower.split(" about ").nth(1) {
                return Some((false, Some(keyword.trim().to_string())));
            }
        } else if input_lower.contains(" for ") {
            if let Some(keyword) = input_lower.split(" for ").nth(1) {
                return Some((false, Some(keyword.trim().to_string())));
            }
        } else {
            return Some((true, None));
        }
    }

    None
}

/// Extract target from policy command
fn extract_target(input: &str) -> Option<String> {
    // Common patterns
    let patterns = [
        "my ", "the ", "touch ", "modify ", "change ", "changes to ", "for ",
        "about ", "to ", "in ",
    ];

    let mut remaining = input;
    for pattern in &patterns {
        if let Some(pos) = remaining.find(pattern) {
            remaining = &remaining[pos + pattern.len()..];
        }
    }

    if remaining.is_empty() {
        return None;
    }

    // Clean up and return
    Some(remaining.trim().to_string())
}

/// Infer domain from target description
fn infer_domain(target: &str) -> PolicyDomain {
    let target_lower = target.to_lowercase();

    if target_lower.contains("package") || target_lower.contains("pacman") || target_lower.contains("install") || target_lower.contains("remove") {
        PolicyDomain::Packages
    } else if target_lower.contains("service") || target_lower.contains("systemd") || target_lower.contains("daemon") {
        PolicyDomain::Services
    } else if target_lower.contains("network") || target_lower.contains("wifi") || target_lower.contains("ethernet") {
        PolicyDomain::Network
    } else if target_lower.contains("user") || target_lower.contains("account") {
        PolicyDomain::Users
    } else if target_lower.contains("file") || target_lower.contains("directory") || target_lower.contains("folder") {
        PolicyDomain::Filesystem
    } else if target_lower.contains("config") || target_lower.contains("/etc") || target_lower.contains("~/.") {
        PolicyDomain::Config
    } else {
        PolicyDomain::General
    }
}

/// Extract file paths from target description
fn extract_paths(target: &str) -> Vec<String> {
    let mut paths = Vec::new();

    // Look for absolute paths
    if target.starts_with('/') {
        // Extract the path (up to the first space or end)
        let path = target.split_whitespace().next().unwrap_or(target);
        paths.push(path.to_string());
    }

    // Look for home directory paths
    if target.contains("~/") {
        if let Some(start) = target.find("~/") {
            let remaining = &target[start..];
            let path = remaining.split_whitespace().next().unwrap_or(remaining);
            paths.push(path.to_string());
        }
    }

    // Look for /etc patterns
    if target.contains("/etc") {
        if let Some(start) = target.find("/etc") {
            let remaining = &target[start..];
            let path = remaining.split_whitespace().next().unwrap_or(remaining);
            paths.push(path.to_string());
        }
    }

    paths
}

/// Extract relevant tags from target description
fn extract_tags(target: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let target_lower = target.to_lowercase();

    // Common configuration file patterns
    let tag_patterns = [
        ("ssh", vec!["ssh", "network", "auth"]),
        ("vim", vec!["vim", "editor"]),
        ("pacman", vec!["pacman", "packages"]),
        ("systemd", vec!["systemd", "services"]),
        ("network", vec!["network"]),
        ("config", vec!["config"]),
    ];

    for (keyword, keyword_tags) in &tag_patterns {
        if target_lower.contains(keyword) {
            tags.extend(keyword_tags.iter().map(|s| s.to_string()));
        }
    }

    tags.sort();
    tags.dedup();
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_plan(
        domain: PolicyDomain,
        risk: PolicyRiskLevel,
        paths: Vec<&str>,
        tags: Vec<&str>,
    ) -> PlannedAction {
        PlannedAction {
            domain,
            risk_level: risk,
            target_paths: paths.iter().map(|s| s.to_string()).collect(),
            target_services: vec![],
            target_packages: vec![],
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_default_policy_exists() {
        let policy = PolicySet::default_policy();
        assert!(!policy.rules.is_empty());
        assert_eq!(policy.version, 1);
    }

    #[test]
    fn test_deny_high_risk_by_default() {
        let policy = PolicySet::default_policy();
        let plan = make_test_plan(PolicyDomain::Config, PolicyRiskLevel::High, vec![], vec![]);

        let decision = evaluate_plan_against_policy(&policy, &plan);
        assert!(!decision.allowed);
        assert!(!decision.notes.is_empty());
    }

    #[test]
    fn test_allow_safe_operations() {
        let policy = PolicySet::default_policy();
        let plan = make_test_plan(
            PolicyDomain::Config,
            PolicyRiskLevel::Safe,
            vec!["~/.vimrc"],
            vec![],
        );

        let decision = evaluate_plan_against_policy(&policy, &plan);
        assert!(decision.allowed);
    }

    #[test]
    fn test_deny_ssh_config_changes() {
        let policy = PolicySet::default_policy();
        let plan = make_test_plan(
            PolicyDomain::Config,
            PolicyRiskLevel::Moderate,
            vec!["/etc/ssh/sshd_config"],
            vec!["ssh"],
        );

        let decision = evaluate_plan_against_policy(&policy, &plan);
        assert!(!decision.allowed);
        assert!(decision.notes.iter().any(|n| n.contains("R-006")));
    }

    #[test]
    fn test_require_confirm_for_moderate_config() {
        let policy = PolicySet::default_policy();
        let plan = make_test_plan(
            PolicyDomain::Config,
            PolicyRiskLevel::Moderate,
            vec!["~/.vimrc"],
            vec!["vim"],
        );

        let decision = evaluate_plan_against_policy(&policy, &plan);
        assert!(decision.allowed);
        assert!(decision.require_confirm);
    }

    #[test]
    fn test_strong_confirm_for_etc_changes() {
        let policy = PolicySet::default_policy();
        let plan = make_test_plan(
            PolicyDomain::Config,
            PolicyRiskLevel::Moderate,
            vec!["/etc/anna/config.toml"],
            vec![],
        );

        let decision = evaluate_plan_against_policy(&policy, &plan);
        assert!(decision.allowed);
        assert!(decision.require_strong_confirm);
    }

    #[test]
    fn test_add_and_remove_rule() {
        let mut policy = PolicySet::default_policy();
        let initial_version = policy.version;
        let initial_count = policy.rules.len();

        let new_rule = PolicyRule {
            id: "R-TEST".to_string(),
            description: "Test rule".to_string(),
            domain: PolicyDomain::Config,
            max_risk: PolicyRiskLevel::Safe,
            path_globs: vec![],
            tags: vec![],
            action: PolicyAction::Allow,
            created_at: Utc::now(),
            source: PolicySource::System,
        };

        policy.add_rule(new_rule);
        assert_eq!(policy.rules.len(), initial_count + 1);
        assert_eq!(policy.version, initial_version + 1);

        let removed = policy.remove_rule("R-TEST");
        assert!(removed);
        assert_eq!(policy.rules.len(), initial_count);
        assert_eq!(policy.version, initial_version + 2);
    }

    #[test]
    fn test_rules_for_domain() {
        let policy = PolicySet::default_policy();
        let config_rules = policy.rules_for_domain(PolicyDomain::Config);
        assert!(!config_rules.is_empty());
    }

    #[test]
    fn test_rules_with_tag() {
        let policy = PolicySet::default_policy();
        let ssh_rules = policy.rules_with_tag("ssh");
        assert!(!ssh_rules.is_empty());
    }

    #[test]
    fn test_path_glob_matching() {
        assert!(path_matches_glob("/etc/ssh/sshd_config", "/etc/ssh/sshd_config"));
        assert!(path_matches_glob("/etc/ssh/sshd_config", "/etc/*"));
        assert!(path_matches_glob("/etc/pacman.conf", "/etc/*"));
        assert!(!path_matches_glob("/home/user/.vimrc", "/etc/*"));
    }

    #[test]
    fn test_deny_critical_system_files() {
        let policy = PolicySet::default_policy();
        let plan = make_test_plan(
            PolicyDomain::Config,
            PolicyRiskLevel::Moderate,
            vec!["/etc/shadow"],
            vec![],
        );

        let decision = evaluate_plan_against_policy(&policy, &plan);
        assert!(!decision.allowed);
        assert!(decision.notes.iter().any(|n| n.contains("R-007")));
    }

    // ============================================================
    // Integration Tests (v6.52.0)
    // ============================================================

    #[test]
    fn integration_test_policy_persistence() {
        use std::fs;
        let temp_dir = std::env::temp_dir();
        let policy_path = temp_dir.join("test_policy.json");

        // Clean up any existing file
        let _ = fs::remove_file(&policy_path);

        // Create and save policy
        let mut policy = PolicySet::default_policy();
        policy.add_rule(PolicyRule {
            id: "R-TEST".to_string(),
            description: "Test rule".to_string(),
            domain: PolicyDomain::Network,
            max_risk: PolicyRiskLevel::Moderate,
            path_globs: vec![],
            tags: vec![],
            action: PolicyAction::Deny,
            created_at: chrono::Utc::now(),
            source: PolicySource::Default,
        });

        save_policy_set(&policy, &policy_path).expect("Failed to save policy");

        // Load and verify
        let loaded = load_policy_set(&policy_path);
        assert_eq!(loaded.version, policy.version);
        assert!(loaded.rules.iter().any(|r| r.id == "R-TEST"));

        // Clean up
        let _ = fs::remove_file(&policy_path);
    }

    #[test]
    fn integration_test_natural_language_policy_parser() {
        // Test denial parsing
        let cmd = parse_policy_command("never touch my ssh config");
        assert!(cmd.is_some());
        let draft = cmd.unwrap();
        assert_eq!(draft.action, PolicyAction::Deny);
        assert!(draft.tags.contains(&"ssh".to_string()));

        // Test approval parsing
        let cmd2 = parse_policy_command("always allow user config changes");
        assert!(cmd2.is_some());
        let draft2 = cmd2.unwrap();
        assert_eq!(draft2.action, PolicyAction::Allow);

        // Test query parsing
        let query = parse_policy_query("show your rules about ssh");
        assert!(query.is_some());
        let (list_all, topic) = query.unwrap();
        assert!(!list_all);
        assert_eq!(topic, Some("ssh".to_string()));
    }

    #[test]
    fn integration_test_planner_policy_integration() {
        use crate::planner_core::{CommandPlan, PlannedCommand, StepRiskLevel};

        let policy = PolicySet::default_policy();

        // Create a plan with an SSH config command
        let mut plan = CommandPlan {
            commands: vec![PlannedCommand {
                command: "sed".to_string(),
                args: vec!["-i".to_string(), "s/PermitRootLogin yes/PermitRootLogin no/".to_string(), "/etc/ssh/sshd_config".to_string()],
                purpose: "Disable root login in SSH".to_string(),
                requires_tools: vec!["sed".to_string()],
                risk_level: StepRiskLevel::Medium,
                writes_files: true,
                requires_root: true,
                expected_outcome: Some("Root login disabled".to_string()),
                validation_hint: Some("Check sshd_config".to_string()),
            }],
            safety_level: crate::planner_core::SafetyLevel::Risky,
            fallbacks: vec![],
            expected_output: String::new(),
            reasoning: "Disable root SSH login for security".to_string(),
            goal_description: Some("Secure SSH configuration".to_string()),
            assumptions: vec![],
            confidence: 0.8,
            policy_decision: None,
        };

        // Check against policy
        let decision = plan.check_against_policy(&policy);

        // Should have a decision
        assert!(plan.policy_decision.is_some());

        // SSH config changes should be blocked (by R-002 or other high-risk rules)
        assert!(!decision.allowed);
        assert!(!decision.matched_rules.is_empty());
    }

    #[test]
    fn integration_test_executor_policy_enforcement() {
        use crate::executor_core::execute_plan;
        use crate::planner_core::{CommandPlan, SafetyLevel};

        // Create a plan that's blocked by policy
        let policy = PolicySet::default_policy();
        let mut plan = CommandPlan {
            commands: vec![],
            safety_level: SafetyLevel::ReadOnly,
            fallbacks: vec![],
            expected_output: String::new(),
            reasoning: "Test".to_string(),
            goal_description: None,
            assumptions: vec![],
            confidence: 0.8,
            policy_decision: None,
        };

        // Add a blocking policy decision
        plan.check_against_policy(&policy);
        if let Some(ref mut decision) = plan.policy_decision {
            decision.allowed = false;
            decision.notes.push("Test denial".to_string());
        }

        // Executor should refuse to execute
        let result = execute_plan(&plan);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Policy denial"));
    }

    #[test]
    fn integration_test_episode_policy_metadata() {
        use crate::action_episodes::EpisodeBuilder;

        let policy = PolicySet::default_policy();
        let plan = make_test_plan(
            PolicyDomain::Config,
            PolicyRiskLevel::Moderate,
            vec!["/etc/ssh/sshd_config"],
            vec!["ssh"],
        );

        let decision = evaluate_plan_against_policy(&policy, &plan);

        // Create episode with policy metadata
        let episode = EpisodeBuilder::new("Configure SSH")
            .with_policy_decision(decision.clone())
            .build();

        assert!(episode.blocked_by_policy);
        assert!(episode.policy_decision.is_some());

        let stored_decision = episode.policy_decision.unwrap();
        assert_eq!(stored_decision.allowed, decision.allowed);
        assert_eq!(stored_decision.matched_rules, decision.matched_rules);
    }

    #[test]
    fn integration_test_change_journal_policy_queries() {
        use crate::change_journal::{EpisodeSummary, EpisodeFilter};
        use crate::action_episodes::{EpisodeBuilder, ActionEpisode};

        let policy = PolicySet::default_policy();
        let plan = make_test_plan(
            PolicyDomain::Config,
            PolicyRiskLevel::High,
            vec!["/boot/grub/grub.cfg"],
            vec!["boot"],
        );

        let decision = evaluate_plan_against_policy(&policy, &plan);

        // Create episode blocked by policy
        let episode = EpisodeBuilder::new("Modify bootloader")
            .with_policy_decision(decision)
            .build();

        // Create summary
        let summary = EpisodeSummary::from_episode(&episode);

        assert!(summary.blocked_by_policy);
        assert!(summary.policy_denial_reason.is_some());

        // Test filter
        let filter = EpisodeFilter {
            blocked_by_policy_only: true,
            ..Default::default()
        };

        // Filter should have the field (compilation test)
        assert!(filter.blocked_by_policy_only);
    }
}
