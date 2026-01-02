// v0.0.716: Settings Missive Tests (Phase 292)
// Tests for missive system

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_missive_type_display() {
        assert_eq!(format!("{}", MissiveType::Formal), "formal");
        assert_eq!(format!("{}", MissiveType::Business), "business");
    }

    #[test]
    fn test_delivery_display() {
        assert_eq!(format!("{}", MissiveDelivery::Standard), "standard");
        assert_eq!(format!("{}", MissiveDelivery::Certified), "certified");
    }

    #[test]
    fn test_config_new() {
        let c = MissiveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MissiveConfig::new("test")
            .missive_type(MissiveType::Personal)
            .delivery(MissiveDelivery::Express);
        assert_eq!(c.missive_type, MissiveType::Personal);
        assert_eq!(c.delivery, MissiveDelivery::Express);
    }

    #[test]
    fn test_letter_new() {
        let l = MissiveLetter::new("l1", "Subject", "Content");
        assert_eq!(l.id, "l1");
    }

    #[test]
    fn test_letter_builder() {
        let l = MissiveLetter::new("l1", "Subject", "Content")
            .from("sender")
            .to("recipient");
        assert_eq!(l.from, "sender");
        assert_eq!(l.to, "recipient");
    }

    #[test]
    fn test_letter_deliver() {
        let mut l = MissiveLetter::new("l1", "Subject", "Content");
        l.deliver();
        assert!(l.delivered);
    }

    #[test]
    fn test_enclosure_new() {
        let e = MissiveEnclosure::new("key", "value", "l1");
        assert_eq!(e.letter_id, "l1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MissiveStats::default();
        let mut letter = MissiveLetter::new("l1", "Subject", "Content");
        letter.deliver();
        s.update(&[letter], MissiveType::Formal);
        assert_eq!(s.total_missives, 1);
        assert_eq!(s.delivered, 1);
    }

    #[test]
    fn test_missive_new() {
        let m = SettingsMissive::new(MissiveConfig::default());
        assert_eq!(m.letter_count(), 0);
    }

    #[test]
    fn test_missive_add_letter() {
        let mut m = SettingsMissive::new(MissiveConfig::default());
        m.add_letter(MissiveLetter::new("l1", "Subject", "Content"));
        assert_eq!(m.letter_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MissiveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MissiveRegistry::new();
        r.register("m1", SettingsMissive::new(MissiveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_missive_query() {
        assert!(is_missive_query("settings missive"));
        assert!(!is_missive_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = missive_fun_fact();
        assert!(fact.contains("missive"));
    }
}
