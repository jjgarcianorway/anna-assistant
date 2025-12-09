//! Deterministic answer types (v0.0.176).

/// Result from deterministic answerer with metadata
pub struct DeterministicResult {
    pub answer: String,
    #[allow(dead_code)]
    pub grounded: bool,
    pub parsed_data_count: usize, // Number of parsed entries (0 = empty)
    pub route_class: String,      // Query class used for routing (for trace)
}
