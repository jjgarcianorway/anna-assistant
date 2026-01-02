//! LLM state management and model tracking.

use anna_shared::status::{BenchmarkResult, LlmState, ModelInfo, ProgressInfo};
use anna_shared::truth_ledger::Veracity;

use super::types::DaemonStateInner;

impl DaemonStateInner {
    pub fn set_llm_phase(&mut self, phase: &str) {
        self.llm.phase = Some(phase.to_string());
    }

    #[allow(dead_code)]
    pub fn set_llm_progress(&mut self, current: u64, total: u64, speed: u64, eta: u64) {
        self.llm.progress = Some(ProgressInfo {
            current_bytes: current,
            total_bytes: total,
            speed_bytes_per_sec: speed,
            eta_seconds: eta,
        });
    }

    #[allow(dead_code)]
    pub fn clear_llm_progress(&mut self) {
        self.llm.progress = None;
    }

    pub fn set_llm_ready(&mut self) {
        self.llm.state = LlmState::Ready;
        self.llm.phase = None;
        self.llm.progress = None;
        self.state = anna_shared::status::DaemonState::Running;
    }

    pub fn set_benchmark_result(&mut self, cpu: &str, ram: &str, gpu: &str) {
        self.llm.benchmark = Some(BenchmarkResult {
            cpu: cpu.to_string(),
            ram: ram.to_string(),
            gpu: gpu.to_string(),
        });
    }

    pub fn add_model(&mut self, name: &str, role: &str, size: u64) {
        self.llm.models.push(ModelInfo {
            name: name.to_string(),
            role: role.to_string(),
            size_bytes: size,
            pulled: true,
        });
    }

    /// Calculate the system's overall truthfulness score based on the TruthLedger.
    /// Score is between 0.0 and 1.0. 1.0 means perfect truthfulness.
    pub fn calculate_truthfulness_score(&self) -> f64 {
        let claims = self.truth_ledger.get_all_claims();
        if claims.is_empty() {
            return 1.0; // Perfect score if no claims
        }

        let mut verified_count = 0;
        let mut disputed_count = 0;

        for claim in claims {
            match claim.veracity {
                Veracity::Verified => verified_count += 1,
                Veracity::Disputed => disputed_count += 1,
                _ => {} // Ignore Unknown or Pending for score calculation for now
            }
        }

        let total_assessable_claims = (verified_count + disputed_count) as f64;
        if total_assessable_claims == 0.0 {
            return 1.0; // Still perfect if no assessable claims
        }

        // Simple score: (verified - disputed) / total_assessable_claims
        // This gives a range from -1.0 (all disputed) to 1.0 (all verified)
        let score = (verified_count as f64 - disputed_count as f64) / total_assessable_claims;

        // Normalize to 0.0 - 1.0 range: (score + 1.0) / 2.0
        (score + 1.0) / 2.0
    }
}
