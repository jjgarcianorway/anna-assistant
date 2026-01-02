// v0.0.710: Settings Brief - Types (Phase 286)
// Brief types, scopes, and data structures

use serde::{Deserialize, Serialize};

/// Brief type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BriefType {
    /// Executive brief
    #[default]
    Executive,
    /// Technical brief
    Technical,
    /// Operational brief
    Operational,
    /// Strategic brief
    Strategic,
}

impl std::fmt::Display for BriefType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executive => write!(f, "executive"),
            Self::Technical => write!(f, "technical"),
            Self::Operational => write!(f, "operational"),
            Self::Strategic => write!(f, "strategic"),
        }
    }
}

/// Brief scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BriefScope {
    /// Department scope
    #[default]
    Department,
    /// Organization scope
    Organization,
    /// Project scope
    Project,
    /// System scope
    System,
}

impl std::fmt::Display for BriefScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Department => write!(f, "department"),
            Self::Organization => write!(f, "organization"),
            Self::Project => write!(f, "project"),
            Self::System => write!(f, "system"),
        }
    }
}

/// Brief config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefConfig {
    /// Name
    pub name: String,
    /// Brief type
    pub brief_type: BriefType,
    /// Scope
    pub scope: BriefScope,
    /// Max points
    pub max_points: usize,
}

impl BriefConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            brief_type: BriefType::Executive,
            scope: BriefScope::Department,
            max_points: 25,
        }
    }

    /// Set type
    pub fn brief_type(mut self, bt: BriefType) -> Self {
        self.brief_type = bt;
        self
    }

    /// Set scope
    pub fn scope(mut self, s: BriefScope) -> Self {
        self.scope = s;
        self
    }

    /// Set max points
    pub fn max_points(mut self, max: usize) -> Self {
        self.max_points = max;
        self
    }
}

impl Default for BriefConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Brief point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefPoint {
    /// Point ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Priority
    pub priority: u8,
    /// Action required
    pub action_required: bool,
}

impl BriefPoint {
    /// Create new point
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            priority: 1,
            action_required: false,
        }
    }

    /// Set description
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    /// Set priority
    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    /// Set action required
    pub fn action_required(mut self, ar: bool) -> Self {
        self.action_required = ar;
        self
    }
}

/// Brief attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefAttachment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Point ID
    pub point_id: String,
}

impl BriefAttachment {
    /// Create new attachment
    pub fn new(key: impl Into<String>, value: impl Into<String>, point_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            point_id: point_id.into(),
        }
    }
}
