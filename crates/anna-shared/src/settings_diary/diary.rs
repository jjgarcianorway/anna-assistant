// v0.0.694: Settings Diary (Phase 270)
// Settings diary main struct

use std::collections::HashMap;
use crate::settings_diary::config::DiaryConfig;
use crate::settings_diary::entry::DiaryEntry;
use crate::settings_diary::page::DailyPage;
use crate::settings_diary::stats::DiaryStats;
use crate::settings_diary::types::{DiaryEntryType, DiaryImportance};

/// Settings diary
#[derive(Debug, Clone, Default)]
pub struct SettingsDiary {
    /// Config
    config: DiaryConfig,
    /// Pages by date
    pages: HashMap<String, DailyPage>,
    /// Stats
    stats: DiaryStats,
    /// Next ID
    next_id: usize,
}

impl SettingsDiary {
    /// Create new diary
    pub fn new(config: DiaryConfig) -> Self {
        Self {
            config,
            pages: HashMap::new(),
            stats: DiaryStats::default(),
            next_id: 1,
        }
    }

    /// Get or create page for date
    fn get_or_create_page(&mut self, date: &str) -> &mut DailyPage {
        if !self.pages.contains_key(date) {
            self.pages.insert(date.to_string(), DailyPage::new(date));
            self.stats.set_days(self.pages.len());
        }
        self.pages.get_mut(date).unwrap()
    }

    /// Add note
    pub fn add_note(&mut self, date: &str, content: &str) -> usize {
        let entry = DiaryEntry::new(self.next_id, DiaryEntryType::Note, content);
        let id = self.next_id;
        self.next_id += 1;
        self.stats.record(&entry);
        self.get_or_create_page(date).add(entry);
        id
    }

    /// Add change
    pub fn add_change(&mut self, date: &str, key: &str, content: &str) -> usize {
        let entry = DiaryEntry::new(self.next_id, DiaryEntryType::Change, content)
            .related_key(key);
        let id = self.next_id;
        self.next_id += 1;
        self.stats.record(&entry);
        self.get_or_create_page(date).add(entry);
        id
    }

    /// Add alert
    pub fn add_alert(&mut self, date: &str, content: &str, importance: DiaryImportance) -> usize {
        let entry = DiaryEntry::new(self.next_id, DiaryEntryType::Alert, content)
            .importance(importance);
        let id = self.next_id;
        self.next_id += 1;
        self.stats.record(&entry);
        self.get_or_create_page(date).add(entry);
        id
    }

    /// Get page
    pub fn get_page(&self, date: &str) -> Option<&DailyPage> {
        self.pages.get(date)
    }

    /// Get stats
    pub fn stats(&self) -> &DiaryStats {
        &self.stats
    }

    /// Day count
    pub fn day_count(&self) -> usize {
        self.pages.len()
    }
}
