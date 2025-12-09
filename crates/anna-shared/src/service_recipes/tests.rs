//! Tests for service_recipes module (v0.0.214).

#[cfg(test)]
mod tests {
    use crate::service_recipes::{
        catalog::{find_service, known_services},
        types::{ServiceAction, ServiceRisk},
    };

    #[test]
    fn test_service_action_display() {
        assert_eq!(ServiceAction::Start.display_name(), "start");
        assert_eq!(ServiceAction::Enable.display_name(), "enable");
    }

    #[test]
    fn test_service_action_opposite() {
        assert_eq!(ServiceAction::Start.opposite(), Some(ServiceAction::Stop));
        assert_eq!(
            ServiceAction::Enable.opposite(),
            Some(ServiceAction::Disable)
        );
        assert_eq!(ServiceAction::Restart.opposite(), None);
    }

    #[test]
    fn test_find_service() {
        assert!(find_service("sshd").is_some());
        assert!(find_service("ssh").is_some()); // alias
        assert!(find_service("docker").is_some());
        assert!(find_service("nonexistent").is_none());
    }

    #[test]
    fn test_protected_service() {
        let dbus = find_service("dbus").unwrap();
        assert_eq!(dbus.risk, ServiceRisk::Protected);
        assert!(!dbus.risk.allows_modification());
    }

    #[test]
    fn test_systemctl_command() {
        let docker = find_service("docker").unwrap();
        assert_eq!(
            docker.command_for(ServiceAction::Start),
            "systemctl start docker.service"
        );
    }

    #[test]
    fn test_known_services_count() {
        let services = known_services();
        assert!(services.len() >= 10);
    }
}
