// v0.0.695: Settings Folio (Phase 271)
// Portfolio of settings collections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Folio type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FolioType {
    /// Active folio
    #[default]
    Active,
    /// Archived folio
    Archived,
    /// Template folio
    Template,
    /// Backup folio
    Backup,
}

impl std::fmt::Display for FolioType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
            Self::Template => write!(f, "template"),
            Self::Backup => write!(f, "backup"),
        }
    }
}

/// Folio status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FolioStatus {
    /// Open
    #[default]
    Open,
    /// Closed
    Closed,
    /// Locked
    Locked,
    /// Pending
    Pending,
}

impl std::fmt::Display for FolioStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Closed => write!(f, "closed"),
            Self::Locked => write!(f, "locked"),
            Self::Pending => write!(f, "pending"),
        }
    }
}

/// Folio config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioConfig {
    /// Name
    pub name: String,
    /// Folio type
    pub folio_type: FolioType,
    /// Description
    pub description: String,
    /// Max sections
    pub max_sections: usize,
}

impl FolioConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            folio_type: FolioType::Active,
            description: String::new(),
            max_sections: 100,
        }
    }

    /// Set type
    pub fn folio_type(mut self, ft: FolioType) -> Self {
        self.folio_type = ft;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set max sections
    pub fn max_sections(mut self, max: usize) -> Self {
        self.max_sections = max;
        self
    }
}

impl Default for FolioConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Folio section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioSection {
    /// Section ID
    pub id: String,
    /// Name
    pub name: String,
    /// Settings
    pub settings: HashMap<String, String>,
    /// Order
    pub order: usize,
}

impl FolioSection {
    /// Create new section
    pub fn new(id: impl Into<String>, name: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            settings: HashMap::new(),
            order,
        }
    }

    /// Add setting
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Get setting
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Setting count
    pub fn count(&self) -> usize {
        self.settings.len()
    }
}

/// Folio item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Section ID
    pub section_id: String,
    /// Notes
    pub notes: Option<String>,
}

impl FolioItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, section_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            section_id: section_id.into(),
            notes: None,
        }
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Folio stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolioStats {
    /// Total sections
    pub total_sections: usize,
    /// Total settings
    pub total_settings: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FolioStats {
    /// Update from folio
    pub fn update(&mut self, sections: &[FolioSection], folio_type: FolioType) {
        self.total_sections = sections.len();
        self.total_settings = sections.iter().map(|s| s.count()).sum();
        *self.by_type.entry(folio_type.to_string()).or_insert(0) += 1;
    }

    /// Avg settings per section
    pub fn avg_per_section(&self) -> f64 {
        if self.total_sections == 0 { 0.0 } else { self.total_settings as f64 / self.total_sections as f64 }
    }
}

/// Settings folio
#[derive(Debug, Clone, Default)]
pub struct SettingsFolio {
    /// Config
    config: FolioConfig,
    /// Sections
    sections: Vec<FolioSection>,
    /// Status
    status: FolioStatus,
    /// Stats
    stats: FolioStats,
}

impl SettingsFolio {
    /// Create new folio
    pub fn new(config: FolioConfig) -> Self {
        Self {
            config,
            sections: Vec::new(),
            status: FolioStatus::Open,
            stats: FolioStats::default(),
        }
    }

    /// Add section
    pub fn add_section(&mut self, id: &str, name: &str) -> bool {
        if self.sections.len() >= self.config.max_sections {
            return false;
        }
        let order = self.sections.len();
        self.sections.push(FolioSection::new(id, name, order));
        self.update_stats();
        true
    }

    /// Get section
    pub fn get_section(&self, id: &str) -> Option<&FolioSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// Get section mut
    pub fn get_section_mut(&mut self, id: &str) -> Option<&mut FolioSection> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    /// Add setting to section
    pub fn add_setting(&mut self, section_id: &str, key: &str, value: &str) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.add(key, value);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.sections, self.config.folio_type);
    }

    /// Lock folio
    pub fn lock(&mut self) {
        self.status = FolioStatus::Locked;
    }

    /// Close folio
    pub fn close(&mut self) {
        self.status = FolioStatus::Closed;
    }

    /// Get status
    pub fn status(&self) -> FolioStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &FolioStats {
        &self.stats
    }

    /// Section count
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

/// Folio registry
#[derive(Debug, Clone, Default)]
pub struct FolioRegistry {
    /// Folios by ID
    folios: HashMap<String, SettingsFolio>,
}

impl FolioRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register folio
    pub fn register(&mut self, id: impl Into<String>, folio: SettingsFolio) {
        self.folios.insert(id.into(), folio);
    }

    /// Unregister folio
    pub fn unregister(&mut self, id: &str) -> bool {
        self.folios.remove(id).is_some()
    }

    /// Get folio
    pub fn get(&self, id: &str) -> Option<&SettingsFolio> {
        self.folios.get(id)
    }

    /// Get folio mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFolio> {
        self.folios.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.folios.len()
    }
}

/// Format folio registry
pub fn format_folio_registry(registry: &FolioRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Folio Registry:\n");
    output.push_str(&format!("  Folios: {}\n", registry.count()));
    output
}

/// Check if query is about folio
pub fn is_folio_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings folio") || lower.contains("folio settings") || lower.contains("settings portfolio")
}

/// Fun fact about folio
pub fn folio_fun_fact() -> &'static str {
    "Anna's settings folio organizes configurations into elegant sections!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folio_type_display() {
        assert_eq!(format!("{}", FolioType::Active), "active");
        assert_eq!(format!("{}", FolioType::Template), "template");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", FolioStatus::Open), "open");
        assert_eq!(format!("{}", FolioStatus::Locked), "locked");
    }

    #[test]
    fn test_config_new() {
        let c = FolioConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = FolioConfig::new("test")
            .folio_type(FolioType::Template)
            .max_sections(50);
        assert_eq!(c.folio_type, FolioType::Template);
        assert_eq!(c.max_sections, 50);
    }

    #[test]
    fn test_section_new() {
        let s = FolioSection::new("s1", "Section 1", 0);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_section_add() {
        let mut s = FolioSection::new("s1", "Section 1", 0);
        s.add("key", "value");
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn test_item_new() {
        let i = FolioItem::new("key", "value", "s1");
        assert_eq!(i.section_id, "s1");
    }

    #[test]
    fn test_item_notes() {
        let i = FolioItem::new("key", "value", "s1").notes("important");
        assert!(i.notes.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = FolioStats::default();
        let sections = vec![FolioSection::new("s1", "Section", 0)];
        s.update(&sections, FolioType::Active);
        assert_eq!(s.total_sections, 1);
    }

    #[test]
    fn test_folio_new() {
        let f = SettingsFolio::new(FolioConfig::default());
        assert_eq!(f.section_count(), 0);
    }

    #[test]
    fn test_folio_add_section() {
        let mut f = SettingsFolio::new(FolioConfig::default());
        f.add_section("s1", "Section 1");
        assert_eq!(f.section_count(), 1);
    }

    #[test]
    fn test_folio_add_setting() {
        let mut f = SettingsFolio::new(FolioConfig::default());
        f.add_section("s1", "Section 1");
        let added = f.add_setting("s1", "key", "value");
        assert!(added);
    }

    #[test]
    fn test_folio_lock() {
        let mut f = SettingsFolio::new(FolioConfig::default());
        f.lock();
        assert_eq!(f.status(), FolioStatus::Locked);
    }

    #[test]
    fn test_registry_new() {
        let r = FolioRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FolioRegistry::new();
        r.register("f1", SettingsFolio::new(FolioConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_folio_query() {
        assert!(is_folio_query("settings folio"));
        assert!(!is_folio_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = folio_fun_fact();
        assert!(fact.contains("folio"));
    }
}
