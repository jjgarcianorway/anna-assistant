//! Anna Common Library
//!
//! Shared messaging, formatting, and utilities for Anna Assistant.
//! This library provides a unified interface for human-friendly output
//! across all Anna components (installer, annactl, annad).

pub mod config;
pub mod config_governance;
pub mod locale;
pub mod messaging;
pub mod persona;
pub mod privilege;
pub mod tui;

pub use config::{default_config, load_config, save_config, AnnaConfig};
pub use config_governance::{
    ensure_banners, get_config_value, load_effective_config, reset_user_config,
    save_effective_snapshot, set_user_config, ConfigOrigin, ConfigPaths, ConfigValue,
    EffectiveConfig, CONFIG_BANNER,
};
pub use locale::{detect_locale, format_duration, format_timestamp, LocaleInfo};
pub use messaging::{
    anna_box, anna_error, anna_info, anna_narrative, anna_ok, anna_say, anna_warn, AnnaMessage,
    MessageType,
};
pub use persona::{
    bundled_personas, explain_persona_choice, get_persona_state, save_persona_state, set_persona,
    Persona, PersonaMode, PersonaState, PersonaTraits,
};
pub use privilege::{is_root, needs_privilege, request_privilege, PrivilegeRequest};
pub use tui::{
    accent, bullet, code, dim, err, header, hint, kv, ok, primary, progress, progress_bar, section,
    status, table, warn, Level, TermCaps,
};
