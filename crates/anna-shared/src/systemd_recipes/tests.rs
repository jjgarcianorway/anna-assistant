//! Tests for systemd_recipes module (v0.0.233).

#[cfg(test)]
mod tests {
    use crate::systemd_recipes::{detect_feature, match_query, SystemdFeature};

    #[test]
    fn test_match_create_service() {
        let recipe = match_query("how do I create a systemd service");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SystemdFeature::CreateService);
    }

    #[test]
    fn test_match_create_timer() {
        let recipe = match_query("create a systemd timer for scheduled tasks");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SystemdFeature::CreateTimer);
    }

    #[test]
    fn test_match_user_service() {
        let recipe = match_query("how to create a user systemd service");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SystemdFeature::CreateUserService);
    }

    #[test]
    fn test_match_view_logs() {
        let recipe = match_query("view systemd service logs");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SystemdFeature::ViewLogs);
    }

    #[test]
    fn test_match_debug_service() {
        let recipe = match_query("debug failing systemd service");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SystemdFeature::DebugService);
    }

    #[test]
    fn test_match_harden() {
        let recipe = match_query("harden systemd service security");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SystemdFeature::HardenService);
    }

    #[test]
    fn test_no_match_unrelated() {
        let recipe = match_query("what is the weather");
        assert!(recipe.is_none());
    }

    #[test]
    fn test_detect_feature_timer() {
        assert_eq!(
            detect_feature("schedule a task with timer"),
            Some(SystemdFeature::CreateTimer)
        );
    }

    #[test]
    fn test_detect_feature_socket() {
        assert_eq!(
            detect_feature("socket activation for my service"),
            Some(SystemdFeature::SocketActivation)
        );
    }
}
