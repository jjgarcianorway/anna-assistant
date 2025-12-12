//! EvidenceBundle (Part B) - v0.0.441.
//!
//! Core data model for structured evidence.
//!
//! All probes output into this structure:
//! - facts: Atomic, namespaced, typed values
//! - raw: Original probe output
//! - confidence: Per-domain confidence scores
//! - missing: Facts that could not be collected

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Typed fact value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FactValue {
    /// Numeric value (integer or float).
    Number(f64),
    /// String value.
    String(String),
    /// Boolean value.
    Bool(bool),
    /// List of strings.
    List(Vec<String>),
    /// Null/missing.
    Null,
}

impl FactValue {
    /// Create a numeric fact.
    pub fn number(n: f64) -> Self {
        Self::Number(n)
    }

    /// Create a string fact.
    pub fn string(s: &str) -> Self {
        Self::String(s.to_string())
    }

    /// Create a boolean fact.
    pub fn bool(b: bool) -> Self {
        Self::Bool(b)
    }

    /// Create a list fact.
    pub fn list(items: Vec<String>) -> Self {
        Self::List(items)
    }

    /// Check if null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get as number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as list.
    pub fn as_list(&self) -> Option<&[String]> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

    /// Format for display.
    pub fn display(&self) -> String {
        match self {
            Self::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{:.1}", n)
                }
            }
            Self::String(s) => s.clone(),
            Self::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
            Self::List(l) => l.join(", "),
            Self::Null => "N/A".to_string(),
        }
    }
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

/// Builder for EvidenceBundle.
pub struct EvidenceBundleBuilder {
    bundle: EvidenceBundle,
}

impl EvidenceBundleBuilder {
    /// Create new builder.
    pub fn new(case_id: &str) -> Self {
        Self {
            bundle: EvidenceBundle::new(case_id),
        }
    }

    /// Add a numeric fact.
    pub fn fact_number(mut self, name: &str, value: f64) -> Self {
        self.bundle.add_fact(name, FactValue::Number(value));
        self
    }

    /// Add a string fact.
    pub fn fact_string(mut self, name: &str, value: &str) -> Self {
        self.bundle
            .add_fact(name, FactValue::String(value.to_string()));
        self
    }

    /// Add a boolean fact.
    pub fn fact_bool(mut self, name: &str, value: bool) -> Self {
        self.bundle.add_fact(name, FactValue::Bool(value));
        self
    }

    /// Add a list fact.
    pub fn fact_list(mut self, name: &str, items: Vec<String>) -> Self {
        self.bundle.add_fact(name, FactValue::List(items));
        self
    }

    /// Add raw output.
    pub fn raw(mut self, probe_id: &str, output: &str) -> Self {
        self.bundle.add_raw(probe_id, output);
        self
    }

    /// Mark missing.
    pub fn missing(mut self, fact_name: &str) -> Self {
        self.bundle.mark_missing(fact_name);
        self
    }

    /// Build the bundle.
    pub fn build(self) -> EvidenceBundle {
        self.bundle
    }
}

/// Extract domain from fact name.
pub fn fact_domain(fact_name: &str) -> &str {
    fact_name.split('.').next().unwrap_or(fact_name)
}

/// Standard fact extractors for common probes.
pub mod extractors {
    use super::*;

    /// Extract memory facts from `free -h` output.
    pub fn extract_memory(output: &str) -> HashMap<String, FactValue> {
        let mut facts = HashMap::new();

        for line in output.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Some(total) = parse_size_gib(parts.get(1).unwrap_or(&"")) {
                        facts.insert("memory.total_gib".to_string(), FactValue::Number(total));
                    }
                    if let Some(used) = parse_size_gib(parts.get(2).unwrap_or(&"")) {
                        facts.insert("memory.used_gib".to_string(), FactValue::Number(used));
                    }
                    if let Some(free) = parse_size_gib(parts.get(3).unwrap_or(&"")) {
                        facts.insert("memory.free_gib".to_string(), FactValue::Number(free));
                    }
                    if let Some(available) = parse_size_gib(parts.get(6).unwrap_or(&"")) {
                        facts.insert(
                            "memory.available_gib".to_string(),
                            FactValue::Number(available),
                        );
                    }
                }
            }
        }

        facts
    }

    /// Extract boot facts from `systemd-analyze` output.
    pub fn extract_boot_time(output: &str) -> HashMap<String, FactValue> {
        let mut facts = HashMap::new();

        // Parse "Startup finished in Xs (firmware) + Xs (loader) + Xs (kernel) + Xs (userspace) = Xs"
        if let Some(total) = extract_total_seconds(output) {
            facts.insert("boot.total_time_s".to_string(), FactValue::Number(total));
        }

        facts
    }

    /// Extract blame list from `systemd-analyze blame` output.
    pub fn extract_blame(output: &str) -> HashMap<String, FactValue> {
        let mut facts = HashMap::new();
        let mut services = Vec::new();

        for line in output.lines().take(10) {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                let time = parts[0];
                let service = parts[1];
                services.push(format!("{} ({})", service, time));
            }
        }

        if !services.is_empty() {
            // First one is slowest
            if let Some(first) = services.first() {
                facts.insert(
                    "boot.slowest_service".to_string(),
                    FactValue::String(first.clone()),
                );
            }
            facts.insert("boot.blame".to_string(), FactValue::List(services));
        }

        facts
    }

    /// Extract disk facts from `df -h` output.
    pub fn extract_disk(output: &str) -> HashMap<String, FactValue> {
        let mut facts = HashMap::new();

        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let mount = parts[5];
                let used_pct = parts[4].trim_end_matches('%');

                if mount == "/" {
                    if let Ok(pct) = used_pct.parse::<f64>() {
                        facts.insert("disk.root_used_pct".to_string(), FactValue::Number(pct));
                    }
                    if let Some(avail) = parse_size_gib(parts[3]) {
                        facts.insert("disk.root_free_gib".to_string(), FactValue::Number(avail));
                    }
                }
            }
        }

        facts
    }

    /// Extract failed services from `systemctl --failed` output.
    pub fn extract_failed_services(output: &str) -> HashMap<String, FactValue> {
        let mut facts = HashMap::new();
        let mut failed = Vec::new();

        for line in output.lines() {
            if line.contains("failed") && line.contains(".service") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(service) = parts.first() {
                    failed.push(service.to_string());
                }
            }
        }

        facts.insert(
            "services.failed_count".to_string(),
            FactValue::Number(failed.len() as f64),
        );
        facts.insert("services.failed_list".to_string(), FactValue::List(failed));

        facts
    }

    /// Parse size string (e.g., "16Gi", "500Mi") to GiB.
    fn parse_size_gib(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("Gi") || s.ends_with("G") {
            s.trim_end_matches("Gi").trim_end_matches("G").parse().ok()
        } else if s.ends_with("Mi") || s.ends_with("M") {
            s.trim_end_matches("Mi")
                .trim_end_matches("M")
                .parse::<f64>()
                .ok()
                .map(|m| m / 1024.0)
        } else if s.ends_with("Ki") || s.ends_with("K") {
            s.trim_end_matches("Ki")
                .trim_end_matches("K")
                .parse::<f64>()
                .ok()
                .map(|k| k / (1024.0 * 1024.0))
        } else {
            s.parse().ok()
        }
    }

    /// Extract total seconds from systemd-analyze output.
    fn extract_total_seconds(output: &str) -> Option<f64> {
        // Look for "= Xs" or "= Xmin Xs"
        if let Some(idx) = output.find('=') {
            let after_eq = &output[idx + 1..];
            let total_str = after_eq.trim();

            // Parse "Xmin Ys" or "Xs"
            if total_str.contains("min") {
                let parts: Vec<&str> = total_str.split_whitespace().collect();
                let mut total = 0.0;
                for part in parts {
                    if part.ends_with("min") {
                        if let Ok(mins) = part.trim_end_matches("min").parse::<f64>() {
                            total += mins * 60.0;
                        }
                    } else if part.ends_with('s') && !part.ends_with("ms") {
                        if let Ok(secs) = part.trim_end_matches('s').parse::<f64>() {
                            total += secs;
                        }
                    }
                }
                return Some(total);
            } else if total_str.ends_with('s') {
                return total_str.trim_end_matches('s').trim().parse().ok();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_value_types() {
        let num = FactValue::number(17.5);
        assert_eq!(num.as_number(), Some(17.5));
        assert_eq!(num.display(), "17.5");

        let s = FactValue::string("hello");
        assert_eq!(s.as_string(), Some("hello"));

        let b = FactValue::bool(true);
        assert_eq!(b.as_bool(), Some(true));
        assert_eq!(b.display(), "Yes");

        let list = FactValue::list(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            list.as_list(),
            Some(vec!["a".to_string(), "b".to_string()].as_slice())
        );
    }

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
    fn test_evidence_bundle_builder() {
        let bundle = EvidenceBundleBuilder::new("DSK-0127")
            .fact_number("memory.free_gib", 17.0)
            .fact_string("cpu.model", "AMD Ryzen 9")
            .fact_bool("disk.trim_enabled", true)
            .build();

        assert_eq!(bundle.get_number("memory.free_gib"), Some(17.0));
        assert_eq!(bundle.get_string("cpu.model"), Some("AMD Ryzen 9"));
        assert_eq!(bundle.get_bool("disk.trim_enabled"), Some(true));
    }

    #[test]
    fn test_memory_extraction() {
        let output =
            "              total        used        free      shared  buff/cache   available
Mem:           31Gi       8.2Gi        15Gi       1.2Gi       7.8Gi        21Gi";

        let facts = extractors::extract_memory(output);
        assert!(facts.get("memory.total_gib").is_some());
        assert!(facts.get("memory.free_gib").is_some());
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
