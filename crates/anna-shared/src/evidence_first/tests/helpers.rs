//! Integration test helpers.
//!
//! Helper functions for creating mock probe outputs and other test utilities.

#[cfg(test)]
use crate::evidence_first::probe_plan::ProbeOutput;

/// Create a mock probe output for testing.
#[cfg(test)]
pub fn mock_probe_output(primitive_id: &str, output: &str) -> ProbeOutput {
    ProbeOutput {
        primitive_id: primitive_id.to_string(),
        raw_output: output.to_string(),
        parsed: None,
        exit_code: Some(0),
        execution_time_ms: 100,
        error: None,
    }
}
