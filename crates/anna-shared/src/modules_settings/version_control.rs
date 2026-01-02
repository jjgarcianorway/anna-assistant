//! Settings version control modules (v0.0.659-676)

#[path = "../settings_restorer/mod.rs"]
pub mod settings_restorer;
#[path = "../settings_versioner/mod.rs"]
pub mod settings_versioner;
#[path = "../settings_differ/mod.rs"]
pub mod settings_differ;
#[path = "../settings_patcher/mod.rs"]
pub mod settings_patcher; // v0.0.662: Now modular
#[path = "../settings_graph/mod.rs"]
pub mod settings_graph; // v0.0.663: Now modular
#[path = "../settings_resolution/mod.rs"]
pub mod settings_resolution;
#[path = "../settings_validator_hub/mod.rs"]
pub mod settings_validator_hub;
#[path = "../settings_transform/mod.rs"]
pub mod settings_transform;
#[path = "../settings_normalization/mod.rs"]
pub mod settings_normalization;
#[path = "../settings_denormalization/mod.rs"]
pub mod settings_denormalization;
#[path = "../settings_indexer/mod.rs"]
pub mod settings_indexer;
#[path = "../settings_query_engine/mod.rs"]
pub mod settings_query_engine;
#[path = "../settings_aggregation/mod.rs"]
pub mod settings_aggregation;
#[path = "../settings_projector/mod.rs"]
pub mod settings_projector;
#[path = "../settings_selector/mod.rs"]
pub mod settings_selector;
#[path = "../settings_filter/mod.rs"]
pub mod settings_filter;
#[path = "../settings_sorter/mod.rs"]
pub mod settings_sorter;
#[path = "../settings_grouper/mod.rs"]
pub mod settings_grouper;
