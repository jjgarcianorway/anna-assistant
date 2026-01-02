//! Canary Tests - Diagnosis (Part 3 of 4) - v0.0.437.
//!
//! Fixed tests that MUST pass for any release.
//! Any regression here BLOCKS release.
//!
//! Test cases:
//! - Diagnosis must have conclusion
//! - Uncertain diagnosis must not use confident language

#[cfg(test)]
mod diagnosis_canary_tests {
    use crate::question_contract::diagnosis::*;

    // ============================================
    // CANARY 5: Diagnosis must have conclusion
    // ============================================

    #[test]
    fn canary_diagnosis_requires_conclusion() {
        // A diagnosis without conclusion is invalid
        let incomplete = DiagnosisConclusion {
            conclusion: ConclusionState::Likely,
            primary_cause: None, // Missing!
            confidence: 0.8,
            supporting_evidence: vec![],
            alternatives: vec![],
        };

        let validation = incomplete.validate();
        assert!(!validation.is_valid());

        // Complete diagnosis
        let complete =
            DiagnosisConclusion::likely("slow disk I/O", 0.85, vec!["ev_iostat".to_string()]);
        assert!(complete.validate().is_valid());
    }

    #[test]
    fn canary_uncertain_no_confident_language() {
        let conclusion = DiagnosisConclusion::uncertain(
            vec!["option A".to_string(), "option B".to_string()],
            vec![],
        );

        // Bad: confident language
        let bad_text = "The problem is definitely caused by X.";
        let result = ConclusionLanguageValidator::validate(&conclusion, bad_text);
        assert!(!result.is_valid());

        // Good: hedging language
        let good_text = "The problem might be caused by X, but I'm uncertain.";
        let result = ConclusionLanguageValidator::validate(&conclusion, good_text);
        assert!(result.is_valid());
    }
}
