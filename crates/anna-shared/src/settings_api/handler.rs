// v0.0.580: Settings API Handler (Phase 156)
// Handler implementation for Settings API

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::types::{ApiOperation, ApiRequest, ApiResponse, SettingValue};

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
