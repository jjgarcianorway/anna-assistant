// v0.0.787: Settings Enclave (Phase 363)
// Exclusive enclave for settings community

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enclave type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EnclaveType {
    /// Exclusive enclave
    #[default]
    Exclusive,
    /// Private enclave
    Private,
    /// Gated enclave
    Gated,
    /// Elite enclave
    Elite,
}

impl std::fmt::Display for EnclaveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exclusive => write!(f, "exclusive"),
            Self::Private => write!(f, "private"),
            Self::Gated => write!(f, "gated"),
            Self::Elite => write!(f, "elite"),
        }
    }
}

/// Enclave status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EnclaveStatus {
    /// Active status
    #[default]
    Active,
    /// Secured status
    Secured,
    /// Restricted status
    Restricted,
    /// Protected status
    Protected,
}

impl std::fmt::Display for EnclaveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Secured => write!(f, "secured"),
            Self::Restricted => write!(f, "restricted"),
            Self::Protected => write!(f, "protected"),
        }
    }
}

/// Enclave config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveConfig {
    /// Name
    pub name: String,
    /// Enclave type
    pub enclave_type: EnclaveType,
    /// Status
    pub status: EnclaveStatus,
    /// Max members
    pub max_members: usize,
}

impl EnclaveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enclave_type: EnclaveType::Exclusive,
            status: EnclaveStatus::Active,
            max_members: 100,
        }
    }

    /// Set type
    pub fn enclave_type(mut self, et: EnclaveType) -> Self {
        self.enclave_type = et;
        self
    }

    /// Set status
    pub fn status(mut self, s: EnclaveStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max members
    pub fn max_members(mut self, max: usize) -> Self {
        self.max_members = max;
        self
    }
}

impl Default for EnclaveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Enclave member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveMember {
    /// Member ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Quarter number
    pub quarter: u32,
    /// Admitted
    pub admitted: bool,
}

impl EnclaveMember {
    /// Create new member
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            quarter: 0,
            admitted: true,
        }
    }

    /// Set quarter
    pub fn quarter(mut self, q: u32) -> Self {
        self.quarter = q;
        self
    }

    /// Make admitted
    pub fn make_admitted(&mut self) {
        self.admitted = true;
    }

    /// Make pending
    pub fn make_pending(&mut self) {
        self.admitted = false;
    }
}

/// Enclave steward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveSteward {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Member ID
    pub member_id: String,
}

impl EnclaveSteward {
    /// Create new steward
    pub fn new(key: impl Into<String>, name: impl Into<String>, member_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            member_id: member_id.into(),
        }
    }
}

/// Enclave stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnclaveStats {
    /// Total members
    pub total_members: usize,
    /// Admitted members
    pub admitted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl EnclaveStats {
    /// Update from members
    pub fn update(&mut self, members: &[EnclaveMember], enclave_type: EnclaveType) {
        self.total_members = members.len();
        self.admitted = members.iter().filter(|m| m.admitted).count();
        *self.by_type.entry(enclave_type.to_string()).or_insert(0) += 1;
    }

    /// Admission rate
    pub fn admission_rate(&self) -> f64 {
        if self.total_members == 0 { 0.0 } else { self.admitted as f64 / self.total_members as f64 * 100.0 }
    }
}

/// Settings enclave
#[derive(Debug, Clone, Default)]
pub struct SettingsEnclave {
    /// Config
    config: EnclaveConfig,
    /// Members
    members: Vec<EnclaveMember>,
    /// Stewards
    stewards: Vec<EnclaveSteward>,
    /// Stats
    stats: EnclaveStats,
}

impl SettingsEnclave {
    /// Create new enclave system
    pub fn new(config: EnclaveConfig) -> Self {
        Self {
            config,
            members: Vec::new(),
            stewards: Vec::new(),
            stats: EnclaveStats::default(),
        }
    }

    /// Add member
    pub fn add_member(&mut self, member: EnclaveMember) -> bool {
        if self.members.len() >= self.config.max_members {
            return false;
        }
        self.members.push(member);
        self.update_stats();
        true
    }

    /// Get member
    pub fn get_member(&self, id: &str) -> Option<&EnclaveMember> {
        self.members.iter().find(|m| m.id == id)
    }

    /// Get member mut
    pub fn get_member_mut(&mut self, id: &str) -> Option<&mut EnclaveMember> {
        self.members.iter_mut().find(|m| m.id == id)
    }

    /// Add steward
    pub fn add_steward(&mut self, steward: EnclaveSteward) {
        self.stewards.push(steward);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.members, self.config.enclave_type);
    }

    /// Get stats
    pub fn stats(&self) -> &EnclaveStats {
        &self.stats
    }

    /// Member count
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

/// Enclave registry
#[derive(Debug, Clone, Default)]
pub struct EnclaveRegistry {
    /// Enclaves by ID
    enclaves: HashMap<String, SettingsEnclave>,
}

impl EnclaveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register enclave
    pub fn register(&mut self, id: impl Into<String>, enclave: SettingsEnclave) {
        self.enclaves.insert(id.into(), enclave);
    }

    /// Unregister enclave
    pub fn unregister(&mut self, id: &str) -> bool {
        self.enclaves.remove(id).is_some()
    }

    /// Get enclave
    pub fn get(&self, id: &str) -> Option<&SettingsEnclave> {
        self.enclaves.get(id)
    }

    /// Get enclave mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEnclave> {
        self.enclaves.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.enclaves.len()
    }
}

/// Format enclave registry
pub fn format_enclave_registry(registry: &EnclaveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Enclave Registry:\n");
    output.push_str(&format!("  Enclaves: {}\n", registry.count()));
    output
}

/// Check if query is about enclave
pub fn is_enclave_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings enclave") || lower.contains("enclave settings") || lower.contains("exclusive enclave")
}

/// Fun fact about enclave
pub fn enclave_fun_fact() -> &'static str {
    "Anna's settings enclave hosts an exclusive community of configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enclave_type_display() {
        assert_eq!(format!("{}", EnclaveType::Exclusive), "exclusive");
        assert_eq!(format!("{}", EnclaveType::Private), "private");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", EnclaveStatus::Active), "active");
        assert_eq!(format!("{}", EnclaveStatus::Protected), "protected");
    }

    #[test]
    fn test_config_new() {
        let c = EnclaveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = EnclaveConfig::new("test")
            .enclave_type(EnclaveType::Gated)
            .status(EnclaveStatus::Secured);
        assert_eq!(c.enclave_type, EnclaveType::Gated);
        assert_eq!(c.status, EnclaveStatus::Secured);
    }

    #[test]
    fn test_member_new() {
        let m = EnclaveMember::new("m1", "Title", "Content");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_member_builder() {
        let m = EnclaveMember::new("m1", "Title", "Content")
            .quarter(1);
        assert_eq!(m.quarter, 1);
    }

    #[test]
    fn test_member_admission() {
        let mut m = EnclaveMember::new("m1", "Title", "Content");
        m.make_pending();
        assert!(!m.admitted);
        m.make_admitted();
        assert!(m.admitted);
    }

    #[test]
    fn test_steward_new() {
        let s = EnclaveSteward::new("key", "name", "m1");
        assert_eq!(s.member_id, "m1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = EnclaveStats::default();
        let member = EnclaveMember::new("m1", "Title", "Content");
        s.update(&[member], EnclaveType::Exclusive);
        assert_eq!(s.total_members, 1);
        assert_eq!(s.admitted, 1);
    }

    #[test]
    fn test_enclave_new() {
        let e = SettingsEnclave::new(EnclaveConfig::default());
        assert_eq!(e.member_count(), 0);
    }

    #[test]
    fn test_enclave_add_member() {
        let mut e = SettingsEnclave::new(EnclaveConfig::default());
        e.add_member(EnclaveMember::new("m1", "Title", "Content"));
        assert_eq!(e.member_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = EnclaveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = EnclaveRegistry::new();
        r.register("e1", SettingsEnclave::new(EnclaveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_enclave_query() {
        assert!(is_enclave_query("settings enclave"));
        assert!(!is_enclave_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = enclave_fun_fact();
        assert!(fact.contains("enclave"));
    }
}
