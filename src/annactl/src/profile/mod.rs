//! System profiling subsystem

pub mod collector;
pub mod checks;
pub mod render;

pub use collector::ProfileCollector;
pub use checks::run_checks;
pub use render::render_profile;
