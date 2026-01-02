//! Deterministic formatting tests for fastpath module (v0.0.39)

use anna_shared::fastpath::{try_fast_path, FastPathInput, FastPathPolicy};
use anna_shared::snapshot::SystemSnapshot;

fn fresh_snapshot() -> SystemSnapshot {
    let mut s = SystemSnapshot::now();
    s.add_disk("/", 45);
    s.add_disk("/home", 60);
    s.set_memory(16_000_000_000, 8_000_000_000); // 50% usage
    s
}

#[test]
fn test_fast_path_answer_deterministic() {
    let policy = FastPathPolicy::default();
    let snapshot = fresh_snapshot();

    // Run twice, should get identical results
    let input1 = FastPathInput {
        request: "disk usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };
    let input2 = FastPathInput {
        request: "disk usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result1 = try_fast_path(&input1);
    let result2 = try_fast_path(&input2);

    assert_eq!(result1.answer_text, result2.answer_text);
    assert_eq!(result1.reliability_hint, result2.reliability_hint);
}
