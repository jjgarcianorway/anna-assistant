//! Policy Engine v0.0.14
//!
//! Local-first policy system that controls what Anna may do:
//! - capabilities.toml: which tools are enabled
//! - risk.toml: risk levels, confirmations, thresholds
//! - blocked.toml: blocked packages/services/paths
//! - helpers.toml: helper definitions and tracking rules
//!
//! All major allow/deny decisions are policy-driven, not hardcoded.
//! Policy reads are evidence sources with Evidence IDs (POL#####).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::RwLock;

/// Policy directory
pub const POLICY_DIR: &str = "/etc/anna/policy";

/// Individual policy files
pub const CAPABILITIES_FILE: &str = "/etc/anna/policy/capabilities.toml";
pub const RISK_FILE: &str = "/etc/anna/policy/risk.toml";
pub const BLOCKED_FILE: &str = "/etc/anna/policy/blocked.toml";
pub const HELPERS_FILE: &str = "/etc/anna/policy/helpers.toml";

/// Policy schema version
pub const POLICY_SCHEMA_VERSION: u32 = 1;

/// Evidence ID prefix for policy reads
pub const POLICY_EVIDENCE_PREFIX: &str = "POL";

/// Generate a unique policy evidence ID
pub fn generate_policy_evidence_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{}{:05}", POLICY_EVIDENCE_PREFIX, ts % 100000)
}

// =============================================================================
// Capabilities Policy
// =============================================================================

/// Capabilities policy - which tools are enabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesPolicy {
    /// Schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Read-only tools configuration
    #[serde(default)]
    pub read_only_tools: ReadOnlyToolsConfig,

    /// Mutation tools configuration
    #[serde(default)]
    pub mutation_tools: MutationToolsConfig,

    /// Global tool settings
    #[serde(default)]
    pub global: GlobalToolSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadOnlyToolsConfig {
    /// Whether read-only tools are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Specific tools to disable (allowlist approach: all enabled except these)
    #[serde(default)]
    pub disabled_tools: Vec<String>,

    /// Maximum evidence bytes per tool result
    #[serde(default = "default_max_evidence_bytes")]
    pub max_evidence_bytes: usize,
}

impl Default for ReadOnlyToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            disabled_tools: Vec::new(),
            max_evidence_bytes: default_max_evidence_bytes(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationToolsConfig {
    /// Whether mutation tools are enabled at all
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Specific mutation tools to disable
    #[serde(default)]
    pub disabled_tools: Vec<String>,

    /// File edit settings
    #[serde(default)]
    pub file_edit: FileEditPolicy,

    /// Systemd settings
    #[serde(default)]
    pub systemd: SystemdPolicy,

    /// Package management settings
    #[serde(default)]
    pub packages: PackagePolicy,
}

impl Default for MutationToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            disabled_tools: Vec::new(),
            file_edit: FileEditPolicy::default(),
            systemd: SystemdPolicy::default(),
            packages: PackagePolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditPolicy {
    /// Whether file editing is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Allowed path prefixes for editing
    #[serde(default = "default_allowed_paths")]
    pub allowed_paths: Vec<String>,

    /// Explicitly blocked paths (takes precedence)
    #[serde(default = "default_blocked_paths")]
    pub blocked_paths: Vec<String>,

    /// Maximum file size in bytes
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: u64,

    /// Only allow text files
    #[serde(default = "default_true")]
    pub text_only: bool,
}

impl Default for FileEditPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_paths: default_allowed_paths(),
            blocked_paths: default_blocked_paths(),
            max_file_size_bytes: default_max_file_size(),
            text_only: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdPolicy {
    /// Whether systemd operations are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Allowed operations
    #[serde(default = "default_systemd_operations")]
    pub allowed_operations: Vec<String>,

    /// Blocked units (never touch these)
    #[serde(default = "default_blocked_units")]
    pub blocked_units: Vec<String>,

    /// Protected units (require high-risk confirmation)
    #[serde(default = "default_protected_units")]
    pub protected_units: Vec<String>,
}

impl Default for SystemdPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_operations: default_systemd_operations(),
            blocked_units: default_blocked_units(),
            protected_units: default_protected_units(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePolicy {
    /// Whether package operations are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum packages per operation
    #[serde(default = "default_max_packages")]
    pub max_packages_per_operation: u32,

    /// Blocked package patterns (kernel, bootloader, etc.)
    #[serde(default = "default_blocked_packages")]
    pub blocked_patterns: Vec<String>,

    /// Protected packages (require high-risk confirmation)
    #[serde(default = "default_protected_packages")]
    pub protected_patterns: Vec<String>,

    /// Categories of packages that are blocked
    #[serde(default = "default_blocked_categories")]
    pub blocked_categories: Vec<String>,
}

impl Default for PackagePolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_packages_per_operation: default_max_packages(),
            blocked_patterns: default_blocked_packages(),
            protected_patterns: default_protected_packages(),
            blocked_categories: default_blocked_categories(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalToolSettings {
    /// Timeout for tool execution in milliseconds
    #[serde(default = "default_tool_timeout")]
    pub timeout_ms: u64,

    /// Whether to log all tool calls
    #[serde(default = "default_true")]
    pub audit_logging: bool,
}

impl Default for GlobalToolSettings {
    fn default() -> Self {
        Self {
            timeout_ms: default_tool_timeout(),
            audit_logging: true,
        }
    }
}

impl Default for CapabilitiesPolicy {
    fn default() -> Self {
        Self {
            schema_version: POLICY_SCHEMA_VERSION,
            read_only_tools: ReadOnlyToolsConfig::default(),
            mutation_tools: MutationToolsConfig::default(),
            global: GlobalToolSettings::default(),
        }
    }
}

// =============================================================================
// Risk Policy
// =============================================================================

/// Risk policy - risk levels, confirmations, thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPolicy {
    /// Schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Risk level definitions
    #[serde(default)]
    pub levels: RiskLevels,

    /// Confirmation settings
    #[serde(default)]
    pub confirmations: ConfirmationSettings,

    /// Thresholds
    #[serde(default)]
    pub thresholds: RiskThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLevels {
    /// Read-only operations
    #[serde(default)]
    pub read_only: RiskLevelConfig,

    /// Low-risk operations
    #[serde(default)]
    pub low: RiskLevelConfig,

    /// Medium-risk operations
    #[serde(default)]
    pub medium: RiskLevelConfig,

    /// High-risk operations
    #[serde(default)]
    pub high: RiskLevelConfig,
}

impl Default for RiskLevels {
    fn default() -> Self {
        Self {
            read_only: RiskLevelConfig {
                requires_confirmation: false,
                confirmation_phrase: String::new(),
                min_reliability_score: 0,
                description: "Safe observation only".to_string(),
            },
            low: RiskLevelConfig {
                requires_confirmation: true,
                confirmation_phrase: "y".to_string(),
                min_reliability_score: 50,
                description: "Reversible, local changes".to_string(),
            },
            medium: RiskLevelConfig {
                requires_confirmation: true,
                confirmation_phrase: "I CONFIRM (medium risk)".to_string(),
                min_reliability_score: 70,
                description: "Config edits, service restarts, installs".to_string(),
            },
            high: RiskLevelConfig {
                requires_confirmation: true,
                confirmation_phrase: "I assume the risk".to_string(),
                min_reliability_score: 85,
                description: "Destructive, potentially irreversible".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskLevelConfig {
    /// Whether confirmation is required
    pub requires_confirmation: bool,

    /// Exact phrase required (empty = any confirmation)
    pub confirmation_phrase: String,

    /// Minimum Junior reliability score
    pub min_reliability_score: u8,

    /// Description for users
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationSettings {
    /// Confirmation phrase for forget/delete operations
    #[serde(default = "default_forget_confirmation")]
    pub forget_phrase: String,

    /// Confirmation phrase for reset operations
    #[serde(default = "default_reset_confirmation")]
    pub reset_phrase: String,

    /// Confirmation phrase for uninstall operations
    #[serde(default = "default_uninstall_confirmation")]
    pub uninstall_phrase: String,

    /// Timeout for confirmation in seconds
    #[serde(default = "default_confirmation_timeout")]
    pub timeout_seconds: u64,
}

impl Default for ConfirmationSettings {
    fn default() -> Self {
        Self {
            forget_phrase: default_forget_confirmation(),
            reset_phrase: default_reset_confirmation(),
            uninstall_phrase: default_uninstall_confirmation(),
            timeout_seconds: default_confirmation_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    /// Minimum reliability for any mutation
    #[serde(default = "default_min_mutation_reliability")]
    pub min_mutation_reliability: u8,

    /// Minimum reliability for package operations
    #[serde(default = "default_min_package_reliability")]
    pub min_package_reliability: u8,

    /// Maximum concurrent mutations
    #[serde(default = "default_max_concurrent_mutations")]
    pub max_concurrent_mutations: u32,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            min_mutation_reliability: default_min_mutation_reliability(),
            min_package_reliability: default_min_package_reliability(),
            max_concurrent_mutations: default_max_concurrent_mutations(),
        }
    }
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            schema_version: POLICY_SCHEMA_VERSION,
            levels: RiskLevels::default(),
            confirmations: ConfirmationSettings::default(),
            thresholds: RiskThresholds::default(),
        }
    }
}

// =============================================================================
// Blocked Policy
// =============================================================================

/// Blocked policy - explicitly blocked items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedPolicy {
    /// Schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Blocked packages
    #[serde(default)]
    pub packages: BlockedPackages,

    /// Blocked services
    #[serde(default)]
    pub services: BlockedServices,

    /// Blocked paths
    #[serde(default)]
    pub paths: BlockedPaths,

    /// Blocked commands
    #[serde(default)]
    pub commands: BlockedCommands,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockedPackages {
    /// Exact package names
    #[serde(default)]
    pub exact: Vec<String>,

    /// Pattern matches (glob-style)
    #[serde(default)]
    pub patterns: Vec<String>,

    /// Blocked categories with reasons
    #[serde(default)]
    pub categories: Vec<BlockedCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedCategory {
    /// Category name
    pub name: String,
    /// Reason for blocking
    pub reason: String,
    /// Package patterns in this category
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockedServices {
    /// Exact service names
    #[serde(default)]
    pub exact: Vec<String>,

    /// Pattern matches
    #[serde(default)]
    pub patterns: Vec<String>,

    /// Critical system services (never touch)
    #[serde(default = "default_critical_services")]
    pub critical: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockedPaths {
    /// Exact paths
    #[serde(default)]
    pub exact: Vec<String>,

    /// Path prefixes
    #[serde(default = "default_blocked_path_prefixes")]
    pub prefixes: Vec<String>,

    /// Path patterns (glob-style)
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockedCommands {
    /// Blocked shell commands
    #[serde(default)]
    pub exact: Vec<String>,

    /// Blocked command patterns
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl Default for BlockedPolicy {
    fn default() -> Self {
        Self {
            schema_version: POLICY_SCHEMA_VERSION,
            packages: BlockedPackages {
                exact: vec![],
                patterns: vec![],
                categories: vec![
                    BlockedCategory {
                        name: "kernel".to_string(),
                        reason: "Kernel modifications require manual intervention".to_string(),
                        patterns: vec!["linux".to_string(), "linux-*".to_string(), "kernel*".to_string()],
                    },
                    BlockedCategory {
                        name: "bootloader".to_string(),
                        reason: "Bootloader changes can render system unbootable".to_string(),
                        patterns: vec!["grub".to_string(), "systemd-boot".to_string(), "refind".to_string(), "syslinux".to_string()],
                    },
                    BlockedCategory {
                        name: "init".to_string(),
                        reason: "Init system changes are critical".to_string(),
                        patterns: vec!["systemd".to_string(), "openrc".to_string(), "runit".to_string()],
                    },
                ],
            },
            services: BlockedServices {
                exact: vec![],
                patterns: vec![],
                critical: default_critical_services(),
            },
            paths: BlockedPaths {
                exact: vec![],
                prefixes: default_blocked_path_prefixes(),
                patterns: vec![],
            },
            commands: BlockedCommands::default(),
        }
    }
}

// =============================================================================
// Helpers Policy
// =============================================================================

/// Helpers policy - what counts as a helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpersPolicy {
    /// Schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Helper definitions
    #[serde(default)]
    pub definitions: Vec<HelperDefinition>,

    /// Tracking settings
    #[serde(default)]
    pub tracking: HelperTrackingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperDefinition {
    /// Package name
    pub package: String,

    /// Why this is a helper
    pub purpose: String,

    /// Category (instrumentation, diagnostics, etc.)
    pub category: String,

    /// Whether it's optional
    #[serde(default)]
    pub optional: bool,

    /// Commands it provides
    #[serde(default)]
    pub provides_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperTrackingSettings {
    /// Whether to track helper installations
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether to offer helper removal on uninstall
    #[serde(default = "default_true")]
    pub offer_removal_on_uninstall: bool,

    /// State file location
    #[serde(default = "default_helpers_state_file")]
    pub state_file: String,
}

impl Default for HelperTrackingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            offer_removal_on_uninstall: true,
            state_file: default_helpers_state_file(),
        }
    }
}

impl Default for HelpersPolicy {
    fn default() -> Self {
        Self {
            schema_version: POLICY_SCHEMA_VERSION,
            definitions: vec![
                HelperDefinition {
                    package: "smartmontools".to_string(),
                    purpose: "SMART disk health monitoring".to_string(),
                    category: "instrumentation".to_string(),
                    optional: true,
                    provides_commands: vec!["smartctl".to_string()],
                },
                HelperDefinition {
                    package: "nvme-cli".to_string(),
                    purpose: "NVMe drive management".to_string(),
                    category: "instrumentation".to_string(),
                    optional: true,
                    provides_commands: vec!["nvme".to_string()],
                },
                HelperDefinition {
                    package: "lm_sensors".to_string(),
                    purpose: "Hardware sensor monitoring".to_string(),
                    category: "instrumentation".to_string(),
                    optional: true,
                    provides_commands: vec!["sensors".to_string()],
                },
                HelperDefinition {
                    package: "iw".to_string(),
                    purpose: "Wireless interface management".to_string(),
                    category: "diagnostics".to_string(),
                    optional: true,
                    provides_commands: vec!["iw".to_string()],
                },
                HelperDefinition {
                    package: "ethtool".to_string(),
                    purpose: "Ethernet device diagnostics".to_string(),
                    category: "diagnostics".to_string(),
                    optional: true,
                    provides_commands: vec!["ethtool".to_string()],
                },
            ],
            tracking: HelperTrackingSettings::default(),
        }
    }
}

// =============================================================================
// Combined Policy
// =============================================================================

/// Combined policy from all files
#[derive(Debug, Clone)]
pub struct Policy {
    /// Capabilities policy
    pub capabilities: CapabilitiesPolicy,

    /// Risk policy
    pub risk: RiskPolicy,

    /// Blocked policy
    pub blocked: BlockedPolicy,

    /// Helpers policy
    pub helpers: HelpersPolicy,

    /// When loaded
    pub loaded_at: DateTime<Utc>,

    /// File checksums for validation
    pub checksums: HashMap<String, String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            capabilities: CapabilitiesPolicy::default(),
            risk: RiskPolicy::default(),
            blocked: BlockedPolicy::default(),
            helpers: HelpersPolicy::default(),
            loaded_at: Utc::now(),
            checksums: HashMap::new(),
        }
    }
}

/// Policy validation result
#[derive(Debug, Clone)]
pub struct PolicyValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Policy {
    /// Load policy from files
    pub fn load() -> Result<Self, PolicyError> {
        let mut policy = Policy::default();
        let mut checksums = HashMap::new();

        // Load capabilities
        if Path::new(CAPABILITIES_FILE).exists() {
            let content = fs::read_to_string(CAPABILITIES_FILE)
                .map_err(|e| PolicyError::IoError(CAPABILITIES_FILE.to_string(), e.to_string()))?;
            checksums.insert(CAPABILITIES_FILE.to_string(), compute_checksum(&content));
            policy.capabilities = toml::from_str(&content)
                .map_err(|e| PolicyError::ParseError(CAPABILITIES_FILE.to_string(), e.to_string()))?;
        }

        // Load risk
        if Path::new(RISK_FILE).exists() {
            let content = fs::read_to_string(RISK_FILE)
                .map_err(|e| PolicyError::IoError(RISK_FILE.to_string(), e.to_string()))?;
            checksums.insert(RISK_FILE.to_string(), compute_checksum(&content));
            policy.risk = toml::from_str(&content)
                .map_err(|e| PolicyError::ParseError(RISK_FILE.to_string(), e.to_string()))?;
        }

        // Load blocked
        if Path::new(BLOCKED_FILE).exists() {
            let content = fs::read_to_string(BLOCKED_FILE)
                .map_err(|e| PolicyError::IoError(BLOCKED_FILE.to_string(), e.to_string()))?;
            checksums.insert(BLOCKED_FILE.to_string(), compute_checksum(&content));
            policy.blocked = toml::from_str(&content)
                .map_err(|e| PolicyError::ParseError(BLOCKED_FILE.to_string(), e.to_string()))?;
        }

        // Load helpers
        if Path::new(HELPERS_FILE).exists() {
            let content = fs::read_to_string(HELPERS_FILE)
                .map_err(|e| PolicyError::IoError(HELPERS_FILE.to_string(), e.to_string()))?;
            checksums.insert(HELPERS_FILE.to_string(), compute_checksum(&content));
            policy.helpers = toml::from_str(&content)
                .map_err(|e| PolicyError::ParseError(HELPERS_FILE.to_string(), e.to_string()))?;
        }

        policy.checksums = checksums;
        policy.loaded_at = Utc::now();

        // Validate
        let validation = policy.validate();
        if !validation.valid {
            return Err(PolicyError::ValidationError(validation.errors.join("; ")));
        }

        Ok(policy)
    }

    /// Validate policy
    pub fn validate(&self) -> PolicyValidation {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check schema versions
        if self.capabilities.schema_version != POLICY_SCHEMA_VERSION {
            warnings.push(format!(
                "capabilities.toml schema version {} differs from expected {}",
                self.capabilities.schema_version, POLICY_SCHEMA_VERSION
            ));
        }

        // Check confirmation phrases aren't empty for required confirmations
        if self.risk.levels.medium.requires_confirmation && self.risk.levels.medium.confirmation_phrase.is_empty() {
            errors.push("Medium risk requires confirmation but phrase is empty".to_string());
        }

        if self.risk.levels.high.requires_confirmation && self.risk.levels.high.confirmation_phrase.is_empty() {
            errors.push("High risk requires confirmation but phrase is empty".to_string());
        }

        // Check reliability thresholds are valid
        if self.risk.thresholds.min_mutation_reliability > 100 {
            errors.push("min_mutation_reliability cannot exceed 100".to_string());
        }

        PolicyValidation {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Check if a tool is enabled
    pub fn is_tool_enabled(&self, tool_name: &str, is_mutation: bool) -> bool {
        if is_mutation {
            if !self.capabilities.mutation_tools.enabled {
                return false;
            }
            !self.capabilities.mutation_tools.disabled_tools.contains(&tool_name.to_string())
        } else {
            if !self.capabilities.read_only_tools.enabled {
                return false;
            }
            !self.capabilities.read_only_tools.disabled_tools.contains(&tool_name.to_string())
        }
    }

    /// Check if a path is allowed for editing
    pub fn is_path_allowed(&self, path: &str) -> PolicyCheckResult {
        let evidence_id = generate_policy_evidence_id();

        // Check blocked paths first
        for blocked in &self.blocked.paths.exact {
            if path == blocked {
                return PolicyCheckResult::blocked(
                    &format!("Path '{}' is explicitly blocked in blocked.toml", path),
                    &evidence_id,
                    "blocked.toml:paths.exact",
                );
            }
        }

        for prefix in &self.blocked.paths.prefixes {
            if path.starts_with(prefix) {
                return PolicyCheckResult::blocked(
                    &format!("Path '{}' matches blocked prefix '{}' in blocked.toml", path, prefix),
                    &evidence_id,
                    "blocked.toml:paths.prefixes",
                );
            }
        }

        // Check capabilities allowed paths
        if !self.capabilities.mutation_tools.file_edit.enabled {
            return PolicyCheckResult::blocked(
                "File editing is disabled in capabilities.toml",
                &evidence_id,
                "capabilities.toml:mutation_tools.file_edit.enabled",
            );
        }

        let allowed = self.capabilities.mutation_tools.file_edit.allowed_paths
            .iter()
            .any(|allowed| path.starts_with(allowed));

        if !allowed {
            return PolicyCheckResult::blocked(
                &format!("Path '{}' is not in allowed paths list in capabilities.toml", path),
                &evidence_id,
                "capabilities.toml:mutation_tools.file_edit.allowed_paths",
            );
        }

        // Check blocked in capabilities
        for blocked in &self.capabilities.mutation_tools.file_edit.blocked_paths {
            if path.starts_with(blocked) {
                return PolicyCheckResult::blocked(
                    &format!("Path '{}' matches blocked path '{}' in capabilities.toml", path, blocked),
                    &evidence_id,
                    "capabilities.toml:mutation_tools.file_edit.blocked_paths",
                );
            }
        }

        PolicyCheckResult::allowed(&evidence_id)
    }

    /// Check if a package is allowed for installation/removal
    pub fn is_package_allowed(&self, package: &str) -> PolicyCheckResult {
        let evidence_id = generate_policy_evidence_id();

        // Check exact blocks
        if self.blocked.packages.exact.contains(&package.to_string()) {
            return PolicyCheckResult::blocked(
                &format!("Package '{}' is explicitly blocked in blocked.toml", package),
                &evidence_id,
                "blocked.toml:packages.exact",
            );
        }

        // Check patterns
        for pattern in &self.blocked.packages.patterns {
            if matches_pattern(package, pattern) {
                return PolicyCheckResult::blocked(
                    &format!("Package '{}' matches blocked pattern '{}' in blocked.toml", package, pattern),
                    &evidence_id,
                    "blocked.toml:packages.patterns",
                );
            }
        }

        // Check categories
        for category in &self.blocked.packages.categories {
            for pattern in &category.patterns {
                if matches_pattern(package, pattern) {
                    return PolicyCheckResult::blocked(
                        &format!("Package '{}' is in blocked category '{}': {} [blocked.toml]",
                            package, category.name, category.reason),
                        &evidence_id,
                        &format!("blocked.toml:packages.categories.{}", category.name),
                    );
                }
            }
        }

        // Check capabilities patterns
        for pattern in &self.capabilities.mutation_tools.packages.blocked_patterns {
            if matches_pattern(package, pattern) {
                return PolicyCheckResult::blocked(
                    &format!("Package '{}' matches blocked pattern '{}' in capabilities.toml", package, pattern),
                    &evidence_id,
                    "capabilities.toml:mutation_tools.packages.blocked_patterns",
                );
            }
        }

        PolicyCheckResult::allowed(&evidence_id)
    }

    /// Check if a service is allowed for operations
    pub fn is_service_allowed(&self, service: &str) -> PolicyCheckResult {
        let evidence_id = generate_policy_evidence_id();

        // Check critical services
        for critical in &self.blocked.services.critical {
            if service == critical || service.starts_with(&format!("{}.", critical)) {
                return PolicyCheckResult::blocked(
                    &format!("Service '{}' is a critical system service [blocked.toml]", service),
                    &evidence_id,
                    "blocked.toml:services.critical",
                );
            }
        }

        // Check exact blocks
        if self.blocked.services.exact.contains(&service.to_string()) {
            return PolicyCheckResult::blocked(
                &format!("Service '{}' is explicitly blocked in blocked.toml", service),
                &evidence_id,
                "blocked.toml:services.exact",
            );
        }

        // Check blocked units in capabilities
        for blocked in &self.capabilities.mutation_tools.systemd.blocked_units {
            if service == blocked || matches_pattern(service, blocked) {
                return PolicyCheckResult::blocked(
                    &format!("Service '{}' matches blocked unit '{}' in capabilities.toml", service, blocked),
                    &evidence_id,
                    "capabilities.toml:mutation_tools.systemd.blocked_units",
                );
            }
        }

        PolicyCheckResult::allowed(&evidence_id)
    }

    /// Check if a systemd operation is allowed
    pub fn is_systemd_operation_allowed(&self, operation: &str) -> PolicyCheckResult {
        let evidence_id = generate_policy_evidence_id();

        if !self.capabilities.mutation_tools.systemd.enabled {
            return PolicyCheckResult::blocked(
                "Systemd operations are disabled in capabilities.toml",
                &evidence_id,
                "capabilities.toml:mutation_tools.systemd.enabled",
            );
        }

        if !self.capabilities.mutation_tools.systemd.allowed_operations.contains(&operation.to_string()) {
            return PolicyCheckResult::blocked(
                &format!("Systemd operation '{}' is not in allowed operations list", operation),
                &evidence_id,
                "capabilities.toml:mutation_tools.systemd.allowed_operations",
            );
        }

        PolicyCheckResult::allowed(&evidence_id)
    }

    /// Get confirmation phrase for a risk level
    pub fn get_confirmation_phrase(&self, risk: &str) -> Option<&str> {
        match risk {
            "read_only" => None,
            "low" => {
                if self.risk.levels.low.requires_confirmation {
                    Some(&self.risk.levels.low.confirmation_phrase)
                } else {
                    None
                }
            }
            "medium" => {
                if self.risk.levels.medium.requires_confirmation {
                    Some(&self.risk.levels.medium.confirmation_phrase)
                } else {
                    None
                }
            }
            "high" => {
                if self.risk.levels.high.requires_confirmation {
                    Some(&self.risk.levels.high.confirmation_phrase)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get minimum reliability for a risk level
    pub fn get_min_reliability(&self, risk: &str) -> u8 {
        match risk {
            "read_only" => self.risk.levels.read_only.min_reliability_score,
            "low" => self.risk.levels.low.min_reliability_score,
            "medium" => self.risk.levels.medium.min_reliability_score,
            "high" => self.risk.levels.high.min_reliability_score,
            _ => 0,
        }
    }

    /// Get max file size for edits
    pub fn get_max_file_size(&self) -> u64 {
        self.capabilities.mutation_tools.file_edit.max_file_size_bytes
    }

    /// Get max packages per operation
    pub fn get_max_packages(&self) -> u32 {
        self.capabilities.mutation_tools.packages.max_packages_per_operation
    }

    /// Check if a service requires elevated confirmation
    pub fn is_protected_service(&self, service: &str) -> bool {
        self.capabilities.mutation_tools.systemd.protected_units
            .iter()
            .any(|p| service == p || matches_pattern(service, p))
    }

    /// Check if a package requires elevated confirmation
    pub fn is_protected_package(&self, package: &str) -> bool {
        self.capabilities.mutation_tools.packages.protected_patterns
            .iter()
            .any(|p| matches_pattern(package, p))
    }
}

/// Result of a policy check
#[derive(Debug, Clone)]
pub struct PolicyCheckResult {
    pub allowed: bool,
    pub reason: String,
    pub evidence_id: String,
    pub policy_rule: String,
}

impl PolicyCheckResult {
    pub fn allowed(evidence_id: &str) -> Self {
        Self {
            allowed: true,
            reason: String::new(),
            evidence_id: evidence_id.to_string(),
            policy_rule: String::new(),
        }
    }

    pub fn blocked(reason: &str, evidence_id: &str, rule: &str) -> Self {
        Self {
            allowed: false,
            reason: reason.to_string(),
            evidence_id: evidence_id.to_string(),
            policy_rule: rule.to_string(),
        }
    }

    /// Format as evidence citation
    pub fn format_citation(&self) -> String {
        if self.allowed {
            format!("[{}]", self.evidence_id)
        } else {
            format!("[{}] Policy: {} ({})", self.evidence_id, self.reason, self.policy_rule)
        }
    }
}

/// Policy error types
#[derive(Debug, Clone)]
pub enum PolicyError {
    IoError(String, String),
    ParseError(String, String),
    ValidationError(String),
    NotFound(String),
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::IoError(file, err) => write!(f, "Failed to read {}: {}", file, err),
            PolicyError::ParseError(file, err) => write!(f, "Failed to parse {}: {}", file, err),
            PolicyError::ValidationError(err) => write!(f, "Policy validation failed: {}", err),
            PolicyError::NotFound(file) => write!(f, "Policy file not found: {}", file),
        }
    }
}

impl std::error::Error for PolicyError {}

// =============================================================================
// Global Policy Cache
// =============================================================================

lazy_static::lazy_static! {
    static ref POLICY_CACHE: RwLock<Option<Policy>> = RwLock::new(None);
}

/// Get the current policy (cached)
pub fn get_policy() -> Policy {
    {
        let cache = POLICY_CACHE.read().unwrap();
        if let Some(ref policy) = *cache {
            return policy.clone();
        }
    }

    // Load and cache
    let policy = Policy::load().unwrap_or_default();
    {
        let mut cache = POLICY_CACHE.write().unwrap();
        *cache = Some(policy.clone());
    }
    policy
}

/// Reload policy from files
pub fn reload_policy() -> Result<Policy, PolicyError> {
    let policy = Policy::load()?;
    {
        let mut cache = POLICY_CACHE.write().unwrap();
        *cache = Some(policy.clone());
    }
    Ok(policy)
}

/// Clear policy cache (forces reload on next access)
pub fn clear_policy_cache() {
    let mut cache = POLICY_CACHE.write().unwrap();
    *cache = None;
}

// =============================================================================
// Helper Functions
// =============================================================================

fn compute_checksum(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn matches_pattern(text: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        // Simple glob matching
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 1 {
            return text == pattern;
        }

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if i == 0 {
                // Must start with first part
                if !text.starts_with(part) {
                    return false;
                }
                pos = part.len();
            } else if i == parts.len() - 1 {
                // Must end with last part
                if !text.ends_with(part) {
                    return false;
                }
            } else {
                // Must contain middle part
                if let Some(found) = text[pos..].find(part) {
                    pos += found + part.len();
                } else {
                    return false;
                }
            }
        }
        true
    } else {
        text == pattern
    }
}

// =============================================================================
// Default Value Functions
// =============================================================================

fn default_schema_version() -> u32 { POLICY_SCHEMA_VERSION }
fn default_true() -> bool { true }
fn default_max_evidence_bytes() -> usize { 8192 }
fn default_max_file_size() -> u64 { 1_048_576 } // 1 MiB
fn default_tool_timeout() -> u64 { 30_000 } // 30 seconds
fn default_max_packages() -> u32 { 5 }
fn default_min_mutation_reliability() -> u8 { 70 }
fn default_min_package_reliability() -> u8 { 75 }
fn default_max_concurrent_mutations() -> u32 { 1 }
fn default_confirmation_timeout() -> u64 { 300 } // 5 minutes

fn default_forget_confirmation() -> String { "I CONFIRM (forget)".to_string() }
fn default_reset_confirmation() -> String { "I CONFIRM (reset)".to_string() }
fn default_uninstall_confirmation() -> String { "I CONFIRM (uninstall)".to_string() }

fn default_helpers_state_file() -> String { "/var/lib/anna/internal/helpers_state.json".to_string() }

fn default_allowed_paths() -> Vec<String> {
    vec![
        "/etc/".to_string(),
        "/home/".to_string(),
        "/root/".to_string(),
        "/var/lib/anna/".to_string(),
        "/tmp/".to_string(),
    ]
}

fn default_blocked_paths() -> Vec<String> {
    vec![
        "/etc/shadow".to_string(),
        "/etc/passwd".to_string(),
        "/etc/sudoers".to_string(),
        "/etc/ssh/sshd_config".to_string(),
    ]
}

fn default_systemd_operations() -> Vec<String> {
    vec![
        "restart".to_string(),
        "reload".to_string(),
        "start".to_string(),
        "stop".to_string(),
        "enable".to_string(),
        "disable".to_string(),
        "daemon-reload".to_string(),
    ]
}

fn default_blocked_units() -> Vec<String> {
    vec![
        "systemd-*".to_string(),
        "dbus.service".to_string(),
        "dbus.socket".to_string(),
    ]
}

fn default_protected_units() -> Vec<String> {
    vec![
        "sshd.service".to_string(),
        "NetworkManager.service".to_string(),
        "systemd-resolved.service".to_string(),
    ]
}

fn default_blocked_packages() -> Vec<String> {
    vec![
        "linux".to_string(),
        "linux-*".to_string(),
        "grub".to_string(),
        "systemd".to_string(),
        "glibc".to_string(),
        "base".to_string(),
        "filesystem".to_string(),
    ]
}

fn default_protected_packages() -> Vec<String> {
    vec![
        "sudo".to_string(),
        "openssh".to_string(),
        "networkmanager".to_string(),
    ]
}

fn default_blocked_categories() -> Vec<String> {
    vec![
        "kernel".to_string(),
        "bootloader".to_string(),
        "init".to_string(),
    ]
}

fn default_critical_services() -> Vec<String> {
    vec![
        "init".to_string(),
        "systemd-journald".to_string(),
        "systemd-udevd".to_string(),
        "systemd-logind".to_string(),
    ]
}

fn default_blocked_path_prefixes() -> Vec<String> {
    vec![
        "/boot/".to_string(),
        "/proc/".to_string(),
        "/sys/".to_string(),
        "/dev/".to_string(),
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = Policy::default();
        assert!(policy.capabilities.read_only_tools.enabled);
        assert!(policy.capabilities.mutation_tools.enabled);
        assert!(!policy.blocked.packages.categories.is_empty());
    }

    #[test]
    fn test_policy_validation() {
        let policy = Policy::default();
        let validation = policy.validate();
        assert!(validation.valid);
    }

    #[test]
    fn test_path_check_allowed() {
        let policy = Policy::default();
        let result = policy.is_path_allowed("/etc/nginx/nginx.conf");
        assert!(result.allowed);
    }

    #[test]
    fn test_path_check_blocked() {
        let policy = Policy::default();
        let result = policy.is_path_allowed("/etc/shadow");
        assert!(!result.allowed);
        assert!(result.reason.contains("blocked"));
    }

    #[test]
    fn test_path_check_boot_blocked() {
        let policy = Policy::default();
        let result = policy.is_path_allowed("/boot/grub/grub.cfg");
        assert!(!result.allowed);
    }

    #[test]
    fn test_package_check_allowed() {
        let policy = Policy::default();
        let result = policy.is_package_allowed("nginx");
        assert!(result.allowed);
    }

    #[test]
    fn test_package_check_blocked_kernel() {
        let policy = Policy::default();
        let result = policy.is_package_allowed("linux");
        assert!(!result.allowed);
        assert!(result.reason.contains("kernel") || result.reason.contains("blocked"));
    }

    #[test]
    fn test_package_check_blocked_pattern() {
        let policy = Policy::default();
        let result = policy.is_package_allowed("linux-headers");
        assert!(!result.allowed);
    }

    #[test]
    fn test_service_check_allowed() {
        let policy = Policy::default();
        let result = policy.is_service_allowed("nginx.service");
        assert!(result.allowed);
    }

    #[test]
    fn test_service_check_critical_blocked() {
        let policy = Policy::default();
        let result = policy.is_service_allowed("systemd-journald.service");
        assert!(!result.allowed);
        assert!(result.reason.contains("critical"));
    }

    #[test]
    fn test_systemd_operation_allowed() {
        let policy = Policy::default();
        let result = policy.is_systemd_operation_allowed("restart");
        assert!(result.allowed);
    }

    #[test]
    fn test_systemd_operation_blocked() {
        let policy = Policy::default();
        let result = policy.is_systemd_operation_allowed("mask");
        assert!(!result.allowed);
    }

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("linux-headers", "linux-*"));
        assert!(matches_pattern("linux", "linux"));
        assert!(!matches_pattern("nginx", "linux-*"));
        assert!(matches_pattern("systemd-journald", "systemd-*"));
    }

    #[test]
    fn test_confirmation_phrase() {
        let policy = Policy::default();
        assert_eq!(policy.get_confirmation_phrase("medium"), Some("I CONFIRM (medium risk)"));
        assert_eq!(policy.get_confirmation_phrase("read_only"), None);
    }

    #[test]
    fn test_min_reliability() {
        let policy = Policy::default();
        assert_eq!(policy.get_min_reliability("medium"), 70);
        assert_eq!(policy.get_min_reliability("high"), 85);
    }

    #[test]
    fn test_evidence_id_generation() {
        let id = generate_policy_evidence_id();
        assert!(id.starts_with(POLICY_EVIDENCE_PREFIX));
    }

    #[test]
    fn test_policy_check_result_format() {
        let blocked = PolicyCheckResult::blocked(
            "Package is blocked",
            "POL12345",
            "blocked.toml:packages.exact"
        );
        let citation = blocked.format_citation();
        assert!(citation.contains("POL12345"));
        assert!(citation.contains("Policy:"));
    }
}
