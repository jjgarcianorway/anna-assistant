//! Intent Map Table - Maps intent to department and default probes.
//!
//! This is NOT per-question. It is per-intent.
//! The taxonomy is hardcoded, not the answers.
//!
//! Part of the Deterministic Intent Map (v0.0.439).

use std::collections::HashMap;

use super::intent_mapping::IntentMapping;
use super::intent_mappings_desktop::register_desktop_mappings;
use super::intent_mappings_hardware::register_hardware_mappings;
use super::intent_mappings_network::register_network_mappings;
use super::intent_mappings_performance::register_performance_mappings;
use super::intent_mappings_security::register_security_mappings;
use super::intent_mappings_services::register_services_mappings;
use super::intent_mappings_storage::register_storage_mappings;
use super::intent_schema::{CanonicalIntent, Department};

/// The deterministic intent mapping table.
pub struct IntentMapTable {
    mappings: HashMap<CanonicalIntent, IntentMapping>,
}

impl IntentMapTable {
    /// Build the canonical intent map.
    pub fn build() -> Self {
        let mut mappings = HashMap::new();

        // Register all department mappings
        register_performance_mappings(&mut mappings);
        register_storage_mappings(&mut mappings);
        register_services_mappings(&mut mappings);
        register_network_mappings(&mut mappings);
        register_hardware_mappings(&mut mappings);
        register_desktop_mappings(&mut mappings);
        register_security_mappings(&mut mappings);

        // Unknown intent
        mappings.insert(
            CanonicalIntent::Unknown,
            IntentMapping {
                intent: CanonicalIntent::Unknown,
                department: Department::Services, // Default to services
                required_probes: vec![],
                optional_probes: vec![],
                can_answer_from_probes: false,
                description: "Unknown intent - needs clarification",
            },
        );

        Self { mappings }
    }

    /// Get mapping for an intent.
    pub fn get(&self, intent: CanonicalIntent) -> Option<&IntentMapping> {
        self.mappings.get(&intent)
    }

    /// Get the correct department for an intent.
    pub fn get_department(&self, intent: CanonicalIntent) -> Department {
        self.mappings
            .get(&intent)
            .map(|m| m.department)
            .unwrap_or(Department::Services)
    }

    /// Get required probes for an intent.
    pub fn get_required_probes(&self, intent: CanonicalIntent) -> Vec<&str> {
        self.mappings
            .get(&intent)
            .map(|m| m.required_probes.clone())
            .unwrap_or_default()
    }

    /// Get optional probes for an intent.
    pub fn get_optional_probes(&self, intent: CanonicalIntent) -> Vec<&str> {
        self.mappings
            .get(&intent)
            .map(|m| m.optional_probes.clone())
            .unwrap_or_default()
    }

    /// Check if intent can be answered directly from probes.
    pub fn can_answer_directly(&self, intent: CanonicalIntent) -> bool {
        self.mappings
            .get(&intent)
            .map(|m| m.can_answer_from_probes)
            .unwrap_or(false)
    }

    /// List all intents for a department.
    pub fn intents_for_department(&self, dept: Department) -> Vec<CanonicalIntent> {
        self.mappings
            .values()
            .filter(|m| m.department == dept)
            .map(|m| m.intent)
            .collect()
    }
}

impl Default for IntentMapTable {
    fn default() -> Self {
        Self::build()
    }
}

/// Global intent map (lazy static alternative).
pub fn get_intent_map() -> IntentMapTable {
    IntentMapTable::build()
}
