//! Tests for status_snapshot module (v0.0.211).

#[cfg(test)]
mod tests {
    use crate::specialists::SpecialistRole;
    use crate::status_snapshot::{
        DaemonInfo, ModelsInfo, RoleModelBinding, StatusSnapshot, UpdateResult, VersionInfo,
    };
    use crate::teams::Team;

    #[test]
    fn test_version_info_new() {
        let v = VersionInfo::new("0.0.29");
        assert_eq!(v.annactl, "0.0.29");
        assert_eq!(v.git_tag_current, Some("v0.0.29".to_string()));
    }

    #[test]
    fn test_daemon_info_running() {
        let d = DaemonInfo::running(1234, 3600);
        assert!(d.running);
        assert_eq!(d.pid, Some(1234));
        assert_eq!(d.uptime_s, Some(3600));
    }

    #[test]
    fn test_daemon_info_not_running() {
        let d = DaemonInfo::not_running();
        assert!(!d.running);
        assert!(d.pid.is_none());
    }

    #[test]
    fn test_update_result_default() {
        let r = UpdateResult::default();
        assert_eq!(r, UpdateResult::NotChecked);
    }

    #[test]
    fn test_status_snapshot_health() {
        let mut snap = StatusSnapshot::new();

        // Initial state
        assert_eq!(snap.health_status(), "DAEMON_DOWN");

        // Daemon running
        snap.daemon = DaemonInfo::running(1234, 100);
        assert_eq!(snap.health_status(), "OLLAMA_MISSING");

        // Ollama present but not running
        snap.models.ollama_present = true;
        assert_eq!(snap.health_status(), "OLLAMA_DOWN");

        // Ollama running
        snap.models.ollama_running = true;
        assert_eq!(snap.health_status(), "OK");

        // With missing model
        snap.models.roles.push(RoleModelBinding {
            team: Team::General,
            role: SpecialistRole::Junior,
            model_name: "test".to_string(),
            model_present: false,
        });
        assert_eq!(snap.health_status(), "MODELS_PENDING");
    }

    #[test]
    fn test_models_info_missing() {
        let mut m = ModelsInfo::default();
        m.ollama_present = true;
        m.ollama_running = true;
        m.roles.push(RoleModelBinding {
            team: Team::Storage,
            role: SpecialistRole::Junior,
            model_name: "llama3.2".to_string(),
            model_present: false,
        });
        m.roles.push(RoleModelBinding {
            team: Team::Storage,
            role: SpecialistRole::Senior,
            model_name: "llama3.2".to_string(),
            model_present: true,
        });

        assert!(!m.is_ready());
        assert_eq!(m.missing_models(), vec!["llama3.2"]);
    }
}
