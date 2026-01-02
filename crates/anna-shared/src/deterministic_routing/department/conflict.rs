//! Department Conflict Tracking - v0.0.439.
//!
//! Tracks conflicts between translator suggestions and canonical mappings.

use super::super::intent_schema::{CanonicalIntent, Department};

/// A conflict between translator suggestion and canonical mapping.
#[derive(Debug, Clone)]
pub struct DepartmentConflict {
    /// The intent in question.
    pub intent: CanonicalIntent,
    /// What translator suggested.
    pub translator_suggested: Department,
    /// The canonical (correct) department.
    pub canonical_department: Department,
}

impl DepartmentConflict {
    /// Format as log message.
    pub fn log_message(&self) -> String {
        format!(
            "[route] Translator suggested {} but mapping says {}, overridden.",
            self.translator_suggested.label(),
            self.canonical_department.label()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_log_message() {
        let conflict = DepartmentConflict {
            intent: CanonicalIntent::BootPerf,
            translator_suggested: Department::Desktop,
            canonical_department: Department::Performance,
        };

        let msg = conflict.log_message();
        assert!(msg.contains("Desktop"));
        assert!(msg.contains("Performance"));
        assert!(msg.contains("overridden"));
    }
}
