// v0.0.683: Settings Zipper (Phase 259)
// Zip and unzip settings collections together

mod types;
mod config;
mod zipper;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{
    ZipMode,
    UnzipMode,
    ZippedPair,
    ZipResult,
    UnzipResult,
    ZipperStats,
};

pub use config::ZipperConfig;
pub use zipper::SettingsZipper;
pub use registry::{ZipperRegistry, format_zipper_registry};
pub use utils::{is_zipper_query, zipper_fun_fact};
