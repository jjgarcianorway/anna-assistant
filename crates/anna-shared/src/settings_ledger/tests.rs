// v0.0.693: Settings Ledger Tests (Phase 269)
// Test suite for the settings ledger

#[cfg(test)]
mod tests {
    use crate::settings_ledger::*;

    #[test]
    fn test_entry_type_display() {
        assert_eq!(format!("{}", LedgerEntryType::Set), "set");
        assert_eq!(format!("{}", LedgerEntryType::Update), "update");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", LedgerStatus::Active), "active");
        assert_eq!(format!("{}", LedgerStatus::Sealed), "sealed");
    }

    #[test]
    fn test_config_new() {
        let c = LedgerConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = LedgerConfig::new("test")
            .max_entries(500)
            .immutable(false);
        assert_eq!(c.max_entries, 500);
        assert!(!c.immutable);
    }

    #[test]
    fn test_entry_new() {
        let e = LedgerEntry::new(1, LedgerEntryType::Set, "key");
        assert!(e.is_modification());
    }

    #[test]
    fn test_entry_values() {
        let e = LedgerEntry::new(1, LedgerEntryType::Update, "key")
            .prev_value("old")
            .value("new");
        assert_eq!(e.prev_value, Some("old".to_string()));
        assert_eq!(e.value, Some("new".to_string()));
    }

    #[test]
    fn test_page_new() {
        let p = LedgerPage::new(1);
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_page_add() {
        let mut p = LedgerPage::new(1);
        p.add(LedgerEntry::new(1, LedgerEntryType::Set, "key"));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_page_seal() {
        let mut p = LedgerPage::new(1);
        p.seal();
        let added = p.add(LedgerEntry::new(1, LedgerEntryType::Set, "key"));
        assert!(!added);
    }

    #[test]
    fn test_stats_record() {
        let mut s = LedgerStats::default();
        s.record(&LedgerEntry::new(1, LedgerEntryType::Set, "key"));
        assert_eq!(s.total_entries, 1);
    }

    #[test]
    fn test_ledger_new() {
        let l = SettingsLedger::new(LedgerConfig::default());
        assert_eq!(l.page_count(), 1);
    }

    #[test]
    fn test_ledger_record_set() {
        let mut l = SettingsLedger::new(LedgerConfig::default());
        let seq = l.record_set("key", "value");
        assert_eq!(seq, 1);
        assert_eq!(l.stats().total_entries, 1);
    }

    #[test]
    fn test_ledger_record_update() {
        let mut l = SettingsLedger::new(LedgerConfig::default());
        let seq = l.record_update("key", "old", "new");
        assert_eq!(seq, 1);
    }

    #[test]
    fn test_ledger_get_entry() {
        let mut l = SettingsLedger::new(LedgerConfig::default());
        l.record_set("key", "value");
        let entry = l.get_entry(1);
        assert!(entry.is_some());
    }

    #[test]
    fn test_registry_new() {
        let r = LedgerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = LedgerRegistry::new();
        r.register("l1", SettingsLedger::new(LedgerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_ledger_query() {
        assert!(is_ledger_query("settings ledger"));
        assert!(!is_ledger_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = ledger_fun_fact();
        assert!(fact.contains("ledger"));
    }
}
