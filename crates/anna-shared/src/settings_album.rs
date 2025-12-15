// v0.0.696: Settings Album (Phase 272)
// Collection album of settings snapshots

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Album type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlbumType {
    /// Standard album
    #[default]
    Standard,
    /// Collection album
    Collection,
    /// Archive album
    Archive,
    /// Snapshot album
    Snapshot,
}

impl std::fmt::Display for AlbumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Collection => write!(f, "collection"),
            Self::Archive => write!(f, "archive"),
            Self::Snapshot => write!(f, "snapshot"),
        }
    }
}

/// Album status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlbumStatus {
    /// Empty
    #[default]
    Empty,
    /// Partial
    Partial,
    /// Complete
    Complete,
    /// Sealed
    Sealed,
}

impl std::fmt::Display for AlbumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Partial => write!(f, "partial"),
            Self::Complete => write!(f, "complete"),
            Self::Sealed => write!(f, "sealed"),
        }
    }
}

/// Album config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumConfig {
    /// Name
    pub name: String,
    /// Album type
    pub album_type: AlbumType,
    /// Description
    pub description: String,
    /// Max pages
    pub max_pages: usize,
}

impl AlbumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            album_type: AlbumType::Standard,
            description: String::new(),
            max_pages: 50,
        }
    }

    /// Set type
    pub fn album_type(mut self, at: AlbumType) -> Self {
        self.album_type = at;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set max pages
    pub fn max_pages(mut self, max: usize) -> Self {
        self.max_pages = max;
        self
    }
}

impl Default for AlbumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Album page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumPage {
    /// Page number
    pub number: usize,
    /// Title
    pub title: String,
    /// Items
    pub items: Vec<AlbumItem>,
    /// Notes
    pub notes: Option<String>,
}

impl AlbumPage {
    /// Create new page
    pub fn new(number: usize, title: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            items: Vec::new(),
            notes: None,
        }
    }

    /// Add item
    pub fn add(&mut self, item: AlbumItem) {
        self.items.push(item);
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Item count
    pub fn count(&self) -> usize {
        self.items.len()
    }
}

/// Album item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Label
    pub label: Option<String>,
    /// Timestamp
    pub timestamp: String,
}

impl AlbumItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, timestamp: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            label: None,
            timestamp: timestamp.into(),
        }
    }

    /// Set label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Album stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlbumStats {
    /// Total pages
    pub total_pages: usize,
    /// Total items
    pub total_items: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AlbumStats {
    /// Update from album
    pub fn update(&mut self, pages: &[AlbumPage], album_type: AlbumType) {
        self.total_pages = pages.len();
        self.total_items = pages.iter().map(|p| p.count()).sum();
        *self.by_type.entry(album_type.to_string()).or_insert(0) += 1;
    }

    /// Avg items per page
    pub fn avg_per_page(&self) -> f64 {
        if self.total_pages == 0 { 0.0 } else { self.total_items as f64 / self.total_pages as f64 }
    }
}

/// Settings album
#[derive(Debug, Clone, Default)]
pub struct SettingsAlbum {
    /// Config
    config: AlbumConfig,
    /// Pages
    pages: Vec<AlbumPage>,
    /// Status
    status: AlbumStatus,
    /// Stats
    stats: AlbumStats,
}

impl SettingsAlbum {
    /// Create new album
    pub fn new(config: AlbumConfig) -> Self {
        Self {
            config,
            pages: Vec::new(),
            status: AlbumStatus::Empty,
            stats: AlbumStats::default(),
        }
    }

    /// Add page
    pub fn add_page(&mut self, title: &str) -> bool {
        if self.pages.len() >= self.config.max_pages {
            return false;
        }
        let number = self.pages.len() + 1;
        self.pages.push(AlbumPage::new(number, title));
        self.update_status();
        self.update_stats();
        true
    }

    /// Get page
    pub fn get_page(&self, number: usize) -> Option<&AlbumPage> {
        self.pages.iter().find(|p| p.number == number)
    }

    /// Get page mut
    pub fn get_page_mut(&mut self, number: usize) -> Option<&mut AlbumPage> {
        self.pages.iter_mut().find(|p| p.number == number)
    }

    /// Add item to page
    pub fn add_item(&mut self, page_number: usize, item: AlbumItem) -> bool {
        if let Some(page) = self.get_page_mut(page_number) {
            page.add(item);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update status
    fn update_status(&mut self) {
        self.status = if self.pages.is_empty() {
            AlbumStatus::Empty
        } else if self.pages.len() < self.config.max_pages {
            AlbumStatus::Partial
        } else {
            AlbumStatus::Complete
        };
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.pages, self.config.album_type);
    }

    /// Seal album
    pub fn seal(&mut self) {
        self.status = AlbumStatus::Sealed;
    }

    /// Get status
    pub fn status(&self) -> AlbumStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &AlbumStats {
        &self.stats
    }

    /// Page count
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

/// Album registry
#[derive(Debug, Clone, Default)]
pub struct AlbumRegistry {
    /// Albums by ID
    albums: HashMap<String, SettingsAlbum>,
}

impl AlbumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register album
    pub fn register(&mut self, id: impl Into<String>, album: SettingsAlbum) {
        self.albums.insert(id.into(), album);
    }

    /// Unregister album
    pub fn unregister(&mut self, id: &str) -> bool {
        self.albums.remove(id).is_some()
    }

    /// Get album
    pub fn get(&self, id: &str) -> Option<&SettingsAlbum> {
        self.albums.get(id)
    }

    /// Get album mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAlbum> {
        self.albums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.albums.len()
    }
}

/// Format album registry
pub fn format_album_registry(registry: &AlbumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Album Registry:\n");
    output.push_str(&format!("  Albums: {}\n", registry.count()));
    output
}

/// Check if query is about album
pub fn is_album_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings album") || lower.contains("album settings") || lower.contains("settings snapshot")
}

/// Fun fact about album
pub fn album_fun_fact() -> &'static str {
    "Anna's settings album preserves snapshots of your configuration history!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_album_type_display() {
        assert_eq!(format!("{}", AlbumType::Standard), "standard");
        assert_eq!(format!("{}", AlbumType::Snapshot), "snapshot");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AlbumStatus::Empty), "empty");
        assert_eq!(format!("{}", AlbumStatus::Sealed), "sealed");
    }

    #[test]
    fn test_config_new() {
        let c = AlbumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AlbumConfig::new("test")
            .album_type(AlbumType::Archive)
            .max_pages(25);
        assert_eq!(c.album_type, AlbumType::Archive);
        assert_eq!(c.max_pages, 25);
    }

    #[test]
    fn test_page_new() {
        let p = AlbumPage::new(1, "Page 1");
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_page_add() {
        let mut p = AlbumPage::new(1, "Page 1");
        p.add(AlbumItem::new("key", "value", "2025-12-15"));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_item_new() {
        let i = AlbumItem::new("key", "value", "2025-12-15");
        assert_eq!(i.key, "key");
    }

    #[test]
    fn test_item_label() {
        let i = AlbumItem::new("key", "value", "2025-12-15").label("important");
        assert!(i.label.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = AlbumStats::default();
        let pages = vec![AlbumPage::new(1, "Page")];
        s.update(&pages, AlbumType::Standard);
        assert_eq!(s.total_pages, 1);
    }

    #[test]
    fn test_album_new() {
        let a = SettingsAlbum::new(AlbumConfig::default());
        assert_eq!(a.page_count(), 0);
    }

    #[test]
    fn test_album_add_page() {
        let mut a = SettingsAlbum::new(AlbumConfig::default());
        a.add_page("Page 1");
        assert_eq!(a.page_count(), 1);
    }

    #[test]
    fn test_album_add_item() {
        let mut a = SettingsAlbum::new(AlbumConfig::default());
        a.add_page("Page 1");
        let added = a.add_item(1, AlbumItem::new("key", "value", "2025-12-15"));
        assert!(added);
    }

    #[test]
    fn test_album_seal() {
        let mut a = SettingsAlbum::new(AlbumConfig::default());
        a.seal();
        assert_eq!(a.status(), AlbumStatus::Sealed);
    }

    #[test]
    fn test_registry_new() {
        let r = AlbumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AlbumRegistry::new();
        r.register("a1", SettingsAlbum::new(AlbumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_album_query() {
        assert!(is_album_query("settings album"));
        assert!(!is_album_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = album_fun_fact();
        assert!(fact.contains("album"));
    }
}
