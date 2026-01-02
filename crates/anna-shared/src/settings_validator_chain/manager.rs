// v0.0.597: Settings Validator Chain - Manager Module
// Validator chain manager

use std::collections::HashMap;

use super::chain::ValidationChain;

/// Validator chain manager
#[derive(Debug, Clone, Default)]
pub struct ValidatorChainManager {
    /// Named chains
    chains: HashMap<String, ValidationChain>,
    /// Default chain
    default_chain: ValidationChain,
}

impl ValidatorChainManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add chain
    pub fn add_chain(&mut self, name: impl Into<String>, chain: ValidationChain) {
        self.chains.insert(name.into(), chain);
    }

    /// Get chain
    pub fn get_chain(&self, name: &str) -> Option<&ValidationChain> {
        self.chains.get(name)
    }

    /// Get chain mut
    pub fn get_chain_mut(&mut self, name: &str) -> Option<&mut ValidationChain> {
        self.chains.get_mut(name)
    }

    /// Remove chain
    pub fn remove_chain(&mut self, name: &str) -> Option<ValidationChain> {
        self.chains.remove(name)
    }

    /// Set default chain
    pub fn set_default(&mut self, chain: ValidationChain) {
        self.default_chain = chain;
    }

    /// Get default chain
    pub fn default_chain(&self) -> &ValidationChain {
        &self.default_chain
    }

    /// List chain names
    pub fn chain_names(&self) -> Vec<&String> {
        self.chains.keys().collect()
    }

    /// Chain count
    pub fn chain_count(&self) -> usize {
        self.chains.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_new() {
        let manager = ValidatorChainManager::new();
        assert_eq!(manager.chain_count(), 0);
    }
}
