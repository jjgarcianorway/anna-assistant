// v0.0.651: Settings Mapper Module (Phase 227)
// Mapper for key transformations and field mapping

pub mod mapper;
pub mod registry;
pub mod types;
pub mod utils;

// Re-export types
pub use types::{
    MapperConfig, MapperStats, MappingDirection, MappingResult, MappingRule, MappingType,
};

// Re-export mapper
pub use mapper::SettingsMapper;

// Re-export registry
pub use registry::{format_mapper_registry, SettingsMapperRegistry};

// Re-export utils
pub use utils::{is_mapper_query, mapper_fun_fact};
