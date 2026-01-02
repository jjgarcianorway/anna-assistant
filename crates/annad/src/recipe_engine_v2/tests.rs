//! Tests for recipe engine v2.

#[cfg(test)]
mod tests {
    use crate::recipe_engine_v2::params::extract_param_value;

    #[test]
    fn test_extract_service_name() {
        assert_eq!(
            extract_param_value("why is nginx service failing", "service_name", ""),
            Some("nginx".to_string())
        );
        assert_eq!(
            extract_param_value("check sshd service status", "service_name", ""),
            Some("sshd".to_string())
        );
    }

    #[test]
    fn test_extract_package_name() {
        assert_eq!(
            extract_param_value("is vim installed", "package_name", ""),
            Some("vim".to_string())
        );
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            extract_param_value("check disk usage on /home", "mount_path", ""),
            Some("/home".to_string())
        );
    }
}
