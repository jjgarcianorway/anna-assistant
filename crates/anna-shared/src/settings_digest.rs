// v0.0.709: Settings Digest (Phase 285)
// Condensed digest of settings summaries

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Digest type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DigestType {
    /// Daily digest
    #[default]
    Daily,
    /// Weekly digest
    Weekly,
    /// Monthly digest
    Monthly,
    /// Custom digest
    Custom,
}

impl std::fmt::Display for DigestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Digest format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DigestFormat {
    /// Summary format
    #[default]
    Summary,
    /// Detailed format
    Detailed,
    /// Highlights format
    Highlights,
    /// Full format
    Full,
}

impl std::fmt::Display for DigestFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "summary"),
            Self::Detailed => write!(f, "detailed"),
            Self::Highlights => write!(f, "highlights"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Digest config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestConfig {
    /// Name
    pub name: String,
    /// Digest type
    pub digest_type: DigestType,
    /// Format
    pub format: DigestFormat,
    /// Max sections
    pub max_sections: usize,
}

impl DigestConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            digest_type: DigestType::Daily,
            format: DigestFormat::Summary,
            max_sections: 20,
        }
    }

    /// Set type
    pub fn digest_type(mut self, dt: DigestType) -> Self {
        self.digest_type = dt;
        self
    }

    /// Set format
    pub fn format(mut self, f: DigestFormat) -> Self {
        self.format = f;
        self
    }

    /// Set max sections
    pub fn max_sections(mut self, max: usize) -> Self {
        self.max_sections = max;
        self
    }
}

impl Default for DigestConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Digest section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSection {
    /// Section ID
    pub id: String,
    /// Title
    pub title: String,
    /// Summary
    pub summary: String,
    /// Items
    pub items: Vec<DigestItem>,
    /// Order
    pub order: usize,
}

impl DigestSection {
    /// Create new section
    pub fn new(id: impl Into<String>, title: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: String::new(),
            items: Vec::new(),
            order,
        }
    }

    /// Set summary
    pub fn summary(mut self, s: impl Into<String>) -> Self {
        self.summary = s.into();
        self
    }

    /// Add item
    pub fn add(&mut self, item: DigestItem) {
        self.items.push(item);
    }

    /// Item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

/// Digest item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Highlight
    pub highlight: bool,
}

impl DigestItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            highlight: false,
        }
    }

    /// Set highlight
    pub fn highlight(mut self, h: bool) -> Self {
        self.highlight = h;
        self
    }
}

/// Digest stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigestStats {
    /// Total sections
    pub total_sections: usize,
    /// Total items
    pub total_items: usize,
    /// Highlighted items
    pub highlighted: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
}

impl DigestStats {
    /// Update from digest
    pub fn update(&mut self, sections: &[DigestSection], format: DigestFormat) {
        self.total_sections = sections.len();
        self.total_items = sections.iter().map(|s| s.item_count()).sum();
        self.highlighted = sections.iter()
            .flat_map(|s| &s.items)
            .filter(|i| i.highlight)
            .count();
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
    }

    /// Highlight rate
    pub fn highlight_rate(&self) -> f64 {
        if self.total_items == 0 { 0.0 } else { self.highlighted as f64 / self.total_items as f64 * 100.0 }
    }
}

/// Settings digest
#[derive(Debug, Clone, Default)]
pub struct SettingsDigest {
    /// Config
    config: DigestConfig,
    /// Sections
    sections: Vec<DigestSection>,
    /// Stats
    stats: DigestStats,
}

impl SettingsDigest {
    /// Create new digest
    pub fn new(config: DigestConfig) -> Self {
        Self {
            config,
            sections: Vec::new(),
            stats: DigestStats::default(),
        }
    }

    /// Add section
    pub fn add_section(&mut self, section: DigestSection) -> bool {
        if self.sections.len() >= self.config.max_sections {
            return false;
        }
        self.sections.push(section);
        self.update_stats();
        true
    }

    /// Get section
    pub fn get_section(&self, id: &str) -> Option<&DigestSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// Get section mut
    pub fn get_section_mut(&mut self, id: &str) -> Option<&mut DigestSection> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    /// Add item to section
    pub fn add_item(&mut self, section_id: &str, item: DigestItem) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.add(item);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.sections, self.config.format);
    }

    /// Get stats
    pub fn stats(&self) -> &DigestStats {
        &self.stats
    }

    /// Section count
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

/// Digest registry
#[derive(Debug, Clone, Default)]
pub struct DigestRegistry {
    /// Digests by ID
    digests: HashMap<String, SettingsDigest>,
}

impl DigestRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register digest
    pub fn register(&mut self, id: impl Into<String>, digest: SettingsDigest) {
        self.digests.insert(id.into(), digest);
    }

    /// Unregister digest
    pub fn unregister(&mut self, id: &str) -> bool {
        self.digests.remove(id).is_some()
    }

    /// Get digest
    pub fn get(&self, id: &str) -> Option<&SettingsDigest> {
        self.digests.get(id)
    }

    /// Get digest mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDigest> {
        self.digests.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.digests.len()
    }
}

/// Format digest registry
pub fn format_digest_registry(registry: &DigestRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Digest Registry:\n");
    output.push_str(&format!("  Digests: {}\n", registry.count()));
    output
}

/// Check if query is about digest
pub fn is_digest_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings digest") || lower.contains("digest settings") || lower.contains("settings summary")
}

/// Fun fact about digest
pub fn digest_fun_fact() -> &'static str {
    "Anna's settings digest condenses configuration changes into easy-to-read summaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_type_display() {
        assert_eq!(format!("{}", DigestType::Daily), "daily");
        assert_eq!(format!("{}", DigestType::Weekly), "weekly");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", DigestFormat::Summary), "summary");
        assert_eq!(format!("{}", DigestFormat::Detailed), "detailed");
    }

    #[test]
    fn test_config_new() {
        let c = DigestConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DigestConfig::new("test")
            .digest_type(DigestType::Weekly)
            .format(DigestFormat::Highlights);
        assert_eq!(c.digest_type, DigestType::Weekly);
        assert_eq!(c.format, DigestFormat::Highlights);
    }

    #[test]
    fn test_section_new() {
        let s = DigestSection::new("s1", "Section 1", 1);
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_section_add() {
        let mut s = DigestSection::new("s1", "Section 1", 1);
        s.add(DigestItem::new("key", "value"));
        assert_eq!(s.item_count(), 1);
    }

    #[test]
    fn test_item_new() {
        let i = DigestItem::new("key", "value");
        assert_eq!(i.key, "key");
    }

    #[test]
    fn test_item_highlight() {
        let i = DigestItem::new("key", "value").highlight(true);
        assert!(i.highlight);
    }

    #[test]
    fn test_stats_update() {
        let mut s = DigestStats::default();
        let mut section = DigestSection::new("s1", "Section", 1);
        section.add(DigestItem::new("key", "value").highlight(true));
        s.update(&[section], DigestFormat::Summary);
        assert_eq!(s.total_sections, 1);
        assert_eq!(s.highlighted, 1);
    }

    #[test]
    fn test_digest_new() {
        let d = SettingsDigest::new(DigestConfig::default());
        assert_eq!(d.section_count(), 0);
    }

    #[test]
    fn test_digest_add_section() {
        let mut d = SettingsDigest::new(DigestConfig::default());
        d.add_section(DigestSection::new("s1", "Section 1", 1));
        assert_eq!(d.section_count(), 1);
    }

    #[test]
    fn test_digest_add_item() {
        let mut d = SettingsDigest::new(DigestConfig::default());
        d.add_section(DigestSection::new("s1", "Section 1", 1));
        let added = d.add_item("s1", DigestItem::new("key", "value"));
        assert!(added);
    }

    #[test]
    fn test_registry_new() {
        let r = DigestRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DigestRegistry::new();
        r.register("d1", SettingsDigest::new(DigestConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_digest_query() {
        assert!(is_digest_query("settings digest"));
        assert!(!is_digest_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = digest_fun_fact();
        assert!(fact.contains("digest"));
    }
}
