//! Batch verification (v0.0.198).

use serde::{Deserialize, Serialize};

use super::runners::run_verification;
use super::types::{VerificationStep, VerifyResult};

/// Pre-action verification batch
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreActionVerify {
    pub steps: Vec<VerificationStep>,
    pub results: Vec<VerifyResult>,
    pub all_passed: bool,
}

impl PreActionVerify {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, step: VerificationStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Run all verification steps
    pub fn run(mut self) -> Self {
        self.results = self.steps.iter().map(run_verification).collect();
        self.all_passed = self
            .results
            .iter()
            .zip(&self.steps)
            .all(|(r, s)| r.passed || !s.mandatory);
        self
    }

    /// Get failed mandatory steps
    pub fn failed_mandatory(&self) -> Vec<(&VerificationStep, &VerifyResult)> {
        self.steps
            .iter()
            .zip(&self.results)
            .filter(|(s, r)| s.mandatory && !r.passed)
            .collect()
    }

    /// Summary for transcript
    pub fn summary(&self) -> String {
        let passed = self.results.iter().filter(|r| r.passed).count();
        let total = self.results.len();
        if self.all_passed {
            format!("Verified {}/{} checks passed", passed, total)
        } else {
            let failed: Vec<_> = self
                .failed_mandatory()
                .iter()
                .map(|(s, r)| {
                    format!(
                        "{}: {}",
                        s.description,
                        r.error.as_deref().unwrap_or("failed")
                    )
                })
                .collect();
            format!("Verification failed: {}", failed.join("; "))
        }
    }
}

/// Post-action verification batch
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostActionVerify {
    pub steps: Vec<VerificationStep>,
    pub results: Vec<VerifyResult>,
    pub success: bool,
}

impl PostActionVerify {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, step: VerificationStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Run all verification steps
    pub fn run(mut self) -> Self {
        self.results = self.steps.iter().map(run_verification).collect();
        self.success = self.results.iter().all(|r| r.passed);
        self
    }

    /// Get confirmation message for transcript
    pub fn confirmation(&self) -> String {
        if self.success {
            "Change verified successfully".to_string()
        } else {
            let failed: Vec<_> = self
                .results
                .iter()
                .filter(|r| !r.passed)
                .map(|r| r.error.as_deref().unwrap_or("unknown"))
                .collect();
            format!("Change may not have applied: {}", failed.join("; "))
        }
    }
}
