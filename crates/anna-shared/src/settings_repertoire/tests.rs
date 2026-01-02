// v0.0.703: Settings Repertoire Tests (Phase 279)
// Test cases for settings repertoire

#[cfg(test)]
mod tests {
    use crate::settings_repertoire::*;

    #[test]
    fn test_repertoire_type_display() {
        assert_eq!(format!("{}", RepertoireType::Standard), "standard");
        assert_eq!(format!("{}", RepertoireType::Classic), "classic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RepertoireStatus::Rehearsing), "rehearsing");
        assert_eq!(format!("{}", RepertoireStatus::Performing), "performing");
    }

    #[test]
    fn test_config_new() {
        let c = RepertoireConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RepertoireConfig::new("test")
            .repertoire_type(RepertoireType::Modern)
            .season("2025");
        assert_eq!(c.repertoire_type, RepertoireType::Modern);
        assert_eq!(c.season, "2025");
    }

    #[test]
    fn test_piece_new() {
        let p = RepertoirePiece::new("p1", "Piece 1", "Composer");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_piece_builder() {
        let p = RepertoirePiece::new("p1", "Piece 1", "Composer")
            .difficulty(5)
            .practiced(true);
        assert_eq!(p.difficulty, 5);
        assert!(p.practiced);
    }

    #[test]
    fn test_item_new() {
        let i = RepertoireItem::new("key", "value", "p1");
        assert_eq!(i.piece_id, "p1");
    }

    #[test]
    fn test_item_notes() {
        let i = RepertoireItem::new("key", "value", "p1").notes("Performance note");
        assert!(i.notes.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = RepertoireStats::default();
        let pieces = vec![RepertoirePiece::new("p1", "Piece", "Composer").practiced(true)];
        s.update(&pieces);
        assert_eq!(s.total_pieces, 1);
        assert_eq!(s.practiced_pieces, 1);
    }

    #[test]
    fn test_repertoire_new() {
        let r = SettingsRepertoire::new(RepertoireConfig::default());
        assert_eq!(r.piece_count(), 0);
    }

    #[test]
    fn test_repertoire_add_piece() {
        let mut r = SettingsRepertoire::new(RepertoireConfig::default());
        r.add_piece(RepertoirePiece::new("p1", "Piece 1", "Composer"));
        assert_eq!(r.piece_count(), 1);
    }

    #[test]
    fn test_repertoire_ready() {
        let mut r = SettingsRepertoire::new(RepertoireConfig::default());
        r.ready();
        assert_eq!(r.status(), RepertoireStatus::Ready);
    }

    #[test]
    fn test_repertoire_perform() {
        let mut r = SettingsRepertoire::new(RepertoireConfig::default());
        r.perform();
        assert_eq!(r.status(), RepertoireStatus::Performing);
    }

    #[test]
    fn test_registry_new() {
        let r = RepertoireRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RepertoireRegistry::new();
        r.register("r1", SettingsRepertoire::new(RepertoireConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_repertoire_query() {
        assert!(is_repertoire_query("settings repertoire"));
        assert!(!is_repertoire_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = repertoire_fun_fact();
        assert!(fact.contains("repertoire"));
    }
}
