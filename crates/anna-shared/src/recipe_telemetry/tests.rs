//! Tests for recipe telemetry.

#[cfg(test)]
mod tests {
    use crate::recipe_telemetry::{
        helpers::{record_learning, record_resolution},
        telemetry::RecipeTelemetry,
        types::{LearningEventType, ResolutionSource},
    };

    #[test]
    fn test_telemetry_basic() {
        let mut telemetry = RecipeTelemetry::new();

        record_resolution(
            &mut telemetry,
            "ticket1",
            ResolutionSource::Recipe,
            Some("vim_syntax"),
            Some("configure_editor"),
            Some("desktop"),
            100,
        );

        record_resolution(
            &mut telemetry,
            "ticket2",
            ResolutionSource::Specialist,
            None,
            Some("check_disk"),
            Some("storage"),
            500,
        );

        assert_eq!(telemetry.stats.total_resolutions, 2);
        assert_eq!(telemetry.stats.by_recipe, 1);
        assert_eq!(telemetry.stats.by_specialist, 1);
        assert_eq!(telemetry.self_reliance_rate(), 50.0);
    }

    #[test]
    fn test_learning_events() {
        let mut telemetry = RecipeTelemetry::new();

        record_learning(
            &mut telemetry,
            LearningEventType::RecipeCreated,
            "vim_syntax",
            Some("ticket1"),
            "Created from successful ticket",
        );

        assert_eq!(telemetry.stats.recipes_created, 1);
        assert_eq!(telemetry.learning_events.len(), 1);
    }

    #[test]
    fn test_summary() {
        let mut telemetry = RecipeTelemetry::new();

        for i in 0..10 {
            let source = if i < 6 {
                ResolutionSource::Recipe
            } else {
                ResolutionSource::Specialist
            };
            record_resolution(
                &mut telemetry,
                &format!("t{}", i),
                source,
                None,
                None,
                None,
                100,
            );
        }

        let summary = telemetry.summary(5, 1);
        assert!(summary.contains("6 recipes"));
        assert!(summary.contains("5 active"));
        assert!(summary.contains("1 disabled"));
        assert!(summary.contains("60%")); // 6/10 = 60% self-reliance
    }
}
