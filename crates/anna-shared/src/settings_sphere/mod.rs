// v0.0.741: Settings Sphere (Phase 317)
// Influence sphere for settings reach

mod types;
mod sphere;
mod registry;
mod utils;

// Re-export all public types and functions to maintain the same API
pub use types::{
    SphereType,
    SphereStatus,
    SphereConfig,
    SphereInterest,
    SphereEntity,
    SphereStats,
};

pub use sphere::SettingsSphere;
pub use registry::SphereRegistry;
pub use utils::{
    format_sphere_registry,
    is_sphere_query,
    sphere_fun_fact,
};
