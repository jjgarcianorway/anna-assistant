# Language Contract

**Version:** 1.0.0
**Status:** Canonical Specification
**Last Updated:** 2025-01-14

## 1. Purpose

### What This Contract Solves

Anna Assistant operates across multiple components (annactl, annad, UI libraries, LLM layer) and must provide a **deterministic, consistent user experience** regardless of:

- User's system locale
- Terminal capabilities
- Component boundaries
- LLM model capabilities

Without this contract, language selection would be:
- **Non-deterministic** - Different components might choose different languages
- **Inconsistent** - Tone, formality, and emoji usage would vary unpredictably
- **Fragile** - Terminal capability mismatches would cause broken output
- **Confusing** - Users would see mixed languages in a single interaction

### Why Every Module Must Obey

This contract is **authoritative**. All Anna components must:

1. **Never implement their own language selection logic**
2. **Always query the canonical configuration source**
3. **Never mix languages in a single response**
4. **Always respect terminal capability constraints**
5. **Never fail due to terminal limitations**

Violation of this contract creates a degraded user experience and breaks Anna's core promise: **"I speak your language, adapt to your terminal, and never surprise you."**

---

## 2. Deterministic Language Priority System

This hierarchy is **immutable law** for all Anna components.

### Priority Order (Highest to Lowest)

```
1. Explicit User Instruction
   ↓ (if not set)
2. Persistent User Preference from Database
   ↓ (if not set)
3. System Locale (LANG/LC_MESSAGES)
   ↓ (if not detected)
4. English Fallback
```

### Rule 1: Explicit User Instruction (Highest Priority)

**Definition:** The user directly tells Anna to use a specific language during the current session.

**Examples:**
- `annactl "use Spanish"`
- `annactl "cambia al español"`
- In REPL: `"Anna, speak Norwegian"`

**Behavior:**
- Takes effect **immediately**
- Persists to database **automatically**
- Overrides all other sources
- Applies to **all subsequent output** in that session and future sessions

**Implementation:**
```rust
// User says: "use Spanish"
config.set_user_language(Language::Spanish);
db.save_language_config(&config).await?;
```

### Rule 2: Persistent User Preference

**Definition:** Previously saved language preference from the database.

**Storage:** `user_preferences.language` in SQLite context database

**Behavior:**
- Loaded on startup
- Survives restarts
- Can be cleared to revert to system locale

**Implementation:**
```rust
let config = db.load_language_config().await?;
let lang = config.effective_language();
```

### Rule 3: System Locale

**Definition:** Language detected from environment variables.

**Detection Order:**
1. `LC_ALL`
2. `LC_MESSAGES`
3. `LANG`

**Parsing:**
- Extract language code from `LANG=es_ES.UTF-8` → `es`
- Map to supported language: `es` → `Language::Spanish`
- If unsupported, continue to fallback

**Implementation:**
```rust
fn detect_system_language() -> Option<Language> {
    if let Ok(lang) = env::var("LANG") {
        let lang_code = lang.split('_').next()?.split('.').next()?;
        return Language::from_str(lang_code);
    }
    None
}
```

### Rule 4: English Fallback

**Definition:** Default when all other sources are unavailable.

**Behavior:**
- Always available
- Never fails
- Used as last resort

**Rationale:** English has the widest support and prevents crashes.

### Enforcement

**Every component must:**
```rust
let config = LanguageConfig::new(); // Auto-detects system locale
let effective = config.effective_language(); // Applies priority rules
let profile = effective.profile(); // Get translations and tone
```

**Never:**
```rust
// ❌ WRONG - Direct locale check bypasses priority
if env::var("LANG").unwrap_or_default().starts_with("es") {
    println!("Hola");
}
```

---

## 3. Terminal Capabilities Contract

### Capability Detection

Anna must detect terminal capabilities **once at startup** and cache the result:

```rust
pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    pub unicode_support: bool,
    pub emoji_support: bool,
    pub is_tty: bool,
}
```

### Color Support Levels

| Level | Description | Detection | ANSI Codes |
|-------|-------------|-----------|------------|
| `None` | No colors | `TERM=dumb` or not a TTY | None |
| `Basic16` | 16 colors | `TERM=xterm` | `\x1b[31m` (red) |
| `Extended256` | 256 colors | `TERM=xterm-256color` | `\x1b[38;5;196m` |
| `TrueColor` | 24-bit RGB | `COLORTERM=truecolor` | `\x1b[38;2;255;0;0m` |

**Fallback Rule:** Always degrade gracefully.

```
TrueColor → Extended256 → Basic16 → None
```

### Unicode Support

**Detection:**
```rust
fn detect_unicode_support() -> bool {
    for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(val) = env::var(var) {
            if val.to_uppercase().contains("UTF-8") {
                return true;
            }
        }
    }
    false
}
```

**Fallback Behavior:**

| Feature | Unicode Supported | Unicode NOT Supported |
|---------|-------------------|----------------------|
| Box drawing | `│ ─ ┌ ┐` | `\| - + +` |
| Bullets | `•` | `*` |
| Arrows | `→` | `->` |
| Checkmarks | `✓` | `[OK]` |
| Crosses | `✗` | `[X]` |

### Emoji Support

**Detection:**
```rust
emoji_support = unicode_support && is_tty
```

**Usage Rules:**

| Language | Emoji Style | Example |
|----------|-------------|---------|
| English | Moderate | `✓ Success` |
| Spanish | Moderate | `✓ Éxito` |
| Norwegian | Moderate | `✓ Suksess` |
| German | Minimal | `Erfolg` (no emoji) |
| French | Moderate | `✓ Succès` |
| Portuguese | Moderate | `✓ Sucesso` |

**When `emoji_support = false`:**
```
✓ → [OK]
✗ → [X]
⚠️ → [!]
🔒 → [SECURE]
```

### TTY Detection

```rust
is_tty = std::io::stdout().is_terminal()
```

**Behavior:**

| Output Target | Color | Emoji | Formatting |
|---------------|-------|-------|------------|
| Interactive TTY | Yes | Yes | Full |
| Pipe/Redirect | No | No | Plain text |
| Log file | No | No | Timestamps only |

---

## 4. Tone and Personality Contract

### Language Profiles

Each language has a mandatory tone profile:

```rust
pub struct ToneProfile {
    pub formality: Formality,
    pub use_contractions: bool,
    pub emoji_style: EmojiStyle,
}
```

### English (Casual)

- **Formality:** Casual
- **Contractions:** Yes ("I'm", "you're", "can't")
- **Emoji:** Moderate
- **Pronouns:** Direct second person ("you", "your")
- **Confirmation:** Friendly ("Got it!", "Done!")
- **Errors:** Empathetic ("Oops, that didn't work. Let's try...")

### Spanish (Casual)

- **Formality:** Casual (tú, not usted)
- **Contractions:** Yes ("pa'" for "para", informal speech)
- **Emoji:** Moderate
- **Pronouns:** Direct second person ("tú", "tu")
- **Confirmation:** Warm ("¡Listo!", "¡Perfecto!")
- **Errors:** Supportive ("No pude hacer eso. Intenta...")

### Norwegian (Casual)

- **Formality:** Casual
- **Contractions:** Yes (Norwegian allows contractions)
- **Emoji:** Moderate
- **Pronouns:** Direct second person ("du", "din")
- **Confirmation:** Direct ("Ferdig!", "Greit!")
- **Errors:** Practical ("Det fungerte ikke. Prøv...")

### German (Polite)

- **Formality:** Polite (Sie, not du)
- **Contractions:** No (formal German avoids contractions)
- **Emoji:** Minimal
- **Pronouns:** Formal second person ("Sie", "Ihr")
- **Confirmation:** Professional ("Erledigt", "Verstanden")
- **Errors:** Clear ("Das hat nicht funktioniert. Versuchen Sie...")

### French (Polite)

- **Formality:** Polite (vous, not tu)
- **Contractions:** Yes (standard French contractions: "l'", "d'")
- **Emoji:** Moderate
- **Pronouns:** Formal second person ("vous", "votre")
- **Confirmation:** Courteous ("Terminé", "D'accord")
- **Errors:** Respectful ("Cela n'a pas fonctionné. Essayez...")

### Portuguese (Casual)

- **Formality:** Casual (você, not o senhor/a senhora)
- **Contractions:** Yes (Brazilian Portuguese uses contractions)
- **Emoji:** Moderate
- **Pronouns:** Direct second person ("você", "seu")
- **Confirmation:** Friendly ("Pronto!", "Tudo certo!")
- **Errors:** Encouraging ("Isso não funcionou. Tente...")

### Application Rule

**Every module must:**
```rust
let profile = config.profile();
let tone = profile.tone;

// Use translations
println!("{}", profile.translations.success);

// Respect formality
if tone.formality == Formality::Polite {
    // Use formal pronouns, avoid casual language
}

// Apply emoji style
if tone.emoji_style == EmojiStyle::Minimal {
    // Strip emojis or use text alternatives
}
```

---

## 5. Interface Contract

### The UI Abstraction Layer

**Principle:** No component may directly format output. All formatting must go through the UI abstraction layer.

**Prohibited:**
```rust
// ❌ WRONG - Direct formatting
println!("✓ Success!");
println!("\x1b[32mDone\x1b[0m");
```

**Required:**
```rust
// ✅ CORRECT - UI abstraction
ui.success("Done");
ui.section_header("System Status");
ui.bullet_list(&items);
```

### UI Layer Responsibilities

The UI layer must:

1. **Apply terminal capabilities automatically**
   - Strip colors if `color_support == None`
   - Replace emojis if `emoji_support == false`
   - Use ASCII box-drawing if `unicode_support == false`

2. **Apply language profile**
   - Use correct translations
   - Respect tone and formality
   - Apply emoji style rules

3. **Handle degradation gracefully**
   - Never crash due to unsupported features
   - Always provide readable fallback

### Required UI Components

Every module must have access to:

```rust
pub trait UILayer {
    fn success(&self, message: &str);
    fn error(&self, message: &str);
    fn warning(&self, message: &str);
    fn info(&self, message: &str);

    fn section_header(&self, icon: &str, title: &str);
    fn bullet_list(&self, items: &[&str]);
    fn numbered_list(&self, items: &[&str]);

    fn progress(&self, current: usize, total: usize, label: &str);
    fn spinner(&self, message: &str);

    fn prompt_yes_no(&self, question: &str) -> bool;
    fn prompt_choice(&self, question: &str, choices: &[&str]) -> usize;
}
```

---

## 6. Interaction Rules

### Language Change Flow

**User Request:**
```
User: "Anna, use Spanish"
```

**System Behavior:**
1. Parse intent: `Intent::Language { language: Some("spanish") }`
2. Validate language is supported
3. Load current config from database
4. Update: `config.set_user_language(Language::Spanish)`
5. Persist: `db.save_language_config(&config).await?`
6. **Confirm in NEW language:** `"Idioma cambiado ✓"`
7. Apply to all subsequent output

**Critical:** The confirmation must be in the **target language**, not the source language.

### Immediate Effect

Language changes take effect **immediately** for:
- Current session
- All future sessions
- REPL interactions
- All commands
- All annad responses
- LLM-generated content

### Persistence Rules

**When to persist:**
- User explicitly sets a language
- Immediately after validation

**When NOT to persist:**
- Auto-detection of system locale
- Temporary session overrides (none exist - all changes persist)

**Storage location:**
- SQLite database: `context.db` → `user_preferences.language`
- User mode: `~/.local/share/anna/context.db`
- System mode: `/var/lib/anna/context.db`

### LLM Layer Alignment

**The LLM must be instructed:**

```
CRITICAL INSTRUCTION:
You are responding in {language_name}.
You must ONLY output in {language_name}.
Do not mix languages.
Do not apologize for language limitations.
Use the tone profile: {formality}, {emoji_style}.
```

**Example:**
```
CRITICAL INSTRUCTION:
You are responding in Spanish.
You must ONLY output in Spanish.
Do not mix languages.
Do not apologize for language limitations.
Use the tone profile: Casual, Moderate emoji.
```

### Repair Flow Language Compliance

**All repair flows must:**
1. Load language config before generating output
2. Use profile translations for all UI text
3. Display commands in monospace (language-agnostic)
4. Explain command effects in user's language

**Example (Spanish):**
```
Voy a ejecutar:
  sudo pacman -Syu

Esto actualizará todos los paquetes del sistema.
¿Quieres continuar? [sí/no]:
```

---

## 7. Safety and Fallback Rules

### Never Fail on Missing Fonts

**Rule:** If a character cannot be rendered, substitute it. Never crash.

**Implementation:**
```rust
fn safe_render(text: &str, caps: &TerminalCapabilities) -> String {
    if !caps.unicode_support {
        text.replace("✓", "[OK]")
            .replace("✗", "[X]")
            .replace("→", "->")
            .replace("│", "|")
    } else {
        text.to_string()
    }
}
```

### Never Output Unsupported Characters

**Detection:**
- Test terminal capabilities at startup
- Cache results
- Apply substitutions automatically

**Example:**
```rust
if !caps.emoji_support {
    // Replace all emojis with text equivalents
    output = strip_emojis(output);
}
```

### Always Degrade Gracefully

**Degradation Levels:**

| Full Capability | Medium | Minimal | ASCII-Only |
|-----------------|--------|---------|------------|
| TrueColor, Unicode, Emoji | 256-color, Unicode | 16-color, ASCII | No color, ASCII |
| `✓ Done` | `✓ Done` | `[OK] Done` | `[OK] Done` |
| `🔒 Secure` | `🔒 Secure` | `[SECURE]` | `[SECURE]` |
| Red error text | Red error text | `[ERROR]` | `[ERROR]` |

**Rule:** Output must always be readable at the lowest capability level.

### Always Log Terminal Capability Profile

**On startup, log:**
```
[INFO] Terminal capabilities detected:
  Color support: Extended256
  Unicode support: true
  Emoji support: true
  TTY: true
```

**Rationale:** Essential for debugging user reports of garbled output.

---

## 8. Required Tests

### Mandatory Test Coverage

#### 1. Language Parsing Tests

```rust
#[test]
fn test_language_from_string() {
    assert_eq!(Language::from_str("english"), Some(Language::English));
    assert_eq!(Language::from_str("español"), Some(Language::Spanish));
    assert_eq!(Language::from_str("es"), Some(Language::Spanish));
    assert_eq!(Language::from_str("invalid"), None);
}
```

#### 2. Priority System Tests

```rust
#[test]
fn test_priority_user_over_system() {
    let mut config = LanguageConfig::new();
    config.system_language = Some(Language::Spanish);
    config.set_user_language(Language::English);
    assert_eq!(config.effective_language(), Language::English);
}

#[test]
fn test_priority_system_over_fallback() {
    let mut config = LanguageConfig::new();
    config.system_language = Some(Language::Norwegian);
    config.user_language = None;
    assert_eq!(config.effective_language(), Language::Norwegian);
}

#[test]
fn test_priority_fallback() {
    let config = LanguageConfig {
        user_language: None,
        system_language: None,
        terminal: TerminalCapabilities::detect(),
    };
    assert_eq!(config.effective_language(), Language::English);
}
```

#### 3. Persistence Tests

```rust
#[tokio::test]
async fn test_language_persistence() {
    let db = ContextDb::open(DbLocation::Custom(temp_path)).await.unwrap();

    let mut config = LanguageConfig::new();
    config.set_user_language(Language::Spanish);
    db.save_language_config(&config).await.unwrap();

    let loaded = db.load_language_config().await.unwrap();
    assert_eq!(loaded.user_language, Some(Language::Spanish));
}
```

#### 4. Terminal Capability Tests

```rust
#[test]
fn test_color_detection() {
    env::set_var("COLORTERM", "truecolor");
    let caps = TerminalCapabilities::detect();
    assert_eq!(caps.color_support, ColorSupport::TrueColor);
}

#[test]
fn test_unicode_detection() {
    env::set_var("LANG", "en_US.UTF-8");
    let caps = TerminalCapabilities::detect();
    assert!(caps.unicode_support);
}
```

#### 5. Emoji/ASCII Fallback Tests

```rust
#[test]
fn test_emoji_fallback() {
    let caps = TerminalCapabilities {
        emoji_support: false,
        ..Default::default()
    };

    assert_eq!(ui.safe_render("✓ Done", &caps), "[OK] Done");
    assert_eq!(ui.safe_render("⚠️ Warning", &caps), "[!] Warning");
}
```

#### 6. Tone Profile Tests

```rust
#[test]
fn test_german_formality() {
    let profile = Language::German.profile();
    assert_eq!(profile.tone.formality, Formality::Polite);
    assert_eq!(profile.tone.use_contractions, false);
}

#[test]
fn test_english_casual() {
    let profile = Language::English.profile();
    assert_eq!(profile.tone.formality, Formality::Casual);
    assert!(profile.tone.use_contractions);
}
```

#### 7. Intent Detection Tests

```rust
#[test]
fn test_language_intent_detection() {
    assert_eq!(
        route_intent("use Spanish"),
        Intent::Language { language: Some("spanish".to_string()) }
    );
    assert_eq!(
        route_intent("cambia al español"),
        Intent::Language { language: Some("spanish".to_string()) }
    );
}
```

#### 8. Regression Tests

```rust
#[test]
fn test_no_mixed_languages() {
    // Ensure response generator never mixes languages
    let config = LanguageConfig::with_language(Language::Spanish);
    let output = generate_response(&config, "system status");
    assert!(!output.contains("English"));
    assert!(!output.contains("Error")); // Should be "Error" not mixed
}
```

---

## 9. Versioning Rules

### Semantic Versioning

This contract follows [Semantic Versioning 2.0.0](https://semver.org/).

**Format:** `MAJOR.MINOR.PATCH`

**Current Version:** `1.0.0`

### Version Change Rules

#### MAJOR Version (Breaking Changes)

**Increment when:**
- Changing priority order
- Removing a supported language
- Changing terminal capability detection logic
- Modifying tone profile contracts
- Breaking UI abstraction interface

**Example:** `1.0.0` → `2.0.0`

**Announcement:** Breaking changes require:
- Migration guide
- Deprecation notices (at least 1 minor version in advance)
- Updated documentation

#### MINOR Version (Backward-Compatible Changes)

**Increment when:**
- Adding new supported languages
- Adding new terminal capability levels
- Enhancing fallback rules
- Adding new UI components
- Expanding tone profiles

**Example:** `1.0.0` → `1.1.0`

**Announcement:** Minor changes require:
- Update CHANGELOG
- Test new features
- No migration needed

#### PATCH Version (Bug Fixes)

**Increment when:**
- Fixing language detection bugs
- Correcting translations
- Fixing terminal capability edge cases
- Improving fallback behavior

**Example:** `1.0.0` → `1.0.1`

**Announcement:** Patch changes require:
- Bug fix notes in CHANGELOG

### Language Addition Process

**To add a new language (MINOR version bump):**

1. Add to `Language` enum
2. Implement `LanguageProfile` with:
   - Translations for all common strings
   - Tone profile (formality, contractions, emoji style)
   - Native language name
3. Add parsing support in `from_str()`
4. Add intent detection keywords
5. Write tests for the new language
6. Update this document
7. Bump MINOR version

**Required approvals:**
- Native speaker review of translations
- Tone profile validation
- Test coverage verification

---

## Compliance Checklist

Every Anna component must verify:

- [ ] Language priority logic matches Section 2
- [ ] Terminal capabilities detected per Section 3
- [ ] Tone profiles applied per Section 4
- [ ] All output goes through UI layer (Section 5)
- [ ] Language changes persist correctly (Section 6)
- [ ] Fallback rules implemented (Section 7)
- [ ] All required tests pass (Section 8)
- [ ] Version tracked per Section 9

---

## Change Log

### Version 1.0.0 (2025-01-14)

**Initial Release**

- Established canonical language priority system
- Defined terminal capability detection rules
- Specified tone profiles for 6 languages
- Created UI abstraction contract
- Documented safety and fallback requirements
- Defined mandatory test coverage
- Established versioning rules

**Supported Languages:**
- English
- Spanish
- Norwegian
- German
- French
- Portuguese

---

## References

- Implementation: `crates/anna_common/src/language.rs`
- Database persistence: `crates/anna_common/src/context/db.rs`
- Intent detection: `crates/annactl/src/intent_router.rs`
- Tests: `crates/anna_common/src/language.rs#tests`

---

**End of Language Contract v1.0.0**
