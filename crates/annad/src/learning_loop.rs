//! Continuous learning loop for TruthLedger (v0.0.448).
//!
//! Analyzes patterns in the TruthLedger to adjust TrustScores and identify claim types.

use crate::state::SharedState;
use anna_shared::truth_ledger::{Source, TrustScore, TruthLedger, Veracity};
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

/// Periodically analyzes the TruthLedger to adjust TrustScores of sources.
pub async fn start_learning_loop(shared_state: SharedState) {
    let mut interval = interval(Duration::from_secs(3600)); // Run once every hour

    loop {
        interval.tick().await;
        info!("Running TruthLedger continuous learning loop...");
        let mut state_write = shared_state.write().await;
        analyze_truth_ledger(&mut state_write.truth_ledger);
        state_write.truthfulness_score = state_write.calculate_truthfulness_score();
        // Save the truth ledger after analysis
        if let Err(e) = state_write
            .truth_ledger
            .save(crate::state::TRUTH_LEDGER_PATH)
        {
            warn!("Failed to save truth ledger after learning loop: {}", e);
        }
        info!(
            "TruthLedger continuous learning loop completed. Current Truthfulness Score: {:.2}",
            state_write.truthfulness_score
        );
    }
}

/// Analyzes patterns in the TruthLedger to adjust TrustScores of sources.
fn analyze_truth_ledger(truth_ledger: &mut TruthLedger) {
    // Collect scores using owned Source values to avoid borrowing issues
    let mut source_scores: HashMap<Source, (f64, usize)> = HashMap::new(); // (total_score, count)

    for entry in &truth_ledger.entries {
        let (total_score, count) = source_scores
            .entry(entry.source_metadata.source.clone())
            .or_default();
        let score_contribution = match entry.veracity {
            Veracity::Verified => 1.0,
            Veracity::Disputed => -1.0,
            Veracity::Unverified => 0.0,
        };
        *total_score += score_contribution;
        *count += 1;
    }

    // Now iterate mutably to update trust scores
    for entry in &mut truth_ledger.entries {
        if let Some(&(total_score, count)) = source_scores.get(&entry.source_metadata.source) {
            if count > 0 {
                let reliability_score = total_score / count as f64;

                let new_trust_score = match reliability_score {
                    r if r > 0.7 => TrustScore::High,
                    r if r < -0.7 => TrustScore::Low,
                    _ => TrustScore::Medium,
                };

                if new_trust_score != entry.source_metadata.trust_score {
                    info!(
                        "Adjusting TrustScore for source {:?} from {:?} to {:?}",
                        entry.source_metadata.source,
                        entry.source_metadata.trust_score,
                        new_trust_score
                    );
                    entry.source_metadata.trust_score = new_trust_score;
                }
            }
        }
    }
}
