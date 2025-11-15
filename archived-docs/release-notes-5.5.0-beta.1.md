## [5.5.0-beta.1] - 2025-11-14

### Phase Next: Autonomous LLM Setup & Auto-Update

**Anna now sets up her own brain and updates herself automatically.**

This release transforms Anna from a prototype into a production-ready assistant that can bootstrap herself completely autonomously while maintaining absolute transparency and user control.

#### Major Features

**1. First-Run LLM Setup Wizard** (`crates/annactl/src/llm_wizard.rs`)

The first time you talk to Anna, she guides you through setting up her "brain":

  annactl
  # or
  annactl "how are you?"

Anna will:
- Assess your hardware capabilities (RAM, CPU, GPU)
- Present three options with clear trade-offs:
  - **Local model** (privacy-first, automatic) - Recommended
  - **Remote API** (faster, but data leaves machine) - Explicit opt-in with warnings
  - **Skip for now** (limited conversational ability) - Can set up later

**Local Setup (Automatic):**
- Installs Ollama via pacman or AUR (yay)
- Downloads appropriate model based on hardware:
  - Tiny (1.3 GB): llama3.2:1b for 4GB RAM, 2 cores
  - Small (2.0 GB): llama3.2:3b for 8GB RAM, 4 cores
  - Medium (4.7 GB): llama3.1:8b for 16GB RAM, 6+ cores
- Enables and starts Ollama service
- Tests that everything works
- **Zero manual configuration required**

**Remote Setup (Manual):**
- Collects OpenAI-compatible API endpoint
- Stores API key environment variable name
- Configures model name
- Shows clear privacy and cost warnings before collecting any information

**Skip Setup:**
- Anna works with built-in rules and Arch Wiki only
- Can set up brain later: `annactl "set up your brain"`

**2. Hardware Upgrade Detection** (`crates/anna_common/src/llm_upgrade.rs`)

Anna detects when your machine becomes more powerful:
- Stores initial hardware capability at setup time
- Re-assesses on daemon startup
- Detects RAM/CPU improvements
- Offers **one-time** brain upgrade suggestion:

  🚀 My Brain Can Upgrade!

  Great news! Your machine got more powerful.
  I can now upgrade to a better language model:

    New model: llama3.1:8b
    Download size: ~4.7 GB

  To upgrade, ask me: "Upgrade your brain"

Notification shown once, never nags.

**3. Automatic Binary Updates** (`crates/annad/src/auto_updater.rs`)

Every 10 minutes, Anna's daemon:
- Checks GitHub releases for new versions
- Downloads new binaries + SHA256SUMS
- **Verifies checksums cryptographically** (fails on mismatch)
- Backs up current binaries using file backup system
- Atomically swaps binaries in `/usr/local/bin`
- Restarts daemon seamlessly
- Records update for notification

**Safety guarantees:**
- ✅ Cryptographic verification prevents corrupted/malicious binaries
- ✅ Atomic operations - no partial states
- ✅ Automatic backups with rollback capability
- ✅ Respects package manager installations (does not replace AUR/pacman binaries)

**4. Update Notifications** (`crates/annactl/src/main.rs`)

Next time you interact with Anna after an update:

```
✨ I Updated Myself!

I upgraded from v5.4.0 to v5.5.0

What's new:
  • Added automatic brain upgrade detection
  • Improved LLM setup wizard UX
  • Fixed permission handling for Ollama
  • Enhanced changelog parsing

[Then answers your question normally]
```

- Parses CHANGELOG.md for version-specific details
- Shows 2-4 key changes
- Displayed in user's configured language
- Shown **once per update**, then cleared

#### Infrastructure Improvements

**5. Data-Driven Model Profiles** (`crates/anna_common/src/model_profiles.rs`)

```rust
pub struct ModelProfile {
    id: String,              // "ollama-llama3.2-3b"
    engine: String,          // "ollama"
    model_name: String,      // "llama3.2:3b"
    min_ram_gb: u64,
    recommended_cores: usize,
    quality_tier: QualityTier,  // Tiny/Small/Medium/Large
    size_gb: f64,
}
```

- Easy to add new models by updating array
- Hardware-aware selection via `select_model_for_capability()`
- Upgrade detection via `find_upgrade_profile()`

**6. Enhanced LLM Configuration** (`crates/anna_common/src/llm.rs`)

```rust
pub enum LlmMode {
    NotConfigured,  // Triggers wizard
    Local,          // Privacy-first
    Remote,         // Explicit opt-in
    Disabled,       // User declined
}

pub struct LlmConfig {
    mode: LlmMode,
    backend: LlmBackendKind,
    model_profile_id: Option<String>,  // NEW: For upgrade detection
    cost_per_1k_tokens: Option<f64>,   // NEW: Cost tracking
    safety_notes: Vec<String>,         // NEW: User warnings
    // ... existing fields
}
```

**7. Generic Preference Storage** (`crates/anna_common/src/context/db.rs`)

```rust
impl ContextDb {
    pub async fn save_preference(&self, key: &str, value: &str) -> Result<()>
    pub async fn load_preference(&self, key: &str) -> Result<Option<String>>
}
```

Used for:
- Initial hardware capability storage
- Pending brain upgrade suggestions
- Update notification state

#### User Experience Flow

**First interaction:**
```bash
$ annactl "how are you?"

🧠 Setting Up My Brain

Let me check your machine's capabilities...

💻 Hardware Assessment
System: 8GB RAM, 4 CPU cores
Capability: Medium - Good for local LLM with small models

⚙️ Configuration Options
1. Set up a local model automatically (recommended - privacy-first)
2. Configure a remote API (OpenAI-compatible) instead
3. Skip for now and use rule-based assistance only

Choose an option (1-3): 1

🏠 Local Model Setup

I will:
  • Install or enable Ollama if needed
  • Download model: llama3.2:3b (~2.0 GB)
  • Start the service and test it

Proceed with setup? (y/n): y

[Downloads and configures automatically]

✓ My local brain is ready!
I can now understand questions much better while keeping
your data completely private on this machine.

[Now answers original question: "how are you?"]
```

**After auto-update:**
```bash
$ annactl

✨ I Updated Myself!

I upgraded from v5.4.0 to v5.5.0

What's new:
  • Added automatic brain upgrade detection
  • Improved LLM setup wizard UX
  • Fixed permission handling for Ollama

[Continues to REPL normally]
```

#### Testing

Added comprehensive test coverage:
- **21 tests** for LLM configuration and routing
- **6 tests** for hardware upgrade detection
- All tests passing ✅

**Test Coverage:**
- Wizard produces correct configs (local/remote/skip)
- LLM routing handles configured/disabled states safely
- Capability comparison logic (Low < Medium < High)
- Upgrade detection only triggers when truly improved
- No false positives for brain upgrades

#### Documentation

**Updated:**
- `README.md`: Added "First-Run Experience" and "Auto-Update System" sections
- `docs/USER_GUIDE.md`: Added detailed LLM setup and auto-update guides (500+ lines)
- `CHANGELOG.md`: This comprehensive entry

#### Privacy & Safety

**Privacy guarantees:**
- Local LLM is default recommendation
- Remote API requires explicit opt-in with clear warnings
- All LLM output is text-only, never executed
- Telemetry stays local unless user explicitly configures remote API

**Security guarantees:**
- SHA256 checksum verification for all downloads
- Atomic binary operations
- File backup system with rollback
- Package manager detection and respect

#### Performance

- First-run wizard: ~3-10 minutes (includes model download)
- Subsequent startups: No overhead
- Auto-update check: ~100ms every 10 minutes (background)
- Update download + install: ~30 seconds (including daemon restart)

#### Migration Notes

- Existing users: First interaction triggers wizard
- Package-managed installations: Auto-update disabled automatically
- No breaking changes to existing functionality

#### What's Next

This release completes the "bootstrap autonomy" milestone. Anna can now:
- Set up her own brain with zero manual steps
- Keep herself updated automatically
- Detect and suggest hardware-appropriate upgrades
- Operate transparently with one-time notifications

**Version**: 5.5.0-beta.1
**Status**: Production-ready for manual installations
**Tested on**: Arch Linux x86_64

---
