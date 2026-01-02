// v0.0.646: Settings Parser Module (Phase 222)
// Parser for parsing settings from various formats

mod types;
mod config;
mod result;
mod parser;
mod registry;
mod helpers;

// Re-export all public types
pub use types::{ParseSource, ParseMode, ParseError};
pub use config::ParserConfig;
pub use result::{ParseResult, ParserStats};
pub use parser::SettingsParser;
pub use registry::{SettingsParserRegistry, format_parser_registry};
pub use helpers::{is_parser_query, parser_fun_fact};
