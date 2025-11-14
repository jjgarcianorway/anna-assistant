# Language Contract

**Contract Version:** 1.0
**Anna Minimum Version:** 5.3.0-beta.1
**Last Updated:** 2025-11-14

---

## 1. Purpose

This contract governs language selection, tone, and terminal formatting for all user-facing output in Anna Assistant.

**Core Principles:**

1. **No Direct Output Bypass:** No module is allowed to bypass this contract and print directly to `stdout`/`stderr` without going through the UI abstraction layer. All user-facing output must use the `UI` type from `anna_common::display`.

2. **Consistency Over Cleverness:** Anna must speak in a single language per interaction, with consistent tone and formatting. Mixed-language responses are bugs, not features.

3. **Graceful Degradation:** Anna must work in any terminal environment - from modern GUI terminals with TrueColor and emoji support, down to ancient TTY with ASCII-only. Terminal capability detection is automatic and mandatory.

4. **Natural Configuration:** Users change Anna's language through natural commands ("use Spanish", "cambia al español"), never by manually editing config files. Anna manages her own configuration.

**Why This Matters:**

Anna is not just a command-line tool - she's a conversational system administrator. Her personality, tone, and language choice are part of the user experience. This contract ensures that experience remains consistent, accessible, and respectful of user preferences across all interactions.

---

## 2. Language Priority Rules

Anna determines her output language using a strict, deterministic priority system. This ordering is **mandatory** - any deviation is a bug.

### Priority Order (Highest to Lowest)

1. **Explicit User Instruction** (Highest Priority)
   - Natural language commands in the current session
   - Examples: "use English", "cambia al español", "parle français"
   - Takes effect immediately and is persisted to the database

2. **Persistent User Preference from Database**
   - Loaded from `user_preferences` table in context database
   - Survives across sessions
   - Set by previous explicit user instruction

3. **System Locale**
   - Detected from `LANG` or `LC_MESSAGES` environment variables
   - Example: `LANG=es_ES.UTF-8` → Spanish
   - Used when no explicit preference exists

4. **English Fallback** (Lowest Priority)
   - Always available as final fallback
   - Ensures Anna always works, even in degraded environments

### Implementation Location

This logic is implemented in:
```rust
// crates/anna_common/src/language.rs
impl LanguageConfig {
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
}
```

### Rules for Contributors

- **Adding New Languages:** Must plug into this system via `Language` enum and locale detection. Do not invent separate language selection logic.
- **Priority Changes:** Changing this priority order requires a contract version bump and explicit approval.
- **Testing:** Any change to language selection logic must include tests verifying the priority order.

---

## 3. Supported Languages & Profiles

Anna currently supports six languages, each with its own personality profile.

### English (Default)

- **Formality:** Professional but approachable
- **Emoji Usage:** Normal (when terminal supports it)
- **Contractions:** Yes ("I'm", "don't", "you're")
- **Confirmation Tone:** Direct and clear
- **Example Confirmation:**
  ```
  Language changed ✓
  Now speaking: English
  ```

### Spanish (Español)

- **Formality:** Warm and friendly
- **Emoji Usage:** Normal
- **Contractions:** Minimal (more formal Spanish structure)
- **Confirmation Tone:** Warm and conversational
- **Example Confirmation:**
  ```
  Idioma cambiado ✓
  Ahora hablo: Español
  ```

### Norwegian (Norsk)

- **Formality:** Casual and direct
- **Emoji Usage:** Conservative
- **Contractions:** Common in casual speech
- **Confirmation Tone:** Brief and straightforward
- **Example Confirmation:**
  ```
  Språk endret ✓
  Snakker nå: Norsk
  ```

### German (Deutsch)

- **Formality:** More formal
- **Emoji Usage:** Minimal
- **Contractions:** Rare (formal German structure)
- **Confirmation Tone:** Clear and precise
- **Example Confirmation:**
  ```
  Sprache geändert ✓
  Spreche jetzt: Deutsch
  ```

### French (Français)

- **Formality:** Polite and structured
- **Emoji Usage:** Normal
- **Contractions:** Common (je suis → j'suis in casual)
- **Confirmation Tone:** Polite and complete
- **Example Confirmation:**
  ```
  Langue changée ✓
  Je parle maintenant: Français
  ```

### Portuguese (Português)

- **Formality:** Friendly and conversational
- **Emoji Usage:** Normal
- **Contractions:** Common
- **Confirmation Tone:** Warm and clear
- **Example Confirmation:**
  ```
  Idioma alterado ✓
  Falando agora: Português
  ```

### Implementation Location

Language profiles are implemented in:
```rust
// crates/anna_common/src/language.rs
impl LanguageConfig {
    pub fn profile(&self) -> LanguageProfile {
        // Returns LanguageProfile with:
        // - language: Language
        // - translations: Translations
        // - tone: ToneProfile
    }
}
```

---

## 4. Terminal Capabilities Contract

Anna automatically detects terminal capabilities and adapts her output accordingly. Users should never see broken glyphs, missing colors, or garbled box-drawing characters.

### Color Support Levels

| Level | Description | Detection | Usage |
|-------|-------------|-----------|-------|
| **None** | No colors | `TERM=dumb` or not a TTY | Plain text only |
| **Basic16** | 16 ANSI colors | `TERM=xterm` | Basic colors for success/error/warning |
| **Extended256** | 256 colors | `TERM=xterm-256color` | Richer color palette |
| **TrueColor** | 24-bit RGB | `COLORTERM=truecolor` | Full color spectrum |

**Detection Order:**
1. Check `COLORTERM` for `truecolor`
2. Check `TERM` for `256color`
3. Check `TERM` for basic xterm support
4. Check if `stdout` is a TTY
5. Fallback to `None`

### Unicode Support

**Detection:**
- Check `LANG` environment variable for UTF-8 encoding
- Example: `LANG=en_US.UTF-8` → Unicode supported
- Fallback: ASCII mode if no UTF-8 detected

**Usage:**
- Unicode: Box-drawing characters (─, ┌, ┐, └, ┘, │)
- ASCII: Plain text equivalents (-, +, |)

### Emoji Support & Fallback

Anna uses emoji when appropriate and safe. When emoji are not supported, automatic text substitution occurs.

**Emoji → Text Mapping:**

| Emoji | Text Equivalent | Usage |
|-------|-----------------|-------|
| ✓ | [OK] | Success confirmations |
| ✗ | [X] | Failures |
| ⚠️ | [!] | Warnings |
| 🔐 | [SECURE] | Security-related actions |
| 💡 | [TIP] | Suggestions |
| 📊 | [STAT] | Statistics |
| 🏛️ | [WIKI] | Arch Wiki references |
| 📖 | [DOCS] | Documentation |
| 📄 | [MAN] | Man pages |
| 🔧 | [FIX] | Repair actions |

**Detection:**
- Emoji support is assumed if terminal has Unicode support
- Can be disabled explicitly via terminal capabilities

### Box-Drawing Character Substitution

| Unicode | ASCII | Usage |
|---------|-------|-------|
| ┌ | + | Top-left corner |
| ┐ | + | Top-right corner |
| └ | + | Bottom-left corner |
| ┘ | + | Bottom-right corner |
| ─ | - | Horizontal line |
| │ | \| | Vertical line |

### Mandatory Rules

1. **Always Detect:** Terminal capabilities must be detected on every UI instantiation
2. **Never Crash:** If detection fails, default to ASCII-only, no-color mode
3. **Respect User:** If `NO_COLOR` environment variable is set, disable colors
4. **Graceful Fallback:** Prefer readable ASCII over broken Unicode

### Implementation Location

Terminal capability detection is implemented in:
```rust
// crates/anna_common/src/language.rs
impl TerminalCapabilities {
    pub fn detect() -> Self {
        // Detects color, Unicode, emoji support
        // Returns safe defaults if detection fails
    }
}
```

Fallback rendering is implemented in:
```rust
// crates/anna_common/src/display.rs
impl UI {
    fn render_emoji(&self, emoji: &str) -> String { /* ... */ }
    fn render_box_char(&self, unicode: &str, ascii: &str) -> String { /* ... */ }
}
```

---

## 5. UI Abstraction Layer Contract

All user-facing output must go through the `UI` abstraction layer. Direct use of `println!`, `eprintln!`, or manual ANSI formatting is prohibited in user-facing code.

### Core Principle

**Business logic should provide semantic information, not presentation details.**

✅ **Correct:**
```rust
let ui = UI::auto();
ui.error("Failed to connect to daemon");
```

❌ **Wrong:**
```rust
eprintln!("\x1b[31mError:\x1b[0m Failed to connect to daemon");
```

### Key Methods

#### Status Messages
```rust
ui.success("Operation completed");    // Green checkmark + message
ui.error("Operation failed");         // Red X + message
ui.warning("Potential issue");        // Yellow warning + message
ui.info("General information");       // Blue info + message
```

#### Structural Elements
```rust
ui.section_header("🔐", "Security Settings");  // Section with icon
ui.bullet_list(&["Item 1", "Item 2"]);         // Bulleted list
ui.numbered_list(&["Step 1", "Step 2"]);       // Numbered list
```

#### User Interaction
```rust
let confirmed = ui.prompt_yes_no("Continue?");  // Boolean prompt
let choice = ui.prompt_choice("Select:", &["A", "B", "C"]);  // Multi-choice
```

#### Advanced Display
```rust
ui.progress(3, 10, "Processing");      // Progress indicator
ui.spinner("Loading...");              // Thinking indicator
ui.box_content("Title", &["Line 1"]); // Content in a box
ui.summary("Results", &[              // Key-value summary
    ("Status", "OK"),
    ("Count", "42"),
]);
```

#### Semantic Commands
```rust
ui.thinking();                                 // "Thinking..." indicator
ui.done("Task completed");                    // Final confirmation
ui.command("sudo systemctl restart annad", true);  // Show command
ui.explain("This is an explanation...");      // Detailed explanation
ui.wiki_link("Arch Linux", "https://...");   // Documentation link
```

### UI Instantiation

**Automatic (Recommended):**
```rust
let ui = UI::auto();  // Loads language config automatically
```

**Explicit (When you have a config):**
```rust
let config = LanguageConfig::new();  // or load from DB
let ui = UI::new(&config);
```

### Legacy Compatibility

For gradual migration, legacy helper functions remain available:
```rust
print_section_header("💡", "Title");  // Internally uses UI
print_thinking();                      // Internally uses UI
```

These are deprecated and should be replaced with `UI` methods in new code.

### Rules for Contributors

1. **No Direct Terminal Output:** Use `UI` methods, not `println!` or ANSI codes
2. **Semantic Over Visual:** Specify what (error, success), not how (red, green)
3. **Let UI Handle Adaptation:** UI layer applies language, tone, emoji, and color based on terminal capabilities
4. **Test Both Modes:** Test your output in both rich terminal and ASCII-only mode

### Implementation Location

```rust
// crates/anna_common/src/display.rs
pub struct UI {
    profile: LanguageProfile,
    caps: TerminalCapabilities,
}

impl UI {
    pub fn success(&self, message: &str) { /* ... */ }
    pub fn error(&self, message: &str) { /* ... */ }
    // ... 15+ methods total
}
```

---

## 6. Interaction Rules for Language Changes

Users change Anna's language through natural commands, not configuration files.

### Supported Command Patterns

Anna recognizes language change requests in multiple forms:

**English:**
- "use English"
- "speak English"
- "switch to English"

**Spanish:**
- "cambia al español"
- "usa español"
- "habla español"

**French:**
- "parle français"
- "utilise le français"

**German:**
- "wechsle zu Deutsch"
- "spreche Deutsch"

**Norwegian:**
- "snakk norsk"
- "bruk norsk"

**Portuguese:**
- "fala português"
- "usa português"

### Implementation Flow

1. **User Issues Command**
   - In REPL: `> use Spanish`
   - One-shot: `annactl "cambia al español"`

2. **Intent Detection**
   - `intent_router.rs` detects Language intent
   - Extracts requested language from command

3. **Language Change**
   - Load current `LanguageConfig` from database
   - Set `user_language` to new preference
   - Save updated config to database

4. **Confirmation (Critical Rule)**
   - **The confirmation MUST be displayed in the NEW language**
   - Create new UI instance with new config
   - Display localized confirmation

**Example:**
```
User: "cambia al español"

Anna:
Idioma cambiado ✓
Ahora hablo: Español
```

### Persistence

- Language preference is immediately saved to `user_preferences` table in context database
- Loaded automatically on next session
- Path: `~/.local/share/anna/context.db` (Linux) or system-appropriate location

### Applies To

This behavior is consistent across:
- REPL (conversational mode)
- One-shot commands (`annactl "..."`)
- Future conversational endpoints

### Implementation Location

```rust
// crates/annactl/src/intent_router.rs
pub enum Intent {
    Language { language: Option<String> },
    // ...
}

pub fn route_intent(input: &str) -> Intent {
    // Detects language change patterns
}

// crates/annactl/src/repl.rs & main.rs
Intent::Language { language } => {
    // Load config, change language, save to DB
    // Confirm in NEW language
}
```

---

## 7. Fallback, Safety, and Error Handling

Anna must never crash due to language or formatting problems. Graceful degradation is mandatory.

### Database Unavailability

**Scenario:** Context database cannot be opened or is corrupted.

**Behavior:**
1. Log warning (if logging is available)
2. Use in-memory `LanguageConfig` for current session
3. Fall back to system locale detection
4. If locale detection fails, use English
5. Continue operating normally

**Implementation:**
```rust
let config = if let Ok(db) = &db_result {
    db.load_language_config().await.unwrap_or_default()
} else {
    LanguageConfig::new()  // Falls back to locale → English
};
```

### Terminal Capability Detection Failure

**Scenario:** Cannot determine terminal capabilities (rare).

**Behavior:**
1. Default to safe mode:
   - `ColorSupport::None`
   - `unicode_support: false`
   - `emoji_support: false`
   - `is_tty: false`
2. Continue operating in ASCII-only, no-color mode
3. All output remains readable

**Implementation:**
```rust
impl TerminalCapabilities {
    pub fn detect() -> Self {
        // Never panics
        // Returns safe defaults if detection fails
    }
}
```

### Language Profile Loading Failure

**Scenario:** Requested language has incomplete translation data (bug).

**Behavior:**
1. Fall back to English translations for missing strings
2. Log error (if logging available)
3. Continue operating

**Current Status:** All supported languages have complete translation tables. This is defensive programming for future extensions.

### NO_COLOR Environment Variable

**Scenario:** User sets `NO_COLOR=1` environment variable.

**Behavior:**
- Disable all color output
- Continue using emoji and Unicode if terminal supports them
- Respect user preference absolutely

**Implementation:**
```rust
pub fn should_use_color() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    std::io::stdout().is_terminal()
}
```

### Startup Logging

At startup or first use, Anna should log (at debug level):
- Detected terminal capability profile
- Chosen language and source (user preference, locale, or fallback)
- Color support level
- Unicode/Emoji availability

This aids debugging user issues related to formatting.

**Example Debug Log:**
```
DEBUG: Terminal capabilities detected: color=256, unicode=true, emoji=true
DEBUG: Language selected: Spanish (source: user preference)
```

### Never-Fail Guarantee

The following operations must never panic:
- Terminal capability detection
- Language config loading
- Language profile creation
- UI method calls (success, error, etc.)
- Emoji/Unicode fallback rendering

If any of these fail internally, they should log and return safe defaults.

---

## 8. Testing Requirements

The language and UI systems must maintain comprehensive test coverage.

### Mandatory Test Categories

#### 1. Language Parsing Tests

**Purpose:** Verify all supported languages can be parsed from various inputs.

**Tests:**
- `test_language_parsing` - ISO codes, native names, case-insensitive matching
- All language codes (en, es, no, de, fr, pt)
- Invalid inputs return `None`

**Location:** `crates/anna_common/src/language.rs::tests`

#### 2. Language Priority System Tests

**Purpose:** Verify deterministic priority order is enforced.

**Tests:**
- `test_language_priority` - User > System > English hierarchy
- Explicit user choice overrides system locale
- Clearing user preference reverts to system/English
- Database persistence survives across sessions

**Location:** `crates/anna_common/src/language.rs::tests`

#### 3. Terminal Capability Tests

**Purpose:** Verify capability detection and fallback behavior.

**Tests:**
- `test_terminal_detection` - Color levels properly detected
- `test_emoji_support_detection` - Emoji vs ASCII fallback
- All color support levels valid (None/Basic16/Extended256/TrueColor)
- Forced ASCII mode works correctly

**Location:** `crates/anna_common/src/language.rs::tests`

#### 4. Tone Profile Tests

**Purpose:** Verify language-specific tone settings.

**Tests:**
- `test_tone_profiles` - Each language has correct tone profile
- Formality levels set correctly
- Contraction usage matches language expectations
- Emoji style appropriate for language

**Location:** `crates/anna_common/src/language.rs::tests`

#### 5. Translation Loading Tests

**Purpose:** Verify translations load correctly for each language.

**Tests:**
- `test_translation_loading` - All translation strings present
- `test_language_native_names` - Native names correct
- Spanish has "sí"/"no" for yes/no
- French has "oui"/"non"
- All supported languages have complete translation tables

**Location:** `crates/anna_common/src/language.rs::tests`

#### 6. No Language Mixing Tests (Critical)

**Purpose:** Verify single-language responses, no contamination.

**Tests:**
- `test_no_language_mixing` - Single config produces single language
- User preference always wins over system locale
- Translations all in same language
- No mixed English/Spanish fragments

**Location:** `crates/anna_common/src/language.rs::tests`

#### 7. UI Integration Tests

**Purpose:** Verify UI layer respects language and terminal capabilities.

**Tests:**
- `test_language_integration` - UI uses correct language profile
- Color fallback works
- Emoji substitution works
- Box-drawing substitution works

**Location:** `crates/anna_common/src/display.rs::tests`

#### 8. Regression Tests

**Purpose:** Prevent reintroduction of known bugs.

**Tests to Add:**
- Mixed language responses (if this bug occurs)
- Broken Unicode in ASCII terminals (if this bug occurs)
- Language change not persisting (if this bug occurs)

### Running Tests

```bash
# All language tests
cargo test -p anna_common language

# All display tests
cargo test -p anna_common display

# Full test suite
cargo test --workspace
```

### Test Coverage Requirements

- Language parsing: 100% of supported languages
- Priority system: All 4 priority levels
- Terminal capabilities: All 4 color levels + Unicode/Emoji
- Tone profiles: All 6 languages
- No language mixing: At least 2 language combinations

### Adding New Tests

When adding new language features:
1. Add test for the feature
2. Add regression test if fixing a bug
3. Ensure existing tests still pass
4. Update this section of the contract

---

## 9. Versioning

This contract is versioned independently of Anna's release version.

**Current Version:** 1.0

### Semantic Versioning

- **Major version (X.0):** Breaking changes to rules or behavior
  - Example: Changing language priority order
  - Example: Removing a supported language
  - Example: Incompatible changes to UI method signatures

- **Minor version (1.X):** Backward-compatible additions
  - Example: Adding a new supported language
  - Example: Adding new UI methods
  - Example: Extending terminal capability detection

- **Patch version (1.0.X):** Clarifications, fixes, documentation updates
  - Example: Fixing typos in this document
  - Example: Adding examples
  - Example: Clarifying ambiguous rules

### Compatibility

**Anna Minimum Version:** 5.3.0-beta.1

This contract requires Anna version 5.3.0-beta.1 or later, which includes:
- Complete language system (`anna_common::language`)
- UI abstraction layer (`anna_common::display`)
- Database persistence for language preferences
- Intent routing for natural language commands

### Changing the Contract

**Before Making Breaking Changes:**
1. Discuss the change with the team
2. Document the reason for the change
3. Update all affected code
4. Update all tests
5. Bump the contract version
6. Update Anna's minimum version if required

**After Making Changes:**
1. Update this document
2. Commit with message: `docs: update LANGUAGE_CONTRACT to vX.Y`
3. Announce the change to contributors

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-11-14 | Initial contract based on Task 3 implementation |

---

## 10. Modules That Must Comply

This section tracks which modules currently follow the Language Contract and which are pending migration.

### ✅ Fully Compliant Modules

These modules correctly use the UI abstraction layer and respect language preferences:

- **`crates/annactl/src/repl.rs`**
  - REPL (conversational interface)
  - All intent handlers use `UI` methods
  - Language change confirmations in new language
  - Error messages use `UI.error()`

- **`crates/annactl/src/main.rs`**
  - One-shot command handler (`handle_one_shot_query`)
  - All intent handlers updated
  - Personality and language change flows integrated

- **`crates/annactl/src/suggestion_display.rs`**
  - Suggestion formatting uses `UI` methods
  - Section headers, bullet lists, emoji fallback

- **`crates/annactl/src/intent_router.rs`**
  - Detects language change intents
  - Multilingual command pattern recognition

- **`crates/anna_common/src/display.rs`**
  - Core UI abstraction implementation
  - Terminal capability detection
  - Emoji and Unicode fallback

- **`crates/anna_common/src/language.rs`**
  - Language priority system
  - Tone profiles and translations
  - Database persistence

### ⚠️ Partially Compliant Modules

These modules exist but have not been fully updated to use the UI abstraction:

- **`crates/annactl/src/report_display.rs`**
  - Report generation (called less frequently)
  - Uses some legacy display functions
  - **Migration Priority:** Medium

- **`crates/annactl/src/action_executor.rs`**
  - Action execution and confirmation prompts
  - Low UI impact (mostly subprocess execution)
  - **Migration Priority:** Low

- **`crates/anna_common/src/personality.rs`**
  - Personality trait settings (humor, verbosity)
  - Currently separate from language system
  - **Migration Priority:** Medium (should eventually merge with tone profiles)

### ❌ Non-Compliant Modules

These modules do not use the UI abstraction and may produce output that violates the contract:

- **Daemon modules (`crates/annad/src/**`)**
  - Daemon logging and notifications
  - Background processes with no direct user interaction
  - **Migration Priority:** Low (daemon output is for logs, not user-facing)
  - **Exception:** System logs may use direct `tracing` macros

- **`crates/annactl/src/repair_command.rs`** (if exists)
  - Self-repair commands
  - **Migration Priority:** Medium

- **Legacy test output**
  - Some tests may print directly for debugging
  - **Migration Priority:** Very Low (test output is not user-facing)

### Migration Roadmap

**High Priority (Next Release):**
- None remaining - core interface is compliant

**Medium Priority (Future Release):**
- `report_display.rs` - Convert to UI abstraction
- `personality.rs` - Integrate with language tone profiles
- `repair_command.rs` - Update prompts and confirmations

**Low Priority (Incremental):**
- `action_executor.rs` - Standardize subprocess output formatting
- Daemon modules - Consider structured logging that respects locale

**Out of Scope:**
- Pure background processes (no user interaction)
- Internal debug logging (use `tracing` crate)
- Test scaffolding

### Adding New Modules

When creating new user-facing modules:
1. ✅ Use `UI::auto()` for output
2. ✅ Never use `println!` or `eprintln!` for user messages
3. ✅ Load `LanguageConfig` if you need translations
4. ✅ Add tests that verify language and terminal fallback behavior
5. ✅ Update this section to list your module as compliant

---

## Appendix: Quick Reference

### Creating a UI Instance

```rust
use anna_common::display::UI;

// Automatic (recommended)
let ui = UI::auto();

// Explicit with config
use anna_common::language::LanguageConfig;
let config = LanguageConfig::new();
let ui = UI::new(&config);
```

### Displaying Messages

```rust
ui.success("Operation completed successfully");
ui.error("Failed to connect to database");
ui.warning("This action cannot be undone");
ui.info("System is up to date");
```

### Detecting Language

```rust
use anna_common::language::{Language, LanguageConfig};

let config = LanguageConfig::new();
let lang = config.effective_language();

match lang {
    Language::English => { /* ... */ },
    Language::Spanish => { /* ... */ },
    // ...
}
```

### Changing Language

```rust
use anna_common::language::{Language, LanguageConfig};
use anna_common::context::db::{ContextDb, DbLocation};

let mut config = LanguageConfig::new();
config.set_user_language(Language::Spanish);

// Save to database
let db_location = DbLocation::auto_detect();
if let Ok(db) = ContextDb::open(db_location).await {
    db.save_language_config(&config).await?;
}

// Confirm in new language
let ui = UI::new(&config);
let profile = config.profile();
ui.success(&format!("{} ✓", profile.translations.language_changed));
```

### Checking Terminal Capabilities

```rust
use anna_common::language::TerminalCapabilities;

let caps = TerminalCapabilities::detect();

if caps.use_emojis() {
    println!("✓ Emoji supported");
} else {
    println!("[OK] Emoji not supported, using text");
}
```

---

## Contact & Contributions

For questions about this contract:
- Open an issue on GitHub with tag `[language-contract]`
- Reference specific section numbers for clarity

For proposed changes:
- Open a PR with changes to this document
- Include rationale for the change
- Update version number if breaking

---

**End of Language Contract v1.0**
