//! Probe execution statistics (v0.0.184).

use serde::{Deserialize, Serialize};

/// Probe execution summary
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeStats {
    /// Number of probes planned by router/translator
    pub planned: usize,
    /// Number of probes that succeeded (exit_code == 0)
    pub succeeded: usize,
    /// Number of probes that failed (exit_code != 0, not timeout)
    pub failed: usize,
    /// Number of probes that timed out
    pub timed_out: usize,
}

impl ProbeStats {
    /// Create stats from probe results
    pub fn from_results(planned: usize, results: &[crate::rpc::ProbeResult]) -> Self {
        let succeeded = results.iter().filter(|p| p.exit_code == 0).count();
        let timed_out = results
            .iter()
            .filter(|p| p.stderr.to_lowercase().contains("timeout"))
            .count();
        let failed = results
            .iter()
            .filter(|p| p.exit_code != 0 && !p.stderr.to_lowercase().contains("timeout"))
            .count();

        Self {
            planned,
            succeeded,
            failed,
            timed_out,
        }
    }
}

impl std::fmt::Display for ProbeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{} probes succeeded", self.succeeded, self.planned)?;
        if self.failed > 0 {
            write!(f, ", {} failed", self.failed)?;
        }
        if self.timed_out > 0 {
            write!(f, ", {} timed out", self.timed_out)?;
        }
        Ok(())
    }
}
