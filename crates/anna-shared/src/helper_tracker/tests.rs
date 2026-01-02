//! Tests for helper tracker

#[cfg(test)]
mod tests {
    use super::super::formatting::*;
    use super::super::tracker::HelperTracker;
    use super::super::types::*;
    use super::super::utils::*;

    fn make_helper(name: &str, source: InstallerSource) -> HelperRecord {
        HelperRecord {
            name: name.to_string(),
            package_name: Some(name.to_string()),
            installed_by: source,
            purpose: detect_purpose(name),
            description: format!("{} helper", name),
            installed_at: 1234567890,
            usage_count: 0,
            last_used: None,
            available: true,
            install_reason: None,
            ticket_id: None,
        }
    }

    #[test]
    fn test_installer_source() {
        assert_eq!(InstallerSource::Anna.name(), "Anna");
        assert_eq!(InstallerSource::User.symbol(), "U");
    }

    #[test]
    fn test_helper_purpose() {
        assert_eq!(HelperPurpose::SystemInfo.name(), "System Info");
        assert_eq!(HelperPurpose::NetworkDiag.name(), "Network Diagnostics");
    }

    #[test]
    fn test_detect_purpose() {
        assert_eq!(detect_purpose("netstat"), HelperPurpose::NetworkDiag);
        assert_eq!(detect_purpose("htop"), HelperPurpose::ProcessMon);
        assert_eq!(detect_purpose("sysinfo"), HelperPurpose::SystemInfo);
        assert_eq!(detect_purpose("git"), HelperPurpose::Development);
        assert_eq!(detect_purpose("ffmpeg"), HelperPurpose::Multimedia);
    }

    #[test]
    fn test_helper_tracker_register() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        assert_eq!(tracker.total_count(), 1);
        assert!(tracker.has("htop"));
    }

    #[test]
    fn test_record_usage() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        assert!(tracker.record_usage("htop", 1234567890));
        assert_eq!(tracker.total_usage, 1);
        assert_eq!(tracker.get("htop").unwrap().usage_count, 1);
    }

    #[test]
    fn test_mark_unavailable() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        assert!(tracker.mark_unavailable("htop"));
        assert!(!tracker.get("htop").unwrap().available);
        assert_eq!(tracker.available_count(), 0);
    }

    #[test]
    fn test_anna_vs_user_installed() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));
        tracker.register(make_helper("vim", InstallerSource::User));
        tracker.register(make_helper("netstat", InstallerSource::Anna));

        assert_eq!(tracker.anna_installed().len(), 2);
        assert_eq!(tracker.user_installed().len(), 1);
    }

    #[test]
    fn test_removable_on_uninstall() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));
        tracker.register(make_helper("vim", InstallerSource::User));

        let removable = tracker.removable_on_uninstall();
        assert_eq!(removable.len(), 1);
        assert_eq!(removable[0].name, "htop");
    }

    #[test]
    fn test_most_used() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));
        tracker.register(make_helper("vim", InstallerSource::User));

        tracker.record_usage("htop", 1);
        tracker.record_usage("htop", 2);
        tracker.record_usage("vim", 3);

        let (name, count) = tracker.most_used().unwrap();
        assert_eq!(name, "htop");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_format_helper_tracker() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        let output = format_helper_tracker(&tracker);
        assert!(output.contains("Helper Tools"));
        assert!(output.contains("Total helpers: 1"));
    }

    #[test]
    fn test_is_helper_query() {
        assert!(is_helper_query("what helpers are installed?"));
        assert!(is_helper_query("show me available tools"));
        assert!(is_helper_query("what did anna install?"));
        assert!(!is_helper_query("what is the weather?"));
    }

    #[test]
    fn test_helper_fun_fact() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        let fact = helper_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
