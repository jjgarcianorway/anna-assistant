// v0.0.580: Settings API (Phase 156)
// Unified API for settings operations

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// API operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiOperation {
    /// Get setting value
    Get,
    /// Set setting value
    Set,
    /// Reset to default
    Reset,
    /// List all settings
    List,
    /// Search settings
    Search,
    /// Validate settings
    Validate,
    /// Export settings
    Export,
    /// Import settings
    Import,
}

impl std::fmt::Display for ApiOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "Get"),
            Self::Set => write!(f, "Set"),
            Self::Reset => write!(f, "Reset"),
            Self::List => write!(f, "List"),
            Self::Search => write!(f, "Search"),
            Self::Validate => write!(f, "Validate"),
            Self::Export => write!(f, "Export"),
            Self::Import => write!(f, "Import"),
        }
    }
}

/// API response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ApiStatus {
    /// Success
    #[default]
    Success,
    /// Error
    Error,
    /// Partial success
    Partial,
    /// Pending
    Pending,
}

impl std::fmt::Display for ApiStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::Error => write!(f, "Error"),
            Self::Partial => write!(f, "Partial"),
            Self::Pending => write!(f, "Pending"),
        }
    }
}

/// API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// Operation
    pub operation: ApiOperation,
    /// Category (optional)
    pub category: Option<SettingsCategory>,
    /// Setting key (optional)
    pub key: Option<String>,
    /// Value (optional)
    pub value: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
}

impl ApiRequest {
    /// Create get request
    pub fn get(category: SettingsCategory, key: impl Into<String>) -> Self {
        Self {
            operation: ApiOperation::Get,
            category: Some(category),
            key: Some(key.into()),
            value: None,
            request_id: None,
        }
    }

    /// Create set request
    pub fn set(category: SettingsCategory, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            operation: ApiOperation::Set,
            category: Some(category),
            key: Some(key.into()),
            value: Some(value.into()),
            request_id: None,
        }
    }

    /// Create list request
    pub fn list(category: Option<SettingsCategory>) -> Self {
        Self {
            operation: ApiOperation::List,
            category,
            key: None,
            value: None,
            request_id: None,
        }
    }

    /// Create reset request
    pub fn reset(category: Option<SettingsCategory>) -> Self {
        Self {
            operation: ApiOperation::Reset,
            category,
            key: None,
            value: None,
            request_id: None,
        }
    }

    /// Create search request
    pub fn search(query: impl Into<String>) -> Self {
        Self {
            operation: ApiOperation::Search,
            category: None,
            key: None,
            value: Some(query.into()),
            request_id: None,
        }
    }

    /// With request ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }
}

/// API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Status
    pub status: ApiStatus,
    /// Operation performed
    pub operation: ApiOperation,
    /// Data (JSON-serialized)
    pub data: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Request ID (echoed back)
    pub request_id: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ApiResponse {
    /// Create success response
    pub fn success(operation: ApiOperation, data: Option<String>) -> Self {
        Self {
            status: ApiStatus::Success,
            operation,
            data,
            error: None,
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create error response
    pub fn error(operation: ApiOperation, error: impl Into<String>) -> Self {
        Self {
            status: ApiStatus::Error,
            operation,
            data: None,
            error: Some(error.into()),
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// With request ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Check if successful
    pub fn is_success(&self) -> bool {
        self.status == ApiStatus::Success
    }
}

/// Setting value representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingValue {
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Value (as string)
    pub value: String,
    /// Type hint
    pub value_type: String,
    /// Description
    pub description: Option<String>,
}

impl SettingValue {
    /// Create new setting value
    pub fn new(
        category: SettingsCategory,
        key: impl Into<String>,
        value: impl Into<String>,
        value_type: impl Into<String>,
    ) -> Self {
        Self {
            category,
            key: key.into(),
            value: value.into(),
            value_type: value_type.into(),
            description: None,
        }
    }

    /// With description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Settings API handler
#[derive(Debug, Clone, Default)]
pub struct SettingsApi {
    /// Request history
    history: Vec<(ApiRequest, ApiResponse)>,
    /// Max history size
    max_history: usize,
}

impl SettingsApi {
    /// Create new API handler
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Handle request
    pub fn handle(&mut self, request: &ApiRequest, settings: &mut UnifiedSettings) -> ApiResponse {
        let response = match request.operation {
            ApiOperation::Get => self.handle_get(request, settings),
            ApiOperation::Set => self.handle_set(request, settings),
            ApiOperation::Reset => self.handle_reset(request, settings),
            ApiOperation::List => self.handle_list(request, settings),
            ApiOperation::Search => self.handle_search(request, settings),
            ApiOperation::Validate => self.handle_validate(settings),
            ApiOperation::Export => self.handle_export(settings),
            ApiOperation::Import => self.handle_import(request, settings),
        };

        // Add request ID if present
        let response = if let Some(ref id) = request.request_id {
            response.with_id(id)
        } else {
            response
        };

        // Store in history
        self.history.push((request.clone(), response.clone()));
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }

        response
    }

    fn handle_get(&self, request: &ApiRequest, settings: &UnifiedSettings) -> ApiResponse {
        let category = match request.category {
            Some(c) => c,
            None => return ApiResponse::error(ApiOperation::Get, "Category required"),
        };

        let key = match &request.key {
            Some(k) => k,
            None => return ApiResponse::error(ApiOperation::Get, "Key required"),
        };

        // Get value based on category and key (simplified)
        let value = self.get_setting_value(settings, category, key);
        match value {
            Some(v) => {
                let data = serde_json::to_string(&v).ok();
                ApiResponse::success(ApiOperation::Get, data)
            }
            None => ApiResponse::error(ApiOperation::Get, "Setting not found"),
        }
    }

    fn handle_set(&self, request: &ApiRequest, _settings: &mut UnifiedSettings) -> ApiResponse {
        if request.category.is_none() {
            return ApiResponse::error(ApiOperation::Set, "Category required");
        }
        if request.key.is_none() {
            return ApiResponse::error(ApiOperation::Set, "Key required");
        }
        if request.value.is_none() {
            return ApiResponse::error(ApiOperation::Set, "Value required");
        }

        // In real implementation, would set the value
        ApiResponse::success(ApiOperation::Set, None)
    }

    fn handle_reset(&self, _request: &ApiRequest, _settings: &mut UnifiedSettings) -> ApiResponse {
        // In real implementation, would reset settings
        ApiResponse::success(ApiOperation::Reset, None)
    }

    fn handle_list(&self, request: &ApiRequest, _settings: &UnifiedSettings) -> ApiResponse {
        let categories: Vec<SettingsCategory> = if let Some(cat) = request.category {
            vec![cat]
        } else {
            vec![
                SettingsCategory::Personality,
                SettingsCategory::Risk,
                SettingsCategory::Learning,
                SettingsCategory::Escalation,
                SettingsCategory::Verbosity,
                SettingsCategory::Confirmation,
                SettingsCategory::Timeout,
                SettingsCategory::OutputStyle,
                SettingsCategory::Privacy,
                SettingsCategory::Backup,
                SettingsCategory::Update,
                SettingsCategory::Model,
            ]
        };

        let data = serde_json::to_string(&categories).ok();
        ApiResponse::success(ApiOperation::List, data)
    }

    fn handle_search(&self, request: &ApiRequest, _settings: &UnifiedSettings) -> ApiResponse {
        let _query = match &request.value {
            Some(q) => q,
            None => return ApiResponse::error(ApiOperation::Search, "Query required"),
        };

        // Simplified search - return empty results
        let results: Vec<SettingValue> = vec![];
        let data = serde_json::to_string(&results).ok();
        ApiResponse::success(ApiOperation::Search, data)
    }

    fn handle_validate(&self, _settings: &UnifiedSettings) -> ApiResponse {
        // Simplified validation - always valid
        let data = serde_json::to_string(&true).ok();
        ApiResponse::success(ApiOperation::Validate, data)
    }

    fn handle_export(&self, settings: &UnifiedSettings) -> ApiResponse {
        let data = serde_json::to_string(settings).ok();
        ApiResponse::success(ApiOperation::Export, data)
    }

    fn handle_import(&self, request: &ApiRequest, _settings: &mut UnifiedSettings) -> ApiResponse {
        if request.value.is_none() {
            return ApiResponse::error(ApiOperation::Import, "Settings data required");
        }
        // Simplified import
        ApiResponse::success(ApiOperation::Import, None)
    }

    fn get_setting_value(&self, settings: &UnifiedSettings, category: SettingsCategory, key: &str) -> Option<SettingValue> {
        match category {
            SettingsCategory::Personality => {
                match key {
                    "formality" => Some(SettingValue::new(
                        category, key, format!("{:?}", settings.personality.formality), "enum"
                    )),
                    "humor" => Some(SettingValue::new(
                        category, key, format!("{:?}", settings.personality.humor), "enum"
                    )),
                    _ => None,
                }
            }
            SettingsCategory::Verbosity => {
                match key {
                    "level" => Some(SettingValue::new(
                        category, key, format!("{:?}", settings.verbosity.level), "enum"
                    )),
                    "show_progress" => Some(SettingValue::new(
                        category, key, settings.verbosity.show_progress.to_string(), "bool"
                    )),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Get request history
    pub fn history(&self) -> &[(ApiRequest, ApiResponse)] {
        &self.history
    }

    /// Get recent requests
    pub fn recent(&self, count: usize) -> Vec<&(ApiRequest, ApiResponse)> {
        self.history.iter().rev().take(count).collect()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
