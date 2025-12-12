//! Probe-only Fallback (Part E) - v0.0.438.
//!
//! If the specialist times out or fails to parse, we still have probe data.
//! Generate a minimal but useful answer from probes alone:
//! - For status questions: return the raw values
//! - For diagnostics: list the collected data points
//!
//! Never say "I don't know" if we have probe data—present it clearly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single probe result for fallback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeData {
    /// Probe ID.
    pub probe_id: String,
    /// Probe display name.
    pub display_name: String,
    /// Raw value from probe.
    pub value: String,
    /// Unit if applicable.
    pub unit: Option<String>,
    /// Whether this is a key metric.
    pub is_key_metric: bool,
}

impl ProbeData {
    /// Create new probe data.
    pub fn new(probe_id: &str, display_name: &str, value: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            display_name: display_name.to_string(),
            value: value.to_string(),
            unit: None,
            is_key_metric: false,
        }
    }

    /// Set unit.
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = Some(unit.to_string());
        self
    }

    /// Mark as key metric.
    pub fn as_key_metric(mut self) -> Self {
        self.is_key_metric = true;
        self
    }

    /// Format for display.
    pub fn format_display(&self) -> String {
        match &self.unit {
            Some(unit) => format!("{}: {} {}", self.display_name, self.value, unit),
            None => format!("{}: {}", self.display_name, self.value),
        }
    }
}

/// Result of probe-only fallback generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOnlyResult {
    /// Whether we had enough data to generate an answer.
    pub has_data: bool,
    /// Key metrics found.
    pub key_metrics: Vec<ProbeData>,
    /// All probe data.
    pub all_probes: Vec<ProbeData>,
    /// Generated summary.
    pub summary: String,
    /// Reason for fallback.
    pub fallback_reason: String,
}

impl ProbeOnlyResult {
    /// Create empty result (no data).
    pub fn empty(reason: &str) -> Self {
        Self {
            has_data: false,
            key_metrics: Vec::new(),
            all_probes: Vec::new(),
            summary: String::new(),
            fallback_reason: reason.to_string(),
        }
    }

    /// Create result with probe data.
    pub fn with_data(probes: Vec<ProbeData>, reason: &str) -> Self {
        let key_metrics: Vec<_> = probes.iter()
            .filter(|p| p.is_key_metric)
            .cloned()
            .collect();

        let summary = Self::generate_summary(&probes);

        Self {
            has_data: !probes.is_empty(),
            key_metrics,
            all_probes: probes,
            summary,
            fallback_reason: reason.to_string(),
        }
    }

    /// Generate summary from probes.
    fn generate_summary(probes: &[ProbeData]) -> String {
        if probes.is_empty() {
            return "No probe data available.".to_string();
        }

        let key_metrics: Vec<_> = probes.iter()
            .filter(|p| p.is_key_metric)
            .collect();

        if !key_metrics.is_empty() {
            key_metrics.iter()
                .map(|p| p.format_display())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            // Use first few probes
            probes.iter()
                .take(3)
                .map(|p| p.format_display())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    /// Format as user-facing answer.
    pub fn format_answer(&self) -> String {
        if !self.has_data {
            return format!("Unable to retrieve data. ({})", self.fallback_reason);
        }

        let mut parts = Vec::new();

        // Key metrics first
        if !self.key_metrics.is_empty() {
            for m in &self.key_metrics {
                parts.push(m.format_display());
            }
        } else {
            // All probes
            for p in &self.all_probes {
                parts.push(p.format_display());
            }
        }

        parts.join("\n")
    }
}

/// Fallback answer generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackAnswer {
    /// The probe-only result.
    pub result: ProbeOnlyResult,
    /// Confidence (lower for fallback).
    pub confidence: f64,
    /// Whether this is a complete answer.
    pub is_complete: bool,
    /// Warning message for user.
    pub warning: Option<String>,
}

impl FallbackAnswer {
    /// Create from probe result.
    pub fn from_probes(result: ProbeOnlyResult) -> Self {
        let confidence = if result.has_data { 0.6 } else { 0.1 };
        let is_complete = result.has_data && !result.key_metrics.is_empty();

        Self {
            result,
            confidence,
            is_complete,
            warning: Some("Answer generated from probe data only.".to_string()),
        }
    }

    /// Format for user.
    pub fn format_for_user(&self) -> String {
        let answer = self.result.format_answer();

        if let Some(warning) = &self.warning {
            format!("{}\n\n_{}_", answer, warning)
        } else {
            answer
        }
    }
}

/// Engine for generating probe-only fallback answers.
pub struct ProbeFallbackEngine {
    /// Known probe ID to display name mappings.
    display_names: HashMap<String, String>,
    /// Known key metric probe IDs.
    key_metrics: Vec<String>,
}

impl ProbeFallbackEngine {
    /// Create new engine.
    pub fn new() -> Self {
        let mut display_names = HashMap::new();
        display_names.insert("sys.mem.free".to_string(), "Free Memory".to_string());
        display_names.insert("sys.mem.total".to_string(), "Total Memory".to_string());
        display_names.insert("sys.disk.free".to_string(), "Free Disk".to_string());
        display_names.insert("sys.cpu.usage".to_string(), "CPU Usage".to_string());
        display_names.insert("sys.load".to_string(), "Load Average".to_string());
        display_names.insert("net.ping".to_string(), "Ping".to_string());
        display_names.insert("proc.count".to_string(), "Processes".to_string());

        let key_metrics = vec![
            "sys.mem.free".to_string(),
            "sys.disk.free".to_string(),
            "sys.cpu.usage".to_string(),
        ];

        Self {
            display_names,
            key_metrics,
        }
    }

    /// Register a display name.
    pub fn register_display_name(&mut self, probe_id: &str, name: &str) {
        self.display_names.insert(probe_id.to_string(), name.to_string());
    }

    /// Mark a probe as key metric.
    pub fn mark_key_metric(&mut self, probe_id: &str) {
        if !self.key_metrics.contains(&probe_id.to_string()) {
            self.key_metrics.push(probe_id.to_string());
        }
    }

    /// Get display name for probe.
    pub fn get_display_name(&self, probe_id: &str) -> String {
        self.display_names
            .get(probe_id)
            .cloned()
            .unwrap_or_else(|| probe_id.to_string())
    }

    /// Check if probe is key metric.
    pub fn is_key_metric(&self, probe_id: &str) -> bool {
        self.key_metrics.contains(&probe_id.to_string())
    }

    /// Generate fallback from raw probe results.
    pub fn generate_fallback(
        &self,
        probe_results: &HashMap<String, String>,
        fallback_reason: &str,
    ) -> FallbackAnswer {
        if probe_results.is_empty() {
            return FallbackAnswer::from_probes(ProbeOnlyResult::empty(fallback_reason));
        }

        let probes: Vec<ProbeData> = probe_results
            .iter()
            .map(|(id, value)| {
                let mut data = ProbeData::new(id, &self.get_display_name(id), value);
                if self.is_key_metric(id) {
                    data = data.as_key_metric();
                }
                data
            })
            .collect();

        FallbackAnswer::from_probes(ProbeOnlyResult::with_data(probes, fallback_reason))
    }
}

impl Default for ProbeFallbackEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_data() {
        let data = ProbeData::new("sys.mem.free", "Free Memory", "4.2")
            .with_unit("GB")
            .as_key_metric();

        assert!(data.is_key_metric);
        assert_eq!(data.format_display(), "Free Memory: 4.2 GB");
    }

    #[test]
    fn test_probe_only_result_empty() {
        let result = ProbeOnlyResult::empty("timeout");
        assert!(!result.has_data);
        assert!(result.all_probes.is_empty());
    }

    #[test]
    fn test_probe_only_result_with_data() {
        let probes = vec![
            ProbeData::new("sys.mem.free", "Free Memory", "4.2 GB").as_key_metric(),
            ProbeData::new("sys.cpu.usage", "CPU", "25%"),
        ];

        let result = ProbeOnlyResult::with_data(probes, "specialist_timeout");
        assert!(result.has_data);
        assert_eq!(result.key_metrics.len(), 1);
    }

    #[test]
    fn test_fallback_answer() {
        let probes = vec![
            ProbeData::new("sys.mem.free", "Free Memory", "4.2 GB").as_key_metric(),
        ];
        let result = ProbeOnlyResult::with_data(probes, "timeout");
        let answer = FallbackAnswer::from_probes(result);

        assert!(answer.is_complete);
        assert!(answer.confidence > 0.5);
    }

    #[test]
    fn test_fallback_engine() {
        let engine = ProbeFallbackEngine::new();

        let mut results = HashMap::new();
        results.insert("sys.mem.free".to_string(), "4.2 GB".to_string());
        results.insert("sys.cpu.usage".to_string(), "25%".to_string());

        let answer = engine.generate_fallback(&results, "specialist_timeout");
        assert!(answer.result.has_data);
        assert!(!answer.result.key_metrics.is_empty());
    }

    #[test]
    fn test_fallback_engine_empty() {
        let engine = ProbeFallbackEngine::new();
        let results = HashMap::new();

        let answer = engine.generate_fallback(&results, "no_probes");
        assert!(!answer.result.has_data);
        assert!(answer.confidence < 0.5);
    }
}
