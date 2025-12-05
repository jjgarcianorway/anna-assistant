//! Tests for clarify_v2 module.

use anna_shared::clarify_v2::{
    ClarifyOption, ClarifyRequest, ClarifyResponse, VerifyFailureTracker,
};

#[test]
fn test_clarify_request_format() {
    let req = ClarifyRequest::new("test", "Which editor?")
        .add_option(ClarifyOption::tool(1, "vim"))
        .add_option(ClarifyOption::tool(2, "nano"));

    let menu = req.format_menu();
    assert!(menu.contains("[1] vim"));
    assert!(menu.contains("[2] nano"));
    assert!(menu.contains("[0] Cancel"));
    assert!(menu.contains("[9] Something else"));
}

#[test]
fn test_parse_numeric_response() {
    let req = ClarifyRequest::new("test", "Which?")
        .add_option(ClarifyOption::tool(1, "vim"));

    let resp = ClarifyResponse::parse("1", &req);
    assert_eq!(resp.selected, Some(1));
    assert!(!resp.cancelled);
}

#[test]
fn test_parse_cancel() {
    let req = ClarifyRequest::new("test", "Which?");

    let resp = ClarifyResponse::parse("0", &req);
    assert!(resp.cancelled);

    let resp = ClarifyResponse::parse("cancel", &req);
    assert!(resp.cancelled);
}

#[test]
fn test_parse_free_text() {
    let req = ClarifyRequest::new("test", "Which?");

    let resp = ClarifyResponse::parse("emacs", &req);
    assert_eq!(resp.free_text, Some("emacs".to_string()));
}

#[test]
fn test_single_option() {
    let req = ClarifyRequest::new("test", "Which?")
        .add_option(ClarifyOption::tool(1, "vim"));

    assert!(req.is_single_option());
    assert_eq!(req.single_option_value(), Some("vim"));
}

#[test]
fn test_verify_failure_tracker() {
    let mut tracker = VerifyFailureTracker::new();

    assert!(!tracker.should_reclarify("editor"));

    tracker.record_failure("editor");
    assert!(!tracker.should_reclarify("editor"));

    tracker.record_failure("editor");
    assert!(tracker.should_reclarify("editor"));

    tracker.clear("editor");
    assert!(!tracker.should_reclarify("editor"));
}

// === v0.45.3 Golden Tests ===

#[test]
fn test_ttl_seconds_default() {
    let req = ClarifyRequest::new("test", "Which?");
    assert_eq!(req.ttl_seconds, 300, "Default TTL should be 300 seconds");
}

#[test]
fn test_ttl_seconds_custom() {
    let req = ClarifyRequest::new("test", "Which?").with_ttl(60);
    assert_eq!(req.ttl_seconds, 60, "Custom TTL should be honored");
}

#[test]
fn test_allow_custom_default() {
    let req = ClarifyRequest::new("test", "Which?");
    assert!(req.allow_custom, "allow_custom should default to true");
}

#[test]
fn test_allow_custom_disabled() {
    let req = ClarifyRequest::new("test", "Which?")
        .add_option(ClarifyOption::tool(1, "vim"))
        .no_custom();

    assert!(!req.allow_custom, "no_custom should disable allow_custom");

    // Menu should not show "Something else" option
    let menu = req.format_menu();
    assert!(!menu.contains("[9] Something else"), "Menu should hide 'Other' when allow_custom=false");
}

#[test]
fn test_free_text_rejected_when_no_custom() {
    let req = ClarifyRequest::new("test", "Which?")
        .add_option(ClarifyOption::tool(1, "vim"))
        .no_custom();

    // Free text should be treated as cancel when allow_custom=false
    let resp = ClarifyResponse::parse("emacs", &req);
    assert!(resp.cancelled, "Free text should cancel when allow_custom=false");
}
