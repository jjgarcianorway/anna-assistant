// v0.0.701: Settings Anthology (Phase 277)
// Curated anthology of settings collections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anthology type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AnthologyType {
    /// Best of anthology
    #[default]
    BestOf,
    /// Complete anthology
    Complete,
    /// Themed anthology
    Themed,
    /// Historical anthology
    Historical,
}

impl std::fmt::Display for AnthologyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BestOf => write!(f, "best_of"),
            Self::Complete => write!(f, "complete"),
            Self::Themed => write!(f, "themed"),
            Self::Historical => write!(f, "historical"),
        }
    }
}

/// Anthology status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AnthologyStatus {
    /// Curating
    #[default]
    Curating,
    /// Complete
    Complete,
    /// Published
    Published,
    /// Archived
    Archived,
}

impl std::fmt::Display for AnthologyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Curating => write!(f, "curating"),
            Self::Complete => write!(f, "complete"),
            Self::Published => write!(f, "published"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

/// Anthology config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthologyConfig {
    /// Name
    pub name: String,
    /// Anthology type
    pub anthology_type: AnthologyType,
    /// Theme
    pub theme: String,
    /// Max works
    pub max_works: usize,
}

impl AnthologyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            anthology_type: AnthologyType::BestOf,
            theme: String::new(),
            max_works: 100,
        }
    }

    /// Set type
    pub fn anthology_type(mut self, at: AnthologyType) -> Self {
        self.anthology_type = at;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    /// Set max works
    pub fn max_works(mut self, max: usize) -> Self {
        self.max_works = max;
        self
    }
}

impl Default for AnthologyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Anthology work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthologyWork {
    /// Work ID
    pub id: String,
    /// Title
    pub title: String,
    /// Author
    pub author: String,
    /// Source
    pub source: String,
    /// Featured
    pub featured: bool,
}

impl AnthologyWork {
    /// Create new work
    pub fn new(id: impl Into<String>, title: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            author: author.into(),
            source: String::new(),
            featured: false,
        }
    }

    /// Set source
    pub fn source(mut self, src: impl Into<String>) -> Self {
        self.source = src.into();
        self
    }

    /// Set featured
    pub fn featured(mut self, feat: bool) -> Self {
        self.featured = feat;
        self
    }
}

/// Anthology piece
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthologyPiece {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Work ID
    pub work_id: String,
    /// Order
    pub order: usize,
}

impl AnthologyPiece {
    /// Create new piece
    pub fn new(key: impl Into<String>, value: impl Into<String>, work_id: impl Into<String>, order: usize) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            work_id: work_id.into(),
            order,
        }
    }
}

/// Anthology stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnthologyStats {
    /// Total works
    pub total_works: usize,
    /// Featured works
    pub featured_works: usize,
    /// Total pieces
    pub total_pieces: usize,
    /// By author
    pub by_author: HashMap<String, usize>,
}

impl AnthologyStats {
    /// Update from anthology
    pub fn update(&mut self, works: &[AnthologyWork]) {
        self.total_works = works.len();
        self.featured_works = works.iter().filter(|w| w.featured).count();
        self.by_author.clear();
        for work in works {
            if !work.author.is_empty() {
                *self.by_author.entry(work.author.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Record piece
    pub fn record_piece(&mut self) {
        self.total_pieces += 1;
    }

    /// Featured rate
    pub fn featured_rate(&self) -> f64 {
        if self.total_works == 0 { 0.0 } else { self.featured_works as f64 / self.total_works as f64 * 100.0 }
    }
}

/// Settings anthology
#[derive(Debug, Clone, Default)]
pub struct SettingsAnthology {
    /// Config
    config: AnthologyConfig,
    /// Works
    works: Vec<AnthologyWork>,
    /// Pieces
    pieces: Vec<AnthologyPiece>,
    /// Status
    status: AnthologyStatus,
    /// Stats
    stats: AnthologyStats,
}

impl SettingsAnthology {
    /// Create new anthology
    pub fn new(config: AnthologyConfig) -> Self {
        Self {
            config,
            works: Vec::new(),
            pieces: Vec::new(),
            status: AnthologyStatus::Curating,
            stats: AnthologyStats::default(),
        }
    }

    /// Add work
    pub fn add_work(&mut self, work: AnthologyWork) -> bool {
        if self.works.len() >= self.config.max_works {
            return false;
        }
        self.works.push(work);
        self.update_stats();
        true
    }

    /// Get work
    pub fn get_work(&self, id: &str) -> Option<&AnthologyWork> {
        self.works.iter().find(|w| w.id == id)
    }

    /// Add piece
    pub fn add_piece(&mut self, piece: AnthologyPiece) {
        self.pieces.push(piece);
        self.stats.record_piece();
    }

    /// Get pieces for work
    pub fn get_pieces(&self, work_id: &str) -> Vec<&AnthologyPiece> {
        let mut result: Vec<_> = self.pieces.iter().filter(|p| p.work_id == work_id).collect();
        result.sort_by_key(|p| p.order);
        result
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.works);
    }

    /// Complete
    pub fn complete(&mut self) {
        self.status = AnthologyStatus::Complete;
    }

    /// Publish
    pub fn publish(&mut self) {
        self.status = AnthologyStatus::Published;
    }

    /// Archive
    pub fn archive(&mut self) {
        self.status = AnthologyStatus::Archived;
    }

    /// Get status
    pub fn status(&self) -> AnthologyStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &AnthologyStats {
        &self.stats
    }

    /// Work count
    pub fn work_count(&self) -> usize {
        self.works.len()
    }
}

/// Anthology registry
#[derive(Debug, Clone, Default)]
pub struct AnthologyRegistry {
    /// Anthologies by ID
    anthologies: HashMap<String, SettingsAnthology>,
}

impl AnthologyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register anthology
    pub fn register(&mut self, id: impl Into<String>, anthology: SettingsAnthology) {
        self.anthologies.insert(id.into(), anthology);
    }

    /// Unregister anthology
    pub fn unregister(&mut self, id: &str) -> bool {
        self.anthologies.remove(id).is_some()
    }

    /// Get anthology
    pub fn get(&self, id: &str) -> Option<&SettingsAnthology> {
        self.anthologies.get(id)
    }

    /// Get anthology mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAnthology> {
        self.anthologies.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.anthologies.len()
    }
}

/// Format anthology registry
pub fn format_anthology_registry(registry: &AnthologyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Anthology Registry:\n");
    output.push_str(&format!("  Anthologies: {}\n", registry.count()));
    output
}

/// Check if query is about anthology
pub fn is_anthology_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings anthology") || lower.contains("anthology settings") || lower.contains("curated settings")
}

/// Fun fact about anthology
pub fn anthology_fun_fact() -> &'static str {
    "Anna's settings anthology curates the best configurations into beautiful collections!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthology_type_display() {
        assert_eq!(format!("{}", AnthologyType::BestOf), "best_of");
        assert_eq!(format!("{}", AnthologyType::Complete), "complete");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AnthologyStatus::Curating), "curating");
        assert_eq!(format!("{}", AnthologyStatus::Published), "published");
    }

    #[test]
    fn test_config_new() {
        let c = AnthologyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AnthologyConfig::new("test")
            .anthology_type(AnthologyType::Themed)
            .theme("Linux configs");
        assert_eq!(c.anthology_type, AnthologyType::Themed);
        assert_eq!(c.theme, "Linux configs");
    }

    #[test]
    fn test_work_new() {
        let w = AnthologyWork::new("w1", "Work 1", "Author");
        assert_eq!(w.id, "w1");
    }

    #[test]
    fn test_work_builder() {
        let w = AnthologyWork::new("w1", "Work 1", "Author")
            .source("config.toml")
            .featured(true);
        assert_eq!(w.source, "config.toml");
        assert!(w.featured);
    }

    #[test]
    fn test_piece_new() {
        let p = AnthologyPiece::new("key", "value", "w1", 1);
        assert_eq!(p.work_id, "w1");
        assert_eq!(p.order, 1);
    }

    #[test]
    fn test_stats_update() {
        let mut s = AnthologyStats::default();
        let works = vec![AnthologyWork::new("w1", "Work", "Author").featured(true)];
        s.update(&works);
        assert_eq!(s.total_works, 1);
        assert_eq!(s.featured_works, 1);
    }

    #[test]
    fn test_anthology_new() {
        let a = SettingsAnthology::new(AnthologyConfig::default());
        assert_eq!(a.work_count(), 0);
    }

    #[test]
    fn test_anthology_add_work() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.add_work(AnthologyWork::new("w1", "Work 1", "Author"));
        assert_eq!(a.work_count(), 1);
    }

    #[test]
    fn test_anthology_add_piece() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.add_piece(AnthologyPiece::new("key", "value", "w1", 1));
        assert_eq!(a.stats().total_pieces, 1);
    }

    #[test]
    fn test_anthology_complete() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.complete();
        assert_eq!(a.status(), AnthologyStatus::Complete);
    }

    #[test]
    fn test_anthology_publish() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.publish();
        assert_eq!(a.status(), AnthologyStatus::Published);
    }

    #[test]
    fn test_registry_new() {
        let r = AnthologyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AnthologyRegistry::new();
        r.register("a1", SettingsAnthology::new(AnthologyConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_anthology_query() {
        assert!(is_anthology_query("settings anthology"));
        assert!(!is_anthology_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = anthology_fun_fact();
        assert!(fact.contains("anthology"));
    }
}
