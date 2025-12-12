//! Deterministic Routing - v0.0.439.
//!
//! Fixes the "Sofia handles everything" bug with deterministic routing
//! and an intent-to-department map that is NOT hardcoded per question.
//!
//! The taxonomy and evidence model are hardcoded, not the answers.
//!
//! Pipeline:
//! Intent → Evidence requirements → Department → Probe bundle → (Optional) specialist
//!
//! Key principles:
//! - Boot questions ALWAYS route to Performance (not Desktop)
//! - GPU questions ALWAYS route to Hardware (not Storage)
//! - Disk questions ALWAYS route to Storage
//! - RAM/CPU load questions ALWAYS route to Performance
//! - Department override logging when translator conflicts with mapping

pub mod answer_tiers;
pub mod department;
pub mod evidence_gate;
pub mod intent_map;
pub mod intent_schema;

// Re-exports for convenience
pub use answer_tiers::{
    build_boot_perf_tiers, build_cpu_load_tiers, build_disk_usage_tiers, build_gpu_driver_tiers,
    build_mem_status_tiers, AnswerTier, ClarificationBuilder, TieredAnswer,
    MAX_CLARIFICATION_LENGTH,
};
pub use department::{
    DepartmentConflict, DepartmentOwnership, DepartmentRules, DeterministicRouter, RouteResult,
};
pub use evidence_gate::{DirectAnswer, EvidenceGate, EvidenceStatus, GateDecision, ProbeResult};
pub use intent_map::{get_intent_map, IntentMapTable, IntentMapping};
pub use intent_schema::{
    CanonicalIntent, Department, IntentSchemaParser, ParseError, RiskLevel, TicketIntentSchema,
};

/// Version of the deterministic routing system.
pub const ROUTING_VERSION: &str = "1";

/// Maximum translator output tokens.
pub const TRANSLATOR_MAX_TOKENS: usize = 200;

/// Translator temperature (must be 0 for determinism).
pub const TRANSLATOR_TEMPERATURE: f32 = 0.0;

/// Process a user query through the deterministic routing pipeline.
pub fn route_query(query: &str, translator_output: &str) -> Result<RouteResult, String> {
    // 1. Parse translator output
    let schema = IntentSchemaParser::parse(translator_output).map_err(|e| e.message())?;

    // 2. Apply department rules (may override translator)
    let router = DeterministicRouter::new();
    let result = router.route(schema);

    // 3. Log conflict if any
    if let Some(conflict) = &result.conflict {
        tracing::warn!("{}", conflict.log_message());
    }

    Ok(result)
}

/// Get the canonical department for a query intent.
pub fn get_canonical_department(intent: CanonicalIntent) -> Department {
    let rules = DepartmentRules::new();
    rules.get_authoritative_department(intent)
}

/// Check if a query can be answered directly from probes.
pub fn can_answer_from_probes(intent: CanonicalIntent) -> bool {
    let map = IntentMapTable::build();
    map.can_answer_directly(intent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_departments() {
        // Boot -> Performance
        assert_eq!(
            get_canonical_department(CanonicalIntent::BootPerf),
            Department::Performance
        );
        // GPU -> Hardware
        assert_eq!(
            get_canonical_department(CanonicalIntent::GpuInfo),
            Department::Hardware
        );
        // Disk -> Storage
        assert_eq!(
            get_canonical_department(CanonicalIntent::DiskUsage),
            Department::Storage
        );
        // RAM -> Performance
        assert_eq!(
            get_canonical_department(CanonicalIntent::MemStatus),
            Department::Performance
        );
    }

    #[test]
    fn test_direct_answer_capability() {
        // Facts can be answered directly
        assert!(can_answer_from_probes(CanonicalIntent::MemStatus));
        assert!(can_answer_from_probes(CanonicalIntent::DiskUsage));
        assert!(can_answer_from_probes(CanonicalIntent::BootPerf));
        // "Health" synthesis cannot
        assert!(!can_answer_from_probes(CanonicalIntent::SvcHealth));
    }

    #[test]
    fn test_route_query_with_conflict() {
        let translator_json = r#"{
            "user_query": "why is boot slow?",
            "intent": "boot_perf",
            "department": "Desktop",
            "required_evidence": [],
            "need_clarification": false,
            "risk_level": "read_only"
        }"#;

        let result = route_query("why is boot slow?", translator_json).unwrap();

        // Should be overridden to Performance
        assert!(result.was_overridden);
        assert_eq!(result.schema.department, Department::Performance);
    }

    #[test]
    fn test_route_query_correct_department() {
        let translator_json = r#"{
            "user_query": "what GPU do I have?",
            "intent": "gpu_info",
            "department": "Hardware",
            "required_evidence": ["lspci_gpu"],
            "need_clarification": false,
            "risk_level": "read_only"
        }"#;

        let result = route_query("what GPU do I have?", translator_json).unwrap();

        // No override needed
        assert!(!result.was_overridden);
        assert_eq!(result.schema.department, Department::Hardware);
    }
}
