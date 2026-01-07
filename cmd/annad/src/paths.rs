//! Mode-aware path resolution for Anna daemon
//!
//! This module provides a unified interface for resolving paths based on the
//! installation mode (user vs system). Mode detection happens early and all
//! subsequent path operations use the detected mode.

use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};

/// Installation mode for Anna daemon
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// User mode: data in ~/.anna/, no root required
    User,
    /// System mode: data in /var/lib/anna/, requires root
    System,
}

impl Mode {
    /// Detect installation mode based on environment and filesystem
    pub fn detect() -> Self {
        // Check ANNA_MODE env var first (explicit override)
        if let Ok(mode) = env::var("ANNA_MODE") {
            if mode == "user" {
                return Mode::User;
            } else if mode == "system" {
                return Mode::System;
            }
        }

        // Auto-detect based on what exists
        let system_data = PathBuf::from("/var/lib/anna");
        let system_config = PathBuf::from("/etc/anna");

        // Check if we're running as root or system paths exist
        if nix::unistd::Uid::effective().is_root()
            || (system_data.exists() && system_config.exists())
        {
            return Mode::System;
        }

        // Default to user mode
        Mode::User
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::User => "user",
            Mode::System => "system",
        }
    }
}

/// Path resolver for mode-aware path operations
pub struct Paths {
    mode: Mode,
    data_root: PathBuf,
    config_root: PathBuf,
    runtime_root: PathBuf,
}

impl Paths {
    /// Create a new path resolver for the detected mode
    pub fn new(mode: Mode) -> Result<Self> {
        let (data_root, config_root, runtime_root) = match mode {
            Mode::System => (
                PathBuf::from("/var/lib/anna"),
                PathBuf::from("/etc/anna"),
                PathBuf::from("/run/anna"),
            ),
            Mode::User => {
                let home = env::var("HOME").context("HOME not set")?;
                let home = PathBuf::from(home);
                (
                    home.join(".anna/data"),
                    home.join(".anna/config"),
                    Self::get_user_runtime_dir(&home)?,
                )
            }
        };

        Ok(Self {
            mode,
            data_root,
            config_root,
            runtime_root,
        })
    }

    /// Get user runtime directory (XDG_RUNTIME_DIR or ~/.anna/run)
    fn get_user_runtime_dir(home: &PathBuf) -> Result<PathBuf> {
        if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
            Ok(PathBuf::from(xdg_runtime).join("anna"))
        } else {
            Ok(home.join(".anna/run"))
        }
    }

    #[allow(dead_code)]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Get the root data directory
    pub fn data_root(&self) -> &PathBuf {
        &self.data_root
    }

    /// Get the config directory
    #[allow(dead_code)]
    pub fn config_root(&self) -> &PathBuf {
        &self.config_root
    }

    /// Get the runtime directory (for sockets, pids)
    #[allow(dead_code)]
    pub fn runtime_root(&self) -> &PathBuf {
        &self.runtime_root
    }

    /// Get the socket path for RPC
    pub fn socket(&self) -> PathBuf {
        self.runtime_root.join("annad.sock")
    }

    /// Get the config file path
    pub fn config_file(&self) -> PathBuf {
        self.config_root.join("config.toml")
    }

    /// Get the plans directory
    pub fn plans_dir(&self) -> PathBuf {
        self.data_root.join("plans")
    }

    /// Get the advice directory
    pub fn advice_dir(&self) -> PathBuf {
        self.data_root.join("advice")
    }

    /// Get the persona directory
    pub fn persona_dir(&self) -> PathBuf {
        self.data_root.join("persona")
    }

    /// Get the quickscan reports directory
    pub fn quickscan_dir(&self) -> PathBuf {
        self.data_root.join("quickscan")
    }

    /// Get the signals directory
    pub fn signals_dir(&self) -> PathBuf {
        self.data_root.join("signals")
    }

    /// Get the system snapshot file
    pub fn system_snapshot(&self) -> PathBuf {
        self.data_root.join("system.json")
    }

    /// Ensure all required directories exist with proper permissions
    pub fn ensure_dirs(&self) -> Result<()> {
        // Create runtime dir (for socket)
        fs::create_dir_all(&self.runtime_root)
            .with_context(|| format!("create runtime dir: {}", self.runtime_root.display()))?;

        // Create data dirs
        fs::create_dir_all(&self.data_root)
            .with_context(|| format!("create data root: {}", self.data_root.display()))?;
        fs::create_dir_all(self.plans_dir())
            .with_context(|| format!("create plans dir: {}", self.plans_dir().display()))?;
        fs::create_dir_all(self.advice_dir())
            .with_context(|| format!("create advice dir: {}", self.advice_dir().display()))?;
        fs::create_dir_all(self.persona_dir())
            .with_context(|| format!("create persona dir: {}", self.persona_dir().display()))?;
        fs::create_dir_all(self.quickscan_dir()).with_context(|| {
            format!(
                "create quickscan dir: {}",
                self.quickscan_dir().display()
            )
        })?;

        // Config dir (may not be writable in system mode)
        if self.mode == Mode::User {
            fs::create_dir_all(&self.config_root).with_context(|| {
                format!("create config root: {}", self.config_root.display())
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_detection() {
        // Should default to user mode when not root and system dirs don't exist
        let mode = Mode::detect();
        assert_eq!(mode, Mode::User);
    }

    #[test]
    fn test_user_paths() {
        env::set_var("HOME", "/home/testuser");
        env::set_var("ANNA_MODE", "user");

        let paths = Paths::new(Mode::User).unwrap();
        assert_eq!(paths.mode(), Mode::User);
        assert!(paths.data_root().to_str().unwrap().contains(".anna"));
        assert!(paths.socket().to_str().unwrap().ends_with("annad.sock"));
    }

    #[test]
    fn test_system_paths() {
        let paths = Paths::new(Mode::System).unwrap();
        assert_eq!(paths.mode(), Mode::System);
        assert_eq!(paths.data_root(), &PathBuf::from("/var/lib/anna"));
        assert_eq!(paths.socket(), PathBuf::from("/run/anna/annad.sock"));
    }
}
