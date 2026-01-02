// v0.0.653: Settings Extractor Module (Phase 229)
// Modular organization for settings extraction functionality

mod types;
mod result;
mod extractor;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{ExtractionType, ExtractionMode, ExtractorConfig};
pub use result::{ExtractionResult, ExtractorStats};
pub use extractor::SettingsExtractor;
pub use registry::{SettingsExtractorRegistry, format_extractor_registry};
pub use utils::{is_extractor_query, extractor_fun_fact};
