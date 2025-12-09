//! Person profile struct (v0.0.182).

use serde::{Deserialize, Serialize};

use crate::teams::Team;

use super::shift::Shift;
use super::tier::Tier;

/// A person profile in the IT department roster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonProfile {
    pub person_id: &'static str,
    pub display_name: &'static str,
    pub role_title: &'static str,
    pub team: Team,
    pub tier: Tier,
    /// v0.0.109: Specialization areas for this staff member
    #[serde(skip)]
    pub specializations: &'static [&'static str],
    /// v0.0.110: Preferred work shift
    pub shift: Shift,
}

impl PersonProfile {
    /// Get formatted display: "Name (Role Title)"
    pub fn display(&self) -> String {
        format!("{} ({})", self.display_name, self.role_title)
    }

    /// Get short display for debug: "name/team"
    pub fn debug_tag(&self) -> String {
        format!("{}/{}", self.display_name.to_lowercase(), self.team)
    }

    /// v0.0.109: Get specialization string
    pub fn specialization_str(&self) -> String {
        if self.specializations.is_empty() {
            String::new()
        } else {
            self.specializations.join(", ")
        }
    }

    /// v0.0.110: Check if this person is currently on shift
    pub fn is_on_shift(&self) -> bool {
        self.shift.is_active()
    }

    /// v0.0.110: Get availability status
    pub fn availability_status(&self) -> &'static str {
        if self.is_on_shift() {
            "on shift"
        } else {
            "off shift"
        }
    }
}
