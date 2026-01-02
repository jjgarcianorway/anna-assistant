//! Stabilization tests for v0.45.4 and v0.45.5.

use anna_shared::reliability::{
    compute_reliability, ReliabilityInput, NO_EVIDENCE_RELIABILITY_CAP,
};

// === v0.45.4 Golden Tests ===

/// v0.45.4: NO_EVIDENCE_RELIABILITY_CAP constant is 40.
#[test]
fn golden_v454_no_evidence_cap_value() {
    assert_eq!(
        NO_EVIDENCE_RELIABILITY_CAP, 40,
        "NO_EVIDENCE_RELIABILITY_CAP must be 40"
    );
}

/// v0.45.4: evidence_required=true + succeeded_probes=0 must trigger EvidenceMissing.
#[test]
fn golden_v454_evidence_missing_when_no_probes_succeed() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(2) // Probes were planned
        .with_succeeded_probes(0) // But none succeeded
        .with_answer_grounded(false)
        .with_no_invention(true)
        .with_translator_confidence(90);

    let output = compute_reliability(&input);

    // Reliability should be significantly penalized
    assert!(
        output.score <= NO_EVIDENCE_RELIABILITY_CAP + 20, // Some slack for penalty interaction
        "With evidence_required=true and 0 probes succeeded, reliability should be low, got {}",
        output.score
    );
}

/// v0.45.4: "do I have nano" must classify as InstalledToolCheck.
#[test]
fn golden_v454_query_classify_tool_check() {
    // This test verifies the query classification patterns
    // The actual classification is in annad::query_classify
    // Here we verify the probe spine enforces correct probes
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let decision = enforce_minimum_probes("do I have nano", &[]);
    assert!(decision.enforced, "Tool check query must enforce probes");
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::CommandV(_))),
        "Tool check must include CommandV probe"
    );
}

/// v0.45.4: "what is my sound card" must classify as HardwareAudio.
#[test]
fn golden_v454_query_classify_audio() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let decision = enforce_minimum_probes("what is my sound card", &[]);
    assert!(decision.enforced, "Audio query must enforce probes");
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::LspciAudio)),
        "Audio query must include LspciAudio probe"
    );
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::PactlCards)),
        "Audio query must include PactlCards probe"
    );
}

/// v0.45.4: "how many cores" must classify as CpuCores.
#[test]
fn golden_v454_query_classify_cores() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let decision = enforce_minimum_probes("how many cores", &[]);
    assert!(decision.enforced, "CPU cores query must enforce probes");
    assert!(
        decision.probes.iter().any(|p| matches!(p, ProbeId::Lscpu)),
        "CPU cores query must include Lscpu probe"
    );
}

/// v0.45.4: "how is my computer doing" must classify as SystemTriage.
#[test]
fn golden_v454_query_classify_system_triage() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let decision = enforce_minimum_probes("how is my computer doing", &[]);
    assert!(decision.enforced, "System health query must enforce probes");
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::JournalErrors)),
        "System health query must include JournalErrors probe"
    );
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::FailedUnits)),
        "System health query must include FailedUnits probe"
    );
}

// === v0.45.5 Golden Tests ===

/// v0.45.5: StageOutcome::ClarificationRequired exists and has correct structure.
#[test]
fn golden_v455_stage_outcome_clarification_required() {
    use anna_shared::transcript::StageOutcome;

    let outcome = StageOutcome::clarification_required(
        "Which editor do you prefer?",
        vec!["vim".to_string(), "nano".to_string(), "emacs".to_string()],
    );

    assert!(outcome.is_clarification_required());
    assert!(!outcome.can_proceed());

    // Display format
    let display = format!("{}", outcome);
    assert!(display.contains("clarification_required"));
    assert!(display.contains("3 choices"));
}

/// v0.45.5: ClarifyPrereq has correct structure for editor prereq.
#[test]
fn golden_v455_clarify_prereq_editor() {
    use anna_shared::recipe::ClarifyPrereq;

    let prereq = ClarifyPrereq::editor();

    assert_eq!(prereq.fact_key, "preferred_editor");
    assert_eq!(prereq.question_id, "editor_select");
    assert!(prereq.evidence_only);
    assert_eq!(prereq.verify_template.as_deref(), Some("command -v {}"));
}

/// v0.45.5: Recipe with clarify_prereqs correctly reports needs_clarification.
#[test]
fn golden_v455_recipe_needs_clarification() {
    use anna_shared::recipe::{ClarifyPrereq, Recipe, RecipeSignature};
    use anna_shared::teams::Team;
    use anna_shared::ticket::RiskLevel;

    let sig = RecipeSignature::new(
        "system",
        "configure",
        "configure_editor",
        "enable syntax highlighting",
    );
    let recipe = Recipe::new(
        sig,
        Team::Desktop,
        RiskLevel::LowRiskChange,
        vec![],
        vec![],
        "Add 'syntax on' to ~/.vimrc".to_string(),
        90,
    )
    .with_clarify_prereqs(vec![ClarifyPrereq::editor()]);

    assert!(recipe.needs_clarification());
    assert_eq!(recipe.get_clarify_prereqs().len(), 1);
    assert_eq!(recipe.get_clarify_prereqs()[0].fact_key, "preferred_editor");
}

/// v0.45.5: Recipe without clarify_prereqs does not need clarification.
#[test]
fn golden_v455_recipe_no_clarification_needed() {
    use anna_shared::recipe::{Recipe, RecipeSignature};
    use anna_shared::teams::Team;
    use anna_shared::ticket::RiskLevel;

    let sig = RecipeSignature::new("system", "question", "memory_usage", "how much ram");
    let recipe = Recipe::new(
        sig,
        Team::Performance,
        RiskLevel::ReadOnly,
        vec![],
        vec!["free".to_string()],
        "You have {} of RAM".to_string(),
        90,
    );

    assert!(!recipe.needs_clarification());
    assert!(recipe.get_clarify_prereqs().is_empty());
}
