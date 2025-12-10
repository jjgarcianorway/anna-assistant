//! Interesting facts types (v0.0.291).

use serde::{Deserialize, Serialize};

/// Categories of interesting facts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactCategory {
    /// System performance (CPU, memory, disk trends)
    Performance,
    /// Hardware information (uptime, specs)
    Hardware,
    /// User patterns (usage times, favorite topics)
    UserPattern,
    /// Anna's growth (recipes learned, success rates)
    Growth,
    /// Historical milestones
    Milestone,
}

/// A single interesting fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestingFact {
    /// Category of the fact
    pub category: FactCategory,
    /// The fact text (for LLM to naturalize)
    pub fact: String,
    /// Priority (1=most interesting, 5=least)
    pub priority: u8,
}
