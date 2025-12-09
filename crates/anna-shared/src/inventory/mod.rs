//! Installed software inventory cache (v0.0.188).
//!
//! Caches information about installed tools to prevent asking about
//! non-installed options and to speed up clarification flows.
//!
//! v0.0.41: Added SystemInfo (hostname, user, arch, kernel, package_count,
//! desktops, gpu_present). TTL reduced to 10 minutes for faster updates.
//!
//! v0.0.188: Modularized into domain-focused submodules.
//!
//! Storage: ~/.anna/inventory.json

mod cache;
mod constants;
mod helpers;
mod persistence;
mod system_info;
mod tests;
mod types;

// Re-export main types and functions
pub use cache::{is_inventory_fresh, InventoryCache};
pub use constants::{DESKTOP_PACKAGES, INVENTORY_TTL_SECS, VIP_TOOLS};
pub use helpers::check_tool_installed;
pub use persistence::{
    clear_inventory, filter_installed_options, inventory_path, load_inventory,
    load_or_create_inventory, save_inventory,
};
pub use system_info::SystemInfo;
pub use types::{InventoryItem, InventoryState};
