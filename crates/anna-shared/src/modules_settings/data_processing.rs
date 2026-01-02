//! Settings data processing modules (v0.0.643-658)

#[path = "../settings_sanitizer/mod.rs"]
pub mod settings_sanitizer;
#[path = "../settings_formatter/mod.rs"]
pub mod settings_formatter;
#[path = "../settings_normalizer/mod.rs"]
pub mod settings_normalizer;
#[path = "../settings_parser/mod.rs"]
pub mod settings_parser;
#[path = "../settings_renderer/mod.rs"]
pub mod settings_renderer;
#[path = "../settings_encoder/mod.rs"]
pub mod settings_encoder;
#[path = "../settings_decoder/mod.rs"]
pub mod settings_decoder;
#[path = "../settings_converter/mod.rs"]
pub mod settings_converter;
#[path = "../settings_mapper/mod.rs"]
pub mod settings_mapper;
#[path = "../settings_binder/mod.rs"]
pub mod settings_binder;
#[path = "../settings_extractor/mod.rs"]
pub mod settings_extractor;
#[path = "../settings_injector/mod.rs"]
pub mod settings_injector; // v0.0.654: Now modular
#[path = "../settings_merger.rs"]
pub mod settings_merger;
#[path = "../settings_splitter/mod.rs"]
pub mod settings_splitter;
#[path = "../settings_cloner/mod.rs"]
pub mod settings_cloner;
#[path = "../settings_archiver/mod.rs"]
pub mod settings_archiver; // v0.0.658: Now modular
