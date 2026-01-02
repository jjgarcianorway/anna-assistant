// v0.0.757: Settings Parcel - Types (Phase 333)

use serde::{Deserialize, Serialize};

/// Parcel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ParcelType {
    /// Fee simple parcel
    #[default]
    FeeSimple,
    /// Leasehold parcel
    Leasehold,
    /// Easement parcel
    Easement,
    /// Right-of-way parcel
    RightOfWay,
}

impl std::fmt::Display for ParcelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FeeSimple => write!(f, "fee-simple"),
            Self::Leasehold => write!(f, "leasehold"),
            Self::Easement => write!(f, "easement"),
            Self::RightOfWay => write!(f, "right-of-way"),
        }
    }
}

/// Parcel status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ParcelStatus {
    /// Platted status
    #[default]
    Platted,
    /// Conveyed status
    Conveyed,
    /// Encumbered status
    Encumbered,
    /// Cleared status
    Cleared,
}

impl std::fmt::Display for ParcelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Platted => write!(f, "platted"),
            Self::Conveyed => write!(f, "conveyed"),
            Self::Encumbered => write!(f, "encumbered"),
            Self::Cleared => write!(f, "cleared"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parcel_type_display() {
        assert_eq!(format!("{}", ParcelType::FeeSimple), "fee-simple");
        assert_eq!(format!("{}", ParcelType::Leasehold), "leasehold");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ParcelStatus::Platted), "platted");
        assert_eq!(format!("{}", ParcelStatus::Conveyed), "conveyed");
    }
}
