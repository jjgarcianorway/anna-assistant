// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Comprehensive compendium of settings knowledge

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compendium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompendiumType {
    /// Reference compendium
    #[default]
    Reference,
    /// Tutorial compendium
    Tutorial,
    /// Encyclopedia compendium
    Encyclopedia,
    /// Handbook compendium
    Handbook,
}

impl std::fmt::Display for CompendiumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference => write!(f, "reference"),
            Self::Tutorial => write!(f, "tutorial"),
            Self::Encyclopedia => write!(f, "encyclopedia"),
            Self::Handbook => write!(f, "handbook"),
        }
    }
}

/// Compendium edition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompendiumEdition {
    /// First edition
    #[default]
    First,
    /// Revised edition
    Revised,
    /// Extended edition
    Extended,
    /// Final edition
    Final,
}

impl std::fmt::Display for CompendiumEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::Revised => write!(f, "revised"),
            Self::Extended => write!(f, "extended"),
            Self::Final => write!(f, "final"),
        }
    }
}

/// Compendium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumConfig {
    /// Name
    pub name: String,
    /// Compendium type
    pub compendium_type: CompendiumType,
    /// Edition
    pub edition: CompendiumEdition,
    /// Max volumes
    pub max_volumes: usize,
}

impl CompendiumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compendium_type: CompendiumType::Reference,
            edition: CompendiumEdition::First,
            max_volumes: 100,
        }
    }

    /// Set type
    pub fn compendium_type(mut self, ct: CompendiumType) -> Self {
        self.compendium_type = ct;
        self
    }

    /// Set edition
    pub fn edition(mut self, ed: CompendiumEdition) -> Self {
        self.edition = ed;
        self
    }

    /// Set max volumes
    pub fn max_volumes(mut self, max: usize) -> Self {
        self.max_volumes = max;
        self
    }
}

impl Default for CompendiumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Compendium volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumVolume {
    /// Volume number
    pub number: usize,
    /// Title
    pub title: String,
    /// Subject
    pub subject: String,
    /// Articles
    pub articles: Vec<CompendiumArticle>,
}

impl CompendiumVolume {
    /// Create new volume
    pub fn new(number: usize, title: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            subject: String::new(),
            articles: Vec::new(),
        }
    }

    /// Set subject
    pub fn subject(mut self, subj: impl Into<String>) -> Self {
        self.subject = subj.into();
        self
    }

    /// Add article
    pub fn add(&mut self, article: CompendiumArticle) {
        self.articles.push(article);
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Compendium article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Keywords
    pub keywords: Vec<String>,
}

impl CompendiumArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            keywords: Vec::new(),
        }
    }

    /// Add keyword
    pub fn keyword(mut self, kw: impl Into<String>) -> Self {
        self.keywords.push(kw.into());
        self
    }
}

/// Compendium entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Article ID
    pub article_id: String,
    /// Definition
    pub definition: Option<String>,
}

impl CompendiumEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            article_id: article_id.into(),
            definition: None,
        }
    }

    /// Set definition
    pub fn definition(mut self, def: impl Into<String>) -> Self {
        self.definition = Some(def.into());
        self
    }
}

/// Compendium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompendiumStats {
    /// Total volumes
    pub total_volumes: usize,
    /// Total articles
    pub total_articles: usize,
    /// Total entries
    pub total_entries: usize,
    /// By subject
    pub by_subject: HashMap<String, usize>,
}

impl CompendiumStats {
    /// Update from compendium
    pub fn update(&mut self, volumes: &[CompendiumVolume]) {
        self.total_volumes = volumes.len();
        self.total_articles = volumes.iter().map(|v| v.article_count()).sum();
        self.by_subject.clear();
        for vol in volumes {
            if !vol.subject.is_empty() {
                *self.by_subject.entry(vol.subject.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Record entry
    pub fn record_entry(&mut self) {
        self.total_entries += 1;
    }

    /// Avg articles per volume
    pub fn avg_per_volume(&self) -> f64 {
        if self.total_volumes == 0 { 0.0 } else { self.total_articles as f64 / self.total_volumes as f64 }
    }
}

/// Settings compendium
#[derive(Debug, Clone, Default)]
pub struct SettingsCompendium {
    /// Config
    config: CompendiumConfig,
    /// Volumes
    volumes: Vec<CompendiumVolume>,
    /// Entries
    entries: Vec<CompendiumEntry>,
    /// Stats
    stats: CompendiumStats,
}

impl SettingsCompendium {
    /// Create new compendium
    pub fn new(config: CompendiumConfig) -> Self {
        Self {
            config,
            volumes: Vec::new(),
            entries: Vec::new(),
            stats: CompendiumStats::default(),
        }
    }

    /// Add volume
    pub fn add_volume(&mut self, volume: CompendiumVolume) -> bool {
        if self.volumes.len() >= self.config.max_volumes {
            return false;
        }
        self.volumes.push(volume);
        self.update_stats();
        true
    }

    /// Get volume
    pub fn get_volume(&self, number: usize) -> Option<&CompendiumVolume> {
        self.volumes.iter().find(|v| v.number == number)
    }

    /// Get volume mut
    pub fn get_volume_mut(&mut self, number: usize) -> Option<&mut CompendiumVolume> {
        self.volumes.iter_mut().find(|v| v.number == number)
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: CompendiumEntry) {
        self.entries.push(entry);
        self.stats.record_entry();
    }

    /// Get entries for article
    pub fn get_entries(&self, article_id: &str) -> Vec<&CompendiumEntry> {
        self.entries.iter().filter(|e| e.article_id == article_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.volumes);
    }

    /// Get stats
    pub fn stats(&self) -> &CompendiumStats {
        &self.stats
    }

    /// Volume count
    pub fn volume_count(&self) -> usize {
        self.volumes.len()
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Compendium registry
#[derive(Debug, Clone, Default)]
pub struct CompendiumRegistry {
    /// Compendiums by ID
    compendiums: HashMap<String, SettingsCompendium>,
}

impl CompendiumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register compendium
    pub fn register(&mut self, id: impl Into<String>, compendium: SettingsCompendium) {
        self.compendiums.insert(id.into(), compendium);
    }

    /// Unregister compendium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.compendiums.remove(id).is_some()
    }

    /// Get compendium
    pub fn get(&self, id: &str) -> Option<&SettingsCompendium> {
        self.compendiums.get(id)
    }

    /// Get compendium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCompendium> {
        self.compendiums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.compendiums.len()
    }
}

/// Format compendium registry
pub fn format_compendium_registry(registry: &CompendiumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Compendium Registry:\n");
    output.push_str(&format!("  Compendiums: {}\n", registry.count()));
    output
}

/// Check if query is about compendium
pub fn is_compendium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings compendium") || lower.contains("compendium settings") || lower.contains("settings encyclopedia")
}

/// Fun fact about compendium
pub fn compendium_fun_fact() -> &'static str {
    "Anna's settings compendium is a comprehensive encyclopedia of your configurations! v0.0.700 milestone!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compendium_type_display() {
        assert_eq!(format!("{}", CompendiumType::Reference), "reference");
        assert_eq!(format!("{}", CompendiumType::Encyclopedia), "encyclopedia");
    }

    #[test]
    fn test_edition_display() {
        assert_eq!(format!("{}", CompendiumEdition::First), "first");
        assert_eq!(format!("{}", CompendiumEdition::Final), "final");
    }

    #[test]
    fn test_config_new() {
        let c = CompendiumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CompendiumConfig::new("test")
            .compendium_type(CompendiumType::Encyclopedia)
            .edition(CompendiumEdition::Revised);
        assert_eq!(c.compendium_type, CompendiumType::Encyclopedia);
        assert_eq!(c.edition, CompendiumEdition::Revised);
    }

    #[test]
    fn test_volume_new() {
        let v = CompendiumVolume::new(1, "Volume 1");
        assert_eq!(v.number, 1);
    }

    #[test]
    fn test_volume_add() {
        let mut v = CompendiumVolume::new(1, "Volume 1");
        v.add(CompendiumArticle::new("a1", "Article 1", "Content"));
        assert_eq!(v.article_count(), 1);
    }

    #[test]
    fn test_article_new() {
        let a = CompendiumArticle::new("a1", "Article 1", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_keywords() {
        let a = CompendiumArticle::new("a1", "Article", "Content")
            .keyword("config")
            .keyword("settings");
        assert_eq!(a.keywords.len(), 2);
    }

    #[test]
    fn test_entry_new() {
        let e = CompendiumEntry::new("key", "value", "a1");
        assert_eq!(e.article_id, "a1");
    }

    #[test]
    fn test_entry_definition() {
        let e = CompendiumEntry::new("key", "value", "a1").definition("A configuration key");
        assert!(e.definition.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = CompendiumStats::default();
        let volumes = vec![CompendiumVolume::new(1, "Volume")];
        s.update(&volumes);
        assert_eq!(s.total_volumes, 1);
    }

    #[test]
    fn test_compendium_new() {
        let c = SettingsCompendium::new(CompendiumConfig::default());
        assert_eq!(c.volume_count(), 0);
    }

    #[test]
    fn test_compendium_add_volume() {
        let mut c = SettingsCompendium::new(CompendiumConfig::default());
        c.add_volume(CompendiumVolume::new(1, "Volume 1"));
        assert_eq!(c.volume_count(), 1);
    }

    #[test]
    fn test_compendium_add_entry() {
        let mut c = SettingsCompendium::new(CompendiumConfig::default());
        c.add_entry(CompendiumEntry::new("key", "value", "a1"));
        assert_eq!(c.entry_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CompendiumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CompendiumRegistry::new();
        r.register("c1", SettingsCompendium::new(CompendiumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_compendium_query() {
        assert!(is_compendium_query("settings compendium"));
        assert!(!is_compendium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = compendium_fun_fact();
        assert!(fact.contains("compendium"));
    }
}
