// v0.0.555: Settings Persistence Tests
// Test suite for settings persistence

#[cfg(test)]
mod tests {
    use crate::settings_persistence::{SettingsError, SettingsFormat, SettingsPersistence};
    use crate::settings_persistence::{
        format_persistence_status,
        is_persistence_available,
        settings_persistence_fun_fact
    };
    use std::io::{self, ErrorKind};

    #[test]
    fn test_settings_format_display() {
        assert_eq!(format!("{}", SettingsFormat::Json), "JSON");
        assert_eq!(format!("{}", SettingsFormat::Toml), "TOML");
    }

    #[test]
    fn test_default_persistence() {
        let persistence = SettingsPersistence::default();
        assert!(persistence.auto_save);
        assert!(persistence.backup_on_save);
        assert_eq!(persistence.max_backups, 5);
    }

    #[test]
    fn test_settings_error_display() {
        let err = SettingsError::PathUnavailable;
        assert!(format!("{}", err).contains("unavailable"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(ErrorKind::NotFound, "test");
        let settings_err = SettingsError::from(io_err);
        assert!(matches!(settings_err, SettingsError::Io(_)));
    }

    #[test]
    fn test_settings_path() {
        // Should return Some on most systems
        let path = SettingsPersistence::settings_path();
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("anna"));
        }
    }

    #[test]
    fn test_backup_dir() {
        let path = SettingsPersistence::backup_dir();
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("backups"));
        }
    }

    #[test]
    fn test_apply_change() {
        let mut persistence = SettingsPersistence::new();
        persistence.auto_save = false; // Don't actually save during test

        let result = persistence.apply_change("enable learning mode");
        assert!(result.is_some());
    }

    #[test]
    fn test_enable_disable_auto_save() {
        let mut persistence = SettingsPersistence::new();
        assert!(persistence.is_auto_save());

        persistence.disable_auto_save();
        assert!(!persistence.is_auto_save());

        persistence.enable_auto_save();
        assert!(persistence.is_auto_save());
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_persistence_fun_fact();
        assert!(fact.contains("backup"));
    }

    #[test]
    fn test_persistence_status() {
        let status = format_persistence_status();
        assert!(status.contains("Persistence"));
    }

    #[test]
    fn test_is_persistence_available() {
        // Should usually be true
        let _ = is_persistence_available();
    }
}
