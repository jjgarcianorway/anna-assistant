// v0.0.660: Settings Versioner (Phase 236)
// Versioner for tracking settings configuration versions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Version scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VersionScheme {
    /// Semantic versioning (major.minor.patch)
    #[default]
    Semantic,
    /// Sequential numbering
    Sequential,
    /// Date-based
    DateBased,
    /// Hash-based
    HashBased,
}

impl std::fmt::Display for VersionScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semantic => write!(f, "semantic"),
            Self::Sequential => write!(f, "sequential"),
            Self::DateBased => write!(f, "date_based"),
            Self::HashBased => write!(f, "hash_based"),
        }
    }
}

/// Version bump type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BumpType {
    /// Major version bump
    Major,
    /// Minor version bump
    #[default]
    Minor,
    /// Patch version bump
    Patch,
    /// Auto-detect based on changes
    Auto,
}

impl std::fmt::Display for BumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

/// Versioner config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionerConfig {
    /// Version scheme
    pub scheme: VersionScheme,
    /// Default bump type
    pub default_bump: BumpType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Track history
    pub track_history: bool,
    /// Max history entries
    pub max_history: usize,
}

impl VersionerConfig {
    /// Create new config
    pub fn new(scheme: VersionScheme) -> Self {
        Self {
            scheme,
            default_bump: BumpType::Minor,
            category: None,
            track_history: true,
            max_history: 100,
        }
    }

    /// Set default bump
    pub fn default_bump(mut self, bump: BumpType) -> Self {
        self.default_bump = bump;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set track history
    pub fn track_history(mut self, track: bool) -> Self {
        self.track_history = track;
        self
    }

    /// Set max history
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }
}

impl Default for VersionerConfig {
    fn default() -> Self {
        Self::new(VersionScheme::Semantic)
    }
}

/// Settings version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsVersion {
    /// Version string
    pub version: String,
    /// Major component
    pub major: u32,
    /// Minor component
    pub minor: u32,
    /// Patch component
    pub patch: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Description
    pub description: Option<String>,
}

impl SettingsVersion {
    /// Create new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            version: format!("{}.{}.{}", major, minor, patch),
            major,
            minor,
            patch,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            description: None,
        }
    }

    /// From string
    pub fn from_string(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 3 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            let patch = parts[2].parse().ok()?;
            Some(Self::new(major, minor, patch))
        } else {
            None
        }
    }

    /// With description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Bump version
    pub fn bump(&self, bump_type: BumpType) -> Self {
        let (major, minor, patch) = match bump_type {
            BumpType::Major => (self.major + 1, 0, 0),
            BumpType::Minor => (self.major, self.minor + 1, 0),
            BumpType::Patch => (self.major, self.minor, self.patch + 1),
            BumpType::Auto => (self.major, self.minor, self.patch + 1),
        };
        Self::new(major, minor, patch)
    }
}

impl Default for SettingsVersion {
    fn default() -> Self {
        Self::new(0, 0, 1)
    }
}

/// Version result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResult {
    /// Previous version
    pub previous: Option<SettingsVersion>,
    /// New version
    pub current: SettingsVersion,
    /// Bump type applied
    pub bump_type: BumpType,
    /// Changes count
    pub changes_count: usize,
}

impl VersionResult {
    /// Create new result
    pub fn new(current: SettingsVersion, bump_type: BumpType) -> Self {
        Self {
            previous: None,
            current,
            bump_type,
            changes_count: 0,
        }
    }

    /// With previous
    pub fn with_previous(mut self, prev: SettingsVersion) -> Self {
        self.previous = Some(prev);
        self
    }

    /// With changes count
    pub fn with_changes(mut self, count: usize) -> Self {
        self.changes_count = count;
        self
    }

    /// Was bumped
    pub fn was_bumped(&self) -> bool {
        self.previous.is_some()
    }
}

/// Versioner stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionerStats {
    /// Total versions created
    pub total_versions: usize,
    /// Total bumps
    pub total_bumps: usize,
    /// By bump type
    pub by_bump_type: HashMap<String, usize>,
    /// Current version
    pub current_version: Option<String>,
}

impl VersionerStats {
    /// Record version
    pub fn record(&mut self, bump_type: BumpType, version: &str) {
        self.total_versions += 1;
        self.total_bumps += 1;
        self.current_version = Some(version.to_string());
        *self.by_bump_type.entry(bump_type.to_string()).or_insert(0) += 1;
    }
}

/// Settings versioner
#[derive(Debug, Clone, Default)]
pub struct SettingsVersioner {
    /// Config
    config: VersionerConfig,
    /// Current version
    current: SettingsVersion,
    /// History
    history: Vec<SettingsVersion>,
    /// Stats
    stats: VersionerStats,
}

impl SettingsVersioner {
    /// Create new versioner
    pub fn new(config: VersionerConfig) -> Self {
        Self {
            config,
            current: SettingsVersion::default(),
            history: Vec::new(),
            stats: VersionerStats::default(),
        }
    }

    /// Get current version
    pub fn current(&self) -> &SettingsVersion {
        &self.current
    }

    /// Bump version
    pub fn bump(&mut self, bump_type: BumpType) -> VersionResult {
        let previous = self.current.clone();
        let new_version = self.current.bump(bump_type);

        if self.config.track_history {
            self.history.push(previous.clone());
            if self.history.len() > self.config.max_history {
                self.history.remove(0);
            }
        }

        self.current = new_version.clone();
        self.stats.record(bump_type, &self.current.version);

        VersionResult::new(new_version, bump_type).with_previous(previous)
    }

    /// Bump with description
    pub fn bump_with_description(&mut self, bump_type: BumpType, description: &str) -> VersionResult {
        let mut result = self.bump(bump_type);
        self.current.description = Some(description.to_string());
        result.current.description = Some(description.to_string());
        result
    }

    /// Set version
    pub fn set_version(&mut self, version: SettingsVersion) {
        if self.config.track_history {
            self.history.push(self.current.clone());
        }
        self.current = version;
    }

    /// Get history
    pub fn history(&self) -> &[SettingsVersion] {
        &self.history
    }

    /// Get stats
    pub fn stats(&self) -> &VersionerStats {
        &self.stats
    }

    /// History count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Settings versioner registry
#[derive(Debug, Clone, Default)]
pub struct SettingsVersionerRegistry {
    /// Versioners by ID
    versioners: HashMap<String, SettingsVersioner>,
}

impl SettingsVersionerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register versioner
    pub fn register(&mut self, id: impl Into<String>, versioner: SettingsVersioner) {
        self.versioners.insert(id.into(), versioner);
    }

    /// Unregister versioner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.versioners.remove(id).is_some()
    }

    /// Get versioner
    pub fn get(&self, id: &str) -> Option<&SettingsVersioner> {
        self.versioners.get(id)
    }

    /// Get versioner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsVersioner> {
        self.versioners.get_mut(id)
    }

    /// Versioner count
    pub fn count(&self) -> usize {
        self.versioners.len()
    }
}

/// Format versioner registry
pub fn format_versioner_registry(registry: &SettingsVersionerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Versioner Registry:\n");
    output.push_str(&format!("  Versioners: {}\n", registry.count()));
    output
}

/// Check if query is about versioner
pub fn is_versioner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("versioner") || lower.contains("version settings") || lower.contains("settings version")
}

/// Fun fact about versioner
pub fn versioner_fun_fact() -> &'static str {
    "Anna's settings versioners track every config change!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_scheme_display() {
        assert_eq!(format!("{}", VersionScheme::Semantic), "semantic");
        assert_eq!(format!("{}", VersionScheme::Sequential), "sequential");
    }

    #[test]
    fn test_bump_type_display() {
        assert_eq!(format!("{}", BumpType::Major), "major");
        assert_eq!(format!("{}", BumpType::Minor), "minor");
    }

    #[test]
    fn test_config_new() {
        let c = VersionerConfig::new(VersionScheme::Semantic);
        assert!(c.track_history);
    }

    #[test]
    fn test_config_builder() {
        let c = VersionerConfig::new(VersionScheme::DateBased)
            .default_bump(BumpType::Patch)
            .max_history(50);
        assert_eq!(c.default_bump, BumpType::Patch);
        assert_eq!(c.max_history, 50);
    }

    #[test]
    fn test_version_new() {
        let v = SettingsVersion::new(1, 2, 3);
        assert_eq!(v.version, "1.2.3");
    }

    #[test]
    fn test_version_from_string() {
        let v = SettingsVersion::from_string("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_bump() {
        let v = SettingsVersion::new(1, 0, 0);
        let bumped = v.bump(BumpType::Minor);
        assert_eq!(bumped.version, "1.1.0");
    }

    #[test]
    fn test_result_new() {
        let r = VersionResult::new(SettingsVersion::new(1, 0, 0), BumpType::Minor);
        assert!(!r.was_bumped());
    }

    #[test]
    fn test_result_with_previous() {
        let r = VersionResult::new(SettingsVersion::new(1, 1, 0), BumpType::Minor)
            .with_previous(SettingsVersion::new(1, 0, 0));
        assert!(r.was_bumped());
    }

    #[test]
    fn test_stats_record() {
        let mut s = VersionerStats::default();
        s.record(BumpType::Minor, "1.1.0");
        assert_eq!(s.total_bumps, 1);
        assert_eq!(s.current_version, Some("1.1.0".to_string()));
    }

    #[test]
    fn test_versioner_new() {
        let v = SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic));
        assert_eq!(v.current().version, "0.0.1");
    }

    #[test]
    fn test_versioner_bump() {
        let mut v = SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic));
        let r = v.bump(BumpType::Minor);
        assert_eq!(r.current.version, "0.1.0");
        assert!(r.was_bumped());
    }

    #[test]
    fn test_versioner_history() {
        let mut v = SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic));
        v.bump(BumpType::Minor);
        v.bump(BumpType::Minor);
        assert_eq!(v.history_count(), 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsVersionerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsVersionerRegistry::new();
        r.register("v1", SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_versioner_query() {
        assert!(is_versioner_query("settings versioner"));
        assert!(!is_versioner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = versioner_fun_fact();
        assert!(fact.contains("versioner"));
    }
}
