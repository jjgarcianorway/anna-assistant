// v0.0.678: Settings Partitioner Types
// Partition strategies and predicate types

use serde::{Deserialize, Serialize};

/// Partition strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PartitionStrategy {
    /// Partition by predicate
    #[default]
    ByPredicate,
    /// Partition by count
    ByCount,
    /// Partition by percentage
    ByPercentage,
    /// Partition by hash
    ByHash,
}

impl std::fmt::Display for PartitionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByPredicate => write!(f, "by_predicate"),
            Self::ByCount => write!(f, "by_count"),
            Self::ByPercentage => write!(f, "by_percentage"),
            Self::ByHash => write!(f, "by_hash"),
        }
    }
}

/// Partition predicate type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PredicateType {
    /// Is numeric value
    #[default]
    IsNumeric,
    /// Is non-empty
    IsNonEmpty,
    /// Key contains pattern
    KeyContains,
    /// Value contains pattern
    ValueContains,
}

impl std::fmt::Display for PredicateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsNumeric => write!(f, "is_numeric"),
            Self::IsNonEmpty => write!(f, "is_non_empty"),
            Self::KeyContains => write!(f, "key_contains"),
            Self::ValueContains => write!(f, "value_contains"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_strategy_display() {
        assert_eq!(format!("{}", PartitionStrategy::ByPredicate), "by_predicate");
        assert_eq!(format!("{}", PartitionStrategy::ByCount), "by_count");
    }

    #[test]
    fn test_predicate_type_display() {
        assert_eq!(format!("{}", PredicateType::IsNumeric), "is_numeric");
        assert_eq!(format!("{}", PredicateType::IsNonEmpty), "is_non_empty");
    }
}
