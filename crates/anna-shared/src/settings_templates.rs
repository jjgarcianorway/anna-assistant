// v0.0.569: Settings Templates (Phase 145)
// Create and apply reusable settings templates for different scenarios

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Template manager
#[derive(Debug, Clone, Default)]
pub struct TemplateManager {
    /// All templates
    templates: Vec<SettingsTemplate>,
    /// Next ID
    next_id: u64,
}

impl TemplateManager {
    /// Create new template manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with built-in templates
    pub fn with_defaults() -> Self {
        let mut mgr = Self::new();
        mgr.add_builtin_templates();
        mgr
    }

    /// Add built-in templates
    fn add_builtin_templates(&mut self) {
        // Development template
        let dev_meta = TemplateMeta::new("Development", "Optimized for development work")
            .with_use_case(TemplateUseCase::Development)
            .with_tag("dev")
            .with_tag("verbose");
        let mut template = SettingsTemplate::new(self.next_id, dev_meta, UnifiedSettings::default());
        template.builtin = true;
        self.templates.push(template);
        self.next_id += 1;

        // Production template
        let prod_meta = TemplateMeta::new("Production", "Safe settings for production servers")
            .with_use_case(TemplateUseCase::Production)
            .with_tag("server")
            .with_tag("safe");
        let mut template = SettingsTemplate::new(self.next_id, prod_meta, UnifiedSettings::default());
        template.builtin = true;
        self.templates.push(template);
        self.next_id += 1;

        // Presentation template
        let pres_meta = TemplateMeta::new("Presentation", "Clean output for demos")
            .with_use_case(TemplateUseCase::Presentation)
            .with_tag("demo")
            .with_tag("clean");
        let mut template = SettingsTemplate::new(self.next_id, pres_meta, UnifiedSettings::default());
        template.builtin = true;
        self.templates.push(template);
        self.next_id += 1;

        // Learning template
        let learn_meta = TemplateMeta::new("Learning", "Verbose explanations for learning")
            .with_use_case(TemplateUseCase::Learning)
            .with_tag("education")
            .with_tag("verbose");
        let mut template = SettingsTemplate::new(self.next_id, learn_meta, UnifiedSettings::default());
        template.builtin = true;
        self.templates.push(template);
        self.next_id += 1;
    }

    /// Add a template
    pub fn add(&mut self, meta: TemplateMeta, settings: UnifiedSettings) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let template = SettingsTemplate::new(id, meta, settings);
        self.templates.push(template);
        id
    }

    /// Add partial template
    pub fn add_partial(
        &mut self,
        meta: TemplateMeta,
        settings: UnifiedSettings,
        categories: Vec<SettingsCategory>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let template = SettingsTemplate::partial(id, meta, settings, categories);
        self.templates.push(template);
        id
    }

    /// Remove a template
    pub fn remove(&mut self, id: u64) -> Option<SettingsTemplate> {
        if let Some(pos) = self.templates.iter().position(|t| t.id == id && !t.builtin) {
            Some(self.templates.remove(pos))
        } else {
            None
        }
    }

    /// Get template by ID
    pub fn get(&self, id: u64) -> Option<&SettingsTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Get mutable template
    pub fn get_mut(&mut self, id: u64) -> Option<&mut SettingsTemplate> {
        self.templates.iter_mut().find(|t| t.id == id)
    }

    /// Find templates by name
    pub fn find_by_name(&self, name: &str) -> Vec<&SettingsTemplate> {
        let lower = name.to_lowercase();
        self.templates
            .iter()
            .filter(|t| t.meta.name.to_lowercase().contains(&lower))
            .collect()
    }

    /// Find by use case
    pub fn find_by_use_case(&self, use_case: TemplateUseCase) -> Vec<&SettingsTemplate> {
        self.templates
            .iter()
            .filter(|t| t.meta.use_case == use_case)
            .collect()
    }

    /// Find by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&SettingsTemplate> {
        let lower = tag.to_lowercase();
        self.templates
            .iter()
            .filter(|t| t.meta.tags.iter().any(|t| t.to_lowercase() == lower))
            .collect()
    }

    /// List all templates
    pub fn list(&self) -> &[SettingsTemplate] {
        &self.templates
    }

    /// List built-in templates
    pub fn builtin(&self) -> Vec<&SettingsTemplate> {
        self.templates.iter().filter(|t| t.builtin).collect()
    }

    /// List user templates
    pub fn user_templates(&self) -> Vec<&SettingsTemplate> {
        self.templates.iter().filter(|t| !t.builtin).collect()
    }

    /// Get most used templates
    pub fn most_used(&self, limit: usize) -> Vec<&SettingsTemplate> {
        let mut sorted: Vec<_> = self.templates.iter().collect();
        sorted.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        sorted.into_iter().take(limit).collect()
    }

    /// Apply template to current settings
    pub fn apply(&mut self, id: u64, current: &mut UnifiedSettings) -> bool {
        if let Some(template) = self.templates.iter_mut().find(|t| t.id == id) {
            match template.meta.scope {
                TemplateScope::Full => {
                    *current = template.settings.clone();
                }
                TemplateScope::Single(cat) => {
                    apply_category(current, &template.settings, cat);
                }
                TemplateScope::Partial => {
                    for cat in &template.included_categories.clone() {
                        apply_category(current, &template.settings, *cat);
                    }
                }
            }
            template.mark_used();
            true
        } else {
            false
        }
    }

    /// Count templates
    pub fn count(&self) -> usize {
        self.templates.len()
    }
}

/// Apply a single category from source to target
fn apply_category(target: &mut UnifiedSettings, source: &UnifiedSettings, category: SettingsCategory) {
    match category {
        SettingsCategory::Personality => target.personality = source.personality.clone(),
        SettingsCategory::Risk => target.risk = source.risk.clone(),
        SettingsCategory::Learning => target.learning = source.learning.clone(),
        SettingsCategory::Escalation => target.escalation = source.escalation.clone(),
        SettingsCategory::Verbosity => target.verbosity = source.verbosity.clone(),
        SettingsCategory::Confirmation => target.confirmation = source.confirmation.clone(),
        SettingsCategory::Timeout => target.timeout = source.timeout.clone(),
        SettingsCategory::OutputStyle => target.output_style = source.output_style.clone(),
        SettingsCategory::Privacy => target.privacy = source.privacy.clone(),
        SettingsCategory::Backup => target.backup = source.backup.clone(),
        SettingsCategory::Update => target.update = source.update.clone(),
        SettingsCategory::Model => target.model = source.model.clone(),
        SettingsCategory::Unknown => {}
    }
}

/// Format templates for display
pub fn format_templates(manager: &TemplateManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Templates ===\n\n");

    if manager.count() == 0 {
        output.push_str("No templates available.\n");
        return output;
    }

    // Built-in templates
    let builtin = manager.builtin();
    if !builtin.is_empty() {
        output.push_str("Built-in Templates:\n");
        for t in builtin {
            output.push_str(&format!(
                "  • {} - {} [{}]\n",
                t.meta.name, t.meta.description, t.meta.use_case
            ));
        }
        output.push('\n');
    }

    // User templates
    let user = manager.user_templates();
    if !user.is_empty() {
        output.push_str("User Templates:\n");
        for t in user {
            output.push_str(&format!(
                "  • {} - {} [{}] (used {} times)\n",
                t.meta.name, t.meta.description, t.meta.use_case, t.usage_count
            ));
        }
    }

    output
}

/// Check if query is about templates
pub fn is_template_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("template")
        || lower.contains("create template")
        || lower.contains("apply template")
        || lower.contains("settings template")
}

/// Fun fact about templates
pub fn template_fun_fact() -> &'static str {
    "Settings templates let you quickly switch between different configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_scope_display() {
        assert_eq!(format!("{}", TemplateScope::Full), "Full");
        assert_eq!(format!("{}", TemplateScope::Partial), "Partial");
    }

    #[test]
    fn test_template_use_case_display() {
        assert_eq!(format!("{}", TemplateUseCase::Development), "Development");
        assert_eq!(format!("{}", TemplateUseCase::Production), "Production");
    }

    #[test]
    fn test_template_meta_new() {
        let meta = TemplateMeta::new("Test", "Test template");
        assert_eq!(meta.name, "Test");
        assert_eq!(meta.use_case, TemplateUseCase::Custom);
    }

    #[test]
    fn test_template_meta_builder() {
        let meta = TemplateMeta::new("Test", "Test")
            .with_use_case(TemplateUseCase::Development)
            .with_tag("dev");
        assert_eq!(meta.use_case, TemplateUseCase::Development);
        assert!(meta.tags.contains(&"dev".to_string()));
    }

    #[test]
    fn test_settings_template_new() {
        let meta = TemplateMeta::new("Test", "Test");
        let template = SettingsTemplate::new(1, meta, UnifiedSettings::default());
        assert_eq!(template.id, 1);
        assert!(!template.builtin);
    }

    #[test]
    fn test_settings_template_builtin() {
        let meta = TemplateMeta::new("Test", "Test");
        let template = SettingsTemplate::new(1, meta, UnifiedSettings::default()).builtin();
        assert!(template.builtin);
    }

    #[test]
    fn test_settings_template_includes_category() {
        let meta = TemplateMeta::new("Test", "Test").with_scope(TemplateScope::Full);
        let template = SettingsTemplate::new(1, meta, UnifiedSettings::default());
        assert!(template.includes_category(SettingsCategory::Personality));
    }

    #[test]
    fn test_template_manager_new() {
        let manager = TemplateManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_template_manager_with_defaults() {
        let manager = TemplateManager::with_defaults();
        assert!(manager.count() >= 4);
    }

    #[test]
    fn test_template_manager_add() {
        let mut manager = TemplateManager::new();
        let meta = TemplateMeta::new("Test", "Test");
        let id = manager.add(meta, UnifiedSettings::default());
        assert_eq!(manager.count(), 1);
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_template_manager_remove() {
        let mut manager = TemplateManager::new();
        let meta = TemplateMeta::new("Test", "Test");
        let id = manager.add(meta, UnifiedSettings::default());
        assert!(manager.remove(id).is_some());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_template_manager_find_by_name() {
        let manager = TemplateManager::with_defaults();
        let found = manager.find_by_name("dev");
        assert!(!found.is_empty());
    }

    #[test]
    fn test_template_manager_find_by_use_case() {
        let manager = TemplateManager::with_defaults();
        let found = manager.find_by_use_case(TemplateUseCase::Development);
        assert!(!found.is_empty());
    }

    #[test]
    fn test_format_templates() {
        let manager = TemplateManager::with_defaults();
        let output = format_templates(&manager);
        assert!(output.contains("Templates"));
    }

    #[test]
    fn test_is_template_query() {
        assert!(is_template_query("create a template"));
        assert!(is_template_query("apply template"));
        assert!(!is_template_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = template_fun_fact();
        assert!(fact.contains("template"));
    }
}
