//! System information answer functions (v0.0.212).
//!
//! Uptime, timezone, hostname, OS, architecture, locale, process tree.
//!
//! v0.0.212: Modularized into domain-focused submodules.

mod diagnostics;
mod host;
mod locale;
mod packages;
mod processes;
mod resources;
mod time;

// Re-export all answer functions
pub use diagnostics::{answer_coredump_list, answer_tmp_files, answer_virtualization_info};
pub use host::{answer_hostname, answer_os_info, answer_system_architecture};
pub use locale::answer_system_locale;
pub use packages::answer_package_updates;
pub use processes::answer_process_tree;
pub use resources::{answer_open_files, answer_swap_info, answer_system_load};
pub use time::{answer_last_boot, answer_system_uptime, answer_timezone_info};
