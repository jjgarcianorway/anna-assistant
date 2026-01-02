//! Helper functions for recipes (v0.0.435).

use std::collections::HashMap;

/// Substitute parameters in a string.
pub fn substitute_params(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

/// Get current timestamp.
pub fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::super::citations::CitationStore;
    use super::super::recipes_candidate::RecipeCandidate;
    use super::super::recipes_promoter::RecipePromoter;
    use super::super::recipes_types::{RecipeStep, RecipeTemplate};
    use std::collections::HashMap;

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
