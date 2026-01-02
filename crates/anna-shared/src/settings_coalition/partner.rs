// v0.0.736: Settings Coalition - Partner (Phase 312)
// Coalition partner

use serde::{Deserialize, Serialize};

/// Coalition partner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalitionPartner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Agreement ID
    pub agreement_id: String,
}

impl CoalitionPartner {
    /// Create new partner
    pub fn new(key: impl Into<String>, name: impl Into<String>, agreement_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            agreement_id: agreement_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner_new() {
        let p = CoalitionPartner::new("key", "name", "a1");
        assert_eq!(p.agreement_id, "a1");
    }
}
