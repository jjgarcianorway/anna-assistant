//! Anna Common Library
//!
//! Shared messaging, formatting, and utilities for Anna Assistant.
//! This library provides a unified interface for human-friendly output
//! across all Anna components (installer, annactl, annad).

pub mod messaging;
pub mod config;
pub mod locale;
pub mod privilege;

pub use messaging::{
    anna_say, anna_info, anna_ok, anna_warn, anna_error, anna_narrative,
    anna_box, MessageType, AnnaMessage
};
pub use config::{AnnaConfig, load_config, default_config, save_config};
pub use locale::{detect_locale, format_timestamp, format_duration, LocaleInfo};
pub use privilege::{needs_privilege, request_privilege, is_root, PrivilegeRequest};
