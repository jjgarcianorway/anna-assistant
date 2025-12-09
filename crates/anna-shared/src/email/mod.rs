//! Email notification system for Service Desk Theatre (v0.0.206).
//!
//! Sends email notifications to users when:
//! - A new async ticket is created
//! - IT staff needs clarification (ticket pending user)
//! - Ticket is resolved with answer
//!
//! Uses the system's `sendmail` or `mail` command.
//! User can configure their email with `annactl config email user@example.com`
//!
//! v0.0.114: Added health check, auto-install, and Anna's email address.
//! v0.0.115: Replaced email inbox with file-based inbox (~/.anna/inbox)
//! v0.0.206: Modularized into domain-focused submodules.

mod config;
mod notifications;
mod system;
mod tests;

// Re-export all types and functions
pub use config::{EmailConfig, EmailHealth};
pub use notifications::{format_email, send_notification, EmailNotification};
pub use system::{
    email_package_name, inbox_path, install_email_command, is_email_available, ANNA_EMAIL,
};
