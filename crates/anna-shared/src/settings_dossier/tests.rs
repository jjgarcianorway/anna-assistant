// v0.0.697: Settings Dossier Tests (Phase 273)
// Unit tests for settings dossier

#[cfg(test)]
mod tests {
    use crate::settings_dossier::*;

    #[test]
    fn test_dossier_type_display() {
        assert_eq!(format!("{}", DossierType::Standard), "standard");
        assert_eq!(format!("{}", DossierType::Confidential), "confidential");
    }

    #[test]
    fn test_classification_display() {
        assert_eq!(format!("{}", DossierClassification::Public), "public");
        assert_eq!(format!("{}", DossierClassification::Secret), "secret");
    }

    #[test]
    fn test_config_new() {
        let c = DossierConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DossierConfig::new("test")
            .dossier_type(DossierType::Full)
            .classification(DossierClassification::Internal);
        assert_eq!(c.dossier_type, DossierType::Full);
        assert_eq!(c.classification, DossierClassification::Internal);
    }

    #[test]
    fn test_document_new() {
        let d = DossierDocument::new("d1", "Doc 1", "Content");
        assert_eq!(d.id, "d1");
    }

    #[test]
    fn test_document_restricted() {
        let d = DossierDocument::new("d1", "Doc 1", "Content")
            .classification(DossierClassification::Secret);
        assert!(d.is_restricted());
    }

    #[test]
    fn test_entry_new() {
        let e = DossierEntry::new("key", "value", "d1");
        assert_eq!(e.document_id, "d1");
    }

    #[test]
    fn test_entry_notes() {
        let e = DossierEntry::new("key", "value", "d1").notes("important");
        assert!(e.notes.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = DossierStats::default();
        let docs = vec![DossierDocument::new("d1", "Doc", "Content")];
        s.update(&docs);
        assert_eq!(s.total_documents, 1);
    }

    #[test]
    fn test_dossier_new() {
        let d = SettingsDossier::new(DossierConfig::default());
        assert_eq!(d.document_count(), 0);
    }

    #[test]
    fn test_dossier_add_document() {
        let mut d = SettingsDossier::new(DossierConfig::default());
        d.add_document(DossierDocument::new("d1", "Doc 1", "Content"));
        assert_eq!(d.document_count(), 1);
    }

    #[test]
    fn test_dossier_add_entry() {
        let mut d = SettingsDossier::new(DossierConfig::default());
        d.add_entry(DossierEntry::new("key", "value", "d1"));
        assert_eq!(d.entry_count(), 1);
    }

    #[test]
    fn test_dossier_get_entries() {
        let mut d = SettingsDossier::new(DossierConfig::default());
        d.add_entry(DossierEntry::new("key", "value", "d1"));
        let entries = d.get_entries("d1");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DossierRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DossierRegistry::new();
        r.register("d1", SettingsDossier::new(DossierConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_dossier_query() {
        assert!(is_dossier_query("settings dossier"));
        assert!(!is_dossier_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = dossier_fun_fact();
        assert!(fact.contains("dossier"));
    }
}
