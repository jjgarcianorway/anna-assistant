// v0.0.712: Settings Report Section (Phase 288)
// Report sections and appendices

use serde::{Deserialize, Serialize};

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Section ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Order
    pub order: usize,
    /// Critical
    pub critical: bool,
}

impl ReportSection {
    /// Create new section
    pub fn new(id: impl Into<String>, title: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: String::new(),
            order,
            critical: false,
        }
    }

    /// Set content
    pub fn content(mut self, c: impl Into<String>) -> Self {
        self.content = c.into();
        self
    }

    /// Set critical
    pub fn critical(mut self, cr: bool) -> Self {
        self.critical = cr;
        self
    }
}

/// Report appendix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAppendix {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Section ID
    pub section_id: String,
}

impl ReportAppendix {
    /// Create new appendix
    pub fn new(key: impl Into<String>, value: impl Into<String>, section_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            section_id: section_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_new() {
        let s = ReportSection::new("s1", "Section 1", 1);
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_section_builder() {
        let s = ReportSection::new("s1", "Section 1", 1)
            .content("content")
            .critical(true);
        assert_eq!(s.content, "content");
        assert!(s.critical);
    }

    #[test]
    fn test_appendix_new() {
        let a = ReportAppendix::new("key", "value", "s1");
        assert_eq!(a.section_id, "s1");
    }
}
