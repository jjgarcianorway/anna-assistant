//! Recipe Templates with Promotion Logic (v0.0.435).
//!
//! Recipes are parameterized templates that require proof before promotion.
//! A candidate must succeed N times (default 3) before becoming a promoted recipe.

use super::citations::CitationStore;
use super::probe_plan::ProbeOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A recipe template - parameterized solution pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeTemplate {
    /// Unique recipe ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// What problem this solves.
    pub problem_pattern: String,
    /// Required probes to run.
    pub required_probes: Vec<String>,
    /// Conditions that must be met (probe_id -> expected pattern).
    pub preconditions: HashMap<String, String>,
    /// Solution steps (parameterized).
    pub steps: Vec<RecipeStep>,
    /// Expected outcome.
    pub expected_outcome: String,
    /// Tags for matching.
    pub tags: Vec<String>,
}

impl RecipeTemplate {
    /// Create a new recipe template.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            problem_pattern: String::new(),
            required_probes: Vec::new(),
            preconditions: HashMap::new(),
            steps: Vec::new(),
            expected_outcome: String::new(),
            tags: Vec::new(),
        }
    }

    /// Set problem pattern.
    pub fn with_problem(mut self, pattern: &str) -> Self {
        self.problem_pattern = pattern.to_string();
        self
    }

    /// Add required probe.
    pub fn with_probe(mut self, probe_id: &str) -> Self {
        self.required_probes.push(probe_id.to_string());
        self
    }

    /// Add precondition.
    pub fn with_precondition(mut self, probe_id: &str, pattern: &str) -> Self {
        self.preconditions
            .insert(probe_id.to_string(), pattern.to_string());
        self
    }

    /// Add solution step.
    pub fn with_step(mut self, step: RecipeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Set expected outcome.
    pub fn with_outcome(mut self, outcome: &str) -> Self {
        self.expected_outcome = outcome.to_string();
        self
    }

    /// Add tag.
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Check if preconditions are met based on probe outputs.
    pub fn check_preconditions(&self, outputs: &[ProbeOutput]) -> bool {
        for (probe_id, pattern) in &self.preconditions {
            let found = outputs.iter().any(|o| {
                o.primitive_id == *probe_id
                    && o.success()
                    && o.raw_output
                        .to_lowercase()
                        .contains(&pattern.to_lowercase())
            });
            if !found {
                return false;
            }
        }
        true
    }

    /// Instantiate recipe with parameters.
    pub fn instantiate(&self, params: &HashMap<String, String>) -> RecipeInstance {
        let steps: Vec<String> = self
            .steps
            .iter()
            .map(|s| substitute_params(&s.instruction, params))
            .collect();

        RecipeInstance {
            recipe_id: self.id.clone(),
            parameters: params.clone(),
            steps,
            current_step: 0,
            outcome: None,
        }
    }
}

/// A step in a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// Step number (1-indexed).
    pub number: u8,
    /// Instruction (may contain {param} placeholders).
    pub instruction: String,
    /// Command to run (if any).
    pub command: Option<String>,
    /// Whether this step requires confirmation.
    pub requires_confirmation: bool,
    /// Expected result pattern.
    pub expected_result: Option<String>,
}

impl RecipeStep {
    /// Create a new recipe step.
    pub fn new(number: u8, instruction: &str) -> Self {
        Self {
            number,
            instruction: instruction.to_string(),
            command: None,
            requires_confirmation: false,
            expected_result: None,
        }
    }

    /// With command to execute.
    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    /// Require user confirmation.
    pub fn with_confirmation(mut self) -> Self {
        self.requires_confirmation = true;
        self
    }

    /// With expected result.
    pub fn with_expected(mut self, pattern: &str) -> Self {
        self.expected_result = Some(pattern.to_string());
        self
    }
}

/// An instantiated recipe ready for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInstance {
    /// Recipe ID.
    pub recipe_id: String,
    /// Parameters used.
    pub parameters: HashMap<String, String>,
    /// Instantiated steps.
    pub steps: Vec<String>,
    /// Current step index.
    pub current_step: usize,
    /// Final outcome.
    pub outcome: Option<RecipeOutcome>,
}

impl RecipeInstance {
    /// Get next step.
    pub fn next_step(&self) -> Option<&str> {
        self.steps.get(self.current_step).map(|s| s.as_str())
    }

    /// Advance to next step.
    pub fn advance(&mut self) {
        if self.current_step < self.steps.len() {
            self.current_step += 1;
        }
    }

    /// Mark as complete.
    pub fn complete(&mut self, success: bool) {
        self.outcome = Some(if success {
            RecipeOutcome::Success
        } else {
            RecipeOutcome::Failed
        });
    }

    /// Check if complete.
    pub fn is_complete(&self) -> bool {
        self.outcome.is_some()
    }
}

/// Outcome of recipe execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeOutcome {
    /// Recipe succeeded.
    Success,
    /// Recipe failed.
    Failed,
    /// User cancelled.
    Cancelled,
}

/// A candidate recipe awaiting promotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCandidate {
    /// Template this is based on.
    pub template: RecipeTemplate,
    /// Successful executions.
    pub confirmations: Vec<Confirmation>,
    /// Failed executions.
    pub failures: Vec<Failure>,
    /// When first seen.
    pub first_seen: u64,
    /// When last confirmed.
    pub last_confirmed: Option<u64>,
}

impl RecipeCandidate {
    /// Create a new candidate.
    pub fn new(template: RecipeTemplate) -> Self {
        Self {
            template,
            confirmations: Vec::new(),
            failures: Vec::new(),
            first_seen: timestamp_now(),
            last_confirmed: None,
        }
    }

    /// Record a successful execution.
    pub fn record_success(&mut self, ticket_id: &str, citations: &CitationStore) {
        self.confirmations.push(Confirmation {
            ticket_id: ticket_id.to_string(),
            timestamp: timestamp_now(),
            citation_count: citations.citation_count(),
        });
        self.last_confirmed = Some(timestamp_now());
    }

    /// Record a failed execution.
    pub fn record_failure(&mut self, ticket_id: &str, reason: &str) {
        self.failures.push(Failure {
            ticket_id: ticket_id.to_string(),
            timestamp: timestamp_now(),
            reason: reason.to_string(),
        });
    }

    /// Check if ready for promotion.
    pub fn ready_for_promotion(&self) -> bool {
        self.confirmations.len() >= super::MIN_CONFIRMATIONS_FOR_RECIPE
    }

    /// Get confirmation count.
    pub fn confirmation_count(&self) -> usize {
        self.confirmations.len()
    }

    /// Get failure count.
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Get success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.confirmations.len() + self.failures.len();
        if total == 0 {
            0.0
        } else {
            self.confirmations.len() as f64 / total as f64
        }
    }
}

/// A confirmation of recipe success.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmation {
    /// Ticket where this was confirmed.
    pub ticket_id: String,
    /// When confirmed.
    pub timestamp: u64,
    /// Number of citations supporting.
    pub citation_count: usize,
}

/// A failure record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    /// Ticket where this failed.
    pub ticket_id: String,
    /// When failed.
    pub timestamp: u64,
    /// Reason for failure.
    pub reason: String,
}

/// Manages recipe candidates and promotion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipePromoter {
    /// Candidates awaiting promotion.
    candidates: HashMap<String, RecipeCandidate>,
    /// Promoted recipes.
    promoted: HashMap<String, RecipeTemplate>,
}

impl RecipePromoter {
    /// Create a new promoter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a candidate recipe.
    pub fn add_candidate(&mut self, template: RecipeTemplate) {
        if !self.candidates.contains_key(&template.id) && !self.promoted.contains_key(&template.id)
        {
            self.candidates
                .insert(template.id.clone(), RecipeCandidate::new(template));
        }
    }

    /// Record execution result.
    pub fn record_execution(
        &mut self,
        recipe_id: &str,
        ticket_id: &str,
        success: bool,
        citations: Option<&CitationStore>,
        failure_reason: Option<&str>,
    ) {
        if let Some(candidate) = self.candidates.get_mut(recipe_id) {
            if success {
                if let Some(cites) = citations {
                    candidate.record_success(ticket_id, cites);
                }
            } else if let Some(reason) = failure_reason {
                candidate.record_failure(ticket_id, reason);
            }

            // Check for promotion
            if candidate.ready_for_promotion() {
                self.promote(recipe_id);
            }
        }
    }

    /// Promote a candidate to full recipe.
    fn promote(&mut self, recipe_id: &str) {
        if let Some(candidate) = self.candidates.remove(recipe_id) {
            self.promoted
                .insert(recipe_id.to_string(), candidate.template);
        }
    }

    /// Get a promoted recipe.
    pub fn get_promoted(&self, recipe_id: &str) -> Option<&RecipeTemplate> {
        self.promoted.get(recipe_id)
    }

    /// Get a candidate.
    pub fn get_candidate(&self, recipe_id: &str) -> Option<&RecipeCandidate> {
        self.candidates.get(recipe_id)
    }

    /// Find matching recipes by tags.
    pub fn find_by_tags(&self, tags: &[&str]) -> Vec<&RecipeTemplate> {
        let mut results: Vec<&RecipeTemplate> = self
            .promoted
            .values()
            .filter(|r| tags.iter().any(|t| r.tags.contains(&t.to_string())))
            .collect();

        // Also include candidates with high success rates
        for candidate in self.candidates.values() {
            if candidate.success_rate() > 0.8
                && tags
                    .iter()
                    .any(|t| candidate.template.tags.contains(&t.to_string()))
            {
                results.push(&candidate.template);
            }
        }

        results
    }

    /// List all promoted recipes.
    pub fn list_promoted(&self) -> Vec<&RecipeTemplate> {
        self.promoted.values().collect()
    }

    /// List all candidates.
    pub fn list_candidates(&self) -> Vec<&RecipeCandidate> {
        self.candidates.values().collect()
    }

    /// Get promotion status.
    pub fn status(&self) -> PromoterStatus {
        PromoterStatus {
            promoted_count: self.promoted.len(),
            candidate_count: self.candidates.len(),
            pending_confirmations: self
                .candidates
                .values()
                .map(|c| super::MIN_CONFIRMATIONS_FOR_RECIPE - c.confirmation_count())
                .sum(),
        }
    }
}

/// Status of the recipe promoter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoterStatus {
    /// Number of promoted recipes.
    pub promoted_count: usize,
    /// Number of candidates.
    pub candidate_count: usize,
    /// Total confirmations needed across all candidates.
    pub pending_confirmations: usize,
}

/// Substitute parameters in a string.
fn substitute_params(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
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
    fn test_recipe_template_creation() {
        let recipe = RecipeTemplate::new("restart-service", "Restart Systemd Service")
            .with_problem("service {service} is not responding")
            .with_probe("sys.services.status")
            .with_precondition("sys.services.status", "inactive")
            .with_step(RecipeStep::new(
                1,
                "Check service status: systemctl status {service}",
            ))
            .with_step(
                RecipeStep::new(2, "Restart service: sudo systemctl restart {service}")
                    .with_command("sudo systemctl restart {service}")
                    .with_confirmation(),
            )
            .with_outcome("Service {service} is running")
            .with_tag("systemd");

        assert_eq!(recipe.id, "restart-service");
        assert_eq!(recipe.steps.len(), 2);
        assert!(recipe.tags.contains(&"systemd".to_string()));
    }

    #[test]
    fn test_recipe_instantiation() {
        let recipe = RecipeTemplate::new("test", "Test")
            .with_step(RecipeStep::new(1, "Do something with {service}"));

        let mut params = HashMap::new();
        params.insert("service".to_string(), "nginx".to_string());

        let instance = recipe.instantiate(&params);
        assert_eq!(instance.steps[0], "Do something with nginx");
    }

    #[test]
    fn test_recipe_candidate_promotion() {
        let template = RecipeTemplate::new("test", "Test");
        let mut candidate = RecipeCandidate::new(template);

        assert!(!candidate.ready_for_promotion());

        let store = CitationStore::new();
        candidate.record_success("ticket-1", &store);
        candidate.record_success("ticket-2", &store);
        assert!(!candidate.ready_for_promotion());

        candidate.record_success("ticket-3", &store);
        assert!(candidate.ready_for_promotion());
    }

    #[test]
    fn test_recipe_promoter() {
        let mut promoter = RecipePromoter::new();

        let template = RecipeTemplate::new("test", "Test").with_tag("systemd");
        promoter.add_candidate(template);

        let store = CitationStore::new();

        // Record 3 successes
        for i in 1..=3 {
            promoter.record_execution("test", &format!("ticket-{}", i), true, Some(&store), None);
        }

        // Should be promoted now
        assert!(promoter.get_promoted("test").is_some());
        assert!(promoter.get_candidate("test").is_none());
    }

    #[test]
    fn test_success_rate() {
        let template = RecipeTemplate::new("test", "Test");
        let mut candidate = RecipeCandidate::new(template);

        let store = CitationStore::new();
        candidate.record_success("t1", &store);
        candidate.record_success("t2", &store);
        candidate.record_failure("t3", "failed");
        candidate.record_failure("t4", "failed");

        assert!((candidate.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_recipe_instance_steps() {
        let recipe = RecipeTemplate::new("test", "Test")
            .with_step(RecipeStep::new(1, "Step 1"))
            .with_step(RecipeStep::new(2, "Step 2"));

        let instance = recipe.instantiate(&HashMap::new());

        assert_eq!(instance.next_step(), Some("Step 1"));
    }
}
