// v0.0.696: Settings Album (Phase 272)
// Album registry for managing multiple albums

use std::collections::HashMap;
use super::album::SettingsAlbum;

/// Album registry
#[derive(Debug, Clone, Default)]
pub struct AlbumRegistry {
    /// Albums by ID
    albums: HashMap<String, SettingsAlbum>,
}

impl AlbumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register album
    pub fn register(&mut self, id: impl Into<String>, album: SettingsAlbum) {
        self.albums.insert(id.into(), album);
    }

    /// Unregister album
    pub fn unregister(&mut self, id: &str) -> bool {
        self.albums.remove(id).is_some()
    }

    /// Get album
    pub fn get(&self, id: &str) -> Option<&SettingsAlbum> {
        self.albums.get(id)
    }

    /// Get album mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAlbum> {
        self.albums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.albums.len()
    }
}
