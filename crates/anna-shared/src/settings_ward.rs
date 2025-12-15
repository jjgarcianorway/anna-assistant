// v0.0.752: Settings Ward (Phase 328)
// Electoral ward for settings representation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ward type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WardType {
    /// Electoral ward
    #[default]
    Electoral,
    /// Hospital ward
    Hospital,
    /// Prison ward
    Prison,
    /// Administrative ward
    Administrative,
}

impl std::fmt::Display for WardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Electoral => write!(f, "electoral"),
            Self::Hospital => write!(f, "hospital"),
            Self::Prison => write!(f, "prison"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// Ward status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WardStatus {
    /// Created status
    #[default]
    Created,
    /// Active status
    Active,
    /// Redrawn status
    Redrawn,
    /// Abolished status
    Abolished,
}

impl std::fmt::Display for WardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Active => write!(f, "active"),
            Self::Redrawn => write!(f, "redrawn"),
            Self::Abolished => write!(f, "abolished"),
        }
    }
}

/// Ward config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardConfig {
    /// Name
    pub name: String,
    /// Ward type
    pub ward_type: WardType,
    /// Status
    pub status: WardStatus,
    /// Max motions
    pub max_motions: usize,
}

impl WardConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ward_type: WardType::Electoral,
            status: WardStatus::Created,
            max_motions: 100,
        }
    }

    /// Set type
    pub fn ward_type(mut self, wt: WardType) -> Self {
        self.ward_type = wt;
        self
    }

    /// Set status
    pub fn status(mut self, s: WardStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max motions
    pub fn max_motions(mut self, max: usize) -> Self {
        self.max_motions = max;
        self
    }
}

impl Default for WardConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Ward motion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardMotion {
    /// Motion ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Precinct number
    pub precinct: u32,
    /// Passed
    pub passed: bool,
}

impl WardMotion {
    /// Create new motion
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            precinct: 0,
            passed: true,
        }
    }

    /// Set precinct
    pub fn precinct(mut self, p: u32) -> Self {
        self.precinct = p;
        self
    }

    /// Make passed
    pub fn make_passed(&mut self) {
        self.passed = true;
    }

    /// Make failed
    pub fn make_failed(&mut self) {
        self.passed = false;
    }
}

/// Ward delegate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardDelegate {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Motion ID
    pub motion_id: String,
}

impl WardDelegate {
    /// Create new delegate
    pub fn new(key: impl Into<String>, name: impl Into<String>, motion_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            motion_id: motion_id.into(),
        }
    }
}

/// Ward stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WardStats {
    /// Total motions
    pub total_motions: usize,
    /// Passed motions
    pub passed: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl WardStats {
    /// Update from motions
    pub fn update(&mut self, motions: &[WardMotion], ward_type: WardType) {
        self.total_motions = motions.len();
        self.passed = motions.iter().filter(|m| m.passed).count();
        *self.by_type.entry(ward_type.to_string()).or_insert(0) += 1;
    }

    /// Passed rate
    pub fn passed_rate(&self) -> f64 {
        if self.total_motions == 0 { 0.0 } else { self.passed as f64 / self.total_motions as f64 * 100.0 }
    }
}

/// Settings ward
#[derive(Debug, Clone, Default)]
pub struct SettingsWard {
    /// Config
    config: WardConfig,
    /// Motions
    motions: Vec<WardMotion>,
    /// Delegates
    delegates: Vec<WardDelegate>,
    /// Stats
    stats: WardStats,
}

impl SettingsWard {
    /// Create new ward system
    pub fn new(config: WardConfig) -> Self {
        Self {
            config,
            motions: Vec::new(),
            delegates: Vec::new(),
            stats: WardStats::default(),
        }
    }

    /// Add motion
    pub fn add_motion(&mut self, motion: WardMotion) -> bool {
        if self.motions.len() >= self.config.max_motions {
            return false;
        }
        self.motions.push(motion);
        self.update_stats();
        true
    }

    /// Get motion
    pub fn get_motion(&self, id: &str) -> Option<&WardMotion> {
        self.motions.iter().find(|m| m.id == id)
    }

    /// Get motion mut
    pub fn get_motion_mut(&mut self, id: &str) -> Option<&mut WardMotion> {
        self.motions.iter_mut().find(|m| m.id == id)
    }

    /// Add delegate
    pub fn add_delegate(&mut self, delegate: WardDelegate) {
        self.delegates.push(delegate);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.motions, self.config.ward_type);
    }

    /// Get stats
    pub fn stats(&self) -> &WardStats {
        &self.stats
    }

    /// Motion count
    pub fn motion_count(&self) -> usize {
        self.motions.len()
    }
}

/// Ward registry
#[derive(Debug, Clone, Default)]
pub struct WardRegistry {
    /// Wards by ID
    wards: HashMap<String, SettingsWard>,
}

impl WardRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register ward
    pub fn register(&mut self, id: impl Into<String>, ward: SettingsWard) {
        self.wards.insert(id.into(), ward);
    }

    /// Unregister ward
    pub fn unregister(&mut self, id: &str) -> bool {
        self.wards.remove(id).is_some()
    }

    /// Get ward
    pub fn get(&self, id: &str) -> Option<&SettingsWard> {
        self.wards.get(id)
    }

    /// Get ward mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsWard> {
        self.wards.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.wards.len()
    }
}

/// Format ward registry
pub fn format_ward_registry(registry: &WardRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Ward Registry:\n");
    output.push_str(&format!("  Wards: {}\n", registry.count()));
    output
}

/// Check if query is about ward
pub fn is_ward_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings ward") || lower.contains("ward settings") || lower.contains("electoral ward")
}

/// Fun fact about ward
pub fn ward_fun_fact() -> &'static str {
    "Anna's settings ward establishes electoral representation!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ward_type_display() {
        assert_eq!(format!("{}", WardType::Electoral), "electoral");
        assert_eq!(format!("{}", WardType::Hospital), "hospital");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", WardStatus::Created), "created");
        assert_eq!(format!("{}", WardStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = WardConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = WardConfig::new("test")
            .ward_type(WardType::Administrative)
            .status(WardStatus::Redrawn);
        assert_eq!(c.ward_type, WardType::Administrative);
        assert_eq!(c.status, WardStatus::Redrawn);
    }

    #[test]
    fn test_motion_new() {
        let m = WardMotion::new("m1", "Title", "Content");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_motion_builder() {
        let m = WardMotion::new("m1", "Title", "Content")
            .precinct(1);
        assert_eq!(m.precinct, 1);
    }

    #[test]
    fn test_motion_passed() {
        let mut m = WardMotion::new("m1", "Title", "Content");
        m.make_failed();
        assert!(!m.passed);
        m.make_passed();
        assert!(m.passed);
    }

    #[test]
    fn test_delegate_new() {
        let d = WardDelegate::new("key", "name", "m1");
        assert_eq!(d.motion_id, "m1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = WardStats::default();
        let motion = WardMotion::new("m1", "Title", "Content");
        s.update(&[motion], WardType::Electoral);
        assert_eq!(s.total_motions, 1);
        assert_eq!(s.passed, 1);
    }

    #[test]
    fn test_ward_new() {
        let w = SettingsWard::new(WardConfig::default());
        assert_eq!(w.motion_count(), 0);
    }

    #[test]
    fn test_ward_add_motion() {
        let mut w = SettingsWard::new(WardConfig::default());
        w.add_motion(WardMotion::new("m1", "Title", "Content"));
        assert_eq!(w.motion_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = WardRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = WardRegistry::new();
        r.register("w1", SettingsWard::new(WardConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_ward_query() {
        assert!(is_ward_query("settings ward"));
        assert!(!is_ward_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = ward_fun_fact();
        assert!(fact.contains("ward"));
    }
}
