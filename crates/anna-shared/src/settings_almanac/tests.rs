// v0.0.705: Settings Almanac (Phase 281)
// Tests

#[cfg(test)]
mod tests {
    use crate::settings_almanac::types::{AlmanacType, AlmanacEdition};
    use crate::settings_almanac::config::AlmanacConfig;
    use crate::settings_almanac::chapter::{AlmanacChapter, AlmanacEntry};
    use crate::settings_almanac::stats::AlmanacStats;
    use crate::settings_almanac::almanac::SettingsAlmanac;
    use crate::settings_almanac::registry::AlmanacRegistry;
    use crate::settings_almanac::utils::*;

    #[test]
    fn test_almanac_type_display() {
        assert_eq!(format!("{}", AlmanacType::Annual), "annual");
        assert_eq!(format!("{}", AlmanacType::Technical), "technical");
    }

    #[test]
    fn test_edition_display() {
        assert_eq!(format!("{}", AlmanacEdition::Current), "current");
        assert_eq!(format!("{}", AlmanacEdition::Special), "special");
    }

    #[test]
    fn test_config_new() {
        let c = AlmanacConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AlmanacConfig::new("test")
            .almanac_type(AlmanacType::Seasonal)
            .year(2025);
        assert_eq!(c.almanac_type, AlmanacType::Seasonal);
        assert_eq!(c.year, 2025);
    }

    #[test]
    fn test_chapter_new() {
        let ch = AlmanacChapter::new(1, "Chapter 1", "Week 1");
        assert_eq!(ch.number, 1);
    }

    #[test]
    fn test_chapter_add() {
        let mut ch = AlmanacChapter::new(1, "Chapter 1", "Week 1");
        ch.add(AlmanacEntry::new("key", "value", "2025-12-15"));
        assert_eq!(ch.entry_count(), 1);
    }

    #[test]
    fn test_entry_new() {
        let e = AlmanacEntry::new("key", "value", "2025-12-15");
        assert_eq!(e.key, "key");
    }

    #[test]
    fn test_entry_highlight() {
        let e = AlmanacEntry::new("key", "value", "2025-12-15").highlight(true);
        assert!(e.highlight);
    }

    #[test]
    fn test_stats_update() {
        let mut s = AlmanacStats::default();
        let mut ch = AlmanacChapter::new(1, "Chapter", "Week 1");
        ch.add(AlmanacEntry::new("key", "value", "2025-12-15").highlight(true));
        s.update(&[ch]);
        assert_eq!(s.total_chapters, 1);
        assert_eq!(s.highlighted, 1);
    }

    #[test]
    fn test_almanac_new() {
        let a = SettingsAlmanac::new(AlmanacConfig::default());
        assert_eq!(a.chapter_count(), 0);
    }

    #[test]
    fn test_almanac_add_chapter() {
        let mut a = SettingsAlmanac::new(AlmanacConfig::default());
        a.add_chapter(AlmanacChapter::new(1, "Chapter 1", "Week 1"));
        assert_eq!(a.chapter_count(), 1);
    }

    #[test]
    fn test_almanac_add_entry() {
        let mut a = SettingsAlmanac::new(AlmanacConfig::default());
        a.add_chapter(AlmanacChapter::new(1, "Chapter 1", "Week 1"));
        let added = a.add_entry(1, AlmanacEntry::new("key", "value", "2025-12-15"));
        assert!(added);
    }

    #[test]
    fn test_almanac_edition() {
        let mut a = SettingsAlmanac::new(AlmanacConfig::default());
        a.set_edition(AlmanacEdition::Special);
        assert_eq!(a.edition(), AlmanacEdition::Special);
    }

    #[test]
    fn test_registry_new() {
        let r = AlmanacRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AlmanacRegistry::new();
        r.register("a1", SettingsAlmanac::new(AlmanacConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_almanac_query() {
        assert!(is_almanac_query("settings almanac"));
        assert!(!is_almanac_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = almanac_fun_fact();
        assert!(fact.contains("almanac"));
    }
}
