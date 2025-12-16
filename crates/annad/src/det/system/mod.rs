//! System answer functions (v0.0.805).
//!
//! Handles package updates, swap, timezone, uptime, users, battery, load, boot,
//! hostname, OS info, network connectivity, filesystems, and USB devices.
//!
//! v0.0.187: Modularized into domain-focused submodules.

mod hardware;
mod info;
mod network;
mod packages;
mod time;
mod users;

// Re-export all answer functions
pub use hardware::{
    answer_battery_status, answer_screen_resolution, answer_swap_info, answer_system_load,
    answer_usb_devices, answer_webcam_status,
};
pub use info::{answer_device_type, answer_hostname, answer_os_info};
pub use network::{answer_mounted_filesystems, answer_network_connectivity};
pub use packages::answer_package_updates;
pub use time::{answer_boot_blame, answer_last_boot, answer_system_uptime, answer_timezone_info};
pub use users::answer_logged_in_users;
