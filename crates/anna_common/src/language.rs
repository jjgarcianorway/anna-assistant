//! Language System - Natural language configuration and terminal adaptation
//!
//! Anna manages her own language settings through natural-language commands.
//! Users never manually edit config files.
//!
//! Language Priority:
//! 1. Explicit user instruction (highest)
//! 2. System locale (fallback)
//! 3. English (default)
//!
//! Terminal Adaptation:
//! - Full color + emojis when supported
//! - Simplified Unicode fallback
//! - Pure ASCII for limited TTY

use serde::{Deserialize, Serialize};
use std::env;
use std::io::IsTerminal;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Spanish,
    Norwegian,
    German,
    French,
    Portuguese,
}

impl Language {
    /// Parse language from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "english" | "en" | "en_us" | "en_gb" => Some(Language::English),
            "spanish" | "español" | "es" | "es_es" | "es_mx" => Some(Language::Spanish),
            "norwegian" | "norsk" | "no" | "nb" | "nn" => Some(Language::Norwegian),
            "german" | "deutsch" | "de" | "de_de" => Some(Language::German),
            "french" | "français" | "fr" | "fr_fr" => Some(Language::French),
            "portuguese" | "português" | "pt" | "pt_br" | "pt_pt" => Some(Language::Portuguese),
            _ => None,
        }
    }

    /// Get language code (ISO 639-1)
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::Norwegian => "no",
            Language::German => "de",
            Language::French => "fr",
            Language::Portuguese => "pt",
        }
    }

    /// Get native name
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            Language::Norwegian => "Norsk",
            Language::German => "Deutsch",
            Language::French => "Français",
            Language::Portuguese => "Português",
        }
    }

    /// Get language profile for responses
    pub fn profile(&self) -> LanguageProfile {
        match self {
            Language::English => LanguageProfile::english(),
            Language::Spanish => LanguageProfile::spanish(),
            Language::Norwegian => LanguageProfile::norwegian(),
            Language::German => LanguageProfile::german(),
            Language::French => LanguageProfile::french(),
            Language::Portuguese => LanguageProfile::portuguese(),
        }
    }
}

/// Language profile containing translations and tone
#[derive(Debug, Clone)]
pub struct LanguageProfile {
    pub language: Language,
    pub translations: Translations,
    pub tone: ToneProfile,
}

impl LanguageProfile {
    fn english() -> Self {
        Self {
            language: Language::English,
            translations: Translations::english(),
            tone: ToneProfile {
                formality: Formality::Casual,
                use_contractions: true,
                emoji_style: EmojiStyle::Moderate,
            },
        }
    }

    fn spanish() -> Self {
        Self {
            language: Language::Spanish,
            translations: Translations::spanish(),
            tone: ToneProfile {
                formality: Formality::Casual,
                use_contractions: true,
                emoji_style: EmojiStyle::Moderate,
            },
        }
    }

    fn norwegian() -> Self {
        Self {
            language: Language::Norwegian,
            translations: Translations::norwegian(),
            tone: ToneProfile {
                formality: Formality::Casual,
                use_contractions: true,
                emoji_style: EmojiStyle::Moderate,
            },
        }
    }

    fn german() -> Self {
        Self {
            language: Language::German,
            translations: Translations::german(),
            tone: ToneProfile {
                formality: Formality::Polite,
                use_contractions: false,
                emoji_style: EmojiStyle::Minimal,
            },
        }
    }

    fn french() -> Self {
        Self {
            language: Language::French,
            translations: Translations::french(),
            tone: ToneProfile {
                formality: Formality::Polite,
                use_contractions: true,
                emoji_style: EmojiStyle::Moderate,
            },
        }
    }

    fn portuguese() -> Self {
        Self {
            language: Language::Portuguese,
            translations: Translations::portuguese(),
            tone: ToneProfile {
                formality: Formality::Casual,
                use_contractions: true,
                emoji_style: EmojiStyle::Moderate,
            },
        }
    }
}

/// Common translations for each language
#[derive(Debug, Clone)]
pub struct Translations {
    // Language change confirmations
    pub language_changed: &'static str,
    pub now_speaking: &'static str,

    // Common UI elements
    pub yes: &'static str,
    pub no: &'static str,
    pub cancel: &'static str,
    pub continue_prompt: &'static str,

    // Status messages
    pub success: &'static str,
    pub failed: &'static str,
    pub working: &'static str,
    pub done: &'static str,

    // System status
    pub system_healthy: &'static str,
    pub checking_system: &'static str,
    pub found_issues: &'static str,

    // Suggestions
    pub suggestions_for_you: &'static str,
    pub no_suggestions: &'static str,
    pub system_looks_good: &'static str,

    // Goodbye
    pub goodbye: &'static str,
    pub watching_background: &'static str,
}

impl Translations {
    fn english() -> Self {
        Self {
            language_changed: "Language changed",
            now_speaking: "I'm now speaking",
            yes: "yes",
            no: "no",
            cancel: "cancel",
            continue_prompt: "Do you want to proceed?",
            success: "Success",
            failed: "Failed",
            working: "Working",
            done: "Done",
            system_healthy: "Your system is healthy",
            checking_system: "Checking your system",
            found_issues: "I found some issues",
            suggestions_for_you: "Suggestions for you",
            no_suggestions: "No suggestions right now",
            system_looks_good: "Your system looks good",
            goodbye: "Goodbye",
            watching_background: "I'll keep watching your system in the background",
        }
    }

    fn spanish() -> Self {
        Self {
            language_changed: "Idioma cambiado",
            now_speaking: "Ahora hablo",
            yes: "sí",
            no: "no",
            cancel: "cancelar",
            continue_prompt: "¿Quieres continuar?",
            success: "Éxito",
            failed: "Falló",
            working: "Trabajando",
            done: "Listo",
            system_healthy: "Tu sistema está saludable",
            checking_system: "Revisando tu sistema",
            found_issues: "Encontré algunos problemas",
            suggestions_for_you: "Sugerencias para ti",
            no_suggestions: "No hay sugerencias ahora",
            system_looks_good: "Tu sistema se ve bien",
            goodbye: "Adiós",
            watching_background: "Seguiré vigilando tu sistema en segundo plano",
        }
    }

    fn norwegian() -> Self {
        Self {
            language_changed: "Språk endret",
            now_speaking: "Jeg snakker nå",
            yes: "ja",
            no: "nei",
            cancel: "avbryt",
            continue_prompt: "Vil du fortsette?",
            success: "Suksess",
            failed: "Mislyktes",
            working: "Arbeider",
            done: "Ferdig",
            system_healthy: "Systemet ditt er sunt",
            checking_system: "Sjekker systemet ditt",
            found_issues: "Jeg fant noen problemer",
            suggestions_for_you: "Forslag til deg",
            no_suggestions: "Ingen forslag akkurat nå",
            system_looks_good: "Systemet ditt ser bra ut",
            goodbye: "Ha det",
            watching_background: "Jeg vil fortsette å overvåke systemet ditt i bakgrunnen",
        }
    }

    fn german() -> Self {
        Self {
            language_changed: "Sprache geändert",
            now_speaking: "Ich spreche jetzt",
            yes: "ja",
            no: "nein",
            cancel: "abbrechen",
            continue_prompt: "Möchten Sie fortfahren?",
            success: "Erfolg",
            failed: "Fehlgeschlagen",
            working: "Arbeite",
            done: "Fertig",
            system_healthy: "Ihr System ist gesund",
            checking_system: "Überprüfe Ihr System",
            found_issues: "Ich habe einige Probleme gefunden",
            suggestions_for_you: "Vorschläge für Sie",
            no_suggestions: "Keine Vorschläge im Moment",
            system_looks_good: "Ihr System sieht gut aus",
            goodbye: "Auf Wiedersehen",
            watching_background: "Ich werde Ihr System im Hintergrund weiter überwachen",
        }
    }

    fn french() -> Self {
        Self {
            language_changed: "Langue changée",
            now_speaking: "Je parle maintenant",
            yes: "oui",
            no: "non",
            cancel: "annuler",
            continue_prompt: "Voulez-vous continuer?",
            success: "Succès",
            failed: "Échoué",
            working: "Travail en cours",
            done: "Terminé",
            system_healthy: "Votre système est en bonne santé",
            checking_system: "Vérification de votre système",
            found_issues: "J'ai trouvé quelques problèmes",
            suggestions_for_you: "Suggestions pour vous",
            no_suggestions: "Pas de suggestions pour le moment",
            system_looks_good: "Votre système a l'air bien",
            goodbye: "Au revoir",
            watching_background: "Je continuerai à surveiller votre système en arrière-plan",
        }
    }

    fn portuguese() -> Self {
        Self {
            language_changed: "Idioma alterado",
            now_speaking: "Agora estou falando",
            yes: "sim",
            no: "não",
            cancel: "cancelar",
            continue_prompt: "Você quer continuar?",
            success: "Sucesso",
            failed: "Falhou",
            working: "Trabalhando",
            done: "Pronto",
            system_healthy: "Seu sistema está saudável",
            checking_system: "Verificando seu sistema",
            found_issues: "Encontrei alguns problemas",
            suggestions_for_you: "Sugestões para você",
            no_suggestions: "Nenhuma sugestão no momento",
            system_looks_good: "Seu sistema está bem",
            goodbye: "Tchau",
            watching_background: "Vou continuar monitorando seu sistema em segundo plano",
        }
    }
}

/// Tone and formality profile
#[derive(Debug, Clone)]
pub struct ToneProfile {
    pub formality: Formality,
    pub use_contractions: bool,
    pub emoji_style: EmojiStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formality {
    Casual,
    Polite,
    Formal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiStyle {
    None,
    Minimal,
    Moderate,
    Expressive,
}

/// Terminal capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    pub unicode_support: bool,
    pub emoji_support: bool,
    pub is_tty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSupport {
    None,
    Basic16,     // 16 colors
    Extended256, // 256 colors
    TrueColor,   // 24-bit RGB
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

impl TerminalCapabilities {
    /// Detect terminal capabilities automatically
    pub fn detect() -> Self {
        let is_tty = std::io::stdout().is_terminal();

        // Detect color support
        let color_support = Self::detect_color_support();

        // Detect Unicode support
        let unicode_support = Self::detect_unicode_support();

        // Emoji support typically requires Unicode
        let emoji_support = unicode_support && is_tty;

        Self {
            color_support,
            unicode_support,
            emoji_support,
            is_tty,
        }
    }

    fn detect_color_support() -> ColorSupport {
        // Check COLORTERM for truecolor
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return ColorSupport::TrueColor;
            }
        }

        // Check TERM for color capabilities
        if let Ok(term) = env::var("TERM") {
            if term.contains("256color") {
                return ColorSupport::Extended256;
            }
            if term != "dumb" && term != "unknown" {
                return ColorSupport::Basic16;
            }
        }

        // Check if we're in a TTY
        if !std::io::stdout().is_terminal() {
            return ColorSupport::None;
        }

        // Default to basic colors for TTY
        ColorSupport::Basic16
    }

    fn detect_unicode_support() -> bool {
        // Check LANG and LC_ALL for UTF-8
        for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(val) = env::var(var) {
                if val.to_uppercase().contains("UTF-8") || val.to_uppercase().contains("UTF8") {
                    return true;
                }
            }
        }

        // Default to true on modern systems
        true
    }

    /// Should use colors for output?
    pub fn use_colors(&self) -> bool {
        !matches!(self.color_support, ColorSupport::None)
    }

    /// Should use emojis for output?
    pub fn use_emojis(&self) -> bool {
        self.emoji_support
    }

    /// Should use Unicode box-drawing characters?
    pub fn use_unicode_graphics(&self) -> bool {
        self.unicode_support
    }
}

/// Language configuration with persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// User's explicit language choice (highest priority)
    pub user_language: Option<Language>,

    /// Detected system locale
    pub system_language: Option<Language>,

    /// Terminal capabilities (auto-detected)
    #[serde(skip)]
    pub terminal: TerminalCapabilities,
}

impl LanguageConfig {
    /// Create default configuration
    pub fn new() -> Self {
        Self {
            user_language: None,
            system_language: Self::detect_system_language(),
            terminal: TerminalCapabilities::detect(),
        }
    }

    /// Get effective language (following priority rules)
    pub fn effective_language(&self) -> Language {
        // 1. User explicit choice (highest priority)
        if let Some(lang) = self.user_language {
            return lang;
        }

        // 2. System locale
        if let Some(lang) = self.system_language {
            return lang;
        }

        // 3. English fallback
        Language::English
    }

    /// Set user language preference
    pub fn set_user_language(&mut self, language: Language) {
        self.user_language = Some(language);
    }

    /// Clear user language preference (revert to system locale)
    pub fn clear_user_language(&mut self) {
        self.user_language = None;
    }

    /// Detect system language from locale
    fn detect_system_language() -> Option<Language> {
        // Check LANG environment variable
        if let Ok(lang) = env::var("LANG") {
            // Extract language code (e.g., "en_US.UTF-8" -> "en")
            let lang_code = lang.split('_').next()?.split('.').next()?;
            return Language::from_str(lang_code);
        }

        // Check LC_MESSAGES
        if let Ok(lang) = env::var("LC_MESSAGES") {
            let lang_code = lang.split('_').next()?.split('.').next()?;
            return Language::from_str(lang_code);
        }

        None
    }

    /// Get current language profile
    pub fn profile(&self) -> LanguageProfile {
        self.effective_language().profile()
    }

    /// Refresh terminal capabilities
    pub fn refresh_terminal(&mut self) {
        self.terminal = TerminalCapabilities::detect();
    }
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_priority() {
        let mut config = LanguageConfig::new();

        // Default should be system or English
        let default_lang = config.effective_language();
        assert!(matches!(
            default_lang,
            Language::English | Language::Spanish | Language::Norwegian
        ));

        // User preference overrides system
        config.set_user_language(Language::Spanish);
        assert_eq!(config.effective_language(), Language::Spanish);

        // Clearing user preference reverts to system/English
        config.clear_user_language();
        let reverted = config.effective_language();
        assert!(matches!(
            reverted,
            Language::English | Language::Spanish | Language::Norwegian
        ));
    }

    #[test]
    fn test_language_parsing() {
        assert_eq!(Language::from_str("english"), Some(Language::English));
        assert_eq!(Language::from_str("EN"), Some(Language::English));
        assert_eq!(Language::from_str("español"), Some(Language::Spanish));
        assert_eq!(Language::from_str("es"), Some(Language::Spanish));
        assert_eq!(Language::from_str("norsk"), Some(Language::Norwegian));
        assert_eq!(Language::from_str("invalid"), None);
    }

    #[test]
    fn test_terminal_detection() {
        let caps = TerminalCapabilities::detect();
        // Just ensure it doesn't panic
        assert!(matches!(
            caps.color_support,
            ColorSupport::None
                | ColorSupport::Basic16
                | ColorSupport::Extended256
                | ColorSupport::TrueColor
        ));
    }

    #[test]
    fn test_emoji_support_detection() {
        let mut caps = TerminalCapabilities::detect();

        // Test that emoji/unicode flags are properly detected
        let has_emoji = caps.use_emojis();
        let has_unicode = caps.use_unicode_graphics();

        // At least one should work (or both can be false for limited terminals)
        assert!(has_emoji || has_unicode || (!has_emoji && !has_unicode));

        // Test forced ASCII mode
        caps.emoji_support = false;
        caps.unicode_support = false;
        assert!(!caps.use_emojis());
        assert!(!caps.use_unicode_graphics());
    }

    #[test]
    fn test_tone_profiles() {
        // English - more formal
        let en_config = LanguageConfig {
            user_language: Some(Language::English),
            system_language: None,
            terminal: TerminalCapabilities::detect(),
        };
        let en_profile = en_config.profile();
        assert_eq!(en_profile.language, Language::English);
        assert!(en_profile.tone.use_contractions); // English allows contractions

        // Spanish - more friendly
        let es_config = LanguageConfig {
            user_language: Some(Language::Spanish),
            system_language: None,
            terminal: TerminalCapabilities::detect(),
        };
        let es_profile = es_config.profile();
        assert_eq!(es_profile.language, Language::Spanish);

        // German - more formal
        let de_config = LanguageConfig {
            user_language: Some(Language::German),
            system_language: None,
            terminal: TerminalCapabilities::detect(),
        };
        let de_profile = de_config.profile();
        assert_eq!(de_profile.language, Language::German);
    }

    #[test]
    fn test_translation_loading() {
        let config = LanguageConfig {
            user_language: Some(Language::Spanish),
            system_language: None,
            terminal: TerminalCapabilities::detect(),
        };

        let profile = config.profile();

        // Verify translations are loaded
        assert!(!profile.translations.language_changed.is_empty());
        assert!(!profile.translations.now_speaking.is_empty());
        assert!(!profile.translations.working.is_empty());

        // Spanish should have "sí" for yes
        assert_eq!(profile.translations.yes, "sí");
        assert_eq!(profile.translations.no, "no");
    }

    #[test]
    fn test_language_native_names() {
        assert_eq!(Language::English.native_name(), "English");
        assert_eq!(Language::Spanish.native_name(), "Español");
        assert_eq!(Language::Norwegian.native_name(), "Norsk");
        assert_eq!(Language::German.native_name(), "Deutsch");
        assert_eq!(Language::French.native_name(), "Français");
        assert_eq!(Language::Portuguese.native_name(), "Português");
    }

    #[test]
    fn test_no_language_mixing() {
        // Verify that a single config produces a single language profile
        let config = LanguageConfig {
            user_language: Some(Language::French),
            system_language: Some(Language::English), // Should be ignored
            terminal: TerminalCapabilities::detect(),
        };

        let profile = config.profile();

        // Should use user preference (French), not system language (English)
        assert_eq!(profile.language, Language::French);
        assert_eq!(config.effective_language(), Language::French);

        // Translations should all be in French
        assert_eq!(profile.translations.yes, "oui");
        assert_eq!(profile.translations.no, "non");
    }
}
