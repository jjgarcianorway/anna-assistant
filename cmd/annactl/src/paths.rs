use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    System,
    User,
}

impl InstallMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallMode::System => "system",
            InstallMode::User => "user",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnnaPaths {
    pub mode: InstallMode,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub advice_dir: PathBuf,
    #[allow(dead_code)]
    pub persona_dir: PathBuf,
    #[allow(dead_code)]
    pub signals_dir: PathBuf,
    #[allow(dead_code)]
    pub profiles_dir: PathBuf,
    pub socket_path: PathBuf,
}

impl AnnaPaths {
    /// Detect install mode and return appropriate paths for the current user
    pub fn detect() -> Self {
        let uid = nix::unistd::Uid::effective();
        Self::detect_for_uid(uid.as_raw())
    }

    /// Detect install mode and return paths for a specific UID
    pub fn detect_for_uid(uid: u32) -> Self {
        // Check ANNA_MODE env var first (for dev/testing)
        if let Ok(mode) = env::var("ANNA_MODE") {
            return match mode.as_str() {
                "user" => Self::user_for_uid(uid),
                "system" => Self::system_for_uid(uid),
                _ => Self::auto_detect_for_uid(uid),
            };
        }

        Self::auto_detect_for_uid(uid)
    }

    fn auto_detect_for_uid(uid: u32) -> Self {
        // Check if system paths exist and are accessible
        let system_data = PathBuf::from("/var/lib/anna");
        let system_config = PathBuf::from("/etc/anna");
        let socket_dir = PathBuf::from("/run/anna");

        // If system paths exist, we're in system mode
        if system_data.exists() && system_config.exists() {
            return Self::system_for_uid(uid);
        }

        // Check if socket exists (indicates system mode daemon)
        if socket_dir.join("annad.sock").exists() {
            return Self::system_for_uid(uid);
        }

        // Otherwise, check for user mode installation
        if let Ok(home) = Self::get_home_for_uid(uid) {
            let user_data = home.join(".anna/data");
            let user_config = home.join(".anna/config");

            // If user paths exist, we're in user mode
            if user_data.exists() || user_config.exists() {
                return Self::user_for_uid(uid);
            }
        }

        // Default: system mode (S1 milestone default)
        Self::system_for_uid(uid)
    }

    /// Get paths for system mode with per-user data
    pub fn system_for_uid(uid: u32) -> Self {
        let config_dir = PathBuf::from("/etc/anna");
        let user_root = PathBuf::from(format!("/var/lib/anna/users/{}", uid));
        let socket_path = PathBuf::from("/run/anna/annad.sock");

        Self {
            mode: InstallMode::System,
            data_dir: user_root.clone(),
            config_dir,
            reports_dir: user_root.join("reports"),
            advice_dir: user_root.join("advice"),
            persona_dir: user_root.join("persona"),
            signals_dir: user_root.join("signals"),
            profiles_dir: user_root.join("profiles"),
            socket_path,
        }
    }

    /// Get paths for user mode
    pub fn user_for_uid(uid: u32) -> Self {
        let home = Self::get_home_for_uid(uid).expect("Cannot determine home directory");
        let data_dir = home.join(".anna/data");
        let config_dir = home.join(".anna/config");

        // Prefer XDG_RUNTIME_DIR for socket, fallback to ~/.anna/run
        let socket_path = Self::resolve_user_socket_path(uid);

        Self {
            mode: InstallMode::User,
            data_dir: data_dir.clone(),
            config_dir,
            reports_dir: data_dir.join("reports"),
            advice_dir: data_dir.join("advice"),
            persona_dir: data_dir.join("persona"),
            signals_dir: data_dir.join("signals"),
            profiles_dir: data_dir.join("profiles"),
            socket_path,
        }
    }

    /// Resolve user-mode socket path with XDG_RUNTIME_DIR preference
    pub fn resolve_user_socket_path(uid: u32) -> PathBuf {
        // Try XDG_RUNTIME_DIR first
        if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
            let xdg_socket = PathBuf::from(xdg_runtime).join("anna/annad.sock");
            return xdg_socket;
        }

        // For non-current UID, try standard path /run/user/<uid>
        let current_uid = nix::unistd::Uid::effective().as_raw();
        if uid != current_uid {
            let runtime_dir = PathBuf::from(format!("/run/user/{}", uid));
            if runtime_dir.exists() {
                return runtime_dir.join("anna/annad.sock");
            }
        }

        // Fallback to ~/.anna/run
        if let Ok(home) = Self::get_home_for_uid(uid) {
            home.join(".anna/run/annad.sock")
        } else {
            PathBuf::from(".anna/run/annad.sock")
        }
    }

    /// Try to find an existing socket in user mode (search order)
    pub fn find_user_socket() -> Option<PathBuf> {
        let candidates = vec![
            // 1. XDG_RUNTIME_DIR (preferred)
            env::var("XDG_RUNTIME_DIR")
                .ok()
                .map(|dir| PathBuf::from(dir).join("anna/annad.sock")),
            // 2. ~/.anna/run (fallback)
            dirs::home_dir().map(|home| home.join(".anna/run/annad.sock")),
        ];

        candidates.into_iter().flatten().find(|path| path.exists())
    }

    /// Legacy methods for backward compatibility
    #[allow(dead_code)]
    pub fn system() -> Self {
        let uid = nix::unistd::Uid::effective();
        Self::system_for_uid(uid.as_raw())
    }

    #[allow(dead_code)]
    pub fn user() -> Self {
        let uid = nix::unistd::Uid::effective();
        Self::user_for_uid(uid.as_raw())
    }

    /// Get home directory for a given UID
    fn get_home_for_uid(uid: u32) -> Result<PathBuf, std::io::Error> {
        // If it's the current user, use dirs crate
        if uid == nix::unistd::Uid::effective().as_raw() {
            return dirs::home_dir().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "home not found")
            });
        }

        // For other UIDs, look up via passwd
        use nix::unistd::{Uid, User};
        let user = User::from_uid(Uid::from_raw(uid))
            .map_err(std::io::Error::other)?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "user not found"))?;

        Ok(user.dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_paths_for_uid() {
        let paths = AnnaPaths::system_for_uid(1000);
        assert_eq!(paths.mode, InstallMode::System);
        assert_eq!(paths.data_dir, PathBuf::from("/var/lib/anna/users/1000"));
        assert_eq!(paths.config_dir, PathBuf::from("/etc/anna"));
        assert_eq!(
            paths.reports_dir,
            PathBuf::from("/var/lib/anna/users/1000/reports")
        );
        assert_eq!(
            paths.advice_dir,
            PathBuf::from("/var/lib/anna/users/1000/advice")
        );
        assert_eq!(
            paths.persona_dir,
            PathBuf::from("/var/lib/anna/users/1000/persona")
        );
        assert_eq!(
            paths.signals_dir,
            PathBuf::from("/var/lib/anna/users/1000/signals")
        );
        assert_eq!(
            paths.profiles_dir,
            PathBuf::from("/var/lib/anna/users/1000/profiles")
        );
        assert_eq!(paths.socket_path, PathBuf::from("/run/anna/annad.sock"));
    }

    #[test]
    fn test_user_paths_for_uid() {
        // Use current UID for this test since we need actual home dir
        let uid = nix::unistd::Uid::effective().as_raw();
        let paths = AnnaPaths::user_for_uid(uid);
        assert_eq!(paths.mode, InstallMode::User);
        assert!(paths.data_dir.ends_with(".anna/data"));
        assert!(paths.config_dir.ends_with(".anna/config"));
        assert!(paths.reports_dir.ends_with(".anna/data/reports"));
        assert!(paths.advice_dir.ends_with(".anna/data/advice"));
        assert!(paths.persona_dir.ends_with(".anna/data/persona"));
        assert!(paths.signals_dir.ends_with(".anna/data/signals"));
        assert!(paths.profiles_dir.ends_with(".anna/data/profiles"));
        assert!(paths.socket_path.ends_with(".anna/run/annad.sock"));
    }

    #[test]
    fn test_install_mode_as_str() {
        assert_eq!(InstallMode::System.as_str(), "system");
        assert_eq!(InstallMode::User.as_str(), "user");
    }

    #[test]
    fn test_env_var_override() {
        // Test ANNA_MODE=system
        std::env::set_var("ANNA_MODE", "system");
        let paths = AnnaPaths::detect_for_uid(1000);
        assert_eq!(paths.mode, InstallMode::System);
        std::env::remove_var("ANNA_MODE");

        // Test ANNA_MODE=user
        std::env::set_var("ANNA_MODE", "user");
        let uid = nix::unistd::Uid::effective().as_raw();
        let paths = AnnaPaths::detect_for_uid(uid);
        assert_eq!(paths.mode, InstallMode::User);
        std::env::remove_var("ANNA_MODE");
    }
}
