//! Tests for live request handling.

use anna_shared::progress::{ProgressEvent, RequestStage};

use super::helpers::format_event_key;

#[test]
fn test_event_key_uniqueness() {
    let event1 =
        ProgressEvent::internal_comms(RequestStage::Translator, "Anna", "Test message", 100);
    let event2 =
        ProgressEvent::internal_comms(RequestStage::Translator, "Anna", "Test message", 200);

    let key1 = format_event_key(&event1);
    let key2 = format_event_key(&event2);

    assert_ne!(
        key1, key2,
        "Different timestamps should produce different keys"
    );
}
