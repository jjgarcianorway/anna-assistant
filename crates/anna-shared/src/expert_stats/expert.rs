//! Expert definition.

use serde::{Deserialize, Serialize};

use super::level::ExpertLevel;

/// An expert in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expert {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Department/team
    pub department: String,
    /// Level (junior/senior)
    pub level: ExpertLevel,
}

impl Expert {
    /// Create new expert
    pub fn new(id: &str, name: &str, department: &str, level: ExpertLevel) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            department: department.to_string(),
            level,
        }
    }

    /// Get full title
    pub fn title(&self) -> String {
        format!("{} {} ({})", self.level.display_name(), self.department, self.name)
    }
}
