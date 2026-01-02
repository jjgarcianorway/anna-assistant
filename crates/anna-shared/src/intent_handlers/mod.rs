//! Intent Handlers (v0.0.417).
//!
//! Explicit, deterministic rules for each intent.
//! NO generic tutorials. NO hallucination. DIRECT answers only.
//!
//! Each handler:
//! - Defines required probes
//! - Defines exact transformation from probe data to answer
//! - Returns structured response or explicit failure

mod dispatcher;
mod disk;
mod helpers;
mod memory;
mod packages;
mod services;
mod system;
mod types;

// Re-export public API
pub use dispatcher::dispatch_handler;
pub use disk::handle_check_disk_usage;
pub use memory::{handle_check_free_ram, handle_check_swap_presence, handle_list_top_memory_processes};
pub use packages::{handle_check_package_count, handle_check_package_installed};
pub use services::handle_check_failed_services;
pub use system::{handle_check_boot_time, handle_check_uptime};
pub use types::HandlerResult;
