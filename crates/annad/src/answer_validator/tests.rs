//! Tests for answer validation modules.

#[cfg(test)]
mod tests {
    use anna_shared::rpc::SpecialistDomain;

    use crate::answer_validator::{
        healing::clean_llm_response, thresholds::domain_threshold, types::BASE_ACCEPTABLE_SCORE,
        ValidationIssue,
    };

    #[test]
    fn test_validation_issue_display() {
        let issue = ValidationIssue::UngroundedClaims { count: 3 };
        assert_eq!(issue.to_string(), "3 ungrounded claims");

        let issue = ValidationIssue::InventionDetected {
            claim: "nginx is running".to_string(),
        };
        assert!(issue.to_string().contains("invented"));
    }

    #[test]
    fn test_clean_llm_response() {
        let response = "<think>Let me think...</think>The answer is 42.";
        let cleaned = clean_llm_response(response);
        assert_eq!(cleaned, "The answer is 42.");
    }

    #[test]
    fn test_domain_threshold() {
        // Security has highest threshold
        assert_eq!(domain_threshold(Some(SpecialistDomain::Security)), 90);
        // System/Storage are standard
        assert_eq!(domain_threshold(Some(SpecialistDomain::System)), 80);
        assert_eq!(domain_threshold(Some(SpecialistDomain::Storage)), 80);
        // Network/Packages allow more flexibility
        assert_eq!(domain_threshold(Some(SpecialistDomain::Network)), 75);
        assert_eq!(domain_threshold(Some(SpecialistDomain::Packages)), 75);
        // None falls back to base
        assert_eq!(domain_threshold(None), BASE_ACCEPTABLE_SCORE);
    }
}
