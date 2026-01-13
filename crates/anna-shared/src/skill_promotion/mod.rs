//! Skill Promotion Pipeline - Turn experiences into tested, reusable skills.
//!
//! The learning loop:
//! 1. Experience recorded (memory/mod.rs)
//! 2. Validated for contradictions (validation/mod.rs)
//! 3. → Candidate generated (this module)
//! 4. → Tested against preconditions
//! 5. → Promoted to recipe library
//!
//! v0.3.14: Initial implementation

mod candidate;
mod promotion;
mod testing;

pub use candidate::{
    generate_candidate, Precondition, PreconditionType, RecipeCandidate, RiskLevel, RollbackStep,
};
pub use promotion::{promote_candidate, PromotionResult};
pub use testing::{test_candidate, TestResult, TestStatus};

use crate::memory::Experience;
use crate::recipe::Recipe;

/// The complete skill promotion pipeline
pub struct SkillPipeline {
    /// Minimum usefulness score to consider for promotion
    pub min_usefulness: u32,
    /// Minimum success count across cluster
    pub min_cluster_success: u32,
    /// Whether to require passing tests before promotion
    pub require_tests: bool,
}

impl Default for SkillPipeline {
    fn default() -> Self {
        Self {
            min_usefulness: 3,      // Used 3+ times
            min_cluster_success: 5, // Cluster has 5+ successes
            require_tests: true,
        }
    }
}

impl SkillPipeline {
    /// Evaluate an experience for skill promotion
    pub fn evaluate(&self, experience: &Experience, cluster_success_count: u32) -> Option<RecipeCandidate> {
        // Check minimum thresholds
        if experience.usefulness_score < self.min_usefulness {
            return None;
        }
        if cluster_success_count < self.min_cluster_success {
            return None;
        }

        // Generate candidate
        Some(generate_candidate(experience))
    }

    /// Run the full pipeline: evaluate → test → promote
    pub fn run(
        &self,
        experience: &Experience,
        cluster_success_count: u32,
    ) -> Option<PromotionResult> {
        let candidate = self.evaluate(experience, cluster_success_count)?;

        if self.require_tests {
            let test_result = test_candidate(&candidate);
            if !test_result.passed() {
                return Some(PromotionResult::TestFailed(test_result));
            }
        }

        Some(promote_candidate(candidate))
    }
}
