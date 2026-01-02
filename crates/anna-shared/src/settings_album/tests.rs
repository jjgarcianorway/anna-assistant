// v0.0.696: Settings Album (Phase 272)
// Unit tests for settings album

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_album_type_display() {
        assert_eq!(format!("{}", AlbumType::Standard), "standard");
        assert_eq!(format!("{}", AlbumType::Snapshot), "snapshot");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AlbumStatus::Empty), "empty");
        assert_eq!(format!("{}", AlbumStatus::Sealed), "sealed");
    }

    #[test]
    fn test_config_new() {
        let c = AlbumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AlbumConfig::new("test")
            .album_type(AlbumType::Archive)
            .max_pages(25);
        assert_eq!(c.album_type, AlbumType::Archive);
        assert_eq!(c.max_pages, 25);
    }

    #[test]
    fn test_page_new() {
        let p = AlbumPage::new(1, "Page 1");
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_page_add() {
        let mut p = AlbumPage::new(1, "Page 1");
        p.add(AlbumItem::new("key", "value", "2025-12-15"));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_item_new() {
        let i = AlbumItem::new("key", "value", "2025-12-15");
        assert_eq!(i.key, "key");
    }

    #[test]
    fn test_item_label() {
        let i = AlbumItem::new("key", "value", "2025-12-15").label("important");
        assert!(i.label.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = AlbumStats::default();
        let pages = vec![AlbumPage::new(1, "Page")];
        s.update(&pages, AlbumType::Standard);
        assert_eq!(s.total_pages, 1);
    }

    #[test]
    fn test_album_new() {
        let a = SettingsAlbum::new(AlbumConfig::default());
        assert_eq!(a.page_count(), 0);
    }

    #[test]
    fn test_album_add_page() {
        let mut a = SettingsAlbum::new(AlbumConfig::default());
        a.add_page("Page 1");
        assert_eq!(a.page_count(), 1);
    }

    #[test]
    fn test_album_add_item() {
        let mut a = SettingsAlbum::new(AlbumConfig::default());
        a.add_page("Page 1");
        let added = a.add_item(1, AlbumItem::new("key", "value", "2025-12-15"));
        assert!(added);
    }

    #[test]
    fn test_album_seal() {
        let mut a = SettingsAlbum::new(AlbumConfig::default());
        a.seal();
        assert_eq!(a.status(), AlbumStatus::Sealed);
    }

    #[test]
    fn test_registry_new() {
        let r = AlbumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AlbumRegistry::new();
        r.register("a1", SettingsAlbum::new(AlbumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_album_query() {
        assert!(is_album_query("settings album"));
        assert!(!is_album_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = album_fun_fact();
        assert!(fact.contains("album"));
    }
}
