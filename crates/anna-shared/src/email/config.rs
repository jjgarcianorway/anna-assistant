//! Email configuration (v0.0.206).

use std::fs;
use std::path::PathBuf;

use super::system::{
    count_inbox_queries, email_package_name, inbox_path, install_email_command, is_email_available,
};

/// Email health status
#[derive(Debug, Clone)]
pub struct EmailHealth {
    /// Is email sending available?
    pub can_send: bool,
    /// Package name needed
    pub package_name: &'static str,
    /// Install command
    pub install_cmd: String,
    /// User's configured email
    pub user_email: Option<String>,
    /// Inbox path for async queries
    pub inbox_path: PathBuf,
    /// Does inbox exist?
    pub inbox_exists: bool,
    /// Pending queries in inbox
    pub inbox_count: usize,
}

impl EmailHealth {
    /// Check email system health
    pub fn check() -> Self {
        let config = EmailConfig::load();
        let inbox = inbox_path();
        let inbox_exists = inbox.exists();
        let inbox_count = if inbox_exists {
            count_inbox_queries(&inbox)
        } else {
            0
        };
        Self {
            can_send: is_email_available(),
            package_name: email_package_name(),
            install_cmd: install_email_command(),
            user_email: config.user_email,
            inbox_path: inbox,
            inbox_exists,
            inbox_count,
        }
    }

    /// Is everything ready for email notifications?
    pub fn is_ready(&self) -> bool {
        self.can_send && self.user_email.is_some()
    }
}

/// Email configuration
#[derive(Debug, Clone, Default)]
pub struct EmailConfig {
    /// User's email address
    pub user_email: Option<String>,
    /// Send email notifications
    pub enabled: bool,
}

impl EmailConfig {
    /// Load email config from disk
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => {
                let email = content.trim().to_string();
                if email.is_empty() || !email.contains('@') {
                    Self::default()
                } else {
                    Self {
                        user_email: Some(email),
                        enabled: true,
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }

    /// Save email config
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(ref email) = self.user_email {
            fs::write(&path, email)?;
        }
        Ok(())
    }

    /// Config file path
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".anna").join("email.conf")
    }

    /// Set email address
    pub fn set_email(&mut self, email: &str) {
        self.user_email = Some(email.to_string());
        self.enabled = true;
    }

    /// Clear email (disable notifications)
    pub fn clear(&mut self) {
        self.user_email = None;
        self.enabled = false;
    }
}
