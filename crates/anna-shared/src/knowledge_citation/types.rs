// v0.0.530: Knowledge Citation Tracker - Type Definitions
// Defines the source types and reliability levels for citations

use serde::{Deserialize, Serialize};

/// Type of knowledge source
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CitationSource {
    ArchWiki,
    ManPage,
    HelpCommand,
    InfoPage,
    OfficialDocs,
    LocalWiki,
    ConfigFile,
}

impl Default for CitationSource {
    fn default() -> Self {
        Self::ArchWiki
    }
}

impl std::fmt::Display for CitationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchWiki => write!(f, "Arch Wiki"),
            Self::ManPage => write!(f, "Man Page"),
            Self::HelpCommand => write!(f, "--help"),
            Self::InfoPage => write!(f, "Info Page"),
            Self::OfficialDocs => write!(f, "Official Docs"),
            Self::LocalWiki => write!(f, "Local Wiki"),
            Self::ConfigFile => write!(f, "Config File"),
        }
    }
}

/// Reliability of citation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum CitationReliability {
    Unverified,
    #[default]
    Trusted,
    Verified,
    Authoritative,
}

impl std::fmt::Display for CitationReliability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unverified => write!(f, "Unverified"),
            Self::Trusted => write!(f, "Trusted"),
            Self::Verified => write!(f, "Verified"),
            Self::Authoritative => write!(f, "Authoritative"),
        }
    }
}
