// v0.0.787: Settings Enclave (Phase 363)
// Enclave statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::EnclaveType;
use super::member::EnclaveMember;

/// Enclave stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnclaveStats {
    /// Total members
    pub total_members: usize,
    /// Admitted members
    pub admitted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl EnclaveStats {
    /// Update from members
    pub fn update(&mut self, members: &[EnclaveMember], enclave_type: EnclaveType) {
        self.total_members = members.len();
        self.admitted = members.iter().filter(|m| m.admitted).count();
        *self.by_type.entry(enclave_type.to_string()).or_insert(0) += 1;
    }

    /// Admission rate
    pub fn admission_rate(&self) -> f64 {
        if self.total_members == 0 { 0.0 } else { self.admitted as f64 / self.total_members as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = EnclaveStats::default();
        let member = EnclaveMember::new("m1", "Title", "Content");
        s.update(&[member], EnclaveType::Exclusive);
        assert_eq!(s.total_members, 1);
        assert_eq!(s.admitted, 1);
    }
}
