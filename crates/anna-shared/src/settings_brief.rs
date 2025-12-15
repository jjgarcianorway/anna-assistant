// v0.0.710: Settings Brief (Phase 286)
// Executive briefs for settings overview

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Brief type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BriefType {
    /// Executive brief
    #[default]
    Executive,
    /// Technical brief
    Technical,
    /// Operational brief
    Operational,
    /// Strategic brief
    Strategic,
}

impl std::fmt::Display for BriefType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executive => write!(f, "executive"),
            Self::Technical => write!(f, "technical"),
            Self::Operational => write!(f, "operational"),
            Self::Strategic => write!(f, "strategic"),
        }
    }
}

/// Brief scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BriefScope {
    /// Department scope
    #[default]
    Department,
    /// Organization scope
    Organization,
    /// Project scope
    Project,
    /// System scope
    System,
}

impl std::fmt::Display for BriefScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Department => write!(f, "department"),
            Self::Organization => write!(f, "organization"),
            Self::Project => write!(f, "project"),
            Self::System => write!(f, "system"),
        }
    }
}

/// Brief config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefConfig {
    /// Name
    pub name: String,
    /// Brief type
    pub brief_type: BriefType,
    /// Scope
    pub scope: BriefScope,
    /// Max points
    pub max_points: usize,
}

impl BriefConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            brief_type: BriefType::Executive,
            scope: BriefScope::Department,
            max_points: 25,
        }
    }

    /// Set type
    pub fn brief_type(mut self, bt: BriefType) -> Self {
        self.brief_type = bt;
        self
    }

    /// Set scope
    pub fn scope(mut self, s: BriefScope) -> Self {
        self.scope = s;
        self
    }

    /// Set max points
    pub fn max_points(mut self, max: usize) -> Self {
        self.max_points = max;
        self
    }
}

impl Default for BriefConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Brief point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefPoint {
    /// Point ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Priority
    pub priority: u8,
    /// Action required
    pub action_required: bool,
}

impl BriefPoint {
    /// Create new point
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            priority: 1,
            action_required: false,
        }
    }

    /// Set description
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    /// Set priority
    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    /// Set action required
    pub fn action_required(mut self, ar: bool) -> Self {
        self.action_required = ar;
        self
    }
}

/// Brief attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefAttachment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Point ID
    pub point_id: String,
}

impl BriefAttachment {
    /// Create new attachment
    pub fn new(key: impl Into<String>, value: impl Into<String>, point_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            point_id: point_id.into(),
        }
    }
}

/// Brief stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BriefStats {
    /// Total points
    pub total_points: usize,
    /// Action items
    pub action_items: usize,
    /// High priority
    pub high_priority: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BriefStats {
    /// Update from points
    pub fn update(&mut self, points: &[BriefPoint], brief_type: BriefType) {
        self.total_points = points.len();
        self.action_items = points.iter().filter(|p| p.action_required).count();
        self.high_priority = points.iter().filter(|p| p.priority >= 3).count();
        *self.by_type.entry(brief_type.to_string()).or_insert(0) += 1;
    }

    /// Action rate
    pub fn action_rate(&self) -> f64 {
        if self.total_points == 0 { 0.0 } else { self.action_items as f64 / self.total_points as f64 * 100.0 }
    }
}

/// Settings brief
#[derive(Debug, Clone, Default)]
pub struct SettingsBrief {
    /// Config
    config: BriefConfig,
    /// Points
    points: Vec<BriefPoint>,
    /// Attachments
    attachments: Vec<BriefAttachment>,
    /// Stats
    stats: BriefStats,
}

impl SettingsBrief {
    /// Create new brief
    pub fn new(config: BriefConfig) -> Self {
        Self {
            config,
            points: Vec::new(),
            attachments: Vec::new(),
            stats: BriefStats::default(),
        }
    }

    /// Add point
    pub fn add_point(&mut self, point: BriefPoint) -> bool {
        if self.points.len() >= self.config.max_points {
            return false;
        }
        self.points.push(point);
        self.update_stats();
        true
    }

    /// Get point
    pub fn get_point(&self, id: &str) -> Option<&BriefPoint> {
        self.points.iter().find(|p| p.id == id)
    }

    /// Get point mut
    pub fn get_point_mut(&mut self, id: &str) -> Option<&mut BriefPoint> {
        self.points.iter_mut().find(|p| p.id == id)
    }

    /// Add attachment
    pub fn add_attachment(&mut self, attachment: BriefAttachment) {
        self.attachments.push(attachment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.points, self.config.brief_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BriefStats {
        &self.stats
    }

    /// Point count
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}

/// Brief registry
#[derive(Debug, Clone, Default)]
pub struct BriefRegistry {
    /// Briefs by ID
    briefs: HashMap<String, SettingsBrief>,
}

impl BriefRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register brief
    pub fn register(&mut self, id: impl Into<String>, brief: SettingsBrief) {
        self.briefs.insert(id.into(), brief);
    }

    /// Unregister brief
    pub fn unregister(&mut self, id: &str) -> bool {
        self.briefs.remove(id).is_some()
    }

    /// Get brief
    pub fn get(&self, id: &str) -> Option<&SettingsBrief> {
        self.briefs.get(id)
    }

    /// Get brief mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBrief> {
        self.briefs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.briefs.len()
    }
}

/// Format brief registry
pub fn format_brief_registry(registry: &BriefRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Brief Registry:\n");
    output.push_str(&format!("  Briefs: {}\n", registry.count()));
    output
}

/// Check if query is about brief
pub fn is_brief_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings brief") || lower.contains("brief settings") || lower.contains("executive brief")
}

/// Fun fact about brief
pub fn brief_fun_fact() -> &'static str {
    "Anna's settings brief provides executive-level overviews of configuration states!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brief_type_display() {
        assert_eq!(format!("{}", BriefType::Executive), "executive");
        assert_eq!(format!("{}", BriefType::Technical), "technical");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", BriefScope::Department), "department");
        assert_eq!(format!("{}", BriefScope::Organization), "organization");
    }

    #[test]
    fn test_config_new() {
        let c = BriefConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BriefConfig::new("test")
            .brief_type(BriefType::Strategic)
            .scope(BriefScope::Organization);
        assert_eq!(c.brief_type, BriefType::Strategic);
        assert_eq!(c.scope, BriefScope::Organization);
    }

    #[test]
    fn test_point_new() {
        let p = BriefPoint::new("p1", "Point 1");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_point_builder() {
        let p = BriefPoint::new("p1", "Point 1")
            .priority(3)
            .action_required(true);
        assert_eq!(p.priority, 3);
        assert!(p.action_required);
    }

    #[test]
    fn test_attachment_new() {
        let a = BriefAttachment::new("key", "value", "p1");
        assert_eq!(a.point_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BriefStats::default();
        let point = BriefPoint::new("p1", "Point").action_required(true).priority(3);
        s.update(&[point], BriefType::Executive);
        assert_eq!(s.total_points, 1);
        assert_eq!(s.action_items, 1);
        assert_eq!(s.high_priority, 1);
    }

    #[test]
    fn test_brief_new() {
        let b = SettingsBrief::new(BriefConfig::default());
        assert_eq!(b.point_count(), 0);
    }

    #[test]
    fn test_brief_add_point() {
        let mut b = SettingsBrief::new(BriefConfig::default());
        b.add_point(BriefPoint::new("p1", "Point 1"));
        assert_eq!(b.point_count(), 1);
    }

    #[test]
    fn test_brief_add_attachment() {
        let mut b = SettingsBrief::new(BriefConfig::default());
        b.add_attachment(BriefAttachment::new("key", "value", "p1"));
        assert_eq!(b.attachments.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BriefRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BriefRegistry::new();
        r.register("b1", SettingsBrief::new(BriefConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_brief_query() {
        assert!(is_brief_query("settings brief"));
        assert!(!is_brief_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = brief_fun_fact();
        assert!(fact.contains("brief"));
    }
}
