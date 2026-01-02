// v0.0.654: Settings Injector (Phase 230)
// Injector for inserting settings into configurations

mod types;
mod injector;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain API compatibility
pub use types::{
    InjectionResult,
    InjectionStrategy,
    InjectionType,
    InjectorConfig,
    InjectorStats,
};

pub use injector::SettingsInjector;

pub use registry::{
    SettingsInjectorRegistry,
    format_injector_registry,
};

pub use helpers::{
    is_injector_query,
    injector_fun_fact,
};
