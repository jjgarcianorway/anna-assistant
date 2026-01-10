//! Common Linux patterns that should get instant answers without clarification.
//!
//! v0.0.909: Added to reduce over-clarification (80% rate in testing).
//! These are well-known issues with standard solutions.

mod pacman;
mod errors;
mod recovery;
mod performance;

use anna_shared::rpc::DeepUnderstanding;

/// Check if a question matches a common pattern that has a known solution.
/// Returns Some(DeepUnderstanding) with high confidence if matched.
pub fn match_common_pattern(question: &str) -> Option<DeepUnderstanding> {
    let q = question.to_lowercase();

    // Check each pattern category (order matters - more specific first)
    pacman::match_patterns(&q)
        .or_else(|| recovery::match_patterns(&q))
        .or_else(|| errors::match_patterns(&q))
        .or_else(|| performance::match_patterns(&q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_database_locked() {
        let result = match_common_pattern("pacman says database is locked");
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.confidence, 0.95);
        assert!(!u.needs_confirmation);
    }

    #[test]
    fn test_deleted_usr_bin() {
        let result = match_common_pattern("I accidentally deleted /usr/bin");
        assert!(result.is_some());
        assert!(!result.unwrap().needs_confirmation);
    }

    #[test]
    fn test_fan_idle() {
        let result = match_common_pattern("why does my fan spin up when the system is idle");
        assert!(result.is_some());
    }

    #[test]
    fn test_no_match() {
        let result = match_common_pattern("what is the meaning of life");
        assert!(result.is_none());
    }
}
