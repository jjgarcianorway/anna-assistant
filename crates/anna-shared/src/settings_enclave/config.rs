// v0.0.787: Settings Enclave (Phase 363)
// Enclave configuration

use serde::{Deserialize, Serialize};
use super::types::{EnclaveType, EnclaveStatus};

/// Enclave config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveConfig {
    /// Name
    pub name: String,
    /// Enclave type
    pub enclave_type: EnclaveType,
    /// Status
    pub status: EnclaveStatus,
    /// Max members
    pub max_members: usize,
}

impl EnclaveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enclave_type: EnclaveType::Exclusive,
            status: EnclaveStatus::Active,
            max_members: 100,
        }
    }

    /// Set type
    pub fn enclave_type(mut self, et: EnclaveType) -> Self {
        self.enclave_type = et;
        self
    }

    /// Set status
    pub fn status(mut self, s: EnclaveStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max members
    pub fn max_members(mut self, max: usize) -> Self {
        self.max_members = max;
        self
    }
}

impl Default for EnclaveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = EnclaveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = EnclaveConfig::new("test")
            .enclave_type(EnclaveType::Gated)
            .status(EnclaveStatus::Secured);
        assert_eq!(c.enclave_type, EnclaveType::Gated);
        assert_eq!(c.status, EnclaveStatus::Secured);
    }
}
