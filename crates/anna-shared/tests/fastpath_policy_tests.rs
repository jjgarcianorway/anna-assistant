//! Policy configuration tests for fastpath module (v0.0.39)

use anna_shared::fastpath::{try_fast_path, FastPathAnswer, FastPathClass, FastPathInput, FastPathPolicy};
use anna_shared::snapshot::SystemSnapshot;

#[test]
fn test_custom_snapshot_max_age() {
    let policy = FastPathPolicy {
        snapshot_max_age_secs: 60, // 1 minute
        ..Default::default()
    };

    let mut snapshot = SystemSnapshot::now();
    snapshot.add_disk("/", 45);
    snapshot.set_memory(16_000_000_000, 8_000_000_000);
    // Fresh snapshot should work

    let input = FastPathInput {
        request: "disk usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
}

#[test]
fn test_fast_path_answer_not_handled_helper() {
    let answer = FastPathAnswer::not_handled("test reason");
    assert!(!answer.handled);
    assert!(answer.answer_text.is_empty());
    assert!(answer.trace_note.contains("test reason"));
    assert_eq!(answer.class, FastPathClass::NotFastPath);
}
