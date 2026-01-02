//! Knowledge type and source enums

use serde::{Deserialize, Serialize};

/// Type of knowledge entry
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeType {
    /// Learned recipe
    Recipe,
    /// Stored fact
    Fact,
    /// Cached wiki page
    WikiPage,
    /// Cached man page
    ManPage,
    /// Cached help output
    HelpCache,
    /// User-taught pattern
    UserTaught,
}

impl KnowledgeType {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Recipe => "Recipe",
            Self::Fact => "Fact",
            Self::WikiPage => "Wiki Page",
            Self::ManPage => "Man Page",
            Self::HelpCache => "Help Cache",
            Self::UserTaught => "User Taught",
        }
    }

    /// Plural display name
    pub fn display_plural(&self) -> &'static str {
        match self {
            Self::Recipe => "Recipes",
            Self::Fact => "Facts",
            Self::WikiPage => "Wiki Pages",
            Self::ManPage => "Man Pages",
            Self::HelpCache => "Help Caches",
            Self::UserTaught => "User Taught",
        }
    }
}

/// Source of knowledge
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Built-in seed knowledge
    Seed,
    /// Learned from specialist
    Specialist,
    /// Learned from user interaction
    User,
    /// Fetched from Arch Wiki
    ArchWiki,
    /// Fetched from man pages
    ManPages,
    /// Fetched from help commands
    HelpCommands,
    /// Imported from external source
    Imported,
}

impl KnowledgeSource {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Seed => "Seed",
            Self::Specialist => "Specialist",
            Self::User => "User",
            Self::ArchWiki => "Arch Wiki",
            Self::ManPages => "Man Pages",
            Self::HelpCommands => "Help Commands",
            Self::Imported => "Imported",
        }
    }
}
