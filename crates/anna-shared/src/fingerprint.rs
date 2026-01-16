//! Fact Fingerprinting.
//!
//! Phase 27: Normalized hashing of probe outputs for similarity detection.
//! Used to calibrate confidence against historically similar states.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use xxhash_rust::xxh64::xxh64;

/// A fingerprint of system state derived from probe outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactFingerprint {
    /// xxhash64 of normalized, concatenated probe outputs
    pub hash: u64,
    /// Normalized lines for similarity calculation
    pub normalized_lines: Vec<String>,
    /// Timestamp of fingerprint creation
    pub ts_utc: String,
}

impl FactFingerprint {
    /// Create fingerprint from raw probe outputs.
    pub fn from_probe_outputs(outputs: &[&str]) -> Self {
        let mut all_lines: Vec<String> = Vec::new();

        for output in outputs {
            let normalized = normalize_output(output);
            all_lines.extend(normalized);
        }

        // Sort for deterministic hashing
        all_lines.sort();
        all_lines.dedup();

        // Compute hash
        let combined = all_lines.join("\n");
        let hash = xxh64(combined.as_bytes(), 0);

        Self {
            hash,
            normalized_lines: all_lines,
            ts_utc: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Compute Jaccard similarity with another fingerprint.
    /// Returns 0.0-1.0 where 1.0 is identical.
    pub fn similarity(&self, other: &FactFingerprint) -> f32 {
        if self.normalized_lines.is_empty() && other.normalized_lines.is_empty() {
            return 1.0;
        }
        if self.normalized_lines.is_empty() || other.normalized_lines.is_empty() {
            return 0.0;
        }

        let set_a: HashSet<&String> = self.normalized_lines.iter().collect();
        let set_b: HashSet<&String> = other.normalized_lines.iter().collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Check if fingerprints match exactly by hash.
    pub fn matches(&self, other: &FactFingerprint) -> bool {
        self.hash == other.hash
    }
}

/// Normalize probe output by removing volatile data.
fn normalize_output(output: &str) -> Vec<String> {
    // Regex patterns for volatile data
    lazy_static_patterns();

    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| normalize_line(line))
        .filter(|line| !line.is_empty())
        .collect()
}

/// Normalize a single line.
fn normalize_line(line: &str) -> String {
    let mut normalized = line.to_lowercase();

    // Strip dates (YYYY-MM-DD)
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    normalized = date_re.replace_all(&normalized, "[date]").to_string();

    // Strip times (HH:MM:SS or HH:MM)
    let time_re = Regex::new(r"\d{1,2}:\d{2}(:\d{2})?").unwrap();
    normalized = time_re.replace_all(&normalized, "[time]").to_string();

    // Strip PIDs in paths (/proc/1234/ -> /proc/[pid]/)
    let pid_path_re = Regex::new(r"/proc/\d+/").unwrap();
    normalized = pid_path_re.replace_all(&normalized, "/proc/[pid]/").to_string();

    // Strip standalone PIDs (PID: 12345 -> pid: [pid])
    let pid_re = Regex::new(r"(pid[:\s]*)\d+").unwrap();
    normalized = pid_re.replace_all(&normalized, "$1[pid]").to_string();

    // Strip memory addresses (0x7fff... -> [addr])
    let addr_re = Regex::new(r"0x[0-9a-f]+").unwrap();
    normalized = addr_re.replace_all(&normalized, "[addr]").to_string();

    // Strip percentages with decimals (normalize precision)
    let pct_re = Regex::new(r"(\d+)\.\d+%").unwrap();
    normalized = pct_re.replace_all(&normalized, "$1%").to_string();

    // Strip UUIDs
    let uuid_re = Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
        .unwrap();
    normalized = uuid_re.replace_all(&normalized, "[uuid]").to_string();

    // Collapse multiple spaces
    let space_re = Regex::new(r"\s+").unwrap();
    normalized = space_re.replace_all(&normalized, " ").to_string();

    normalized.trim().to_string()
}

/// Placeholder for lazy_static pattern initialization.
fn lazy_static_patterns() {
    // Patterns are created inline in normalize_line
    // This function exists for future optimization with lazy_static
}

/// Find the most similar historical fingerprint.
pub fn find_best_match<'a>(
    current: &FactFingerprint,
    historical: &'a [FactFingerprint],
) -> Option<(f32, &'a FactFingerprint)> {
    if historical.is_empty() {
        return None;
    }

    let mut best_score = 0.0f32;
    let mut best_match: Option<&FactFingerprint> = None;

    for fp in historical {
        let score = current.similarity(fp);
        if score > best_score {
            best_score = score;
            best_match = Some(fp);
        }
    }

    best_match.map(|fp| (best_score, fp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_line_dates() {
        let line = "Created on 2024-01-15 at startup";
        let normalized = normalize_line(line);
        assert!(normalized.contains("[date]"));
        assert!(!normalized.contains("2024"));
    }

    #[test]
    fn test_normalize_line_times() {
        let line = "Started at 14:30:45";
        let normalized = normalize_line(line);
        // After lowercase, [TIME] becomes [time]
        assert!(normalized.contains("[time]"));
        assert!(!normalized.contains("14:30"));
    }

    #[test]
    fn test_normalize_line_pids() {
        let line = "Process in /proc/12345/status has PID: 12345";
        let normalized = normalize_line(line);
        // After lowercase, [PID] becomes [pid]
        assert!(normalized.contains("/proc/[pid]/"));
        assert!(normalized.contains("[pid]"));
    }

    #[test]
    fn test_fingerprint_from_outputs() {
        let outputs = vec!["Active: active (running)", "Memory: 100M"];
        let fp = FactFingerprint::from_probe_outputs(&outputs);

        assert!(!fp.normalized_lines.is_empty());
        assert!(fp.hash != 0);
    }

    #[test]
    fn test_fingerprint_similarity_identical() {
        let outputs = vec!["Active: active (running)", "Memory: 100M"];
        let fp1 = FactFingerprint::from_probe_outputs(&outputs);
        let fp2 = FactFingerprint::from_probe_outputs(&outputs);

        assert!((fp1.similarity(&fp2) - 1.0).abs() < 0.01);
        assert!(fp1.matches(&fp2));
    }

    #[test]
    fn test_fingerprint_similarity_partial() {
        let outputs1 = vec!["Active: active (running)", "Memory: 100M"];
        let outputs2 = vec!["Active: active (running)", "Memory: 200M"];

        let fp1 = FactFingerprint::from_probe_outputs(&outputs1);
        let fp2 = FactFingerprint::from_probe_outputs(&outputs2);

        let sim = fp1.similarity(&fp2);
        assert!(sim > 0.0 && sim < 1.0); // Partial match
    }

    #[test]
    fn test_fingerprint_similarity_disjoint() {
        let outputs1 = vec!["foo bar baz"];
        let outputs2 = vec!["completely different content"];

        let fp1 = FactFingerprint::from_probe_outputs(&outputs1);
        let fp2 = FactFingerprint::from_probe_outputs(&outputs2);

        assert!((fp1.similarity(&fp2) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize_volatile_data_produces_same_hash() {
        // Same content with different timestamps should hash the same
        let outputs1 = vec!["Started at 2024-01-15 10:30:00"];
        let outputs2 = vec!["Started at 2025-12-01 23:45:59"];

        let fp1 = FactFingerprint::from_probe_outputs(&outputs1);
        let fp2 = FactFingerprint::from_probe_outputs(&outputs2);

        assert!(fp1.matches(&fp2));
    }

    #[test]
    fn test_find_best_match() {
        let current = FactFingerprint::from_probe_outputs(&["active running memory 100m"]);

        let historical = vec![
            FactFingerprint::from_probe_outputs(&["inactive stopped"]),
            FactFingerprint::from_probe_outputs(&["active running memory 200m"]),
            FactFingerprint::from_probe_outputs(&["active running memory 100m"]),
        ];

        let result = find_best_match(&current, &historical);
        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!((score - 1.0).abs() < 0.01); // Exact match
    }
}
