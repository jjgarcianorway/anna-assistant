// v0.0.593: Settings Lock (Phase 169)
// Lock settings to prevent changes

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::unified_settings::SettingsCategory;

/// Lock type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockType {
    /// Read-only (no writes)
    ReadOnly,
    /// Full lock (no reads or writes)
    Full,
    /// Admin only
    AdminOnly,
    /// Temporary lock
    Temporary,
}

impl Default for LockType {
    fn default() -> Self {
        Self::ReadOnly
    }
}

impl std::fmt::Display for LockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "read_only"),
            Self::Full => write!(f, "full"),
            Self::AdminOnly => write!(f, "admin_only"),
            Self::Temporary => write!(f, "temporary"),
        }
    }
}

/// Lock scope
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockScope {
    /// Global lock
    Global,
    /// Category lock
    Category(SettingsCategory),
    /// Key lock
    Key(SettingsCategory, String),
}

impl std::fmt::Display for LockScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, "global"),
            Self::Category(c) => write!(f, "category:{}", c),
            Self::Key(c, k) => write!(f, "key:{}:{}", c, k),
        }
    }
}

/// Lock entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    /// Lock ID
    pub id: String,
    /// Lock type
    pub lock_type: LockType,
    /// Scope
    pub scope: LockScope,
    /// Owner
    pub owner: String,
    /// Reason
    pub reason: Option<String>,
    /// Created time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expires at
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Active
    pub active: bool,
}

impl LockEntry {
    /// Create new lock
    pub fn new(lock_type: LockType, scope: LockScope) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            lock_type,
            scope,
            owner: "system".to_string(),
            reason: None,
            created_at: chrono::Utc::now(),
            expires_at: None,
            active: true,
        }
    }

    /// Set owner
    pub fn owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = owner.into();
        self
    }

    /// Set reason
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Set expiration
    pub fn expires_in(mut self, seconds: i64) -> Self {
        self.expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(seconds));
        self
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }

    /// Check if effective
    pub fn is_effective(&self) -> bool {
        self.active && !self.is_expired()
    }

    /// Release lock
    pub fn release(&mut self) {
        self.active = false;
    }
}

/// Lock check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockCheckResult {
    /// Not locked
    Unlocked,
    /// Locked
    Locked(String),
    /// Admin required
    AdminRequired,
}

impl LockCheckResult {
    /// Check if locked
    pub fn is_locked(&self) -> bool {
        !matches!(self, Self::Unlocked)
    }
}

/// Settings lock manager
#[derive(Debug, Clone, Default)]
pub struct SettingsLockManager {
    /// Active locks
    locks: Vec<LockEntry>,
    /// Permanently locked categories
    permanent: HashSet<SettingsCategory>,
}

impl SettingsLockManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add lock
    pub fn lock(&mut self, entry: LockEntry) -> String {
        let id = entry.id.clone();
        self.locks.push(entry);
        id
    }

    /// Unlock by ID
    pub fn unlock(&mut self, id: &str) -> bool {
        if let Some(lock) = self.locks.iter_mut().find(|l| l.id == id) {
            lock.release();
            return true;
        }
        false
    }

    /// Remove lock by ID
    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.locks.len();
        self.locks.retain(|l| l.id != id);
        self.locks.len() < len
    }

    /// Check if locked
    pub fn check(&self, category: SettingsCategory, key: Option<&str>) -> LockCheckResult {
        // Check permanent locks first
        if self.permanent.contains(&category) {
            return LockCheckResult::Locked("permanently locked".to_string());
        }

        for lock in &self.locks {
            if !lock.is_effective() {
                continue;
            }

            let matches = match &lock.scope {
                LockScope::Global => true,
                LockScope::Category(c) => *c == category,
                LockScope::Key(c, k) => {
                    *c == category && key.map(|key| key == k).unwrap_or(false)
                }
            };

            if matches {
                return match lock.lock_type {
                    LockType::AdminOnly => LockCheckResult::AdminRequired,
                    _ => LockCheckResult::Locked(lock.reason.clone().unwrap_or_default()),
                };
            }
        }

        LockCheckResult::Unlocked
    }

    /// Lock category permanently
    pub fn lock_permanent(&mut self, category: SettingsCategory) {
        self.permanent.insert(category);
    }

    /// Unlock permanent
    pub fn unlock_permanent(&mut self, category: SettingsCategory) {
        self.permanent.remove(&category);
    }

    /// Get all locks
    pub fn all(&self) -> &[LockEntry] {
        &self.locks
    }

    /// Get active locks
    pub fn active(&self) -> Vec<&LockEntry> {
        self.locks.iter().filter(|l| l.is_effective()).collect()
    }

    /// Clean expired locks
    pub fn clean_expired(&mut self) -> usize {
        let len = self.locks.len();
        self.locks.retain(|l| !l.is_expired());
        len - self.locks.len()
    }

    /// Lock count
    pub fn count(&self) -> usize {
        self.locks.len()
    }

    /// Active lock count
    pub fn active_count(&self) -> usize {
        self.locks.iter().filter(|l| l.is_effective()).count()
    }

    /// Clear all locks
    pub fn clear(&mut self) {
        self.locks.clear();
        self.permanent.clear();
    }
}

/// Format locks
pub fn format_locks(manager: &SettingsLockManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Locks ===\n\n");
    output.push_str(&format!(
        "Total: {} ({} active)\n\n",
        manager.count(),
        manager.active_count()
    ));

    for lock in manager.active() {
        output.push_str(&format!(
            "{} [{}] - {}\n",
            lock.scope,
            lock.lock_type,
            lock.reason.as_deref().unwrap_or("no reason")
        ));
    }

    output
}

/// Check if query is about locks
pub fn is_lock_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("lock")
        || lower.contains("unlock")
        || lower.contains("protect")
        || lower.contains("freeze")
}

/// Fun fact about locks
pub fn settings_lock_fun_fact() -> &'static str {
    "Anna can lock settings to prevent accidental or unauthorized changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_type_display() {
        assert_eq!(format!("{}", LockType::ReadOnly), "read_only");
        assert_eq!(format!("{}", LockType::Full), "full");
    }

    #[test]
    fn test_lock_scope_display() {
        assert_eq!(format!("{}", LockScope::Global), "global");
        let cat = LockScope::Category(SettingsCategory::Personality);
        assert!(format!("{}", cat).contains("category"));
    }

    #[test]
    fn test_lock_entry_new() {
        let lock = LockEntry::new(LockType::ReadOnly, LockScope::Global);
        assert!(lock.active);
        assert!(lock.is_effective());
    }

    #[test]
    fn test_lock_entry_builder() {
        let lock = LockEntry::new(LockType::AdminOnly, LockScope::Global)
            .owner("admin")
            .reason("maintenance");
        assert_eq!(lock.owner, "admin");
        assert!(lock.reason.is_some());
    }

    #[test]
    fn test_lock_entry_release() {
        let mut lock = LockEntry::new(LockType::ReadOnly, LockScope::Global);
        lock.release();
        assert!(!lock.active);
        assert!(!lock.is_effective());
    }

    #[test]
    fn test_manager_new() {
        let manager = SettingsLockManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_manager_lock_unlock() {
        let mut manager = SettingsLockManager::new();
        let id = manager.lock(LockEntry::new(LockType::ReadOnly, LockScope::Global));
        assert_eq!(manager.count(), 1);
        assert!(manager.unlock(&id));
    }

    #[test]
    fn test_manager_check_unlocked() {
        let manager = SettingsLockManager::new();
        let result = manager.check(SettingsCategory::Personality, None);
        assert!(!result.is_locked());
    }

    #[test]
    fn test_manager_check_locked() {
        let mut manager = SettingsLockManager::new();
        manager.lock(LockEntry::new(LockType::ReadOnly, LockScope::Global));
        let result = manager.check(SettingsCategory::Personality, None);
        assert!(result.is_locked());
    }

    #[test]
    fn test_manager_permanent() {
        let mut manager = SettingsLockManager::new();
        manager.lock_permanent(SettingsCategory::Risk);
        let result = manager.check(SettingsCategory::Risk, None);
        assert!(result.is_locked());
    }

    #[test]
    fn test_format_locks() {
        let manager = SettingsLockManager::new();
        let output = format_locks(&manager);
        assert!(output.contains("Locks"));
    }

    #[test]
    fn test_is_lock_query() {
        assert!(is_lock_query("lock settings"));
        assert!(is_lock_query("protect category"));
        assert!(!is_lock_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_lock_fun_fact();
        assert!(fact.contains("lock"));
    }
}
