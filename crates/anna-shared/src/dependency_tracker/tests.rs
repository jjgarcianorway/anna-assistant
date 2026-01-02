//! Tests for dependency tracking

#[cfg(test)]
mod tests {
    use super::super::formatting::{dependency_fun_fact, format_dependency_tracker, is_dependency_query};
    use super::super::tracker::DependencyTracker;
    use super::super::types::{DependencyRecord, DependencyStatus, DependencyType};

    fn make_dep(pkg: &str, dep: &str, dep_type: DependencyType, status: DependencyStatus) -> DependencyRecord {
        DependencyRecord {
            package: pkg.to_string(),
            dependency: dep.to_string(),
            dep_type,
            status,
            version_req: Some(">=1.0".to_string()),
            installed_version: Some("1.2.3".to_string()),
            last_check: 1234567890,
        }
    }

    #[test]
    fn test_dependency_type() {
        assert_eq!(DependencyType::Runtime.name(), "Runtime");
        assert_eq!(DependencyType::Build.symbol(), "⚙");
    }

    #[test]
    fn test_dependency_status() {
        assert_eq!(DependencyStatus::Missing.name(), "Missing");
        assert_eq!(DependencyStatus::Missing.symbol(), "✗");
    }

    #[test]
    fn test_add_dependency() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.deps_for("app").len(), 1);
    }

    #[test]
    fn test_reverse_deps() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app1", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));
        tracker.add(make_dep("app2", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        assert_eq!(tracker.reverse_deps("libfoo").len(), 2);
    }

    #[test]
    fn test_broken_packages() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Missing));

        assert!(tracker.has_missing("app"));
        assert!(tracker.broken_packages.contains(&"app".to_string()));
    }

    #[test]
    fn test_safe_to_remove() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        // libfoo is needed by app, not safe to remove
        assert!(!tracker.safe_to_remove("libfoo"));
        // app has nothing depending on it
        assert!(tracker.safe_to_remove("app"));
    }

    #[test]
    fn test_update_status() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Missing));

        assert!(tracker.update_status("app", "libfoo", DependencyStatus::Installed));
        assert_eq!(tracker.deps_for("app")[0].status, DependencyStatus::Installed);
    }

    #[test]
    fn test_by_type() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));
        tracker.add(make_dep("app", "cmake", DependencyType::Build, DependencyStatus::Installed));

        assert_eq!(tracker.by_dep_type(DependencyType::Runtime).len(), 1);
        assert_eq!(tracker.by_dep_type(DependencyType::Build).len(), 1);
    }

    #[test]
    fn test_format_tracker() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        let output = format_dependency_tracker(&tracker);
        assert!(output.contains("Dependency Tracker"));
        assert!(output.contains("Total dependencies: 1"));
    }

    #[test]
    fn test_is_dependency_query() {
        assert!(is_dependency_query("what are the dependencies?"));
        assert!(is_dependency_query("what depends on libfoo"));
        assert!(is_dependency_query("show orphan packages"));
        assert!(!is_dependency_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        let fact = dependency_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
