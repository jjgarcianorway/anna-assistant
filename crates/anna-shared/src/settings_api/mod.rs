// v0.0.580: Settings API Module (Phase 156)
// Unified API for settings operations

mod handler;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use handler::SettingsApi;
pub use types::{ApiOperation, ApiRequest, ApiResponse, ApiStatus, SettingValue};
pub use utils::{format_api_response, is_api_query, settings_api_fun_fact};
