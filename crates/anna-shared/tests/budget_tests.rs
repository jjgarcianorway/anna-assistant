//! Tests for budget.rs

use anna_shared::budget::{
    check_stage_budget, check_total_budget, BudgetCheck, ProbeBudget, Stage, StageBudget,
    StageTiming,
};

#[test]
fn test_stage_budget_defaults() {
    let budget = StageBudget::default();
    assert_eq!(budget.translator_ms, 5_000);
    assert_eq!(budget.probes_ms, 12_000);
    assert_eq!(budget.specialist_ms, 15_000);
    assert_eq!(budget.supervisor_ms, 8_000);
    assert_eq!(budget.total_ms, 25_000);
    assert_eq!(budget.margin_ms, 1_000);
    assert_eq!(budget.effective_total(), 24_000);
}

#[test]
fn test_check_stage_budget_ok() {
    let budget = StageBudget::default();
    let result = check_stage_budget(Stage::Translator, 4_000, &budget);
    assert_eq!(result, BudgetCheck::Ok);
}

#[test]
fn test_check_stage_budget_exceeded() {
    let budget = StageBudget::default();
    let result = check_stage_budget(Stage::Translator, 6_000, &budget);
    assert!(matches!(
        result,
        BudgetCheck::StageExceeded {
            stage: Stage::Translator,
            budget_ms: 5_000,
            elapsed_ms: 6_000,
        }
    ));
}

#[test]
fn test_check_total_budget_ok() {
    let budget = StageBudget::default();
    let result = check_total_budget(20_000, &budget);
    assert_eq!(result, BudgetCheck::Ok);
}

#[test]
fn test_check_total_budget_exceeded() {
    let budget = StageBudget::default();
    // Effective total is 24_000 (25_000 - 1_000 margin)
    let result = check_total_budget(25_000, &budget);
    assert!(matches!(
        result,
        BudgetCheck::TotalExceeded {
            budget_ms: 24_000,
            elapsed_ms: 25_000,
        }
    ));
}

#[test]
fn test_stage_timing() {
    let budget = StageBudget::default();
    let timing = StageTiming::new(Stage::Probes, 15_000, &budget);
    assert!(timing.exceeded);
    assert_eq!(timing.budget_ms, 12_000);
}

#[test]
fn test_budget_check_is_exceeded() {
    assert!(!BudgetCheck::Ok.is_exceeded());
    assert!(BudgetCheck::StageExceeded {
        stage: Stage::Translator,
        budget_ms: 5_000,
        elapsed_ms: 6_000,
    }
    .is_exceeded());
    assert!(BudgetCheck::TotalExceeded {
        budget_ms: 24_000,
        elapsed_ms: 25_000,
    }
    .is_exceeded());
}

// v0.0.36: ProbeBudget tests
#[test]
fn test_probe_budget_defaults() {
    let budget = ProbeBudget::default();
    assert_eq!(budget.max_probes, 4);
    assert_eq!(budget.max_output_bytes, 64_000);
    assert_eq!(budget.per_probe_cap_bytes, 16_000);
}

#[test]
fn test_probe_budget_fast_path() {
    let budget = ProbeBudget::fast_path();
    assert_eq!(budget.max_probes, 4);
    assert_eq!(budget.max_output_bytes, 32_000);
    assert_eq!(budget.per_probe_cap_bytes, 8_000);
}

#[test]
fn test_probe_budget_would_exceed() {
    let budget = ProbeBudget::default();
    assert!(!budget.would_exceed(0, 10_000));
    assert!(!budget.would_exceed(50_000, 10_000)); // 60k < 64k
    assert!(budget.would_exceed(60_000, 10_000)); // 70k > 64k
}

#[test]
fn test_probe_budget_cap_output() {
    let budget = ProbeBudget {
        max_probes: 4,
        max_output_bytes: 100,
        per_probe_cap_bytes: 10,
    };
    // Under limit - unchanged
    assert_eq!(budget.cap_output("hello"), "hello");
    // Over limit - truncated
    let long = "0123456789abcdef";
    let capped = budget.cap_output(long);
    assert!(capped.contains("truncated"));
    assert!(capped.starts_with("0123456789"));
}
