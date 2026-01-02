// v0.0.694: Settings Diary (Phase 270)
// Diary registry

use std::collections::HashMap;
use crate::settings_diary::diary::SettingsDiary;

/// Diary registry
#[derive(Debug, Clone, Default)]
pub struct DiaryRegistry {
    /// Diaries by ID
    diaries: HashMap<String, SettingsDiary>,
}

impl DiaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register diary
    pub fn register(&mut self, id: impl Into<String>, diary: SettingsDiary) {
        self.diaries.insert(id.into(), diary);
    }

    /// Unregister diary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.diaries.remove(id).is_some()
    }

    /// Get diary
    pub fn get(&self, id: &str) -> Option<&SettingsDiary> {
        self.diaries.get(id)
    }

    /// Get diary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDiary> {
        self.diaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.diaries.len()
    }
}
