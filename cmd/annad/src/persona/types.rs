use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Persona {
    AdminPragmatic,
    DevEnthusiast,
    PowerNerd,
    CasualMinimal,
    CreatorWriter,
    #[default]
    Unknown,
}

impl Persona {
    pub fn as_str(&self) -> &'static str {
        match self {
            Persona::AdminPragmatic => "admin-pragmatic",
            Persona::DevEnthusiast => "dev-enthusiast",
            Persona::PowerNerd => "power-nerd",
            Persona::CasualMinimal => "casual-minimal",
            Persona::CreatorWriter => "creator-writer",
            Persona::Unknown => "unknown",
        }
    }
}

impl fmt::Display for Persona {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Persona {
    type Err = PersonaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin-pragmatic" => Ok(Persona::AdminPragmatic),
            "dev-enthusiast" => Ok(Persona::DevEnthusiast),
            "power-nerd" => Ok(Persona::PowerNerd),
            "casual-minimal" => Ok(Persona::CasualMinimal),
            "creator-writer" => Ok(Persona::CreatorWriter),
            "unknown" => Ok(Persona::Unknown),
            _ => Err(PersonaParseError),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PersonaParseError;

impl fmt::Display for PersonaParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid persona name")
    }
}

impl std::error::Error for PersonaParseError {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PersonaSource {
    Default,
    Override,
    Current,
    Inferred,
}

impl PersonaSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PersonaSource::Default => "default",
            PersonaSource::Override => "override",
            PersonaSource::Current => "current",
            PersonaSource::Inferred => "inferred",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaState {
    pub persona: Persona,
    pub confidence: f32,
    pub updated: String,
    pub source: PersonaSource,
    #[serde(default)]
    pub explanations: Vec<String>,
    #[serde(default)]
    pub window_days: u32,
}
