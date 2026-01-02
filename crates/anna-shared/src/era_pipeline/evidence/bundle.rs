//! EvidenceBundle - Core data structure for collected evidence.

use super::fact_value::FactValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Probe execution error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeError {
    /// Probe ID.
    pub probe_id: String,
    /// Error message.
    pub message: String,
    /// Whether probe timed out.
    pub timeout: bool,
}

/// Evidence bundle containing all collected facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// Case ID.
    pub case_id: String,
    /// Atomic facts (namespaced, e.g., "memory.free_gib").
    pub facts: HashMap<String, FactValue>,
    /// Raw probe outputs.
    pub raw: HashMap<String, String>,
    /// Confidence per domain (0.0-1.0).
    pub confidence: HashMap<String, f64>,
    /// Missing facts that could not be collected.
    pub missing: Vec<String>,
    /// Probe execution errors.
    pub errors: Vec<ProbeError>,
}

impl EvidenceBundle {
    /// Create new empty bundle.
    pub fn new(case_id: &str) -> Self {
        Self {
            case_id: case_id.to_string(),
            facts: HashMap::new(),
            raw: HashMap::new(),
            confidence: HashMap::new(),
            missing: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add a fact.
    pub fn add_fact(&mut self, name: &str, value: FactValue) {
        // Update domain confidence
        if let Some(domain) = name.split('.').next() {
            self.confidence.entry(domain.to_string()).or_insert(1.0);
        }
        self.facts.insert(name.to_string(), value);
    }

    /// Add raw probe output.
    pub fn add_raw(&mut self, probe_id: &str, output: &str) {
        self.raw.insert(probe_id.to_string(), output.to_string());
    }

    /// Mark a fact as missing.
    pub fn mark_missing(&mut self, fact_name: &str) {
        if !self.missing.contains(&fact_name.to_string()) {
            self.missing.push(fact_name.to_string());
        }
        // Reduce domain confidence
        if let Some(domain) = fact_name.split('.').next() {
            let conf = self.confidence.entry(domain.to_string()).or_insert(1.0);
            *conf *= 0.5;
        }
    }

    /// Record a probe error.
    pub fn record_error(&mut self, probe_id: &str, message: &str, timeout: bool) {
        self.errors.push(ProbeError {
            probe_id: probe_id.to_string(),
            message: message.to_string(),
            timeout,
        });
    }

    /// Get a fact value.
    pub fn get(&self, name: &str) -> Option<&FactValue> {
        self.facts.get(name)
    }

    /// Get a fact as number.
    pub fn get_number(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|v| v.as_number())
    }

    /// Get a fact as string.
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.as_string())
    }

    /// Get a fact as bool.
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|v| v.as_bool())
    }

    /// Get a fact as list.
    pub fn get_list(&self, name: &str) -> Option<&[String]> {
        self.get(name).and_then(|v| v.as_list())
    }

    /// Check if a fact exists.
    pub fn has(&self, name: &str) -> bool {
        self.facts.contains_key(name)
    }

    /// Check if all required facts are present.
    pub fn has_all(&self, required: &[&str]) -> bool {
        required.iter().all(|f| self.has(f))
    }

    /// Get missing from required list.
    pub fn get_missing(&self, required: &[&str]) -> Vec<String> {
        required
            .iter()
            .filter(|f| !self.has(f))
            .map(|f| f.to_string())
            .collect()
    }

    /// Get domain confidence.
    pub fn domain_confidence(&self, domain: &str) -> f64 {
        self.confidence.get(domain).copied().unwrap_or(0.0)
    }

    /// Get overall confidence.
    pub fn overall_confidence(&self) -> f64 {
        if self.confidence.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.confidence.values().sum();
        sum / self.confidence.len() as f64
    }

    /// Check if bundle is complete (no missing facts).
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.errors.is_empty()
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Serialize to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Extract domain from fact name.
pub fn fact_domain(fact_name: &str) -> &str {
    fact_name.split('.').next().unwrap_or(fact_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_bundle() {
        let mut bundle = EvidenceBundle::new("DSK-0127");
        bundle.add_fact("memory.free_gib", FactValue::Number(17.0));
        bundle.add_fact("memory.total_gib", FactValue::Number(31.0));

        assert!(bundle.has("memory.free_gib"));
        assert!(!bundle.has("cpu.temp"));
        assert_eq!(bundle.get_number("memory.free_gib"), Some(17.0));

        assert!(bundle.has_all(&["memory.free_gib", "memory.total_gib"]));
        assert!(!bundle.has_all(&["memory.free_gib", "cpu.temp"]));
    }

    #[test]
    fn test_domain_confidence() {
        let mut bundle = EvidenceBundle::new("DSK-0127");
        bundle.add_fact("memory.free_gib", FactValue::Number(17.0));
        bundle.add_fact("boot.total_time_s", FactValue::Number(25.6));

        assert_eq!(bundle.domain_confidence("memory"), 1.0);
        assert_eq!(bundle.domain_confidence("boot"), 1.0);

        bundle.mark_missing("memory.used_gib");
        assert!(bundle.domain_confidence("memory") < 1.0);
    }
}
