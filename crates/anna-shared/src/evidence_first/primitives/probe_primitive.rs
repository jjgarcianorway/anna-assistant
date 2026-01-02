//! Probe primitive definition and functionality.

use super::domain::{Domain, ParserId};
use super::precondition::Precondition;

/// A probe primitive definition.
#[derive(Debug, Clone)]
pub struct ProbePrimitive {
    /// Unique identifier (e.g., "sys.boot.analyze").
    pub id: &'static str,
    /// Domain this probe belongs to.
    pub domain: Domain,
    /// Human-readable purpose.
    pub purpose: &'static str,
    /// Command template to execute.
    pub command_template: &'static str,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Parser for output.
    pub parser: ParserId,
    /// Preconditions that must be met.
    pub preconditions: &'static [Precondition],
    /// Related man page (for documentation lookup).
    pub related_man: Option<&'static str>,
    /// Keywords for matching.
    pub keywords: &'static [&'static str],
}

impl ProbePrimitive {
    /// Check if all preconditions are met.
    pub fn can_run(&self) -> bool {
        self.preconditions.iter().all(|p| p.check())
    }

    /// Get the command to execute.
    pub fn command(&self) -> String {
        self.command_template.to_string()
    }

    /// Check if primitive matches keywords.
    pub fn matches_keywords(&self, query: &[&str]) -> bool {
        for q in query {
            let q_lower = q.to_lowercase();
            if self
                .keywords
                .iter()
                .any(|k| k.contains(&q_lower) || q_lower.contains(*k))
            {
                return true;
            }
            if self.purpose.to_lowercase().contains(&q_lower) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_first::PrimitiveLibrary;

    #[test]
    fn test_primitive_matches_keywords() {
        let lib = PrimitiveLibrary::default_library();
        let probe = lib.get("sys.boot.analyze").unwrap();

        assert!(probe.matches_keywords(&["boot"]));
        assert!(probe.matches_keywords(&["slow"]));
        assert!(!probe.matches_keywords(&["network"]));
    }
}
