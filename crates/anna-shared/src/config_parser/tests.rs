//! Tests for config parser.

use super::parser::{is_config_request, parse_config_request};
use super::types::ConfigChange;
use super::utils::extract_email;

#[test]
fn test_parse_learning_mode() {
    assert_eq!(
        parse_config_request("enable learning mode"),
        Some(ConfigChange::LearningMode(true))
    );
    assert_eq!(
        parse_config_request("disable learning mode"),
        Some(ConfigChange::LearningMode(false))
    );
}

#[test]
fn test_parse_formality() {
    assert_eq!(
        parse_config_request("make anna more casual"),
        Some(ConfigChange::Formality(0))
    );
    assert_eq!(
        parse_config_request("be more formal"),
        Some(ConfigChange::Formality(2))
    );
}

#[test]
fn test_parse_verbosity() {
    assert_eq!(
        parse_config_request("be more verbose"),
        Some(ConfigChange::Verbosity(2))
    );
    assert_eq!(
        parse_config_request("give me brief answers"),
        Some(ConfigChange::Verbosity(0))
    );
}

#[test]
fn test_parse_humor() {
    assert_eq!(
        parse_config_request("no jokes please"),
        Some(ConfigChange::Humor(0))
    );
    assert_eq!(
        parse_config_request("be more playful"),
        Some(ConfigChange::Humor(2))
    );
}

#[test]
fn test_parse_internal_comms() {
    assert_eq!(
        parse_config_request("show internal comms"),
        Some(ConfigChange::ShowInternalComms(true))
    );
    assert_eq!(
        parse_config_request("hide internal communications"),
        Some(ConfigChange::ShowInternalComms(false))
    );
}

#[test]
fn test_is_config_request() {
    assert!(is_config_request("Anna, enable learning mode"));
    assert!(is_config_request("make Anna more casual"));
    assert!(is_config_request("Anna disable auto confirm"));
    assert!(is_config_request("change my setting to verbose"));
    assert!(!is_config_request("how much disk space do I have"));
}

#[test]
fn test_not_config_request() {
    assert_eq!(parse_config_request("how much memory"), None);
    assert_eq!(parse_config_request("disk usage"), None);
}

#[test]
fn test_extract_email() {
    assert_eq!(
        extract_email("my email is user@example.com"),
        Some("user@example.com".to_string())
    );
    assert_eq!(
        extract_email("notify me at test@domain.org please"),
        Some("test@domain.org".to_string())
    );
    assert_eq!(extract_email("no email here"), None);
}

#[test]
fn test_parse_email() {
    assert_eq!(
        parse_config_request("my email is user@example.com"),
        Some(ConfigChange::Email("user@example.com".to_string()))
    );
    assert_eq!(
        parse_config_request("notify me at test@domain.org"),
        Some(ConfigChange::Email("test@domain.org".to_string()))
    );
    assert_eq!(
        parse_config_request("disable email notifications"),
        Some(ConfigChange::ClearEmail)
    );
}

#[test]
fn test_is_email_config_request() {
    assert!(is_config_request("my email is user@example.com"));
    assert!(is_config_request("notify me at test@domain.org"));
    assert!(is_config_request("disable email notifications"));
}
