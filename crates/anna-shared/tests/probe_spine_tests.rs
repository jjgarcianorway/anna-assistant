//! Tests for probe spine enforcement.
//!
//! v0.45.2 Golden Regression Tests for the 6 failures that MUST be fixed.

use anna_shared::probe_spine::{
    enforce_minimum_probes, enforce_spine_probes, EvidenceKind, ProbeId, RouteCapability,
};

#[test]
fn test_enforce_spine_when_empty() {
    let cap = RouteCapability {
        evidence_required: true,
        required_evidence: vec![EvidenceKind::Cpu],
        spine_probes: vec![ProbeId::Lscpu],
        can_answer_deterministically: false,
    };

    let (probes, reason) = enforce_spine_probes(&[], &cap);
    assert!(!probes.is_empty(), "Should enforce probes when empty");
    assert!(reason.is_some(), "Should provide reason");
}

#[test]
fn test_no_enforce_when_not_required() {
    let cap = RouteCapability {
        evidence_required: false,
        ..Default::default()
    };

    let (probes, reason) = enforce_spine_probes(&[], &cap);
    assert!(probes.is_empty(), "Should not enforce when not required");
    assert!(reason.is_none());
}

#[test]
fn test_preserve_translator_probes() {
    let cap = RouteCapability {
        evidence_required: true,
        spine_probes: vec![ProbeId::Lscpu],
        ..Default::default()
    };

    let translator = vec!["custom_probe".to_string()];
    let (probes, reason) = enforce_spine_probes(&translator, &cap);
    assert_eq!(probes, translator, "Should preserve translator probes");
    assert!(reason.is_none());
}

// === v0.45.2 Golden Regression Tests ===
// These test the 6 failures that MUST be fixed.

#[test]
fn test_do_i_have_nano_enforces_probes() {
    // FAILURE 1: "Do I have nano?" ran zero probes, timed out
    let decision = enforce_minimum_probes("Do I have nano?", &[]);
    assert!(decision.enforced, "Must enforce probes for package check");
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::PacmanQ(pkg) if pkg == "nano"))
    );
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::CommandV(cmd) if cmd == "nano"))
    );
    assert!(decision.evidence_kinds.contains(&EvidenceKind::Packages));
}

#[test]
fn test_sound_card_enforces_probes() {
    // FAILURE 2: "What is my sound card?" ran zero probes, timed out
    let decision = enforce_minimum_probes("What is my sound card?", &[]);
    assert!(decision.enforced, "Must enforce probes for audio query");
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::LspciAudio)));
    assert!(decision.evidence_kinds.contains(&EvidenceKind::Audio));
}

#[test]
fn test_cpu_temperature_enforces_sensors() {
    // FAILURE 3: "CPU temperature?" returned CPU model (nonsense)
    let decision = enforce_minimum_probes("CPU temperature?", &[]);
    assert!(decision.enforced, "Must enforce sensors for temperature");
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::Sensors)));
    assert!(decision
        .evidence_kinds
        .contains(&EvidenceKind::CpuTemperature));
}

#[test]
fn test_how_many_cores_enforces_lscpu() {
    // FAILURE 4: "How many cores?" returned CPU model with 0 probes
    let decision = enforce_minimum_probes("How many cores?", &[]);
    assert!(decision.enforced, "Must enforce lscpu for core count");
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::Lscpu)));
    assert!(decision.evidence_kinds.contains(&EvidenceKind::Cpu));
}

#[test]
fn test_system_health_enforces_journal() {
    // FAILURE 5: "How is my computer doing?" scary journal counts
    let decision = enforce_minimum_probes("How is my computer doing?", &[]);
    assert!(decision.enforced, "Must enforce journal for system health");
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::JournalErrors)));
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::FailedUnits)));
    assert!(decision.evidence_kinds.contains(&EvidenceKind::Journal));
}

#[test]
fn test_package_name_extraction() {
    let decision = enforce_minimum_probes("do I have nano?", &[]);
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::PacmanQ(pkg) if pkg == "nano")));

    let decision = enforce_minimum_probes("Do I have vim installed?", &[]);
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::PacmanQ(pkg) if pkg == "vim")));

    let decision = enforce_minimum_probes("have I got firefox", &[]);
    assert!(decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::PacmanQ(pkg) if pkg == "firefox")));
}

// === v0.45.3 Minimal Probe Policy Tests ===

use anna_shared::probe_spine::{reduce_probes, Urgency};

#[test]
fn test_reduce_probes_default_max_3() {
    let probes = vec![
        ProbeId::Free,
        ProbeId::Df,
        ProbeId::Lscpu,
        ProbeId::Sensors,
        ProbeId::IpAddr,
    ];
    let reduced = reduce_probes(probes, "memory_usage", Urgency::Normal);
    assert!(reduced.len() <= 3, "Default should be max 3 probes");
}

#[test]
fn test_reduce_probes_system_health_max_4() {
    let probes = vec![
        ProbeId::JournalErrors,
        ProbeId::FailedUnits,
        ProbeId::SystemdAnalyze,
        ProbeId::Df,
        ProbeId::Free,
    ];
    let reduced = reduce_probes(probes, "system_health_summary", Urgency::Normal);
    assert!(reduced.len() <= 4, "System health should be max 4 probes");
    assert!(reduced.len() >= 3, "Should keep important probes");
}

#[test]
fn test_reduce_probes_no_duplicate_journal() {
    let probes = vec![
        ProbeId::JournalErrors,
        ProbeId::JournalWarnings,
        ProbeId::FailedUnits,
    ];
    let reduced = reduce_probes(probes, "system_triage", Urgency::Normal);
    // Should not have both errors and warnings (unless Detailed)
    let has_errors = reduced.iter().any(|p| matches!(p, ProbeId::JournalErrors));
    let has_warnings = reduced.iter().any(|p| matches!(p, ProbeId::JournalWarnings));
    assert!(!(has_errors && has_warnings), "Should not have both errors and warnings");
}

#[test]
fn test_reduce_probes_warnings_query_gets_warnings() {
    use anna_shared::probe_spine::query_wants_warnings;
    assert!(query_wants_warnings("show me warnings"));
    assert!(!query_wants_warnings("show me errors"));
    assert!(!query_wants_warnings("show me errors and warnings"));
}

#[test]
fn test_reduce_probes_errors_query_gets_errors() {
    use anna_shared::probe_spine::query_wants_errors;
    assert!(query_wants_errors("show me errors"));
    assert!(!query_wants_errors("show me warnings"));
    assert!(!query_wants_errors("show me errors and warnings"));
}
