// v0.0.569: Settings Templates - Types (Phase 145)
// Core types for settings templates: enums, metadata, and template structures

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Template scope - what settings the template affects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateScope {
    /// Affects all settings
    Full,
    /// Only specific categories
    Partial,
    /// Single category
    Single(SettingsCategory),
}

impl std::fmt::Display for TemplateScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "Full"),
            Self::Partial => write!(f, "Partial"),
            Self::Single(cat) => write!(f, "{}", cat),
        }
    }
}

/// Template use case
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateUseCase {
    /// Development work
    Development,
    /// Production servers
    Production,
    /// Presentation/demo mode
    Presentation,
    /// Learning/educational
    Learning,
    /// Minimal resource usage
    Minimal,
    /// Maximum features enabled
    Full,
    /// Custom use case
    Custom,
}

impl std::fmt::Display for TemplateUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => write!(f, "Development"),
            Self::Production => write!(f, "Production"),
            Self::Presentation => write!(f, "Presentation"),
            Self::Learning => write!(f, "Learning"),
            Self::Minimal => write!(f, "Minimal"),
            Self::Full => write!(f, "Full"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMeta {
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// Use case
    pub use_case: TemplateUseCase,
    /// Scope
    pub scope: TemplateScope,
    /// Tags for search
    pub tags: Vec<String>,
    /// Author
    pub author: Option<String>,
    /// Version
    pub version: String,
    /// Created timestamp
    pub created: chrono::DateTime<chrono::Utc>,
    /// Last modified
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl TemplateMeta {
    /// Create new template metadata
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            description: description.into(),
            use_case: TemplateUseCase::Custom,
            scope: TemplateScope::Full,
            tags: Vec::new(),
            author: None,
            version: "1.0.0".to_string(),
            created: now,
            modified: now,
        }
    }

    /// Set use case
    pub fn with_use_case(mut self, use_case: TemplateUseCase) -> Self {
        self.use_case = use_case;
        self
    }

    /// Set scope
    pub fn with_scope(mut self, scope: TemplateScope) -> Self {
        self.scope = scope;
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// A settings template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsTemplate {
    /// Unique ID
    pub id: u64,
    /// Metadata
    pub meta: TemplateMeta,
    /// The actual settings (partial or full)
    pub settings: UnifiedSettings,
    /// Categories included (for partial templates)
    pub included_categories: Vec<SettingsCategory>,
    /// Is this a built-in template
    pub builtin: bool,
    /// Usage count
    pub usage_count: u32,
}

impl SettingsTemplate {
    /// Create new template from settings
    pub fn new(id: u64, meta: TemplateMeta, settings: UnifiedSettings) -> Self {
        Self {
            id,
            meta,
            settings,
            included_categories: Vec::new(),
            builtin: false,
            usage_count: 0,
        }
    }

    /// Create partial template with specific categories
    pub fn partial(
        id: u64,
        meta: TemplateMeta,
        settings: UnifiedSettings,
        categories: Vec<SettingsCategory>,
    ) -> Self {
        let mut meta = meta;
        meta.scope = TemplateScope::Partial;
        Self {
            id,
            meta,
            settings,
            included_categories: categories,
            builtin: false,
            usage_count: 0,
        }
    }

    /// Mark as built-in
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }

    /// Increment usage count
    pub fn mark_used(&mut self) {
        self.usage_count += 1;
    }

    /// Check if category is included
    pub fn includes_category(&self, category: SettingsCategory) -> bool {
        match self.meta.scope {
            TemplateScope::Full => true,
            TemplateScope::Single(cat) => cat == category,
            TemplateScope::Partial => self.included_categories.contains(&category),
        }
    }
}
