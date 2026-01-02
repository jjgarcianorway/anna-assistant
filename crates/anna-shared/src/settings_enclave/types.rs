// v0.0.787: Settings Enclave (Phase 363)
// Enclave types and enums

use serde::{Deserialize, Serialize};

/// Enclave type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EnclaveType {
    /// Exclusive enclave
    #[default]
    Exclusive,
    /// Private enclave
    Private,
    /// Gated enclave
    Gated,
    /// Elite enclave
    Elite,
}

impl std::fmt::Display for EnclaveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exclusive => write!(f, "exclusive"),
            Self::Private => write!(f, "private"),
            Self::Gated => write!(f, "gated"),
            Self::Elite => write!(f, "elite"),
        }
    }
}

/// Enclave status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EnclaveStatus {
    /// Active status
    #[default]
    Active,
    /// Secured status
    Secured,
    /// Restricted status
    Restricted,
    /// Protected status
    Protected,
}

impl std::fmt::Display for EnclaveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Secured => write!(f, "secured"),
            Self::Restricted => write!(f, "restricted"),
            Self::Protected => write!(f, "protected"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enclave_type_display() {
        assert_eq!(format!("{}", EnclaveType::Exclusive), "exclusive");
        assert_eq!(format!("{}", EnclaveType::Private), "private");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", EnclaveStatus::Active), "active");
        assert_eq!(format!("{}", EnclaveStatus::Protected), "protected");
    }
}
