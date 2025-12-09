//! Tests for transcript rendering (v0.0.179).

#[cfg(test)]
mod tests {
    use super::super::helpers::{reliability_color, truncate, format_outcome};
    use anna_shared::transcript::StageOutcome;
    use anna_shared::ui::colors;

    #[test]
    fn test_reliability_color() {
        assert_eq!(reliability_color(100), colors::OK);
        assert_eq!(reliability_color(80), colors::OK);
        assert_eq!(reliability_color(79), colors::WARN);
        assert_eq!(reliability_color(49), colors::ERR);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is a~");
    }

    #[test]
    fn test_format_outcome_all_variants() {
        let _ok = format_outcome(&StageOutcome::Ok);
        let _timeout = format_outcome(&StageOutcome::Timeout);
        let _det = format_outcome(&StageOutcome::Deterministic);
        let _budget = format_outcome(&StageOutcome::BudgetExceeded {
            stage: "probes".to_string(),
            budget_ms: 12000,
            elapsed_ms: 15000,
        });
    }
}
