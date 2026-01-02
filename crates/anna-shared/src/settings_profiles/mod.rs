// v0.0.565: Settings Profiles (Phase 141)
// Named settings configurations that users can switch between

mod types;
mod manager;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{ProfileId, ProfileMeta, SettingsProfile};
pub use manager::ProfileManager;
pub use utils::{format_profiles_list, is_profile_query, profiles_fun_fact};
