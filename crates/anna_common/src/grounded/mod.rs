//! Grounded Knowledge System v7.2.0
//!
//! Every piece of data has a verifiable source.
//! No invented numbers. No hallucinated descriptions.
//! No guessing config paths - only from pacman/man/Arch Wiki.
//! Rule-based categories from descriptions and metadata.

pub mod packages;
pub mod commands;
pub mod services;
pub mod errors;
pub mod config;
pub mod category;
pub mod arch_wiki;
pub mod categoriser;

pub use packages::*;
pub use commands::*;
pub use services::*;
pub use errors::*;
pub use config::*;
pub use category::*;
pub use arch_wiki::*;
pub use categoriser::*;
