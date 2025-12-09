//! Tests for report module (v0.0.189).

#[cfg(test)]
mod tests {
    use crate::report::{format_bytes, sanitize_mount, HealthSeverity};

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_bytes(32 * 1024 * 1024 * 1024), "32.0 GB");
    }

    #[test]
    fn test_sanitize_mount() {
        assert_eq!(sanitize_mount("/"), "root");
        assert_eq!(sanitize_mount("/var"), "var");
        assert_eq!(sanitize_mount("/var/log"), "var_log");
    }

    #[test]
    fn test_health_severity_ordering() {
        assert!(HealthSeverity::Critical > HealthSeverity::Warning);
        assert!(HealthSeverity::Warning > HealthSeverity::Ok);
    }
}
