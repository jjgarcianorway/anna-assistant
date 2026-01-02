// v0.0.580: Settings API Tests (Phase 156)
// Tests for Settings API

#![cfg(test)]

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::handler::SettingsApi;
use super::types::{ApiOperation, ApiRequest, ApiResponse, ApiStatus, SettingValue};
use super::utils::{format_api_response, is_api_query, settings_api_fun_fact};

#[test]
fn test_api_operation_display() {
    assert_eq!(format!("{}", ApiOperation::Get), "Get");
    assert_eq!(format!("{}", ApiOperation::Set), "Set");
}

#[test]
fn test_api_status_display() {
    assert_eq!(format!("{}", ApiStatus::Success), "Success");
    assert_eq!(format!("{}", ApiStatus::Error), "Error");
}

#[test]
fn test_api_request_get() {
    let req = ApiRequest::get(SettingsCategory::Personality, "formality");
    assert_eq!(req.operation, ApiOperation::Get);
    assert_eq!(req.category, Some(SettingsCategory::Personality));
}

#[test]
fn test_api_request_set() {
    let req = ApiRequest::set(SettingsCategory::Risk, "tolerance", "High");
    assert_eq!(req.operation, ApiOperation::Set);
    assert_eq!(req.value, Some("High".to_string()));
}

#[test]
fn test_api_request_list() {
    let req = ApiRequest::list(None);
    assert_eq!(req.operation, ApiOperation::List);
}

#[test]
fn test_api_request_with_id() {
    let req = ApiRequest::get(SettingsCategory::Personality, "mode")
        .with_id("req-123");
    assert_eq!(req.request_id, Some("req-123".to_string()));
}

#[test]
fn test_api_response_success() {
    let resp = ApiResponse::success(ApiOperation::Get, Some("test".to_string()));
    assert!(resp.is_success());
    assert_eq!(resp.data, Some("test".to_string()));
}

#[test]
fn test_api_response_error() {
    let resp = ApiResponse::error(ApiOperation::Set, "Failed");
    assert!(!resp.is_success());
    assert_eq!(resp.error, Some("Failed".to_string()));
}

#[test]
fn test_setting_value_new() {
    let sv = SettingValue::new(SettingsCategory::Risk, "tolerance", "High", "enum");
    assert_eq!(sv.key, "tolerance");
    assert_eq!(sv.value, "High");
}

#[test]
fn test_settings_api_new() {
    let api = SettingsApi::new();
    assert!(api.history().is_empty());
}

#[test]
fn test_settings_api_handle_list() {
    let mut api = SettingsApi::new();
    let mut settings = UnifiedSettings::default();
    let req = ApiRequest::list(None);
    let resp = api.handle(&req, &mut settings);
    assert!(resp.is_success());
}

#[test]
fn test_format_api_response() {
    let resp = ApiResponse::success(ApiOperation::Get, Some("test".to_string()));
    let output = format_api_response(&resp);
    assert!(output.contains("Success"));
}

#[test]
fn test_is_api_query() {
    assert!(is_api_query("settings api"));
    assert!(is_api_query("api request"));
    assert!(!is_api_query("hello world"));
}

#[test]
fn test_fun_fact() {
    let fact = settings_api_fun_fact();
    assert!(fact.contains("API"));
}
