//! Tests for docker_recipes module (v0.0.235).

#[cfg(test)]
mod tests {
    use crate::docker_recipes::{detect_feature, match_query, DockerFeature};

    #[test]
    fn test_match_create_compose() {
        let recipe = match_query("how do I create a docker compose file");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DockerFeature::CreateCompose);
    }

    #[test]
    fn test_match_start_services() {
        let recipe = match_query("docker compose start services");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DockerFeature::StartServices);
    }

    #[test]
    fn test_match_stop_services() {
        let recipe = match_query("docker compose stop");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DockerFeature::StopServices);
    }

    #[test]
    fn test_match_view_logs() {
        let recipe = match_query("view docker compose logs");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DockerFeature::ViewLogs);
    }

    #[test]
    fn test_match_exec() {
        let recipe = match_query("docker exec shell into container");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DockerFeature::ExecContainer);
    }

    #[test]
    fn test_match_cleanup() {
        let recipe = match_query("docker cleanup unused images");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DockerFeature::Cleanup);
    }

    #[test]
    fn test_match_debug() {
        let recipe = match_query("debug docker container not working");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DockerFeature::Debug);
    }

    #[test]
    fn test_no_match_unrelated() {
        let recipe = match_query("what is the weather");
        assert!(recipe.is_none());
    }

    #[test]
    fn test_detect_feature_build() {
        assert_eq!(
            detect_feature("docker build image"),
            Some(DockerFeature::BuildImages)
        );
    }

    #[test]
    fn test_detect_feature_pull() {
        assert_eq!(
            detect_feature("docker pull images"),
            Some(DockerFeature::PullImages)
        );
    }
}
