//! Helper functions for the core loop.
//!
//! This module contains utility functions:
//! - Evidence gathering by running probes
//! - String manipulation utilities

use std::collections::HashMap;
use tracing::warn;

use crate::probes;

/// Gather evidence by running probes
pub async fn gather_evidence(probe_cmds: &[String]) -> HashMap<String, String> {
    let mut evidence = HashMap::new();

    for probe in probe_cmds {
        match probes::run_command(probe) {
            Ok(output) => {
                evidence.insert(probe.clone(), output);
            }
            Err(e) => {
                warn!("Evidence probe failed: {} - {}", probe, e);
            }
        }
    }

    evidence
}

/// Capitalize first letter
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("system"), "System");
        assert_eq!(capitalize("network"), "Network");
        assert_eq!(capitalize(""), "");
    }
}
