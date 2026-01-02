//! Recipe management tests.
//!
//! Tests for recipe promotion, instantiation, and failure tracking.

#[cfg(test)]
mod tests {
    use crate::evidence_first::{
        CitationStore,
        RecipePromoter, RecipeStep, RecipeTemplate,
    };
    use std::collections::HashMap;

    /// Test 3: Recipe promotion after N confirmations.
    #[test]
    fn test_recipe_promotion() {
        let mut promoter = RecipePromoter::new();

        // Create a recipe template
        let template = RecipeTemplate::new("restart-service", "Restart Failed Service")
            .with_problem("Service {service} has failed")
            .with_probe("sys.services.failed")
            .with_step(RecipeStep::new(
                1,
                "Check status: systemctl status {service}",
            ))
            .with_step(
                RecipeStep::new(2, "Restart: sudo systemctl restart {service}")
                    .with_command("sudo systemctl restart {service}")
                    .with_confirmation(),
            )
            .with_outcome("Service {service} is running")
            .with_tag("systemd");

        // Add as candidate
        promoter.add_candidate(template);

        // Verify it's a candidate, not promoted
        assert!(
            promoter.get_candidate("restart-service").is_some(),
            "Should be a candidate"
        );
        assert!(
            promoter.get_promoted("restart-service").is_none(),
            "Should not be promoted yet"
        );

        // Record successful executions
        let store = CitationStore::new();

        // First confirmation
        promoter.record_execution("restart-service", "ticket-1", true, Some(&store), None);
        let candidate = promoter.get_candidate("restart-service").unwrap();
        assert_eq!(candidate.confirmation_count(), 1);
        assert!(!candidate.ready_for_promotion());

        // Second confirmation
        promoter.record_execution("restart-service", "ticket-2", true, Some(&store), None);
        let candidate = promoter.get_candidate("restart-service").unwrap();
        assert_eq!(candidate.confirmation_count(), 2);
        assert!(!candidate.ready_for_promotion());

        // Third confirmation - should trigger promotion
        promoter.record_execution("restart-service", "ticket-3", true, Some(&store), None);

        // Should now be promoted
        assert!(
            promoter.get_promoted("restart-service").is_some(),
            "Should be promoted after 3 confirmations"
        );
        assert!(
            promoter.get_candidate("restart-service").is_none(),
            "Should no longer be a candidate"
        );
    }

    /// Test 8: Recipe instantiation with parameters.
    #[test]
    fn test_recipe_instantiation() {
        let template = RecipeTemplate::new("test", "Test Recipe")
            .with_step(RecipeStep::new(1, "Check {service} status"))
            .with_step(RecipeStep::new(2, "Restart {service}"));

        let mut params = HashMap::new();
        params.insert("service".to_string(), "nginx".to_string());

        let instance = template.instantiate(&params);

        assert_eq!(instance.steps[0], "Check nginx status");
        assert_eq!(instance.steps[1], "Restart nginx");
        assert_eq!(instance.next_step(), Some("Check nginx status"));
    }

    /// Test 12: Recipe failure tracking.
    #[test]
    fn test_recipe_failure_tracking() {
        let mut promoter = RecipePromoter::new();

        let template = RecipeTemplate::new("test", "Test");
        promoter.add_candidate(template);

        let store = CitationStore::new();

        // Record mix of successes and failures
        promoter.record_execution("test", "t1", true, Some(&store), None);
        promoter.record_execution("test", "t2", false, None, Some("Service not found"));
        promoter.record_execution("test", "t3", true, Some(&store), None);

        let candidate = promoter.get_candidate("test").unwrap();
        assert_eq!(candidate.confirmation_count(), 2);
        assert_eq!(candidate.failure_count(), 1);

        // Success rate should be 2/3
        let rate = candidate.success_rate();
        assert!((rate - 0.666).abs() < 0.01, "Success rate should be ~66%");
    }
}
