// v0.0.762: Settings Field Types (Phase 338)
// Field type and status enums

use serde::{Deserialize, Serialize};

/// Field type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FieldType {
    /// Arable field
    #[default]
    Arable,
    /// Pastoral field
    Pastoral,
    /// Fallow field
    Fallow,
    /// Orchard field
    Orchard,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arable => write!(f, "arable"),
            Self::Pastoral => write!(f, "pastoral"),
            Self::Fallow => write!(f, "fallow"),
            Self::Orchard => write!(f, "orchard"),
        }
    }
}

/// Field status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FieldStatus {
    /// Prepared status
    #[default]
    Prepared,
    /// Planted status
    Planted,
    /// Growing status
    Growing,
    /// Harvested status
    Harvested,
}

impl std::fmt::Display for FieldStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prepared => write!(f, "prepared"),
            Self::Planted => write!(f, "planted"),
            Self::Growing => write!(f, "growing"),
            Self::Harvested => write!(f, "harvested"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_display() {
        assert_eq!(format!("{}", FieldType::Arable), "arable");
        assert_eq!(format!("{}", FieldType::Pastoral), "pastoral");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", FieldStatus::Prepared), "prepared");
        assert_eq!(format!("{}", FieldStatus::Harvested), "harvested");
    }
}
