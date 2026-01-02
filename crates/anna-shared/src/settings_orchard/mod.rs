// v0.0.766: Settings Orchard (Phase 342)
// Fruit orchard for settings horticulture

mod types;
mod config;
mod fruit;
mod picker;
mod stats;
mod orchard;
mod registry;
#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{OrchardType, OrchardStatus};
pub use config::OrchardConfig;
pub use fruit::OrchardFruit;
pub use picker::OrchardPicker;
pub use stats::OrchardStats;
pub use orchard::SettingsOrchard;
pub use registry::{OrchardRegistry, format_orchard_registry, is_orchard_query, orchard_fun_fact};
