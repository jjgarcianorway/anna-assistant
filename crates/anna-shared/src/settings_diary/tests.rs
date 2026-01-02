// v0.0.694: Settings Diary (Phase 270)
// Tests

#[cfg(test)]
mod tests {
    use crate::settings_diary::types::{DiaryEntryType, DiaryImportance};
    use crate::settings_diary::config::DiaryConfig;
    use crate::settings_diary::entry::DiaryEntry;
    use crate::settings_diary::page::DailyPage;
    use crate::settings_diary::stats::DiaryStats;
    use crate::settings_diary::diary::SettingsDiary;
    use crate::settings_diary::registry::DiaryRegistry;
    use crate::settings_diary::helpers::{is_diary_query, diary_fun_fact};

    #[test]
    fn test_entry_type_display() {
        assert_eq!(format!("{}", DiaryEntryType::Note), "note");
        assert_eq!(format!("{}", DiaryEntryType::Alert), "alert");
    }

    #[test]
    fn test_importance_display() {
        assert_eq!(format!("{}", DiaryImportance::High), "high");
        assert_eq!(format!("{}", DiaryImportance::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = DiaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DiaryConfig::new("test")
            .max_entries_per_day(50)
            .retention_days(7);
        assert_eq!(c.max_entries_per_day, 50);
        assert_eq!(c.retention_days, 7);
    }

    #[test]
    fn test_entry_new() {
        let e = DiaryEntry::new(1, DiaryEntryType::Note, "test note");
        assert!(!e.is_important());
    }

    #[test]
    fn test_entry_important() {
        let e = DiaryEntry::new(1, DiaryEntryType::Alert, "alert")
            .importance(DiaryImportance::High);
        assert!(e.is_important());
    }

    #[test]
    fn test_entry_tags() {
        let e = DiaryEntry::new(1, DiaryEntryType::Note, "note")
            .tag("config")
            .tag("update");
        assert_eq!(e.tags.len(), 2);
    }

    #[test]
    fn test_page_new() {
        let p = DailyPage::new("2025-12-15");
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_page_add() {
        let mut p = DailyPage::new("2025-12-15");
        p.add(DiaryEntry::new(1, DiaryEntryType::Note, "test"));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = DiaryStats::default();
        s.record(&DiaryEntry::new(1, DiaryEntryType::Note, "test"));
        assert_eq!(s.total_entries, 1);
    }

    #[test]
    fn test_diary_new() {
        let d = SettingsDiary::new(DiaryConfig::default());
        assert_eq!(d.day_count(), 0);
    }

    #[test]
    fn test_diary_add_note() {
        let mut d = SettingsDiary::new(DiaryConfig::default());
        d.add_note("2025-12-15", "test note");
        assert_eq!(d.stats().total_entries, 1);
    }

    #[test]
    fn test_diary_add_change() {
        let mut d = SettingsDiary::new(DiaryConfig::default());
        d.add_change("2025-12-15", "app.name", "Changed app name");
        assert_eq!(d.day_count(), 1);
    }

    #[test]
    fn test_diary_get_page() {
        let mut d = SettingsDiary::new(DiaryConfig::default());
        d.add_note("2025-12-15", "test");
        let page = d.get_page("2025-12-15");
        assert!(page.is_some());
    }

    #[test]
    fn test_registry_new() {
        let r = DiaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DiaryRegistry::new();
        r.register("d1", SettingsDiary::new(DiaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_diary_query() {
        assert!(is_diary_query("settings diary"));
        assert!(!is_diary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = diary_fun_fact();
        assert!(fact.contains("diary"));
    }
}
