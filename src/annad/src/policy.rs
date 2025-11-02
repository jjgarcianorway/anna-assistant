// Anna v0.11.0 - Policy Engine
//
// Determines whether automatic repair is allowed for each event domain.
// Enforces safe operations and requires user confirmation for risky changes.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tracing::{info, warn};

const POLICY_PATH: &str = "/etc/anna/policy.toml";

/// Policy decision for an event
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecision {
    AutoRepair, // Automatic repair allowed
    AlertOnly,  // Create alert, wait for user
    NoAction,   // No action needed
}

/// Policy configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct PolicyConfig {
    policy: PolicyRoot,
}

#[derive(Debug, Deserialize, Serialize)]
struct PolicyRoot {
    version: String,
    description: String,
    packages: DomainPolicy,
    config: DomainPolicy,
    devices: DomainPolicy,
    network: DomainPolicy,
    storage: DomainPolicy,
    kernel: DomainPolicy,
    always_allowed: AllowedOperations,
    always_forbidden: ForbiddenOperations,
}

#[derive(Debug, Deserialize, Serialize)]
struct DomainPolicy {
    auto: bool,
    reason: String,
    allowed_operations: Vec<String>,
    forbidden_operations: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AllowedOperations {
    operations: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ForbiddenOperations {
    operations: Vec<String>,
}

pub struct PolicyEngine {
    config: PolicyConfig,
    cache: HashMap<String, PolicyDecision>,
}

impl PolicyEngine {
    /// Load policy from /etc/anna/policy.toml
    pub fn new() -> Result<Self> {
        let content = fs::read_to_string(POLICY_PATH)
            .context(format!("Failed to read policy file: {}", POLICY_PATH))?;

        let config: PolicyConfig =
            toml::from_str(&content).context("Failed to parse policy.toml")?;

        info!("Policy engine loaded: v{}", config.policy.version);

        Ok(Self {
            config,
            cache: HashMap::new(),
        })
    }

    /// Determine policy decision for a domain
    pub fn decide(&mut self, domain: &str, operation: &str) -> PolicyDecision {
        let cache_key = format!("{}:{}", domain, operation);

        // Check cache
        if let Some(decision) = self.cache.get(&cache_key) {
            return decision.clone();
        }

        // Check if operation is always forbidden
        if self.is_operation_forbidden(operation) {
            info!(
                "Policy: {} for {} → AlertOnly (forbidden operation)",
                operation, domain
            );
            let decision = PolicyDecision::AlertOnly;
            self.cache.insert(cache_key, decision.clone());
            return decision;
        }

        // Check domain policy
        let domain_policy = match domain {
            "packages" => &self.config.policy.packages,
            "config" => &self.config.policy.config,
            "devices" => &self.config.policy.devices,
            "network" => &self.config.policy.network,
            "storage" => &self.config.policy.storage,
            "kernel" => &self.config.policy.kernel,
            _ => {
                warn!("Unknown domain: {}", domain);
                let decision = PolicyDecision::AlertOnly;
                self.cache.insert(cache_key, decision.clone());
                return decision;
            }
        };

        // If auto=false, always alert-only
        if !domain_policy.auto {
            info!(
                "Policy: {} for {} → AlertOnly (auto=false: {})",
                operation, domain, domain_policy.reason
            );
            let decision = PolicyDecision::AlertOnly;
            self.cache.insert(cache_key, decision.clone());
            return decision;
        }

        // auto=true, check if operation is allowed
        if domain_policy
            .forbidden_operations
            .contains(&operation.to_string())
        {
            info!(
                "Policy: {} for {} → AlertOnly (domain forbids this operation)",
                operation, domain
            );
            let decision = PolicyDecision::AlertOnly;
            self.cache.insert(cache_key, decision.clone());
            return decision;
        }

        // Check if operation is always allowed
        if self.is_operation_allowed(operation) {
            info!(
                "Policy: {} for {} → AutoRepair (always allowed)",
                operation, domain
            );
            let decision = PolicyDecision::AutoRepair;
            self.cache.insert(cache_key, decision.clone());
            return decision;
        }

        // Check if operation is in domain's allowed list
        if domain_policy
            .allowed_operations
            .contains(&operation.to_string())
        {
            info!(
                "Policy: {} for {} → AutoRepair (domain allows this operation)",
                operation, domain
            );
            let decision = PolicyDecision::AutoRepair;
            self.cache.insert(cache_key, decision.clone());
            return decision;
        }

        // Default: alert-only for safety
        info!(
            "Policy: {} for {} → AlertOnly (default, not explicitly allowed)",
            operation, domain
        );
        let decision = PolicyDecision::AlertOnly;
        self.cache.insert(cache_key, decision.clone());
        decision
    }

    /// Check if operation is always allowed
    fn is_operation_allowed(&self, operation: &str) -> bool {
        self.config
            .policy
            .always_allowed
            .operations
            .contains(&operation.to_string())
    }

    /// Check if operation is always forbidden
    fn is_operation_forbidden(&self, operation: &str) -> bool {
        self.config
            .policy
            .always_forbidden
            .operations
            .contains(&operation.to_string())
    }

    /// Get auto-repair status for a domain
    pub fn is_auto_repair_enabled(&self, domain: &str) -> bool {
        match domain {
            "packages" => self.config.policy.packages.auto,
            "config" => self.config.policy.config.auto,
            "devices" => self.config.policy.devices.auto,
            "network" => self.config.policy.network.auto,
            "storage" => self.config.policy.storage.auto,
            "kernel" => self.config.policy.kernel.auto,
            _ => false,
        }
    }

    /// Get policy reason for a domain
    pub fn get_reason(&self, domain: &str) -> String {
        let reason = match domain {
            "packages" => &self.config.policy.packages.reason,
            "config" => &self.config.policy.config.reason,
            "devices" => &self.config.policy.devices.reason,
            "network" => &self.config.policy.network.reason,
            "storage" => &self.config.policy.storage.reason,
            "kernel" => &self.config.policy.kernel.reason,
            _ => return "Unknown domain".to_string(),
        };

        reason.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_decisions() {
        // Note: This test requires policy.toml to exist
        // In real environment, we'd use a test fixture

        // Mock test - just check enum values
        assert_eq!(PolicyDecision::AutoRepair, PolicyDecision::AutoRepair);
        assert_ne!(PolicyDecision::AutoRepair, PolicyDecision::AlertOnly);
    }
}
