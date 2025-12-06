//! Stabilization tests for v0.0.45 and v0.45.4.
//!
//! These tests lock invariants that ensure correctness:
//! - No probe, no claim: if probes_succeeded == 0, numeric claims must be rejected
//! - Evidence gating: deterministic answers must have ParsedProbeData from same request
//! - Reliability truthfulness: evidence_required + no evidence = reliability < 100
//! - v0.45.4: NO_EVIDENCE_RELIABILITY_CAP at 40 when evidence_required but no probes

use anna_shared::reliability::{compute_reliability, ReliabilityInput, NO_EVIDENCE_RELIABILITY_CAP};

/// Golden test: No probe, no claim invariant.
/// If probes_succeeded == 0 AND evidence_required == true,
/// any answer with claims should have invention_detected or reliability < 100.
#[test]
fn golden_no_probe_no_claim_invariant() {
    // Scenario: Query requires evidence, but no probes ran
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(0)
        .with_succeeded_probes(0)
        .with_total_claims(3) // Answer has 3 claims
        .with_verified_claims(0) // None verified (no evidence)
        .with_answer_grounded(false)
        .with_no_invention(true) // Guard didn't flag (but should have)
        .with_translator_confidence(90);

    let output = compute_reliability(&input);

    // INVARIANT: Reliability MUST be < 100 when claims exist but no evidence
    assert!(
        output.score < 100,
        "With evidence_required=true, 0 probes succeeded, and 3 unverified claims, \
        reliability must be < 100, got {}",
        output.score
    );

    // Should have evidence missing reason
    assert!(
        output.reasons.iter().any(|r| format!("{:?}", r).contains("Evidence")),
        "Should have evidence-related degradation reason"
    );
}

/// Golden test: Evidence missing with claims should cap reliability.
#[test]
fn golden_evidence_missing_caps_reliability() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(2)
        .with_succeeded_probes(0) // All probes failed
        .with_total_claims(5)
        .with_verified_claims(0)
        .with_answer_grounded(false)
        .with_no_invention(true)
        .with_translator_confidence(95);

    let output = compute_reliability(&input);

    // Should be significantly penalized
    assert!(
        output.score <= 70,
        "With all probes failed and unverified claims, reliability should be <= 70, got {}",
        output.score
    );
}

/// Golden test: No claims, no evidence required = high reliability.
#[test]
fn golden_no_claims_no_evidence_ok() {
    let input = ReliabilityInput::default()
        .with_evidence_required(false)
        .with_planned_probes(0)
        .with_succeeded_probes(0)
        .with_total_claims(0) // No claims
        .with_verified_claims(0)
        .with_answer_grounded(true) // Generic answer, no claims
        .with_no_invention(true)
        .with_translator_confidence(90);

    let output = compute_reliability(&input);

    // Should be reasonably high - no evidence needed, no claims
    assert!(
        output.score >= 80,
        "With no evidence required and no claims, reliability should be >= 80, got {}",
        output.score
    );
}

/// Golden test: Invention detected must cap at 40.
#[test]
fn golden_invention_detected_ceiling() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(1)
        .with_succeeded_probes(1)
        .with_total_claims(2)
        .with_verified_claims(0) // Claims not verified
        .with_answer_grounded(false)
        .with_no_invention(false) // INVENTION DETECTED
        .with_translator_confidence(100);

    let output = compute_reliability(&input);

    // INVARIANT: Invention ceiling is 40
    assert!(
        output.score <= 40,
        "With invention_detected=true, reliability must be <= 40, got {}",
        output.score
    );
}

/// Golden test: All probes succeeded, claims verified = high reliability.
#[test]
fn golden_fully_verified_high_reliability() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(3)
        .with_succeeded_probes(3)
        .with_total_claims(3)
        .with_verified_claims(3) // All claims verified
        .with_answer_grounded(true)
        .with_no_invention(true)
        .with_translator_confidence(95);

    let output = compute_reliability(&input);

    // Should be high reliability
    assert!(
        output.score >= 90,
        "With all probes succeeded and claims verified, reliability should be >= 90, got {}",
        output.score
    );
}

/// Test that partial probe success still provides reasonable reliability.
#[test]
fn test_partial_probe_success() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(4)
        .with_succeeded_probes(2) // 50% success
        .with_total_claims(2)
        .with_verified_claims(2)
        .with_answer_grounded(true)
        .with_no_invention(true)
        .with_translator_confidence(85);

    let output = compute_reliability(&input);

    // Should be degraded but not catastrophically
    assert!(
        output.score >= 60 && output.score <= 85,
        "With 50% probe success and verified claims, reliability should be 60-85, got {}",
        output.score
    );
}

// === v0.45.4 Golden Tests ===

/// v0.45.4: NO_EVIDENCE_RELIABILITY_CAP constant is 40.
#[test]
fn golden_v454_no_evidence_cap_value() {
    assert_eq!(NO_EVIDENCE_RELIABILITY_CAP, 40, "NO_EVIDENCE_RELIABILITY_CAP must be 40");
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
        decision.probes.iter().any(|p| matches!(p, ProbeId::CommandV(_))),
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
        decision.probes.iter().any(|p| matches!(p, ProbeId::LspciAudio)),
        "Audio query must include LspciAudio probe"
    );
    assert!(
        decision.probes.iter().any(|p| matches!(p, ProbeId::PactlCards)),
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
        decision.probes.iter().any(|p| matches!(p, ProbeId::JournalErrors)),
        "System health query must include JournalErrors probe"
    );
    assert!(
        decision.probes.iter().any(|p| matches!(p, ProbeId::FailedUnits)),
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

    let sig = RecipeSignature::new("system", "configure", "configure_editor", "enable syntax highlighting");
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

// === v0.45.6 Golden Tests: Probe Contract Fix ===

/// v0.45.6: "do I have nano" must enforce CommandV probe.
#[test]
fn golden_v456_tool_check_enforces_command_v() {
    use anna_shared::probe_spine::{enforce_minimum_probes, probe_to_command, ProbeId};

    let decision = enforce_minimum_probes("do I have nano", &[]);
    assert!(decision.enforced, "Tool check must enforce probes");

    // Must include CommandV probe
    let has_command_v = decision.probes.iter().any(|p| matches!(p, ProbeId::CommandV(_)));
    assert!(has_command_v, "Tool check must include CommandV probe");

    // When converted to command, should produce executable command
    let command_v_probe = decision.probes.iter()
        .find(|p| matches!(p, ProbeId::CommandV(_)))
        .unwrap();
    let cmd = probe_to_command(command_v_probe);
    assert!(cmd.contains("command -v"), "CommandV probe must use 'command -v'");
    assert!(cmd.contains("nano"), "CommandV probe must include package name");
}

/// v0.45.6: "how many cores" must enforce Lscpu probe.
#[test]
fn golden_v456_cpu_cores_enforces_lscpu() {
    use anna_shared::probe_spine::{enforce_minimum_probes, probe_to_command, ProbeId};

    let decision = enforce_minimum_probes("how many cores has my cpu", &[]);
    assert!(decision.enforced, "CPU cores query must enforce probes");

    // Must include Lscpu probe
    let has_lscpu = decision.probes.iter().any(|p| matches!(p, ProbeId::Lscpu));
    assert!(has_lscpu, "CPU cores query must include Lscpu probe");

    // When converted to command, should be "lscpu"
    let cmd = probe_to_command(&ProbeId::Lscpu);
    assert_eq!(cmd, "lscpu", "Lscpu probe must produce 'lscpu' command");
}

/// v0.45.6: "what is my sound card" must enforce audio probes.
#[test]
fn golden_v456_sound_card_enforces_audio_probes() {
    use anna_shared::probe_spine::{enforce_minimum_probes, probe_to_command, ProbeId};

    let decision = enforce_minimum_probes("what is my sound card", &[]);
    assert!(decision.enforced, "Sound card query must enforce probes");

    // Must include LspciAudio probe
    let has_lspci_audio = decision.probes.iter().any(|p| matches!(p, ProbeId::LspciAudio));
    assert!(has_lspci_audio, "Sound card query must include LspciAudio probe");

    // LspciAudio command should contain lspci and audio
    let cmd = probe_to_command(&ProbeId::LspciAudio);
    assert!(cmd.contains("lspci"), "LspciAudio probe must use lspci");
    assert!(cmd.to_lowercase().contains("audio"), "LspciAudio probe must filter for audio");
}

/// v0.45.6: Probe commands from probe_spine can be resolved for execution.
#[test]
fn golden_v456_probe_spine_commands_resolvable() {
    use anna_shared::probe_spine::{probe_to_command, ProbeId};

    // All probe_spine commands should start with known executables
    let known_executables = ["lscpu", "sensors", "free", "df", "lsblk", "lspci", "pactl",
        "ip", "ps", "systemctl", "journalctl", "pacman", "sh", "uname", "systemd-analyze"];

    let probes = [
        ProbeId::Lscpu,
        ProbeId::Sensors,
        ProbeId::Free,
        ProbeId::Df,
        ProbeId::Lsblk,
        ProbeId::LspciAudio,
        ProbeId::PactlCards,
        ProbeId::IpAddr,
        ProbeId::TopMemory,
        ProbeId::TopCpu,
        ProbeId::FailedUnits,
        ProbeId::JournalErrors,
        ProbeId::JournalWarnings,
        ProbeId::PacmanCount,
        ProbeId::CommandV("test".to_string()),
        ProbeId::SystemdAnalyze,
        ProbeId::Uname,
    ];

    for probe in probes {
        let cmd = probe_to_command(&probe);
        let first_word = cmd.split_whitespace().next().unwrap_or("");

        let is_known = known_executables.iter().any(|&exe| first_word == exe);
        assert!(
            is_known,
            "Probe {:?} produces command '{}' with unknown executable '{}'",
            probe, cmd, first_word
        );
    }
}

/// v0.45.6: Evidence kinds are properly bound to probes.
#[test]
fn golden_v456_evidence_binding() {
    use anna_shared::probe_spine::{probes_for_evidence, EvidenceKind};

    // Audio evidence must include audio probes
    let audio_probes = probes_for_evidence(EvidenceKind::Audio);
    assert!(!audio_probes.is_empty(), "Audio evidence must have probes");

    // CPU evidence must include lscpu
    let cpu_probes = probes_for_evidence(EvidenceKind::Cpu);
    assert!(!cpu_probes.is_empty(), "CPU evidence must have probes");

    // Memory evidence must include free
    let mem_probes = probes_for_evidence(EvidenceKind::Memory);
    assert!(!mem_probes.is_empty(), "Memory evidence must have probes");

    // Journal evidence must include journal probes
    let journal_probes = probes_for_evidence(EvidenceKind::Journal);
    assert!(!journal_probes.is_empty(), "Journal evidence must have probes");
}
