// v0.0.580: Settings API Types (Phase 156)
// Type definitions for Settings API

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

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
