// v0.0.709: Settings Digest Types (Phase 285)
// Digest type and format enums

use serde::{Deserialize, Serialize};

/// Digest type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DigestType {
    /// Daily digest
    #[default]
    Daily,
    /// Weekly digest
    Weekly,
    /// Monthly digest
    Monthly,
    /// Custom digest
    Custom,
}

impl std::fmt::Display for DigestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Digest format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DigestFormat {
    /// Summary format
    #[default]
    Summary,
    /// Detailed format
    Detailed,
    /// Highlights format
    Highlights,
    /// Full format
    Full,
}

impl std::fmt::Display for DigestFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "summary"),
            Self::Detailed => write!(f, "detailed"),
            Self::Highlights => write!(f, "highlights"),
            Self::Full => write!(f, "full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_type_display() {
        assert_eq!(format!("{}", DigestType::Daily), "daily");
        assert_eq!(format!("{}", DigestType::Weekly), "weekly");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", DigestFormat::Summary), "summary");
        assert_eq!(format!("{}", DigestFormat::Detailed), "detailed");
    }
}
