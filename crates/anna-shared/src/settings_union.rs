// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Union type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UnionType {
    /// Full union
    #[default]
    Full,
    /// Customs union
    Customs,
    /// Monetary union
    Monetary,
    /// Personal union
    Personal,
}

impl std::fmt::Display for UnionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Customs => write!(f, "customs"),
            Self::Monetary => write!(f, "monetary"),
            Self::Personal => write!(f, "personal"),
        }
    }
}

/// Union status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UnionStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Ratified status
    Ratified,
    /// Integrated status
    Integrated,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for UnionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Ratified => write!(f, "ratified"),
            Self::Integrated => write!(f, "integrated"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}

/// Union config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionConfig {
    /// Name
    pub name: String,
    /// Union type
    pub union_type: UnionType,
    /// Status
    pub status: UnionStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl UnionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            union_type: UnionType::Full,
            status: UnionStatus::Proposed,
            max_provisions: 100,
        }
    }

    /// Set type
    pub fn union_type(mut self, ut: UnionType) -> Self {
        self.union_type = ut;
        self
    }

    /// Set status
    pub fn status(mut self, s: UnionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for UnionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Union provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Binding
    pub binding: bool,
}

impl UnionProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            binding: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make binding
    pub fn make_binding(&mut self) {
        self.binding = true;
    }

    /// Make advisory
    pub fn make_advisory(&mut self) {
        self.binding = false;
    }
}

/// Union member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Provision ID
    pub provision_id: String,
}

impl UnionMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Union stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnionStats {
    /// Total provisions
    pub total_provisions: usize,
    /// Binding provisions
    pub binding: usize,
    /// Integrated count
    pub integrated_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl UnionStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[UnionProvision], union_type: UnionType) {
        self.total_provisions = provisions.len();
        self.binding = provisions.iter().filter(|p| p.binding).count();
        *self.by_type.entry(union_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.binding as f64 / self.total_provisions as f64 * 100.0 }
    }
}

/// Settings union
#[derive(Debug, Clone, Default)]
pub struct SettingsUnion {
    /// Config
    config: UnionConfig,
    /// Provisions
    provisions: Vec<UnionProvision>,
    /// Members
    members: Vec<UnionMember>,
    /// Stats
    stats: UnionStats,
}

impl SettingsUnion {
    /// Create new union system
    pub fn new(config: UnionConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            members: Vec::new(),
            stats: UnionStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: UnionProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&UnionProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut UnionProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: UnionMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.union_type);
    }

    /// Get stats
    pub fn stats(&self) -> &UnionStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}

/// Union registry
#[derive(Debug, Clone, Default)]
pub struct UnionRegistry {
    /// Unions by ID
    unions: HashMap<String, SettingsUnion>,
}

impl UnionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register union
    pub fn register(&mut self, id: impl Into<String>, union: SettingsUnion) {
        self.unions.insert(id.into(), union);
    }

    /// Unregister union
    pub fn unregister(&mut self, id: &str) -> bool {
        self.unions.remove(id).is_some()
    }

    /// Get union
    pub fn get(&self, id: &str) -> Option<&SettingsUnion> {
        self.unions.get(id)
    }

    /// Get union mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsUnion> {
        self.unions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.unions.len()
    }
}

/// Format union registry
pub fn format_union_registry(registry: &UnionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Union Registry:\n");
    output.push_str(&format!("  Unions: {}\n", registry.count()));
    output
}

/// Check if query is about union
pub fn is_union_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings union") || lower.contains("union settings") || lower.contains("political union")
}

/// Fun fact about union
pub fn union_fun_fact() -> &'static str {
    "Anna's settings union establishes political integration!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_type_display() {
        assert_eq!(format!("{}", UnionType::Full), "full");
        assert_eq!(format!("{}", UnionType::Customs), "customs");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", UnionStatus::Proposed), "proposed");
        assert_eq!(format!("{}", UnionStatus::Integrated), "integrated");
    }

    #[test]
    fn test_config_new() {
        let c = UnionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = UnionConfig::new("test")
            .union_type(UnionType::Monetary)
            .status(UnionStatus::Ratified);
        assert_eq!(c.union_type, UnionType::Monetary);
        assert_eq!(c.status, UnionStatus::Ratified);
    }

    #[test]
    fn test_provision_new() {
        let p = UnionProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = UnionProvision::new("p1", "Title", "Content")
            .section(1);
        assert_eq!(p.section, 1);
    }

    #[test]
    fn test_provision_binding() {
        let mut p = UnionProvision::new("p1", "Title", "Content");
        p.make_advisory();
        assert!(!p.binding);
        p.make_binding();
        assert!(p.binding);
    }

    #[test]
    fn test_member_new() {
        let m = UnionMember::new("key", "name", "p1");
        assert_eq!(m.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = UnionStats::default();
        let provision = UnionProvision::new("p1", "Title", "Content");
        s.update(&[provision], UnionType::Full);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.binding, 1);
    }

    #[test]
    fn test_union_new() {
        let u = SettingsUnion::new(UnionConfig::default());
        assert_eq!(u.provision_count(), 0);
    }

    #[test]
    fn test_union_add_provision() {
        let mut u = SettingsUnion::new(UnionConfig::default());
        u.add_provision(UnionProvision::new("p1", "Title", "Content"));
        assert_eq!(u.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = UnionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = UnionRegistry::new();
        r.register("u1", SettingsUnion::new(UnionConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_union_query() {
        assert!(is_union_query("settings union"));
        assert!(!is_union_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = union_fun_fact();
        assert!(fact.contains("union"));
    }
}
