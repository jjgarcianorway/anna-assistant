// v0.0.580: Settings API Utils (Phase 156)
// Utility functions for Settings API

use super::types::ApiResponse;

/// Format API response for display
pub fn format_api_response(response: &ApiResponse) -> String {
    let mut output = String::new();

    output.push_str(&format!("Status: {}\n", response.status));
    output.push_str(&format!("Operation: {}\n", response.operation));

    if let Some(ref data) = response.data {
        output.push_str(&format!("Data: {}\n", data));
    }

    if let Some(ref error) = response.error {
        output.push_str(&format!("Error: {}\n", error));
    }

    output
}

/// Check if query is about API
pub fn is_api_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings api")
        || lower.contains("api request")
        || lower.contains("api call")
}

/// Fun fact about API
pub fn settings_api_fun_fact() -> &'static str {
    "Anna's Settings API provides a unified interface for all settings operations!"
}
