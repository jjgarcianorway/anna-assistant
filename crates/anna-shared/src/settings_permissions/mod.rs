// v0.0.582: Settings Permissions (Phase 158)
// Access control and permissions for settings

mod types;
mod role;
mod manager;
mod utils;

// Re-export all public types to preserve the original API
pub use types::{
    PermissionLevel,
    PermissionAction,
    PermissionResult,
    CategoryPermission,
};

pub use role::PermissionRole;
pub use manager::PermissionManager;

pub use utils::{
    format_permissions,
    is_permissions_query,
    settings_permissions_fun_fact,
};
