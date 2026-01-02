// v0.0.592: Snapshot Tests (Phase 168)
// Tests for settings snapshots

#[cfg(test)]
mod tests {
    use crate::settings_snapshot::*;
    use crate::unified_settings::SettingsCategory;

    #[test]
    fn test_snapshot_type_display() {
        assert_eq!(format!("{}", SnapshotType::Manual), "manual");
        assert_eq!(format!("{}", SnapshotType::Auto), "auto");
    }

    #[test]
    fn test_snapshot_status_display() {
        assert_eq!(format!("{}", SnapshotStatus::Active), "active");
        assert_eq!(format!("{}", SnapshotStatus::Archived), "archived");
    }

    #[test]
    fn test_snapshot_new() {
        let snapshot = SettingsSnapshot::new("test");
        assert_eq!(snapshot.name, "test");
        assert_eq!(snapshot.status, SnapshotStatus::Active);
    }

    #[test]
    fn test_snapshot_add_data() {
        let mut snapshot = SettingsSnapshot::new("test");
        snapshot.add_data(SettingsCategory::Personality, "data");
        assert_eq!(snapshot.category_count(), 1);
        assert!(snapshot.size > 0);
    }

    #[test]
    fn test_snapshot_archive() {
        let mut snapshot = SettingsSnapshot::new("test");
        snapshot.archive();
        assert_eq!(snapshot.status, SnapshotStatus::Archived);
    }

    #[test]
    fn test_manager_new() {
        let manager = SnapshotManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_manager_create() {
        let mut manager = SnapshotManager::new();
        manager.create("test");
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_manager_get() {
        let mut manager = SnapshotManager::new();
        let snapshot = manager.create("test");
        let id = snapshot.id.clone();
        assert!(manager.get(&id).is_some());
    }

    #[test]
    fn test_manager_delete() {
        let mut manager = SnapshotManager::new();
        let snapshot = manager.create("test");
        let id = snapshot.id.clone();
        assert!(manager.delete(&id));
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_format_snapshot() {
        let snapshot = SettingsSnapshot::new("test");
        let output = format_snapshot(&snapshot);
        assert!(output.contains("Snapshot"));
    }

    #[test]
    fn test_format_snapshots() {
        let manager = SnapshotManager::new();
        let output = format_snapshots(&manager);
        assert!(output.contains("Snapshots"));
    }

    #[test]
    fn test_is_snapshot_query() {
        assert!(is_snapshot_query("create snapshot"));
        assert!(is_snapshot_query("checkpoint settings"));
        assert!(!is_snapshot_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_snapshot_fun_fact();
        assert!(fact.contains("snapshot"));
    }
}
