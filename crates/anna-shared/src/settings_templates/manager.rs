// v0.0.569: Settings Templates - Manager (Phase 145)
// Template manager for storing, searching, and applying templates

use crate::unified_settings::{SettingsCategory, UnifiedSettings};
use super::types::{SettingsTemplate, TemplateMeta, TemplateScope, TemplateUseCase};
use super::helpers::apply_category;

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
