// v0.0.696: Settings Album (Phase 272)
// Collection album of settings snapshots

mod types;
mod config;
mod page;
mod stats;
mod album;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{AlbumType, AlbumStatus};
pub use config::AlbumConfig;
pub use page::{AlbumPage, AlbumItem};
pub use stats::AlbumStats;
pub use album::SettingsAlbum;
pub use registry::AlbumRegistry;
pub use helpers::{format_album_registry, is_album_query, album_fun_fact};
