//! Deterministic Router - v0.0.439.
//!
//! Main router that applies all department rules.

use super::super::intent_map_table::IntentMapTable;
use super::super::intent_schema::TicketIntentSchema;
use super::conflict::DepartmentConflict;
use super::rules::DepartmentRules;

/// Route result after applying ownership rules.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// Final schema with correct department.
    pub schema: TicketIntentSchema,
    /// Whether department was overridden.
    pub was_overridden: bool,
    /// Conflict details if overridden.
    pub conflict: Option<DepartmentConflict>,
    /// Required probes from intent map.
    pub required_probes: Vec<String>,
    /// Optional probes from intent map.
    pub optional_probes: Vec<String>,
}

/// Router that applies all rules.
pub struct DeterministicRouter {
    /// Department rules.
    rules: DepartmentRules,
    /// Intent map.
    intent_map: IntentMapTable,
}

impl DeterministicRouter {
    /// Create new router.
    pub fn new() -> Self {
        Self {
            rules: DepartmentRules::new(),
            intent_map: IntentMapTable::build(),
        }
    }

    /// Route a schema, enforcing all rules.
    pub fn route(&self, schema: TicketIntentSchema) -> RouteResult {
        let (corrected, conflict) = self.rules.enforce_ownership(schema);
        let was_overridden = conflict.is_some();

        // Get probes from intent map
        let required_probes = self
            .intent_map
            .get_required_probes(corrected.intent)
            .into_iter()
            .map(String::from)
            .collect();
        let optional_probes = self
            .intent_map
            .get_optional_probes(corrected.intent)
            .into_iter()
            .map(String::from)
            .collect();

        RouteResult {
            schema: corrected,
            was_overridden,
            conflict,
            required_probes,
            optional_probes,
        }
    }

    /// Get department rules.
    pub fn rules(&self) -> &DepartmentRules {
        &self.rules
    }
}

impl Default for DeterministicRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::intent_schema::{CanonicalIntent, Department};
    use super::*;

    #[test]
    fn test_router_full_flow() {
        let router = DeterministicRouter::new();

        // Wrong department from translator
        let schema = TicketIntentSchema::new(
            "what GPU driver am I using?",
            CanonicalIntent::GpuDriver,
            Department::Storage, // WRONG
        );

        let result = router.route(schema);
        assert!(result.was_overridden);
        assert_eq!(result.schema.department, Department::Hardware);
        assert!(!result.required_probes.is_empty());
    }
}
