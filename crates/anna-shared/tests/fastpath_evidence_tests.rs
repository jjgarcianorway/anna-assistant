//! Evidence tracking tests for fastpath module (v0.0.39)

use anna_shared::fastpath::{try_fast_path, FastPathInput, FastPathPolicy};
use anna_shared::snapshot::SystemSnapshot;
use anna_shared::trace::EvidenceKind;

fn high_usage_snapshot() -> SystemSnapshot {
    let mut s = SystemSnapshot::now();
    s.add_disk("/", 92); // Warning
    s.add_disk("/home", 96); // Critical
    s.set_memory(16_000_000_000, 14_000_000_000); // ~88% - high
    s.add_failed_service("nginx.service");
    s
}

#[test]
fn test_evidence_kinds_tracked() {
    let policy = FastPathPolicy::default();
    let snapshot = high_usage_snapshot();
    let input = FastPathInput {
        request: "any errors",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);

    // Should include all relevant evidence kinds
    assert!(result.evidence_used.contains(&EvidenceKind::Memory));
    assert!(result.evidence_used.contains(&EvidenceKind::Disk));
    assert!(result.evidence_used.contains(&EvidenceKind::FailedUnits));
}
