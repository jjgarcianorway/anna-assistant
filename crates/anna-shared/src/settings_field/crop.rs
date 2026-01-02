// v0.0.762: Settings Field Crop (Phase 338)
// Field crop management

use serde::{Deserialize, Serialize};

/// Field crop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCrop {
    /// Crop ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Furrow number
    pub furrow: u32,
    /// Yielded
    pub yielded: bool,
}

impl FieldCrop {
    /// Create new crop
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            furrow: 0,
            yielded: true,
        }
    }

    /// Set furrow
    pub fn furrow(mut self, f: u32) -> Self {
        self.furrow = f;
        self
    }

    /// Make yielded
    pub fn make_yielded(&mut self) {
        self.yielded = true;
    }

    /// Make barren
    pub fn make_barren(&mut self) {
        self.yielded = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crop_new() {
        let c = FieldCrop::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_crop_builder() {
        let c = FieldCrop::new("c1", "Title", "Content")
            .furrow(1);
        assert_eq!(c.furrow, 1);
    }

    #[test]
    fn test_crop_yielded() {
        let mut c = FieldCrop::new("c1", "Title", "Content");
        c.make_barren();
        assert!(!c.yielded);
        c.make_yielded();
        assert!(c.yielded);
    }
}
