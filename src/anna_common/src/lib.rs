//! Anna Common Library
//!
//! Shared messaging, formatting, and utilities for Anna Assistant.
//! This library provides a unified interface for human-friendly output
//! across all Anna components (installer, annactl, annad).

pub mod messaging;
pub mod config;
pub mod locale;
pub mod privilege;
pub mod config_governance;
pub mod persona;

pub use messaging::{
    anna_say, anna_info, anna_ok, anna_warn, anna_error, anna_narrative,
    anna_box, MessageType, AnnaMessage
};
pub use config::{AnnaConfig, load_config, default_config, save_config};
pub use locale::{detect_locale, format_timestamp, format_duration, LocaleInfo};
pub use privilege::{needs_privilege, request_privilege, is_root, PrivilegeRequest};
pub use config_governance::{
    ConfigValue, ConfigOrigin, EffectiveConfig, ConfigPaths,
    load_effective_config, save_effective_snapshot, set_user_config,
    get_config_value, reset_user_config, ensure_banners, CONFIG_BANNER
};
pub use persona::{
    Persona, PersonaTraits, PersonaMode, PersonaState,
    bundled_personas, get_persona_state, save_persona_state,
    set_persona, explain_persona_choice
};
