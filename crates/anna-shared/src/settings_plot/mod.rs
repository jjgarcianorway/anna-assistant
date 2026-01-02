// v0.0.758: Settings Plot (Phase 334)
// Land plot for settings allocation

mod types;
mod config;
mod survey;
mod steward;
mod stats;
mod plot;
mod registry;
mod utils;
#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{PlotType, PlotStatus};
pub use config::PlotConfig;
pub use survey::PlotSurvey;
pub use steward::PlotSteward;
pub use stats::PlotStats;
pub use plot::SettingsPlot;
pub use registry::PlotRegistry;
pub use utils::{format_plot_registry, is_plot_query, plot_fun_fact};
