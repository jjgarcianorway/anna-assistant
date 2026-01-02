// v0.0.704: Gazette Notice (Phase 280)

use serde::{Deserialize, Serialize};

/// Gazette notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteNotice {
    /// Notice ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Effective date
    pub effective_date: String,
    /// Urgent
    pub urgent: bool,
}

impl GazetteNotice {
    /// Create new notice
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            effective_date: String::new(),
            urgent: false,
        }
    }

    /// Set effective date
    pub fn effective_date(mut self, date: impl Into<String>) -> Self {
        self.effective_date = date.into();
        self
    }

    /// Set urgent
    pub fn urgent(mut self, u: bool) -> Self {
        self.urgent = u;
        self
    }
}

/// Gazette entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Notice ID
    pub notice_id: String,
    /// Reference
    pub reference: Option<String>,
}

impl GazetteEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, notice_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            notice_id: notice_id.into(),
            reference: None,
        }
    }

    /// Set reference
    pub fn reference(mut self, ref_: impl Into<String>) -> Self {
        self.reference = Some(ref_.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notice_new() {
        let n = GazetteNotice::new("n1", "Notice 1", "Content");
        assert_eq!(n.id, "n1");
    }

    #[test]
    fn test_notice_builder() {
        let n = GazetteNotice::new("n1", "Notice 1", "Content")
            .effective_date("2025-12-15")
            .urgent(true);
        assert_eq!(n.effective_date, "2025-12-15");
        assert!(n.urgent);
    }

    #[test]
    fn test_entry_new() {
        let e = GazetteEntry::new("key", "value", "n1");
        assert_eq!(e.notice_id, "n1");
    }

    #[test]
    fn test_entry_reference() {
        let e = GazetteEntry::new("key", "value", "n1").reference("REF-001");
        assert!(e.reference.is_some());
    }
}
