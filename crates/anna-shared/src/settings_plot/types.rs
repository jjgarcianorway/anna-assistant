// v0.0.758: Settings Plot (Phase 334)
// Plot type and status enums

use serde::{Deserialize, Serialize};

/// Plot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlotType {
    /// Garden plot
    #[default]
    Garden,
    /// Building plot
    Building,
    /// Cemetery plot
    Cemetery,
    /// Allotment plot
    Allotment,
}

impl std::fmt::Display for PlotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Garden => write!(f, "garden"),
            Self::Building => write!(f, "building"),
            Self::Cemetery => write!(f, "cemetery"),
            Self::Allotment => write!(f, "allotment"),
        }
    }
}

/// Plot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlotStatus {
    /// Allocated status
    #[default]
    Allocated,
    /// Cultivated status
    Cultivated,
    /// Fallow status
    Fallow,
    /// Reserved status
    Reserved,
}

impl std::fmt::Display for PlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allocated => write!(f, "allocated"),
            Self::Cultivated => write!(f, "cultivated"),
            Self::Fallow => write!(f, "fallow"),
            Self::Reserved => write!(f, "reserved"),
        }
    }
}
