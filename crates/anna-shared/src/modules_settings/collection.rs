//! Settings collection modules (v0.0.677-692)

#[path = "../settings_reducer/mod.rs"]
pub mod settings_reducer;
#[path = "../settings_partitioner/mod.rs"]
pub mod settings_partitioner;
#[path = "../settings_flattener/mod.rs"]
pub mod settings_flattener;
#[path = "../settings_expander/mod.rs"]
pub mod settings_expander;
#[path = "../settings_iterator/mod.rs"]
pub mod settings_iterator;
#[path = "../settings_collector/mod.rs"]
pub mod settings_collector;
#[path = "../settings_zipper/mod.rs"]
pub mod settings_zipper;
#[path = "../settings_scanner/mod.rs"]
pub mod settings_scanner;
#[path = "../settings_finder/mod.rs"]
pub mod settings_finder;
#[path = "../settings_counter/mod.rs"]
pub mod settings_counter;
#[path = "../settings_matcher/mod.rs"]
pub mod settings_matcher;
#[path = "../settings_validator/mod.rs"]
pub mod settings_validator;
#[path = "../settings_comparer/mod.rs"]
pub mod settings_comparer; // v0.0.689: Now modular
#[path = "../settings_combiner/mod.rs"]
pub mod settings_combiner;
#[path = "../settings_auditor/mod.rs"]
pub mod settings_auditor;
#[path = "../settings_chronicle/mod.rs"]
pub mod settings_chronicle;
