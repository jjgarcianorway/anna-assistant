// v0.0.677: Settings Reducer Types (Phase 253)
// Core types for settings reduction operations

use serde::{Deserialize, Serialize};

/// Reduce operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReduceOp {
    /// Count entries
    #[default]
    Count,
    /// Sum numeric values
    Sum,
    /// Average numeric values
    Average,
    /// Find minimum
    Min,
    /// Find maximum
    Max,
    /// Concatenate values
    Concat,
}

impl std::fmt::Display for ReduceOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count => write!(f, "count"),
            Self::Sum => write!(f, "sum"),
            Self::Average => write!(f, "average"),
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::Concat => write!(f, "concat"),
        }
    }
}

/// Reduce target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReduceTarget {
    /// Reduce values
    #[default]
    Values,
    /// Reduce keys
    Keys,
    /// Reduce both
    Both,
}

impl std::fmt::Display for ReduceTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Values => write!(f, "values"),
            Self::Keys => write!(f, "keys"),
            Self::Both => write!(f, "both"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_op_display() {
        assert_eq!(format!("{}", ReduceOp::Count), "count");
        assert_eq!(format!("{}", ReduceOp::Sum), "sum");
    }

    #[test]
    fn test_reduce_target_display() {
        assert_eq!(format!("{}", ReduceTarget::Values), "values");
        assert_eq!(format!("{}", ReduceTarget::Keys), "keys");
    }
}
