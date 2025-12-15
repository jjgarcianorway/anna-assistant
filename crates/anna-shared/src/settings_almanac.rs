// v0.0.705: Settings Almanac (Phase 281)
// Yearly almanac of settings information

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Almanac type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlmanacType {
    /// Annual almanac
    #[default]
    Annual,
    /// Seasonal almanac
    Seasonal,
    /// Technical almanac
    Technical,
    /// Historical almanac
    Historical,
}

impl std::fmt::Display for AlmanacType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Annual => write!(f, "annual"),
            Self::Seasonal => write!(f, "seasonal"),
            Self::Technical => write!(f, "technical"),
            Self::Historical => write!(f, "historical"),
        }
    }
}

/// Almanac edition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlmanacEdition {
    /// Current edition
    #[default]
    Current,
    /// Previous edition
    Previous,
    /// Special edition
    Special,
    /// Commemorative edition
    Commemorative,
}

impl std::fmt::Display for AlmanacEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Current => write!(f, "current"),
            Self::Previous => write!(f, "previous"),
            Self::Special => write!(f, "special"),
            Self::Commemorative => write!(f, "commemorative"),
        }
    }
}

/// Almanac config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacConfig {
    /// Name
    pub name: String,
    /// Almanac type
    pub almanac_type: AlmanacType,
    /// Year
    pub year: usize,
    /// Max chapters
    pub max_chapters: usize,
}

impl AlmanacConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            almanac_type: AlmanacType::Annual,
            year: 2025,
            max_chapters: 52,
        }
    }

    /// Set type
    pub fn almanac_type(mut self, at: AlmanacType) -> Self {
        self.almanac_type = at;
        self
    }

    /// Set year
    pub fn year(mut self, year: usize) -> Self {
        self.year = year;
        self
    }

    /// Set max chapters
    pub fn max_chapters(mut self, max: usize) -> Self {
        self.max_chapters = max;
        self
    }
}

impl Default for AlmanacConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Almanac chapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacChapter {
    /// Chapter number
    pub number: usize,
    /// Title
    pub title: String,
    /// Period (e.g., "Week 1", "Q1")
    pub period: String,
    /// Entries
    pub entries: Vec<AlmanacEntry>,
}

impl AlmanacChapter {
    /// Create new chapter
    pub fn new(number: usize, title: impl Into<String>, period: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            period: period.into(),
            entries: Vec::new(),
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: AlmanacEntry) {
        self.entries.push(entry);
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Almanac entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Date
    pub date: String,
    /// Highlights
    pub highlight: bool,
}

impl AlmanacEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            date: date.into(),
            highlight: false,
        }
    }

    /// Set highlight
    pub fn highlight(mut self, h: bool) -> Self {
        self.highlight = h;
        self
    }
}

/// Almanac stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlmanacStats {
    /// Total chapters
    pub total_chapters: usize,
    /// Total entries
    pub total_entries: usize,
    /// Highlighted entries
    pub highlighted: usize,
    /// By period
    pub by_period: HashMap<String, usize>,
}

impl AlmanacStats {
    /// Update from almanac
    pub fn update(&mut self, chapters: &[AlmanacChapter]) {
        self.total_chapters = chapters.len();
        self.total_entries = chapters.iter().map(|c| c.entry_count()).sum();
        self.highlighted = chapters.iter()
            .flat_map(|c| &c.entries)
            .filter(|e| e.highlight)
            .count();
        self.by_period.clear();
        for ch in chapters {
            if !ch.period.is_empty() {
                *self.by_period.entry(ch.period.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Highlight rate
    pub fn highlight_rate(&self) -> f64 {
        if self.total_entries == 0 { 0.0 } else { self.highlighted as f64 / self.total_entries as f64 * 100.0 }
    }
}

/// Settings almanac
#[derive(Debug, Clone, Default)]
pub struct SettingsAlmanac {
    /// Config
    config: AlmanacConfig,
    /// Chapters
    chapters: Vec<AlmanacChapter>,
    /// Edition
    edition: AlmanacEdition,
    /// Stats
    stats: AlmanacStats,
}

impl SettingsAlmanac {
    /// Create new almanac
    pub fn new(config: AlmanacConfig) -> Self {
        Self {
            config,
            chapters: Vec::new(),
            edition: AlmanacEdition::Current,
            stats: AlmanacStats::default(),
        }
    }

    /// Add chapter
    pub fn add_chapter(&mut self, chapter: AlmanacChapter) -> bool {
        if self.chapters.len() >= self.config.max_chapters {
            return false;
        }
        self.chapters.push(chapter);
        self.update_stats();
        true
    }

    /// Get chapter
    pub fn get_chapter(&self, number: usize) -> Option<&AlmanacChapter> {
        self.chapters.iter().find(|c| c.number == number)
    }

    /// Get chapter mut
    pub fn get_chapter_mut(&mut self, number: usize) -> Option<&mut AlmanacChapter> {
        self.chapters.iter_mut().find(|c| c.number == number)
    }

    /// Add entry to chapter
    pub fn add_entry(&mut self, chapter_number: usize, entry: AlmanacEntry) -> bool {
        if let Some(chapter) = self.get_chapter_mut(chapter_number) {
            chapter.add(entry);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.chapters);
    }

    /// Set edition
    pub fn set_edition(&mut self, edition: AlmanacEdition) {
        self.edition = edition;
    }

    /// Get edition
    pub fn edition(&self) -> AlmanacEdition {
        self.edition
    }

    /// Get stats
    pub fn stats(&self) -> &AlmanacStats {
        &self.stats
    }

    /// Chapter count
    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }
}

/// Almanac registry
#[derive(Debug, Clone, Default)]
pub struct AlmanacRegistry {
    /// Almanacs by ID
    almanacs: HashMap<String, SettingsAlmanac>,
}

impl AlmanacRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register almanac
    pub fn register(&mut self, id: impl Into<String>, almanac: SettingsAlmanac) {
        self.almanacs.insert(id.into(), almanac);
    }

    /// Unregister almanac
    pub fn unregister(&mut self, id: &str) -> bool {
        self.almanacs.remove(id).is_some()
    }

    /// Get almanac
    pub fn get(&self, id: &str) -> Option<&SettingsAlmanac> {
        self.almanacs.get(id)
    }

    /// Get almanac mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAlmanac> {
        self.almanacs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.almanacs.len()
    }
}

/// Format almanac registry
pub fn format_almanac_registry(registry: &AlmanacRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Almanac Registry:\n");
    output.push_str(&format!("  Almanacs: {}\n", registry.count()));
    output
}

/// Check if query is about almanac
pub fn is_almanac_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings almanac") || lower.contains("almanac settings") || lower.contains("yearly settings")
}

/// Fun fact about almanac
pub fn almanac_fun_fact() -> &'static str {
    "Anna's settings almanac chronicles your configurations throughout the year!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_almanac_type_display() {
        assert_eq!(format!("{}", AlmanacType::Annual), "annual");
        assert_eq!(format!("{}", AlmanacType::Technical), "technical");
    }

    #[test]
    fn test_edition_display() {
        assert_eq!(format!("{}", AlmanacEdition::Current), "current");
        assert_eq!(format!("{}", AlmanacEdition::Special), "special");
    }

    #[test]
    fn test_config_new() {
        let c = AlmanacConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AlmanacConfig::new("test")
            .almanac_type(AlmanacType::Seasonal)
            .year(2025);
        assert_eq!(c.almanac_type, AlmanacType::Seasonal);
        assert_eq!(c.year, 2025);
    }

    #[test]
    fn test_chapter_new() {
        let ch = AlmanacChapter::new(1, "Chapter 1", "Week 1");
        assert_eq!(ch.number, 1);
    }

    #[test]
    fn test_chapter_add() {
        let mut ch = AlmanacChapter::new(1, "Chapter 1", "Week 1");
        ch.add(AlmanacEntry::new("key", "value", "2025-12-15"));
        assert_eq!(ch.entry_count(), 1);
    }

    #[test]
    fn test_entry_new() {
        let e = AlmanacEntry::new("key", "value", "2025-12-15");
        assert_eq!(e.key, "key");
    }

    #[test]
    fn test_entry_highlight() {
        let e = AlmanacEntry::new("key", "value", "2025-12-15").highlight(true);
        assert!(e.highlight);
    }

    #[test]
    fn test_stats_update() {
        let mut s = AlmanacStats::default();
        let mut ch = AlmanacChapter::new(1, "Chapter", "Week 1");
        ch.add(AlmanacEntry::new("key", "value", "2025-12-15").highlight(true));
        s.update(&[ch]);
        assert_eq!(s.total_chapters, 1);
        assert_eq!(s.highlighted, 1);
    }

    #[test]
    fn test_almanac_new() {
        let a = SettingsAlmanac::new(AlmanacConfig::default());
        assert_eq!(a.chapter_count(), 0);
    }

    #[test]
    fn test_almanac_add_chapter() {
        let mut a = SettingsAlmanac::new(AlmanacConfig::default());
        a.add_chapter(AlmanacChapter::new(1, "Chapter 1", "Week 1"));
        assert_eq!(a.chapter_count(), 1);
    }

    #[test]
    fn test_almanac_add_entry() {
        let mut a = SettingsAlmanac::new(AlmanacConfig::default());
        a.add_chapter(AlmanacChapter::new(1, "Chapter 1", "Week 1"));
        let added = a.add_entry(1, AlmanacEntry::new("key", "value", "2025-12-15"));
        assert!(added);
    }

    #[test]
    fn test_almanac_edition() {
        let mut a = SettingsAlmanac::new(AlmanacConfig::default());
        a.set_edition(AlmanacEdition::Special);
        assert_eq!(a.edition(), AlmanacEdition::Special);
    }

    #[test]
    fn test_registry_new() {
        let r = AlmanacRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AlmanacRegistry::new();
        r.register("a1", SettingsAlmanac::new(AlmanacConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_almanac_query() {
        assert!(is_almanac_query("settings almanac"));
        assert!(!is_almanac_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = almanac_fun_fact();
        assert!(fact.contains("almanac"));
    }
}
