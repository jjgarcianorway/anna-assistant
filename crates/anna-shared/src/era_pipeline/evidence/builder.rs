//! EvidenceBundleBuilder - Fluent builder for constructing evidence bundles.

use super::bundle::EvidenceBundle;
use super::fact_value::FactValue;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
