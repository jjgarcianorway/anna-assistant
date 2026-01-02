//! Probe Plan - Dynamic Probe Composition.
//!
//! Structures for building and managing probe plans.

use super::super::primitives::{Domain, PrimitiveLibrary};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A probe plan built for a specific ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbePlan {
    /// Ticket ID this plan is for.
    pub ticket_id: String,
    /// Selected primitive IDs.
    pub selected_primitives: Vec<String>,
    /// Why each primitive was selected.
    pub selection_reasons: HashMap<String, String>,
    /// Maximum probes to run.
    pub max_probes: usize,
    /// When the plan was created.
    pub created_at: u64,
}

impl ProbePlan {
    /// Create an empty plan for a ticket.
    pub fn new(ticket_id: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            selected_primitives: Vec::new(),
            selection_reasons: HashMap::new(),
            max_probes: super::super::MAX_PROBES_PER_TICKET,
            created_at: timestamp_now(),
        }
    }

    /// Select primitives based on keywords from intent.
    pub fn select_from_keywords(&mut self, keywords: &[&str], library: &PrimitiveLibrary) {
        // Find all primitives matching any of the keywords
        for primitive in library.find_by_keywords(keywords) {
            if self.selected_primitives.len() >= self.max_probes {
                break;
            }
            if !self.selected_primitives.contains(&primitive.id.to_string()) {
                self.selected_primitives.push(primitive.id.to_string());
                self.selection_reasons
                    .insert(primitive.id.to_string(), format!("matched keywords"));
            }
        }
    }

    /// Select primitives for a specific domain.
    pub fn select_for_domain(&mut self, domain: Domain, library: &PrimitiveLibrary) {
        for primitive in library.for_domain(domain) {
            if self.selected_primitives.len() >= self.max_probes {
                break;
            }
            if !self.selected_primitives.contains(&primitive.id.to_string()) {
                self.selected_primitives.push(primitive.id.to_string());
                self.selection_reasons
                    .insert(primitive.id.to_string(), format!("domain {:?}", domain));
            }
        }
    }

    /// Add a specific primitive by ID.
    pub fn add_primitive(&mut self, id: &str, reason: &str) {
        if self.selected_primitives.len() < self.max_probes
            && !self.selected_primitives.contains(&id.to_string())
        {
            self.selected_primitives.push(id.to_string());
            self.selection_reasons
                .insert(id.to_string(), reason.to_string());
        }
    }

    /// Get number of selected probes.
    pub fn probe_count(&self) -> usize {
        self.selected_primitives.len()
    }

    /// Check if plan is empty.
    pub fn is_empty(&self) -> bool {
        self.selected_primitives.is_empty()
    }
}

/// Selection of primitives with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeSelection {
    /// Selected primitive.
    pub primitive_id: String,
    /// Why it was selected.
    pub reason: String,
    /// Priority (lower = run first).
    pub priority: u8,
    /// Parameters to substitute.
    pub parameters: HashMap<String, String>,
}

impl ProbeSelection {
    /// Create a new selection.
    pub fn new(primitive_id: &str, reason: &str) -> Self {
        Self {
            primitive_id: primitive_id.to_string(),
            reason: reason.to_string(),
            priority: 100,
            parameters: HashMap::new(),
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add parameter.
    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
}

/// Get current timestamp.
fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_plan_creation() {
        let plan = ProbePlan::new("ticket-123");
        assert_eq!(plan.ticket_id, "ticket-123");
        assert!(plan.is_empty());
    }

    #[test]
    fn test_probe_plan_select_keywords() {
        let mut plan = ProbePlan::new("test");
        let library = PrimitiveLibrary::new();

        plan.select_from_keywords(&["boot", "slow"], &library);
        assert!(!plan.is_empty());
        assert!(plan
            .selected_primitives
            .contains(&"sys.boot.analyze".to_string()));
    }

    #[test]
    fn test_probe_plan_max_probes() {
        let mut plan = ProbePlan::new("test");
        plan.max_probes = 2;
        let library = PrimitiveLibrary::new();

        plan.select_for_domain(Domain::Boot, &library);
        plan.select_for_domain(Domain::Memory, &library);

        assert!(plan.probe_count() <= 2);
    }

    #[test]
    fn test_probe_selection() {
        let selection = ProbeSelection::new("sys.boot.analyze", "slow boot complaint")
            .with_priority(1)
            .with_param("service", "nginx");

        assert_eq!(selection.primitive_id, "sys.boot.analyze");
        assert_eq!(selection.priority, 1);
        assert_eq!(
            selection.parameters.get("service"),
            Some(&"nginx".to_string())
        );
    }
}
