//! Validation logic for specialist responses
//!
//! This module contains all validation rules and business logic for ensuring
//! specialist responses meet quality standards.

use crate::strict_contract::types::{StrictSpecialistResponse, StrictStatus};

impl StrictSpecialistResponse {
    /// Validate response - returns list of issues
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Summary must not be empty
        if self.summary.trim().is_empty() {
            issues.push("summary is empty".to_string());
        }

        // Summary length check
        if self.summary.len() > 200 {
            issues.push(format!(
                "summary too long ({} > 200 chars)",
                self.summary.len()
            ));
        }

        // Confidence range
        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push(format!(
                "confidence {} out of range [0.0, 1.0]",
                self.confidence
            ));
        }

        // Ok status requires evidence
        if self.status == StrictStatus::Ok && self.confidence >= 0.8 && self.evidence.is_empty() {
            issues.push("status=ok with high confidence but no evidence".to_string());
        }

        // Check for forbidden nonsense patterns
        let forbidden_nonsense = [
            "unknown is installed",
            "unknown is not installed",
            "**unknown**",
            "2 is installed",
            "1 is installed",
            "installed package is not installed",
            "package is installed is not",
        ];

        let summary_lower = self.summary.to_lowercase();
        for f in forbidden_nonsense {
            if summary_lower.contains(f) {
                issues.push(format!("contains forbidden nonsense: '{}'", f));
            }
        }

        // Check for tutorial patterns in summary (MUST be answer, not instructions)
        let tutorial_patterns = [
            "to check ",
            "you can check ",
            "run the command",
            "use the following",
            "here's how to",
            "how to debug",
            "to debug ",
            "you should run",
            "try running",
            "execute the",
            "step 1:",
            "step 2:",
            "first, run",
            "start by running",
        ];

        for pattern in tutorial_patterns {
            if summary_lower.contains(pattern) {
                issues.push(format!("summary contains tutorial pattern: '{}'", pattern));
            }
        }

        // Check for "I can't answer" when we should have data
        let evasion_patterns = [
            "i cannot determine",
            "i can't answer yet",
            "i cannot answer",
            "run annactl status",
            "collect evidence",
            "need more data",
            "insufficient data",
        ];

        // Only flag evasion if status is ok (contradiction)
        if self.status == StrictStatus::Ok {
            for pattern in evasion_patterns {
                if summary_lower.contains(pattern) {
                    issues.push(format!(
                        "status=ok but summary contains evasion: '{}'",
                        pattern
                    ));
                }
            }
        }

        // Check details for tutorial patterns
        for (i, detail) in self.details.iter().enumerate() {
            if detail.len() > 300 {
                issues.push(format!(
                    "details[{}] too long ({} > 300 chars)",
                    i,
                    detail.len()
                ));
            }
            let detail_lower = detail.to_lowercase();
            for f in forbidden_nonsense {
                if detail_lower.contains(f) {
                    issues.push(format!(
                        "details[{}] contains forbidden nonsense: '{}'",
                        i, f
                    ));
                }
            }
        }

        // Actions limit
        if self.actions.len() > 5 {
            issues.push(format!("too many actions ({} > 5)", self.actions.len()));
        }

        // High confidence + failed status is contradictory
        if self.status == StrictStatus::Failed && self.confidence > 0.5 {
            issues.push(format!(
                "failed status with high confidence {} is contradictory",
                self.confidence
            ));
        }

        issues
    }

    /// Check if this is a valid, meaningful response
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Check if this should count as "resolved" for stats
    pub fn is_resolved(&self) -> bool {
        self.status == StrictStatus::Ok
            && self.confidence >= 0.8
            && !self.summary.trim().is_empty()
            && self.is_valid()
    }
}
