//! Package Tracker Tests

#[cfg(test)]
mod tests {
    use super::super::formatting::*;
    use super::super::tracker::PackageTracker;
    use super::super::types::{InstalledBy, PackageManager, PackageRecord};
    use super::super::utils::*;

    fn make_package(name: &str, by: InstalledBy) -> PackageRecord {
        PackageRecord {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            installed_by: by,
            manager: PackageManager::Pacman,
            installed_at: 1234567890,
            reason: Some("test".to_string()),
            ticket_id: None,
            is_installed: true,
            removed_at: None,
        }
    }

    #[test]
    fn test_installed_by() {
        assert_eq!(InstalledBy::Anna.symbol(), "A");
        assert_eq!(InstalledBy::User.description(), "installed by user");
    }

    #[test]
    fn test_package_manager() {
        assert_eq!(PackageManager::Pacman.name(), "pacman");
        assert_eq!(PackageManager::Apt.name(), "apt");
    }

    #[test]
    fn test_package_tracker_record() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.anna_installed_count, 1);
    }

    #[test]
    fn test_package_tracker_removal() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        assert!(tracker.record_removal("vim"));
        assert_eq!(tracker.installed_count(), 0);
        assert_eq!(tracker.removed().len(), 1);
    }

    #[test]
    fn test_anna_installed() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));
        tracker.record_install(make_package("htop", InstalledBy::User));

        assert_eq!(tracker.anna_installed().len(), 1);
        assert_eq!(tracker.user_installed().len(), 1);
    }

    #[test]
    fn test_by_package_manager() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let mut pkg = make_package("code", InstalledBy::User);
        pkg.manager = PackageManager::Flatpak;
        tracker.record_install(pkg);

        assert_eq!(tracker.by_package_manager(PackageManager::Pacman).len(), 1);
        assert_eq!(tracker.by_package_manager(PackageManager::Flatpak).len(), 1);
    }

    #[test]
    fn test_format_package_tracker() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let output = format_package_tracker(&tracker);
        assert!(output.contains("Package Installation History"));
        assert!(output.contains("Total tracked: 1"));
    }

    #[test]
    fn test_is_package_tracker_query() {
        assert!(is_package_tracker_query("show installed packages"));
        assert!(is_package_tracker_query("what packages did anna install?"));
        assert!(is_package_tracker_query("package history"));
        assert!(!is_package_tracker_query("what is my disk space?"));
    }

    #[test]
    fn test_package_fun_fact() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let fact = package_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let compact = format_package_tracker_compact(&tracker);
        assert!(compact.contains("Packages: 1 installed"));

        let oneline = format_package_tracker_oneline(&tracker);
        assert!(oneline.contains("1 packages"));
    }
}
