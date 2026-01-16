//! Probe Effectiveness Statistics.
//!
//! Phase 27: Tracks resolution and abstention rates per probe pattern.
//! Used to inform confidence calculations and adaptive iteration budgets.

use crate::outcome_ledger::{Outcome, OutcomeRecord};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effectiveness metrics for a probe pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEffectivenessRecord {
    /// Probe pattern (e.g., "systemctl status *")
    pub probe_pattern: String,
    /// Resolution rate: resolved / (resolved + failed)
    pub resolution_rate: f32,
    /// Abstention rate: abstained / total
    pub abstention_rate: f32,
    /// Average execution duration in milliseconds
    pub avg_duration_ms: u64,
    /// Number of samples used for calculation
    pub sample_count: usize,
}

/// Per-probe accumulator for building stats.
#[derive(Default)]
struct ProbeAccumulator {
    resolved: u64,
    failed: u64,
    abstained: u64,
    total: u64,
    total_duration_ms: u64,
}

/// Aggregate probe effectiveness from outcome records.
/// Returns a map of probe_pattern -> effectiveness stats.
pub fn aggregate_probe_effectiveness(
    records: &[OutcomeRecord],
) -> HashMap<String, ProbeEffectivenessRecord> {
    let mut accumulators: HashMap<String, ProbeAccumulator> = HashMap::new();

    for record in records {
        // Skip records without probe data
        let probes = match &record.probes_used {
            Some(p) if !p.is_empty() => p,
            _ => continue,
        };

        // Duration per probe (distribute evenly)
        let duration_per_probe = record.duration_ms / probes.len().max(1) as u64;

        for probe in probes {
            let pattern = normalize_probe_pattern(probe);
            let acc = accumulators.entry(pattern).or_default();

            acc.total += 1;
            acc.total_duration_ms += duration_per_probe;

            match record.outcome {
                Outcome::Resolved => acc.resolved += 1,
                Outcome::Failed => acc.failed += 1,
                Outcome::Abstained => acc.abstained += 1,
                _ => {} // Cancelled/Expired don't count
            }
        }
    }

    // Convert accumulators to records
    accumulators
        .into_iter()
        .filter(|(_, acc)| acc.total >= 5) // Minimum sample size
        .map(|(pattern, acc)| {
            let decisive = acc.resolved + acc.failed;
            let resolution_rate = if decisive > 0 {
                acc.resolved as f32 / decisive as f32
            } else {
                0.5 // Unknown prior
            };

            let abstention_rate = if acc.total > 0 {
                acc.abstained as f32 / acc.total as f32
            } else {
                0.0
            };

            let avg_duration_ms = if acc.total > 0 {
                acc.total_duration_ms / acc.total
            } else {
                0
            };

            (
                pattern.clone(),
                ProbeEffectivenessRecord {
                    probe_pattern: pattern,
                    resolution_rate,
                    abstention_rate,
                    avg_duration_ms,
                    sample_count: acc.total as usize,
                },
            )
        })
        .collect()
}

/// Normalize probe command to a pattern.
/// Replaces specific values with wildcards for grouping.
fn normalize_probe_pattern(probe: &str) -> String {
    let parts: Vec<&str> = probe.split_whitespace().collect();
    if parts.is_empty() {
        return probe.to_string();
    }

    let cmd = parts[0];

    // Pattern rules by command
    match cmd {
        "systemctl" => {
            // systemctl status foo.service -> systemctl status *
            if parts.len() >= 2 {
                format!("{} {} *", cmd, parts[1])
            } else {
                probe.to_string()
            }
        }
        "cat" | "head" | "tail" => {
            // cat /etc/foo -> cat /etc/*
            if parts.len() >= 2 {
                let path = parts[1];
                if let Some(parent) = path.rsplit_once('/') {
                    format!("{} {}/*", cmd, parent.0)
                } else {
                    format!("{} *", cmd)
                }
            } else {
                format!("{} *", cmd)
            }
        }
        "journalctl" => {
            // journalctl -u foo -> journalctl -u *
            if parts.len() >= 3 && parts[1] == "-u" {
                format!("{} -u *", cmd)
            } else {
                probe.to_string()
            }
        }
        // Simple commands keep as-is
        "df" | "free" | "uptime" | "uname" | "lsblk" | "lscpu" | "ip" | "ss" | "ps" | "top"
        | "htop" | "iostat" | "vmstat" | "sar" => probe.to_string(),
        // Default: command + wildcard
        _ => format!("{} *", cmd),
    }
}

/// Get top N probes by resolution rate.
pub fn top_probes_by_resolution(
    stats: &HashMap<String, ProbeEffectivenessRecord>,
    n: usize,
) -> Vec<&ProbeEffectivenessRecord> {
    let mut probes: Vec<_> = stats.values().collect();
    probes.sort_by(|a, b| {
        b.resolution_rate
            .partial_cmp(&a.resolution_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    probes.truncate(n);
    probes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent_class::IntentClass;
    use crate::outcome_ledger::RequestMode;

    fn make_record_with_probes(
        id: &str,
        outcome: Outcome,
        duration_ms: u64,
        probes: Vec<String>,
    ) -> OutcomeRecord {
        let mut record = OutcomeRecord::new(
            id,
            RequestMode::Dialogue,
            IntentClass::ReadOnly,
            outcome,
            false,
            duration_ms,
        );
        record.probes_used = Some(probes);
        record
    }

    #[test]
    fn test_normalize_probe_pattern() {
        assert_eq!(
            normalize_probe_pattern("systemctl status nginx.service"),
            "systemctl status *"
        );
        assert_eq!(normalize_probe_pattern("cat /etc/nginx/nginx.conf"), "cat /etc/nginx/*");
        assert_eq!(normalize_probe_pattern("df -h"), "df -h");
        assert_eq!(normalize_probe_pattern("journalctl -u nginx"), "journalctl -u *");
    }

    #[test]
    fn test_aggregate_probe_effectiveness() {
        let records = vec![
            make_record_with_probes("1", Outcome::Resolved, 100, vec!["df -h".to_string()]),
            make_record_with_probes("2", Outcome::Resolved, 100, vec!["df -h".to_string()]),
            make_record_with_probes("3", Outcome::Resolved, 100, vec!["df -h".to_string()]),
            make_record_with_probes("4", Outcome::Resolved, 100, vec!["df -h".to_string()]),
            make_record_with_probes("5", Outcome::Failed, 100, vec!["df -h".to_string()]),
        ];

        let stats = aggregate_probe_effectiveness(&records);
        let df_stats = stats.get("df -h").expect("should have df stats");

        assert_eq!(df_stats.sample_count, 5);
        assert!((df_stats.resolution_rate - 0.8).abs() < 0.01); // 4/5 = 0.8
        assert!((df_stats.abstention_rate - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_minimum_sample_size() {
        // Only 3 samples - should be filtered out
        let records = vec![
            make_record_with_probes("1", Outcome::Resolved, 100, vec!["free -h".to_string()]),
            make_record_with_probes("2", Outcome::Resolved, 100, vec!["free -h".to_string()]),
            make_record_with_probes("3", Outcome::Resolved, 100, vec!["free -h".to_string()]),
        ];

        let stats = aggregate_probe_effectiveness(&records);
        assert!(stats.is_empty()); // Filtered out due to < 5 samples
    }

    #[test]
    fn test_top_probes() {
        let mut stats = HashMap::new();
        stats.insert(
            "df -h".to_string(),
            ProbeEffectivenessRecord {
                probe_pattern: "df -h".to_string(),
                resolution_rate: 0.9,
                abstention_rate: 0.05,
                avg_duration_ms: 50,
                sample_count: 10,
            },
        );
        stats.insert(
            "free -h".to_string(),
            ProbeEffectivenessRecord {
                probe_pattern: "free -h".to_string(),
                resolution_rate: 0.8,
                abstention_rate: 0.1,
                avg_duration_ms: 30,
                sample_count: 8,
            },
        );

        let top = top_probes_by_resolution(&stats, 1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].probe_pattern, "df -h");
    }
}
