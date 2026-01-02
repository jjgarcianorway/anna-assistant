// v0.0.706: Settings Bulletin - Tests (Phase 282)
// Test suite for bulletin module

#[cfg(test)]
mod tests {
    use crate::settings_bulletin::*;

    #[test]
    fn test_bulletin_type_display() {
        assert_eq!(format!("{}", BulletinType::News), "news");
        assert_eq!(format!("{}", BulletinType::Alert), "alert");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", BulletinPriority::Normal), "normal");
        assert_eq!(format!("{}", BulletinPriority::Urgent), "urgent");
    }

    #[test]
    fn test_config_new() {
        let c = BulletinConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BulletinConfig::new("test")
            .bulletin_type(BulletinType::Alert)
            .max_posts(50);
        assert_eq!(c.bulletin_type, BulletinType::Alert);
        assert_eq!(c.max_posts, 50);
    }

    #[test]
    fn test_post_new() {
        let p = BulletinPost::new("p1", "Post 1", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_post_builder() {
        let p = BulletinPost::new("p1", "Post 1", "Content")
            .priority(BulletinPriority::High)
            .pinned(true);
        assert!(p.is_high_priority());
        assert!(p.pinned);
    }

    #[test]
    fn test_item_new() {
        let i = BulletinItem::new("key", "value", "p1");
        assert_eq!(i.post_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BulletinStats::default();
        let posts = vec![BulletinPost::new("p1", "Post", "Content").pinned(true)];
        s.update(&posts, BulletinType::News);
        assert_eq!(s.total_posts, 1);
        assert_eq!(s.pinned_posts, 1);
    }

    #[test]
    fn test_bulletin_new() {
        let b = SettingsBulletin::new(BulletinConfig::default());
        assert_eq!(b.post_count(), 0);
    }

    #[test]
    fn test_bulletin_add_post() {
        let mut b = SettingsBulletin::new(BulletinConfig::default());
        b.add_post(BulletinPost::new("p1", "Post 1", "Content"));
        assert_eq!(b.post_count(), 1);
    }

    #[test]
    fn test_bulletin_get_pinned() {
        let mut b = SettingsBulletin::new(BulletinConfig::default());
        b.add_post(BulletinPost::new("p1", "Post 1", "Content").pinned(true));
        let pinned = b.get_pinned();
        assert_eq!(pinned.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BulletinRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BulletinRegistry::new();
        r.register("b1", SettingsBulletin::new(BulletinConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_bulletin_query() {
        assert!(is_bulletin_query("settings bulletin"));
        assert!(!is_bulletin_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = bulletin_fun_fact();
        assert!(fact.contains("bulletin"));
    }
}
