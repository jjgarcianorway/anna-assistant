// v0.0.787: Settings Enclave (Phase 363)
// Enclave member management

use serde::{Deserialize, Serialize};

/// Enclave member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveMember {
    /// Member ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Quarter number
    pub quarter: u32,
    /// Admitted
    pub admitted: bool,
}

impl EnclaveMember {
    /// Create new member
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            quarter: 0,
            admitted: true,
        }
    }

    /// Set quarter
    pub fn quarter(mut self, q: u32) -> Self {
        self.quarter = q;
        self
    }

    /// Make admitted
    pub fn make_admitted(&mut self) {
        self.admitted = true;
    }

    /// Make pending
    pub fn make_pending(&mut self) {
        self.admitted = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_new() {
        let m = EnclaveMember::new("m1", "Title", "Content");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_member_builder() {
        let m = EnclaveMember::new("m1", "Title", "Content")
            .quarter(1);
        assert_eq!(m.quarter, 1);
    }

    #[test]
    fn test_member_admission() {
        let mut m = EnclaveMember::new("m1", "Title", "Content");
        m.make_pending();
        assert!(!m.admitted);
        m.make_admitted();
        assert!(m.admitted);
    }
}
