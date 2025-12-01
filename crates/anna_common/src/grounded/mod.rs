//! Grounded Knowledge System v7.1.0
//!
//! Every piece of data has a verifiable source.
//! No invented numbers. No hallucinated descriptions.
//! No guessing config paths - only from pacman/man.
//! No hardcoded categories - from desktop/pacman/man.

pub mod packages;
pub mod commands;
pub mod services;
pub mod errors;
pub mod config;
pub mod category;

pub use packages::*;
pub use commands::*;
pub use services::*;
pub use errors::*;
pub use config::*;
pub use category::*;
