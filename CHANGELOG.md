# Changelog

All notable changes to Anna will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.103] - 2025-12-06

### Added - Recipe Feedback System (Phase 23)

**User Feedback for Recipe Answers**

Users can now rate whether a recipe answer was helpful. Feedback adjusts
recipe reliability scores, improving future matches.

**Key Changes**:

- `recipe_feedback.rs` - New module with FeedbackRating enum, RecipeFeedback struct
- `FeedbackRating::Helpful` - Increases reliability_score (+1, max 99) and success_count
- `FeedbackRating::NotHelpful` - Decreases reliability_score (-5, min 50)
- `FeedbackRating::Partial` - Increases success_count only
- `log_feedback()` - Appends all feedback to `~/.anna/feedback_history.jsonl`
- `apply_feedback()` - Updates recipe file on disk

**New CLI Command**:
```
annactl feedback <recipe_id> -r <rating> [-c "optional comment"]
```

Rating options: `helpful`, `partial`, `not-helpful` (aliases: good/yes, ok/meh, bad/no)

**New RPC Method**: `RecipeFeedback`

This closes the learning loop: Anna learns recipes from specialists,
applies them instantly, and improves based on user feedback.

## [0.0.102] - 2025-12-06

### Added - Recipe Direct Answers (Phase 22)

**Recipe-Based Direct Answers - Skip Everything!**

When a recipe matches with high confidence AND has an answer template,
Anna now returns the answer immediately without running probes or calling
specialists. This makes responses near-instant for learned patterns.

**Key Changes**:

- `build_recipe_result()` - Creates ServiceDeskResult directly from recipe
- `can_answer_directly()` - Checks if recipe has an answer template
- Early return in `rpc_handler.rs` when recipe can answer directly
- Refactored `team_to_domain()` helper for cleaner code

**Performance Impact**:

For queries like "enable syntax highlighting in zsh":
- Before: Router → Translator (LLM) → Probes → Specialist (LLM) → Answer
- After: Router → Recipe Match → Answer (instant!)

This completes the recipe learning vision: Anna learns from specialists,
then applies learned recipes instantly without any LLM calls.

## [0.0.101] - 2025-12-06

### Added - Recipe Fast Path Integration (Phase 21)

**Recipe Fast Path - Skip LLM for Learned Queries**

Wired the recipe matcher into the main request handler:

- `recipe_fast_path.rs` - Checks recipes BEFORE calling LLM translator
- Recipe index is built from disk at daemon startup and stored in state
- High-confidence recipe matches skip LLM entirely (score >= 70)

**Key Flow**:
1. User query comes in (e.g., "enable syntax highlighting in zsh")
2. Router classifies as Unknown -> triggers recipe check
3. Recipe fast path matches built-in zsh syntax highlighting recipe
4. Ticket created from recipe, LLM translator skipped entirely
5. User gets instant response with configuration instructions

**New Query Classes**
- `ConfigureShell` - Shell config queries (bash, zsh, fish)
- `ConfigureGit` - Git config queries

**Recipe Sources Checked (in order)**:
1. Learned recipes from RecipeIndex (saved from previous successful results)
2. Built-in shell recipes (colored prompt, git prompt, syntax highlighting, etc.)
3. Built-in git recipes (user identity, aliases, editor, etc.)

This implements the user's vision: Anna LEARNS from specialists and can
apply learned recipes to SIMILAR queries without LLM, making responses
near-instant for known patterns.

## [0.0.100] - 2025-12-06

### Added - Recipe Matcher & Config Recipes (Phase 20)

**Recipe Matcher for Translator Fast-Path**

The translator can now check for matching recipes BEFORE calling the specialist:

- `match_recipe()` - finds similar recipes by tokenized query
- `match_config_recipe()` - finds config recipes for intent+target
- `match_action_recipe()` - finds package/service action recipes
- Substitution extraction for adapting recipes to new targets
- High-confidence matches skip LLM entirely

**Key Insight**: Anna LEARNS from specialists, then the translator can apply
learned recipes to SIMILAR queries without needing the slow LLM path.

**Shell Configuration Recipes** (`shell_recipes.rs`)

Built-in recipes for bash, zsh, and fish:

- Colored prompt (PS1 customization)
- Git branch in prompt
- Syntax highlighting (zsh/fish)
- Auto-suggestions (zsh/fish)
- Colored ls output
- History settings
- Common aliases

**Git Configuration Recipes** (`git_recipes.rs`)

Built-in recipes for .gitconfig:

- User identity (name, email) - with parameter prompts
- Default branch (main)
- Editor selection (vim, nano)
- Colored output
- Common aliases (st, co, br, ci, lg)
- Push/pull defaults
- Credential helpers
- Merge/diff tools

**New RecipeKind Variants**
- `PackageInstall` - for package installation recipes
- `ServiceManage` - for systemd service recipes
- `ShellConfig` - for shell config recipes
- `GitConfig` - for git config recipes

**New Modules**
- `recipe_matcher.rs` - Fast-path recipe matching
- `shell_recipes.rs` - Shell configuration recipes
- `git_recipes.rs` - Git configuration recipes

## [0.0.99] - 2025-12-06

### Added - Natural Language Package & Service Management (Phase 19)

**Package Installation via Natural Language**

Install packages with simple commands:

```bash
annactl "install htop"
annactl "install vim"
annactl "add nano"
```

- Automatic package manager detection (pacman, apt, dnf)
- Cross-distro package name mapping
- Confirmation prompt before installation
- Known package database with descriptions

**Service Management via Natural Language**

Control systemd services naturally:

```bash
annactl "restart docker"
annactl "start sshd"
annactl "enable bluetooth"
```

- Service action detection (start, stop, restart, enable, disable, reload)
- Protected service prevention (journald, dbus, udev cannot be modified)
- Risk level assessment (Low, Medium, High)
- Rollback command suggestions

**New Query Classes**
- `InstallPackage` - Routes "install X" queries
- `ManageService` - Routes "restart X", "start X" queries

**New Module**
- `action_handlers.rs` - Handles package and service actions

## [0.0.98] - 2025-12-06

### Added - Multi-file Transactions & Recipe Systems (Phase 18)

**Multi-file Change Transactions**

Atomic transactions for multi-file config changes:

- `ChangeTransaction` - Groups multiple `ChangePlan` items
- All-or-nothing: if any change fails, all are rolled back
- Automatic rollback in reverse order
- Transaction IDs for tracking

```rust
let mut tx = ChangeTransaction::new("Configure vim settings");
tx.add(syntax_plan);
tx.add(numbers_plan);
let result = apply_transaction(&tx);
// If numbers_plan fails, syntax_plan is rolled back
```

**Package Installation Recipes**

Safe package installation with multi-manager support:

- Supported managers: pacman, apt, dnf, flatpak, snap
- Common package database (vim, git, htop, curl, etc.)
- Cross-distro package name mapping
- Confirmation prompts with install commands

**Service Configuration Recipes**

Safe systemd service management:

- Service actions: start, stop, restart, enable, disable, reload
- Risk levels: Low, Medium, High, Protected
- Protected services cannot be modified (journald, dbus, udev)
- Rollback commands for reversible actions
- Service aliases (e.g., "ssh" -> "sshd.service")

**New Modules**
- `change_transaction.rs` - Multi-file atomic transactions
- `package_recipes.rs` - Package installation recipes
- `service_recipes.rs` - Service configuration recipes

## [0.0.97] - 2025-12-06

### Added - Change History and Undo (Phase 17)

**Change History Tracking**

All config changes are now recorded for audit and undo:

- `annactl history` - View recent config changes
- `annactl undo <id>` - Restore file from backup
- Changes recorded in `~/.anna/change_history.jsonl`

**New Commands**

```bash
# View change history
annactl history

# Undo a specific change
annactl undo abc12345
```

**History Entry Fields**
- ID: Unique 8-character identifier
- Timestamp: When the change was applied
- Description: What was changed
- Target path: File that was modified
- Backup path: Location of backup file
- Status: can undo / undone / no backup

**New Modules**
- `change_history.rs` - History recording and undo logic
- `change_commands.rs` - CLI handlers for history/undo

## [0.0.96] - 2025-12-06

### Added - Desktop Team Editor Config Flow (Phase 16)

**Natural Language Editor Configuration**

Users can now say "enable syntax highlighting" and Anna will:
1. Detect installed editors (vim, nano, emacs, etc.)
2. Show proposed config change with backup path
3. Ask for confirmation before modifying files
4. Apply change using Safe Change Engine

**New ServiceDeskResult Field**

- `proposed_change: Option<ChangePlan>` - Contains change plan for user confirmation
- CLI automatically prompts user when proposed_change is present
- Works in both one-shot mode and REPL mode

**Integration Points**

- `build_editor_config_with_change()` - Creates ChangePlan from EditorRecipe
- `handle_proposed_change()` - CLI confirmation flow
- Backup-first, idempotent config modifications

**Example Flow**
```
anna> enable syntax highlighting
Detected vim installed.
This will add: syntax on to ~/.vimrc

Proposed Change
  File: ~/.vimrc
  Risk: Low
  Backup: ~/.vimrc.anna-backup-20251206-...

Apply this change? [y/N] y
  Change applied successfully.
```

## [0.0.95] - 2025-12-06

### Added - Safe Change Engine (Phase 15)

**Config File Modifications with Backup/Rollback**

Anna can now safely modify config files with automatic backup and rollback:

- **Backup-first**: Creates backup before any modification
- **Idempotent**: Safe to apply same change multiple times
- **Rollback**: Can restore from backup if needed

**New RPC Methods**

- `PlanChange` - Creates a change plan for user confirmation
- `ApplyChange` - Applies a confirmed change plan
- `RollbackChange` - Rolls back a change using backup

**Codebase Refactoring**

- Extracted `editor_recipe_data.rs` from `editor_recipes.rs`
- Both files now under 400 lines for better maintainability
- All change operations require user confirmation

**Change Engine Types**

- `ChangePlan` - Describes planned change with risk level
- `ChangeResult` - Result with applied/noop/error status
- `ChangeOperation` - EnsureLine, AppendLine operations
- `ChangeRisk` - Low/Medium/High risk classification

## [0.0.94] - 2025-12-06

### Added - Recipe Learning System (Phase 14)

**Automatic Recipe Learning**

Anna now learns from successful, high-reliability queries:

- Recipes are saved when: `verified=true` AND `reliability_score >= 80`
- Recipes include: query pattern, probes executed, answer template, domain
- Existing recipes are updated with incremented success count
- Stored in `~/.anna/recipes/{recipe_id}.json`

**New Module: `recipe_learning.rs`**

- `try_learn_from_result()` - Attempts to learn from a ServiceDeskResult
- `LearnResult` - Tracks whether learning occurred and why
- Automatic signature generation from query, domain, and probes
- Team assignment from domain (Storage, Network, Desktop, etc.)
- Intent tag extraction for future RAG-lite retrieval

**Integration Points**
- Main request handler calls `success_with_learning()` on completion
- Debug logging when recipes are learned
- Non-blocking - learning failures don't affect responses

**Learning Criteria**
- Must be verified (answer_grounded = true)
- Must have reliability score >= 80
- Must have answer text
- Must have at least one probe executed
- Clarification queries are not learned

## [0.0.93] - 2025-12-06

### Changed - Documentation Update (Phase 13)

**Project Documentation Refresh**

Updated all project documentation to reflect current version (v0.0.93):

- **README.md**: Updated features, usage examples, output modes, achievement badges
- **ROADMAP.md**: Reorganized with current focus and completed phases
- **FEATURES.md**: Added RPG stats, Service Desk Theatre, greetings, achievements

**Hollywood IT Aesthetic**

Documentation now reflects Anna's cinematic terminal style:
- ASCII-only design (no emojis)
- Named IT personas and team system
- RPG-style progression with achievements

## [0.0.92] - 2025-12-06

### Fixed - Codebase Hygiene (Phase 12)

**Warning Cleanup**

Zero compiler warnings across the entire workspace:

- Fixed unused `unlock` method in achievements.rs (marked `#[cfg(test)]`)
- Fixed unused variable warnings in router.rs tests
- Fixed dead code warning for `line_num` field in test struct
- Applied `cargo fix` to clean up unused imports in test files

**Files Fixed**
- `anna-shared/src/achievements.rs`: Test-only method
- `annad/src/router.rs`: Prefixed unused test variables
- `annad/tests/router_corpus_tests.rs`: Allow dead_code for debug field
- Multiple test files: Removed unused imports via cargo fix

## [0.0.91] - 2025-12-06

### Changed - ASCII-Style Achievement Badges (Phase 11)

**Hollywood IT Aesthetic**

Achievement badges now use ASCII art symbols instead of emojis, matching Anna's
classic terminal style:

```
Achievements
  [1] [10] <3d> (90+) {*}
  [100] Power User - Complete 100 queries
  <7d> Week Warrior - Maintain a 7-day streak
```

**Badge Styles by Category**

- **Milestones**: `[1]` `[10]` `[50]` `[100]` `[500]`
- **Streaks**: `<3d>` `<7d>` `<30d>`
- **Quality**: `(90+)` `(ok)` `(<<)`
- **Teams**: `{*}` `{df}` `{ip}` `{top}`
- **Special**: `~00~` `~05~` `[rx]` `[!!]`
- **Tenure**: `|7d|` `|30d|`

**Technical Changes**
- Renamed `emoji` field to `badge` in Achievement struct
- Updated `format_achievements()` to use ASCII badges
- Updated stats_display.rs to reference `badge` instead of `emoji`

## [0.0.90] - 2025-12-06

### Added - Achievement Badges (Phase 10)

**Gamification Achievements**

Anna now tracks and displays achievement badges in stats.

**New Module: `achievements.rs`**

22 unique achievements across categories:

- **Milestones**: First Contact, Getting Started, Regular User, Power User, Anna Expert
- **Streaks**: On Fire (3-day), Week Warrior (7-day), Monthly Master (30-day)
- **Quality**: Perfect 10, Flawless, Speed Demon
- **Teams**: Well-Rounded, Storage Savvy, Network Guru, Performance Junkie
- **Special**: Night Owl, Early Bird, Recipe Master, Solo Artist
- **Tenure**: One Week In, Month Veteran

**Features**
- `check_achievements()` - Check all achievements against stats
- `unlocked_achievements()` - Get list of earned achievements
- `format_achievements()` - Display badge summary
- `newly_unlocked()` - Detect new achievements for notifications

**Stats Display Integration**
- Achievements section shows badge row and notable unlocks
- Removed legacy RPG stats fallback (event log is now primary)
- File reduced from 476 to 379 lines

## [0.0.89] - 2025-12-06

### Added - Personalized Greetings (Phase 9)

**Time-Aware Greetings**

Anna now greets users based on time of day and remembers their name:

```
Good morning, john! How can I help you today?
Good afternoon, john. What can I do for you?
Good evening, john! Anna Service Desk at your service.
```

**New Module: `greetings.rs`**

Context-aware dialogue for a more natural experience:

- `TimeOfDay` enum with `Morning`, `Afternoon`, `Evening`, `Night`
- `anna_session_greeting()` - personalized session start greeting
- `anna_off_hours_comment()` - friendly late-night comments
- `anna_welcome_back()` - greetings based on time away
- `anna_followup_prompt()` - domain-specific follow-up suggestions
- `anna_patience_phrase()` - team-specific "please wait" messages

**Domain-Specific Context**

Follow-up prompts now match the query domain:
- Storage: "Need me to check anything else about your storage?"
- Network: "Any other network questions?"
- Services: "Should I check any other services?"

**Late Night Personality**

When working late, Anna might say:
- "Burning the midnight oil, I see!"
- "Late night debugging session?"

**Code Quality**
- Split into `dialogue.rs` (234 lines) and `greetings.rs` (257 lines)
- Both files well under 400 line limit
- Full test coverage for new functions

## [0.0.88] - 2025-12-06

### Changed - Codebase Cleanup (Phase 8)

**Removed Unused Code**

Cleaned up all compiler warnings across the workspace:

- **annactl/theatre_render.rs**: Removed unused `Tier` import and `had_junior_review` variable
- **annactl/client.rs**: Removed unused `status_snapshot()` method and `StatusSnapshot` import
- **annactl/display.rs**: Removed unused `print_repl_greeting()`, `print_repl_header()`, and `format_delta_plain()` functions; cleaned up unused imports
- **annactl/transcript_render.rs**: Changed `AnswerSource::Transcript` to unit variant (was storing unused string)
- **annad/deterministic.rs**: Removed unused `find_tool_evidence` and `find_package_evidence` imports
- **annad/handlers.rs**: Removed unused `GlobalStats` import
- **annad/health_brief_builder.rs**: Moved types to test module where they're needed
- **annad/rpc_handler.rs**: Removed unused `error` and `DeterministicResult` imports
- **annad/service_desk.rs**: Removed unused `EvidenceKind` import and `has_probes` variable
- **annad/verify_probes.rs**: Removed unused `VERIFY_PROBE_TIMEOUT_SECS` constant

**Result**
- Zero compiler warnings in the main codebase
- Reduced binary size through dead code elimination
- Cleaner, more maintainable code

## [0.0.87] - 2025-12-06

### Added - Enhanced Theatre Dialogue (Phase 7)

**Dialogue Variety System**

Internal IT communications now feel more natural with varied dialogue:

```
--- Internal ---
Anna: Michael, got a ticket coming your way. Case abc12345

Michael (Network Engineer): Looking at it now.
Michael (Network Engineer): Looks solid, confidence 85%. Good to go.

--- OR ---

Anna: Hey Lars! I have a case for you. xyz98765

Lars (Storage Engineer): On it.
Lars (Storage Engineer): I've reviewed the data. 90% confident.
```

**Features**
- Multiple phrase variations for each dialogue type
- Deterministic selection based on case ID (same case = same dialogue)
- Junior approval phrases vary by confidence level (90%+, 80%+, <80%)
- Junior escalation requests with named senior mentions
- Senior response variations
- Anna post-review phrases (different for escalated vs non-escalated)
- Team-specific checking phrases

**New Module**
- `dialogue.rs`: Centralized dialogue variety system
  - `anna_dispatch_greeting()`: Varied greetings to team members
  - `junior_approval()`: Approval phrases by confidence tier
  - `junior_escalation_request()`: Escalation to senior
  - `senior_response()`: Senior feedback
  - `anna_after_review()`: Post-review phrases
  - `team_checking_phrase()`: Team-specific progress messages

**Code Quality**
- Extracted dialogue functions to keep theatre.rs under 400 lines
- Uses deterministic hashing for consistent dialogue per case

## [0.0.86] - 2025-12-06

### Added - Usage Streaks & Achievements (Phase 6)

**Streak Tracking**

Anna now tracks your usage streaks - consecutive days you've asked for help:

```
Fun Facts
  › Anna since: Dec 1, 2025 (6 days)
  › 🔥 5 day streak! Keep it going!
  › Best streak: 5 days
  › Active on 6 different days
  › Lucky team: Storage & Filesystems (100% success rate)
```

**New Statistics**
- Current streak (with 🔥 emoji for active streaks)
- Best streak ever achieved
- Total active days (unique days with activity)
- Lucky team (team with highest success rate, ≥3 cases, ≥90%)

**Code Changes**
- `streaks.rs`: New module for streak/lucky team calculations
- `event_log.rs`: Added `current_streak`, `best_streak`, `active_days`, `lucky_team`, `lucky_team_rate` fields
- `stats_display.rs`: Shows streak and lucky team achievements
- Modularized streak logic for maintainability

## [0.0.85] - 2025-12-06

### Added - Installation Date & Tenure Tracking (Phase 5)

**Anna Since / Tenure Display**

The stats display now shows when you first started using Anna and how long you've been together:

```
Fun Facts
  › Anna since: Dec 1, 2025 (5 days)
  › Most consulted: Storage & Filesystems (42% of cases)
  ...
```

**Tenure Formatting**
- Shows human-readable dates (e.g., "Dec 6, 2025")
- Formats tenure as "today", "X days", "X weeks", "X months", or "X years, Y months"
- Tracks first and last event timestamps in event log

**Code Changes**
- `event_log.rs`: Added `first_event_ts` and `last_event_ts` to AggregatedEvents
- `time_format.rs`: New module for date/tenure formatting utilities
- `stats_display.rs`: Now uses time_format module, shows "Anna since" in Fun Facts
- Modularized to keep files under 400 lines

## [0.0.84] - 2025-12-06

### Added - Enhanced Stats Display (Phase 4)

**Fun Facts Section**

The `annactl stats` command now includes a "Fun Facts" section with interesting statistics:

```
Fun Facts
  › Most consulted: Storage & Filesystems (42% of cases)
  › Least consulted: Security (3 cases)
  › Fastest answer: 124ms
  › Longest research: 8.2s (that was a tough one!)
  › Zero timeouts! Anna always came through.
  › High performer! Avg reliability of 87%
```

**Statistics Tracked**
- Most consulted team (with percentage of total cases)
- Least consulted team (when multiple teams used)
- Fastest answer time in milliseconds
- Longest research time (shown for answers >5s)
- Zero timeouts achievement badge
- High performer badge (≥85% reliability, ≥10 cases)

**Code Changes**
- `stats_display.rs`: Added `print_fun_stats()` function
- Enhanced RPG stats section with fun facts at the end
- Uses bullet points (›) for visual consistency

## [0.0.83] - 2025-12-06

### Added - Internal IT Communications Toggle (Phase 3)

**Fly-on-the-Wall View of IT Department**

New `--internal` / `-i` flag lets users see the internal IT department communications:

```bash
# One-shot with internal view
annactl --internal "what is my disk usage?"

# REPL with internal view
annactl -i
```

**What you see with --internal:**
- Anna dispatching cases to team members
- Junior specialists reviewing answers
- Senior escalation conversations
- Team collaboration dialogue

**Example Internal Mode Output:**
```
[you] what is my disk usage?

Checking disk space...

--- Internal ---
Anna: Hey Lars! I have a case for you. CN-ABC12345
Lars (Storage Engineer): Got it, Anna. I've reviewed the data. Looks good, confidence 85%.

[anna]
Your disk usage:
- /: 45% used

Storage & Filesystems | Based on system data | 85%
```

**CLI Changes:**
- Added `--internal` / `-i` global flag to annactl
- Works with both REPL and one-shot modes
- Shows `[internal mode]` indicator when active

**Code Changes:**
- `main.rs`: Added show_internal CLI argument
- `commands.rs`: Pass show_internal through handlers
- `transcript_render.rs`: Already had render_with_options, now used

## [0.0.82] - 2025-12-06

### Added - Theatre REPL Greeting (Phase 2)

**Personal, Aware REPL Greeting**

Anna now greets users like a real IT support person who knows them and their system:

```
Anna Service Desk
─────────────────────────────────────────────────────────────
Hello lhoqvso!

It's been about 1 day since you checked with me!

Since the last time, a few things happened:

  › Your boot time increased by 2 seconds.
    This is normal variation, nothing to worry about.
  › No warnings or errors detected - looking good!

Systems ready. Translator: qwen3:1.7b, Specialist: qwen3:8b

But I believe you want to ask me something, don't you?
```

**Features**
- Personalized greeting based on time since last visit
- Health delta report showing what changed
- Boot time tracking with context
- Failed service detection and reporting
- Service recovery notifications
- Disk/memory warning thresholds
- LLM readiness status
- Update availability notifications

**New Module**
- `annactl/src/greeting.rs`: Theatre-style REPL greeting
  - `print_theatre_greeting()`: Main entry point
  - `InteractionInfo`: Tracks user visit history
  - Bullet-pointed change list (max 4 items)

**Code Changes**
- `commands.rs`: REPL now passes status to greeting
- Greeting shows "Since last time" section with health deltas

## [0.0.81] - 2025-12-06

### Added - Service Desk Theatre (Phase 1)

**Narrative UX Layer - Cinematic IT Department Experience**

This release begins the transformation of Anna's output into a "fly on the wall"
IT department experience. Users now see:

- **Named personas**: 18 IT department members with real names
  - Network: Michael (Junior), Ana (Senior)
  - Storage: Lars (Junior), Ines (Senior)
  - Desktop: Sofia (Junior), Erik (Senior)
  - And more across all teams...

- **Theatre rendering**: Clean mode now uses cinematic narrative flow
  - Shows what Anna is checking (not raw probe output)
  - Internal IT communications can be shown with `--internal` flag
  - Professional footer with domain context and reliability

**New Modules**
- `anna-shared/src/theatre.rs`: Core narrative building system
  - `NarrativeSegment`: Typed dialogue chunks with speaker info
  - `Speaker`: Enum for Anna, You, TeamMember, Narrator
  - `NarrativeBuilder`: Fluent API for building narratives
  - `describe_check()`: Human-friendly probe descriptions

- `annactl/src/theatre_render.rs`: Theatre-mode renderer
  - Replaces render_clean in normal mode
  - Shows IT department style output
  - Supports internal communications toggle

**Code Changes**
- `transcript_render.rs`: Now delegates to theatre_render in clean mode
- `lib.rs`: Added theatre module export

**Example Output (debug OFF)**
```
[you] what is my disk usage?

Checking disk space...

[anna]
Your disk usage:
- /: 45% used (120GB of 256GB)
- /home: 78% used (400GB of 512GB)

Storage & Filesystems | Based on system data | 85% | Verified from disk
```

## [0.0.80] - 2025-12-06

### Fixed - STABILIZE: Answer Minimality (B1)

**Answer Minimality Enforcement (B1: Fixed)**
- "free memory" and "available memory" now correctly route to `MemoryFree` class
- Previously these queries incorrectly routed to `MemoryUsage`, returning broader data
- B1 principle: Specific questions should get specific answers
  - "free memory" → MemoryFree (just free/available RAM)
  - "memory usage" → MemoryUsage (used + total)
  - "how much ram" → RamInfo (general RAM info)

**Code Changes**
- `query_classify.rs`: Moved "free memory" and "available memory" patterns to MemoryFree block
- `router_tests.rs`: Fixed outdated test assertions, added B1 minimality tests
  - `test_free_memory_routes_to_memory_free()` - verifies B1 fix
  - `test_memory_usage_distinct_from_memory_free()` - verifies separation

**Tests Added**
- Golden tests for answer minimality enforcement
- Tests verify that "free memory" ≠ "memory usage" ≠ "how much ram"

## [0.0.79] - 2025-12-06

### Fixed - STABILIZE: Stats Tracking

Continuation of stabilization work.

**Stats Consistency (B6: Fixed)**
- Added `GlobalStats` field to `DaemonStateInner` in state.rs
- Stats are now actually tracked and persisted in daemon state
- `handle_stats` now returns actual tracked stats instead of empty `GlobalStats::new()`
- Added `record_request()` method to track:
  - Fast path hits (deterministic routes)
  - Total requests
  - Translator timeouts
  - Specialist timeouts
- Stats are recorded when each request completes in rpc_handler.rs

**Code Changes**
- `state.rs`: Added `stats: GlobalStats` field and `record_request()` method
- `handlers.rs`: Updated `handle_stats` to read from daemon state
- `rpc_handler.rs`: Call `record_request()` when request completes

**Probe Accounting (B3: Verified)**
- Reviewed probe accounting logic
- `ticket_probes_planned` correctly captured from `ticket.needs_probes.len()`
- `ProbeStats::from_results()` creates accurate stats from actual results
- MetaSmallTalk and ConfigFileLocation correctly declare empty probes

## [0.0.78] - 2025-12-06

### Fixed - STABILIZE: Continued Hygiene

Continuation of v0.0.77 stabilization work.

**Reliability Score Alignment (B4: In Progress)**
- Reviewed reliability scoring flow through service_desk.rs
- Deterministic answers correctly set `answer_grounded=true` when `parsed_data_count > 0`
- Evidence required flag properly propagated from route capability
- Score penalties only apply when conditions warrant

**Updater Integrity (B7: Verified)**
- Verified asset verification before reporting available version
- Checksum verification (SHA256SUMS) for all downloads
- Binary version verification before installation
- Atomic pair update with rollback on failure
- Pair consistency check post-installation
- No code changes needed - updater is solid since v0.0.73

**Version Bump**
- Workspace version: 0.0.78
- All tests passing

## [0.0.77] - 2025-12-06

### Fixed - STABILIZE: Routing, Determinism, and Consistency

This is a stabilization release focused on correctness, truthfulness, and consistency.
No new user-facing features; only bug fixes and hygiene improvements.

**Meta/Small-Talk Classification (B2: Fixed)**
- New `QueryClass::MetaSmallTalk` for greetings and meta questions
- Pattern matching for: "how are you", "what is your name", "are you using llm", etc.
- Bypasses LLM translator entirely - instant deterministic response
- No longer routes to domain=security for meta questions

**Deterministic Handlers for Common Safe Questions (B5: Fixed)**
- New `QueryClass::KernelVersion` for "kernel version", "uname" queries
- New `QueryClass::ConfigFileLocation` for "where is vim config", "hyprland config" etc.
- Deterministic config path answers for 20+ common tools:
  - Editors: vim, nvim, nano, emacs
  - Shells: bash, zsh, fish
  - WMs: hyprland, sway, i3
  - Terminals: alacritty, kitty
  - Utilities: tmux, git, ssh, rofi, waybar, dunst, picom, polybar
- Added "uname" to translator probe mappings
- All new handlers bypass LLM for instant, grounded responses

**Query Classification Updates**
- Added MetaSmallTalk, KernelVersion, ConfigFileLocation to `classify_query()`
- Updated `is_fast_path()` to include MetaSmallTalk
- Updated Display and from_str for new QueryClass variants
- All query classes enumerated in evidence contract tests

**Probe Mapping**
- Added `uname` → `uname -a` in translator.rs

## [0.0.76] - 2025-12-06

### Added - Version Sanity + Model Registry + Correctness Fixes

This release ensures version consistency across all components and adds config-driven model registry for future model upgrades.

**Version Consistency (Single Source of Truth)**
- Workspace `Cargo.toml` is the canonical version source
- `anna_shared::VERSION` derived from `CARGO_PKG_VERSION`
- All binaries (annactl, annad) use shared version constant
- VERSION file kept in sync with workspace version
- Hard gate tests fail CI if versions mismatch

**Model Registry Configuration (config.rs)**
- New `ModelRegistryConfig` struct for model selection:
  - `translator`: Fast model for query classification
  - `specialist_default`: Default model for all domains
  - `specialist_overrides`: Domain-specific models (HashMap)
  - `preferred_family`: Model family preference (qwen2.5, qwen3-vl, llama3.2)
- `get_specialist(domain, tier)`: Lookup with fallback chain
- Domain:tier format for granular control (e.g., "security:senior")
- Qwen3-VL placeholders ready for future upgrade
- TOML configuration support with safe defaults

**Status Display Enhancements**
- `annactl status` shows installed, daemon, and available versions
- Update check timing: last_check_at, next_check_at, check_pace
- available_version with check state indicator
- Version mismatch warning in health section

**Tests Added**
- `test_model_registry_default`
- `test_model_registry_get_specialist_default`
- `test_model_registry_get_specialist_with_overrides`
- `test_model_registry_all_models`
- `test_model_registry_parse_toml`
- `test_config_invalid_falls_back_safely`

## [0.0.75] - 2025-12-06

### Added - UX Realism + Stats/RPG + Recipes + Citations

This release adds foundation for UX realism, RPG-style stats progression, recipe learning, and local-only citations.

**Streaming Presentation Protocol (presentation.rs)**
- New `PresentationEvent` enum for UX updates:
  - `RequestStarted`, `TicketCreated`, `StageStart`, `StageEnd`
  - `CheckingSource`, `EvidenceGathered`, `ThinkingUpdate`
  - `ActionProposed`, `ConfirmationNeeded`, `ActionComplete`
  - `AnswerReady`, `ErrorOccurred`
- `PresentationStage` enum: Routing, Probing, Analyzing, Drafting, Verifying
- `Technician` struct with domain-specific personas:
  - Network → Alex, Performance → Sam, Storage → Jordan
  - Desktop → Taylor, Security → Casey
- `TechnicianTier` enum: Frontline, Senior, Manager

**Unified Result Signals (result_signals.rs)**
- `ResultSignals` struct for reliability flag calculation
- `Outcome` enum: Verified, Clarification, Failed, Timeout, Deterministic
- `EvidenceSummary` with probe counts and evidence kinds
- Centralized score calculation with penalties:
  - Invention detection: hard penalty
  - Ungrounded answers: score reduction
  - Clarification: caps score at 70
- Builder methods: `deterministic_with_evidence()`, `clarification_with_evidence()`, `timeout_with_evidence()`, `failed()`

**Event Log Store (event_log.rs)**
- JSONL-based append-only event log
- `EventRecord` with request metadata:
  - `request_id`, `timestamp`, `query_class`, `outcome`
  - `reliability`, `team`, `escalated`, `escalation_tier`
  - `duration_ms`, `interactions`
  - `recipe_used`, `recipe_learned` (optional)
- `EventLog` with rotation support
- `AggregatedEvents` with XP/Level computation:
  - 11 levels from "Apprentice Troubleshooter" to "Grandmaster of Uptime"
  - XP from requests, success rate, reliability, recipes
- `read_recent()` for time-filtered queries

**Enhanced Stats Display (stats_display.rs)**
- Level and XP progress bar visualization
- Shows XP needed for next level
- Success rate with color coding (green/yellow/red)
- Recipes learned/used counts
- Average response time with min/max
- Escalation stats per team

**Recipe Store (recipe_store.rs)**
- `Recipe` struct with full metadata:
  - `id`, `category`, `title`, `triggers`
  - `required_evidence`, `risk`, `steps`, `citations`
  - `learned_from_ticket`, `learned_reliability`
  - `usage_count`, `last_used`
- `RecipeRisk` enum: ReadOnly, ConfigChange, SystemChange, Destructive
- `RecipeStep` with action templates and rollback
- `RecipeStore` with persistence and trigger indexing
- `should_learn_recipe()` with MIN_LEARN_RELIABILITY = 85
- `find_matches()` for query+evidence matching

**Local-Only Citations (citation.rs)**
- `CitationSource` enum:
  - `ManPage { command, section }`
  - `HelpOutput { command }`
  - `ArchWiki { article }`
  - `Internal { topic }`
- `Citation` with source, excerpt, relevance
- `CitationSet` for teaching mode:
  - `cite_man()`, `cite_help()`, `cite_wiki()`, `cite_internal()`
  - `inline_refs()` for answer text
  - `format_footer()` for sources section
- `CitationStore` with wiki directory and caching

**Idle-Only Benchmark Scheduler (benchmark_scheduler.rs)**
- `BenchmarkScheduler` with atomics for lock-free state
- Idle detection: MIN_IDLE_SECS = 30
- Benchmark timeout: MAX_BENCHMARK_SECS = 60
- Cooldown between runs: BENCHMARK_COOLDOWN_SECS = 300
- `record_request()` resets idle timer and interrupts running benchmarks
- `BenchmarkGuard` with RAII cleanup
- `try_start()` with atomic lock acquisition
- `wait_interruptible()` for async cancellation

**Tests Added**
- `test_event_record_builder`
- `test_aggregated_events_xp_calculation`
- `test_xp_to_level_progression`
- `test_recipe_builder`
- `test_recipe_matches`
- `test_recipe_store_operations`
- `test_should_learn_recipe`
- `test_citation_source_display`
- `test_citation_set_enabled`
- `test_truncate_excerpt`
- `test_scheduler_initial_state`
- `test_interrupt_on_request`
- `test_deterministic_with_evidence_is_grounded`
- `test_clarification_with_evidence_is_grounded`

## [0.0.74] - 2025-12-06

### Added - Version Sanity + Model Upgrade + Answer Shaping

This release adds model selection preferences, answer shaping contracts, and editor configuration recipes.

**Model Selector with Qwen3-VL Preference (model_selector.rs)**
- New `ModelSelector` module for intelligent model selection
- Prefers Qwen3-VL family when available (4B for specialist, 2B for translator)
- Falls back to Qwen2.5 → Llama3.2 → Other when preferred models unavailable
- `ModelFamily` enum: Qwen3VL, Qwen25, Llama32, Other
- `ModelRole` enum: Translator, Specialist
- `ModelCandidate`, `ModelSelection`, `ModelBenchmark` types
- `select_model()` function with catalog-based selection
- `detect_family()` helper for model name classification
- Optional benchmark integration for data-driven selection

**Answer Contract Module (answer_contract.rs)**
- Enforces answer shaping to prevent over-sharing facts
- `Verbosity` enum: Minimal, Normal, Teach
- `RequestedField` enum with 17+ specific field types:
  - CpuCores, CpuModel, CpuTemp, RamTotal, RamFree, RamUsed
  - DiskUsage, DiskFree, SoundCard, GpuInfo, NetworkInterfaces
  - ServiceStatus, ProcessList, PackageCount, ToolExists, Generic
- `AnswerContract::from_query()` parses user intent
- `validate_answer()` checks for missing/extra fields
- `trim_answer()` attempts to extract only requested info
- Teaching mode and minimal verbosity detection

**Editor Configuration Recipes (editor_recipes.rs)**
- Safe, idempotent editor configuration
- `Editor` enum: Vim, Neovim, Nano, Emacs, Helix, Micro, VsCode, Kate, Gedit
- `ConfigFeature` enum: SyntaxHighlighting, LineNumbers, WordWrap, Indentation, AutoIndent, ShowWhitespace
- `EditorRecipe` with typed config lines and rollback hints
- `get_recipe()` returns recipe for editor+feature combination
- `line_exists()` with multiline regex for idempotency
- `apply_recipe()` applies changes without duplication
- `describe_changes()` shows [add]/[skip] status
- `confirmation_prompt()` for user approval
- `backup_path()` for safe backups

**Status Model Selection Display**
- `LlmStatus` now includes:
  - `translator_model`: Selected translator model
  - `specialist_model`: Selected specialist model
  - `preferred_family`: Detected model family

**Micro-Benchmark Support (benchmark.rs)**
- New `benchmark.rs` module in annad for model performance testing
- `run_micro_benchmark()` measures tokens/sec and time-to-first-token
- `benchmark_models()` benchmarks multiple models in sequence
- `parse_benchmark_response()` parses ollama timing response
- Short classification prompt for quick benchmarking

**ConfigureEditor Uses Recipes**
- `build_editor_config_answer()` now uses EditorRecipes module
- Dynamic recipe lookup for supported editors (vim, nvim, nano, emacs)
- Falls back to static answers for GUI editors (VS Code, Kate, Gedit)
- Rollback instructions included in answers

**Tests Added**
- `golden_v074_model_selector_prefers_qwen3_vl`
- `golden_v074_model_selector_fallback`
- `golden_v074_answer_contract_from_query`
- `golden_v074_answer_contract_teaching_mode`
- `golden_v074_editor_recipe_vim_syntax`
- `golden_v074_editor_recipe_idempotent`
- `golden_v074_editor_from_tool_name`
- `golden_v074_version_sanity`

## [0.0.73] - 2025-12-06

### Fixed - Version Truth and Single Source of Reality

This release permanently fixes version inconsistencies with a centralized version module and atomic pair updates.

**Single Source of Truth via version.rs**
- New `crates/anna-shared/src/version.rs` module with:
  - `VERSION`: From `env!("CARGO_PKG_VERSION")` (Cargo.toml workspace)
  - `GIT_SHA`: Build-time injection from `git rev-parse --short HEAD`
  - `BUILD_DATE`: Build-time injection in YYYY-MM-DD format
  - `PROTOCOL_VERSION`: RPC compatibility version (currently 2)
  - `VersionInfo` struct for RPC responses
- New `crates/anna-shared/build.rs` for build-time injection

**RPC GetDaemonInfo Method**
- New `GetDaemonInfo` RPC method returns `DaemonInfo`:
  - `version_info`: VersionInfo with version, git_sha, build_date, protocol_version
  - `pid`: Daemon process ID
  - `uptime_secs`: Time since daemon started
- Client uses this for accurate version comparison

**Enhanced Status Display**
- `annactl status` version section now shows:
  - `annactl X.Y.Z (gitsha)`: Client version with git SHA
  - `annad X.Y.Z (gitsha)`: Daemon version with git SHA
  - `[MISMATCH]` warning if client/daemon versions differ
  - `protocol N`: RPC protocol version
- Health section warns on client/daemon version mismatch

**Atomic Pair Update for annactl + annad**
- Auto-update now updates both binaries together
- Download phase: Both binaries downloaded to temp directory
- Verify phase: Checksums validated, `--version` output checked
- Backup phase: Existing binaries backed up for rollback
- Install phase: Both binaries installed atomically
- Consistency phase: Installed versions verified to match
- Rollback: On any failure, original binaries restored

**Installer Version Verification**
- `scripts/install.sh` now verifies both binaries after install
- Queries `annactl --version` and `annad --version`
- Warns if base versions don't match

**Documentation**
- New `VERSIONING.md` documents the version system
- Explains version flow, verification, and troubleshooting

**Tests Added**
- `test_v073_version_module_integration`: VERSION is valid semver, VersionInfo works
- `test_v073_version_matching`: Same version/different SHA matches

## [0.0.72] - 2025-12-06

### Fixed - Status and Update Reality

This release restructures the status display and fixes the update checker contract.

**Restructured `annactl status`**
- Organized into strict factual sections: daemon, version, update, system, llm, health
- Daemon section: state (RUNNING/STARTING/ERROR), pid, uptime, debug_mode
- Version section: annactl, annad (with mismatch warning), protocol version
- Update section: auto_update, check_pace, last_check_at, next_check_at, available_version, available_checked_at
- System section: cpu, ram, gpu
- LLM section: state, provider, phase, progress, models
- Health section: OK/ERROR/DEGRADED with error details if applicable

**Update Checker Contract (No Lies)**
- Renamed `UpdateStatus` fields for clarity:
  - `last_check_at`: When we last attempted a check (success or failure)
  - `next_check_at`: When we'll next check
  - `latest_version`: The latest version from GitHub (preserved on failure)
  - `latest_checked_at`: When we last successfully fetched latest_version
  - `check_state`: UpdateCheckState enum (NeverChecked, Success, Failed, Checking)
- On GitHub check failure: preserve last known version, mark check_state: FAILED
- Never blank out known data on transient failures

**REPL Greeting Baseline (Movie-IT Style)**
- Shows greeting with last interaction time if >12 hours
- Displays 0-2 notable deltas (boot time, service warnings)
- Shows health status (all services running / N services failed)
- Ends with "What can I do for you today?"
- Saves snapshot for next session comparison

**Version Comparison Edge Cases**
- Empty or invalid versions now return false (never upgrade from invalid)
- Fixed test_v070_version_parsing_edge_cases

**Tests Added**
- `test_v072_update_check_state_serialization`: Validates UpdateCheckState enum
- `test_v072_version_preservation_contract`: Documents version preservation on failure

## [0.0.71] - 2025-12-06

### Fixed - Version Truth (Pure Hygiene Release)

This release makes it impossible for annactl, annad, installer, and auto-update to disagree about the version.

**Single Source of Truth**
- Workspace Cargo.toml `version = "0.0.71"` is the ONLY authoritative version
- All binaries display version via `env!("CARGO_PKG_VERSION")`
- Removed all hardcoded version strings from Rust code, scripts, and docs output examples

**Unified Version Display**
- `annactl --version` prints exactly: `annactl vX.Y.Z`
- `annad --version` prints exactly: `annad vX.Y.Z`
- `annactl status` now shows:
  - `installed`: annactl binary version
  - `daemon_ver`: annad binary version (queried over RPC)
  - `available`: from release checker (shows "unknown" until first check)
  - `last_check`: timestamp of last update check
  - `next_check`: countdown to next check
  - `auto_update`: ENABLED/DISABLED state

**Hard Gate Tests (Fail CI if broken)**
- `hard_gate_annactl_version_equals_workspace`: annactl version must match workspace
- `hard_gate_annad_version_equals_workspace`: annad version must match workspace
- `hard_gate_no_conflicting_crate_versions`: No crate may have hardcoded version != workspace

**Auto-Update Invariants**
- Semantic version comparison (0.0.9 < 0.0.10), not string comparison
- Tests for: installed < latest, installed == latest, installed > latest (no downgrade)

## [0.0.70] - 2025-12-06

### Fixed - Version Unification

This release ensures it is impossible for annactl, annad, installer, uninstaller, and auto-update to disagree about the version.

**A) Single Source of Truth**
- Workspace Cargo.toml `version = "0.0.70"` is the ONLY authoritative version
- All crates use `version.workspace = true` to inherit from workspace
- `anna_shared::VERSION` uses `env!("CARGO_PKG_VERSION")` for compile-time resolution
- Removed hardcoded version from install.sh - now fetches from GitHub releases API
- VERSION file synchronized with workspace version

**B) Version Consistency Tests**
- `version_format_is_semver`: Validates X.Y.Z format
- `version_matches_workspace_cargo_toml`: Reads root Cargo.toml and compares
- `version_file_matches_workspace`: Ensures VERSION file is synchronized
- `all_crates_use_shared_version_constant`: Documents the architecture contract

**C) Status Output Contract**
- Changed "version" to "installed" to clearly show THIS binary's version
- "available" now shows "not checked yet" if no update check has been performed
- Added "last_check" field showing when last update check occurred
- "auto_update" now shows colored ENABLED/DISABLED status

**D) Auto-Update Correctness Gate**
- Added `test_v070_installed_older_than_latest`: Verifies update triggers correctly
- Added `test_v070_installed_equals_latest`: Verifies no update when up-to-date
- Added `test_v070_no_downgrade`: Critical test - never downgrade even if remote is older
- Added `test_v070_semver_not_string`: Ensures semantic comparison (0.0.9 < 0.0.10)
- Added `test_v070_version_parsing_edge_cases`: Handles empty/invalid versions

**E) Install Script Improvements**
- Removed hardcoded VERSION="X.X.X" from install.sh
- Added `fetch_version()` function that queries GitHub releases API
- Installer now always installs the latest published release

### Changed
- display.rs: Shows `installed` and `available` versions with proper status
- display.rs: Uses `anna_shared::VERSION` directly instead of daemon status
- install.sh: Dynamically fetches version from GitHub instead of hardcoding

## [0.0.69] - 2025-12-06

### Added - Unified Versioning + REPL Narrative Enhancements

**Goal 1: Unified Versioning**
- Single source of truth: workspace Cargo.toml `version = "0.0.69"`
- All crates use `version.workspace = true`
- VERSION constant uses `env!("CARGO_PKG_VERSION")` for compile-time resolution
- Added version consistency tests in `tests/version_consistency.rs`
- `annactl status` displays: installed version, available version, check_pace, next_check

**Goal 2: REPL "Since Last Time" Summary**
- REPL greeting now shows changes since last session from snapshot comparison
- Tracks: failed services, disk usage changes, memory changes
- Delta format: `[warn]`, `[crit]`, `[fail]`, `[ok]` prefixes (no emojis)
- Persists snapshots to `~/.anna/snapshots/last.json`
- Shows time since last interaction in greeting

**Goal 3: Editor Config Flow (existing v0.0.68 behavior)**
- Multiple editors → numbered list clarification
- Single editor → deterministic config steps
- No editors → grounded negative evidence

**Goal 4: Progress UI (existing v0.0.67 behavior)**
- ASCII spinner during bootstrap and requests
- Stage updates in debug mode (translator, probes, deterministic, specialist, supervisor)

**Goal 5: Documentation Updates**
- README.md version updated to v0.0.69
- VERSION file updated to 0.0.69
- install.sh version updated to 0.0.69

### Changed
- display.rs: Added snapshot collection and delta display in print_repl_header()
- format_delta_plain() outputs clean text with color prefixes, no emojis

## [0.0.68] - 2025-12-06

### Fixed - Audio Parse Correctness

**Goal 1: Audio Evidence Parsing**
- Audio deterministic answer now correctly reports devices when lspci shows "Multimedia audio controller"
- Verified parsing handles all common patterns: "audio device", "multimedia audio controller", "audio controller", "hd audio"
- Added tests for audio parsing with PCI slot extraction (00:1f.3 format)
- Audio grep exit_code=1 is correctly treated as valid negative evidence (no devices found)

**Goal 2: ConfigureEditor Grounding + Probe Accounting**
- Fixed probe accounting for ConfigureEditor route - now uses all 10 router probes instead of spine-reduced subset
- ConfigureEditor only shows editors that were actually probed AND found (exists=true)
- Clarification prompt ends with period ("Reply with the number."), not question mark
- Execution trace properly includes probe stats and evidence kinds
- Added skip_spine_override for configure_editor to preserve router's complete probe list

### Changed
- rpc_handler.rs: Skip spine probe override for ConfigureEditor when router already provided probes
- ConfigureEditor now logs "v0.0.68: ConfigureEditor using router probes:" for traceability

### Tests Added
- test_v068_audio_multimedia_controller_parsing - verifies "Multimedia audio controller" parsing
- test_v068_audio_deterministic_answer_with_device - confirms no false "No audio" when device exists
- test_v068_audio_grep_exit1_is_valid_negative_evidence - validates exit_code=1 handling
- test_v068_configure_editor_grounded_to_probes - ensures only probed editors appear
- test_v068_configure_editor_clarification_ends_with_period - no question marks in prompts
- test_v068_version_consistency - version normalization check

## [0.0.67] - 2025-12-06

### Added - Service Desk Theatre UX Overhaul

**Part A: Service Desk Narrative Renderer**
- New `render.rs` module for debug OFF "movie-terminal" output
- Header block: hostname, username, version, debug mode
- Narrative greeting with time delta, boot status, critical issues
- Case flow blocks: case ID (CN-YYYYMMDD-XXXX), domain dispatch, evidence summary
- Clarification as numbered list ending with period (no question marks)
- Risk/reliability line with evidence kinds
- Spinner animation for async operations

**Part B: REPL Narrative UI**
- Updated REPL header to use narrative greeting
- Shows critical issues count and boot delta
- Clean help hint: "Type a question, or: status, help, exit"

**Part C: annactl stats RPG System**
- New `stats_store.rs` with JSONL-backed persistence
- XP calculation using logistic curve (0-100 scale)
- Titles from "Apprentice Troubleshooter" to "Grandmaster of Uptime"
- RPG profile display with XP bar visualization
- Aggregated metrics: cases, success rate, avg reliability, escalations

**Part D: Local Citations System**
- New `citations.rs` for grounded guidance references
- KnowledgeCache for man pages and Arch Wiki snapshots
- Citation sources: ManPage, HelpOutput, ArchWiki, LocalFile
- find_citation() returns Cited or Uncited with verification ticket
- Excerpt extraction for relevant topic matches

### Changed
- REPL greeting now uses Service Desk narrative style
- Stats display includes RPG profile when data available
- Modularized display.rs into stats_display.rs and progress_display.rs

### Non-Negotiables Enforced
- No icons or emojis in output
- No raw probe output in debug OFF mode
- No question marks in Anna's final text
- No "would you like" phrasing
- Citations required for factual guidance

## [0.0.66] - 2025-12-06

### Fixed - Version Normalization + Regression Fixes

**Part A: Version Normalization**
- Consolidated version to 0.0.66 across all sources (workspace, install script, README)
- Single source of truth: `[workspace.package] version` in root Cargo.toml
- Added version consistency test (`test_v066_version_consistency`)
- Install script now uses VERSION="0.0.66"

**Part B: Audio Evidence Regression Fix**
- Fixed lspci audio output parsing to handle PCI class codes [XXXX] in device lines
- Audio answer format changed from markdown to clean text: "Detected audio hardware: X (PCI Y)"
- Negative evidence now states source: "No audio devices detected (checked lspci and pactl)."
- Added tests: `test_v066_audio_evidence_from_lspci_probe`, `test_v066_audio_negative_evidence_from_empty_grep`, `test_v066_find_audio_evidence_prefers_lspci`

**Part C: ConfigureEditor Regression Fix**
- Multiple editors now show statement with numbered options, no question mark:
  "I can configure syntax highlighting for one of these editors:\n1) vim\n2) code\nReply with the number."
- Single editor answer starts with "Detected X installed." (no markdown)
- All editor answers remove markdown formatting (no **bold**)
- Added tests: `test_v066_editor_answers_no_markdown`, `test_v066_editor_answers_start_with_detected`

**Part D: Debug OFF/ON Output Gating**
- Verified: debug OFF never shows raw probe commands, stdout/stderr, or stage headers
- Verified: debug OFF responses end with statements, not questions
- Verified: reliability and signals computed from actual execution

### Changed
- Audio output format: "Detected audio hardware: <desc> (PCI <slot>)"
- Editor config output: no markdown, starts with "Detected"
- Clarification: statement with numbered options, ends with period

## [0.0.55] - 2025-12-06

### Fixed - Regression Fixes for v0.0.54

**Bug 1: HardwareAudio returns "No audio devices detected" despite lspci showing one**
- Fixed lspci audio parsing to handle PCI class codes in brackets (e.g., `[0403]`)
- Parser now correctly extracts description after "Multimedia audio controller [0403]:"
- Added `extract_lspci_description_v055()` for robust description extraction
- Audio probes now correctly parse: `00:1f.3 Multimedia audio controller [0403]: Intel Corporation...`

**Bug 2: ConfigureEditor shows wrong editors and broken reliability**
- Added `command_v_hx` probe for helix binary (was missing)
- Changed clarification prompt from question to statement: "Select editor to configure"
- Only editors that were actually probed can appear in options
- Reliability signals now correctly reflect executed probes

### Changed
- Router now probes 10 editors including hx (helix binary name)
- Clarification format: no trailing question marks in normal mode

### Tests Added
- `test_v055_lspci_audio_with_class_code`
- `test_v055_lspci_audio_without_class_code`
- `test_v055_lspci_audio_empty_grep`
- `test_v055_lspci_audio_multiple_devices`
- `test_v055_extract_description_with_class_code`

## [0.0.63] - 2025-12-06

### Added - Service Desk Theatre Renderer (UX Overhaul)

**1. Clean mode narrative flow**
- Normal mode now shows "Checking X..." before answers (e.g., "Checking audio hardware...", "Checking installed editors...")
- Evidence source shown in footer when answer is grounded (e.g., "Verified from lspci+pactl")
- Clarification options displayed with numbered list when multiple choices available
- No raw probe output ever leaks in clean mode

**2. New transcript event types**
- `EvidenceSummary`: Human-readable summary of what probes found
- `DeterministicPath`: Which deterministic route was used
- `ProposedAction`: Privileged actions requiring confirmation
- `ActionConfirmationRequest`: User confirmation prompts

**3. Debug mode enhancements**
- All new event types rendered with appropriate formatting
- Risk levels color-coded for ProposedAction (high=red, medium=yellow, low=green)
- Rollback availability shown for actions

### Changed

- `transcript_render.rs`: Refactored `render_clean` for Service Desk Theatre
- Added `describe_probes_checked()` to categorize probes (audio, editor, memory, disk, etc.)
- Added `format_evidence_source()` for footer evidence source text
- Added `render_clarification_options_clean()` for numbered option display

### Tests Added

- `golden_v063_evidence_summary_event`
- `golden_v063_deterministic_path_event`
- `golden_v063_proposed_action_event`
- `golden_v063_action_confirmation_request_event`
- `golden_v063_probe_description_categories`

## [0.0.62] - 2025-12-06

### Fixed - ConfigureEditor Grounded Evidence + Reliability Accounting

**1. Proper probe accounting for ConfigureEditor**
- **Issue**: ConfigureEditor interception returned `probes: 0` and `grounded=✗` even when tool existence probes ran successfully.
- **Fix** (`rpc_handler.rs`): Now correctly counts valid evidence from ToolExists probes and sets execution_trace with probe stats.
- All ConfigureEditor paths (no-editors, single-editor, multi-editor) now include `execution_trace` with accurate probe stats.

**2. Fixed grounding signal for clarification responses**
- **Issue**: `create_clarification_with_options` didn't set `execution_trace` and used hasProbes instead of valid evidence count.
- **Fix** (`service_desk.rs`): Updated to:
  - Count valid evidence using `is_valid_evidence()`
  - Set `answer_grounded`, `probe_coverage`, `translator_confident` based on valid evidence count
  - Build and attach `execution_trace` with `ProbeStats` and `evidence_kinds`

**3. Simplified clarification question text**
- **Fix**: Changed multi-editor clarification question from "Which editor would you like to configure for syntax highlighting?" to "Which editor would you like to configure?" for cleaner output.

### Tests Added

- `golden_v062_configure_editor_tool_evidence_extraction`
- `golden_v062_tool_exists_is_valid_evidence`
- `golden_v062_configure_editor_valid_evidence_count`

## [0.0.61] - 2025-12-06

### Fixed - HardwareAudio Parser + Deterministic Merge

**1. Content-based audio detection**
- **Issue**: Parser only matched command strings like `lspci | grep -i audio`. If the actual command varied, parsing could fail even when output contained audio device info.
- **Fix** (`parsers/mod.rs`): Added `stdout_contains_audio_device()` function to detect audio device output by content patterns:
  - `Audio device:`
  - `Multimedia audio controller:`
  - `Audio controller:`
  - `Multimedia controller:` (if line contains "audio")
- Now parses audio devices by output content, not just command pattern.

**2. pactl content-based detection**
- **Fix**: Added detection of pactl cards output by `Card #` blocks in stdout.
- Works even when command string doesn't contain "cards".

**3. Parser robustness improvements**
- `try_parse_audio_devices()` now tries content-based detection first for lspci.
- Falls back to command pattern matching for backwards compatibility.
- Added `stdout_contains_audio_device()` and `is_lspci_audio_command()` helpers.

### Tests Added

- `golden_v061_lspci_audio_detected_by_output_content`
- `golden_v061_answer_hardware_audio_prefers_positive_lspci`
- `golden_v061_answer_hardware_audio_prefers_positive_pactl`
- `golden_v061_pactl_detected_by_output_content`
- `golden_v061_no_audio_only_when_both_empty`

## [0.0.60] - 2025-12-06

### Fixed - HardwareAudio False Negatives

**1. Expanded lspci audio parsing**
- **Issue**: Parser only recognized "Audio device:" but missed "Multimedia audio controller:" and "Audio controller:" lines from lspci, causing false "No audio devices detected" responses.
- **Fix** (`parsers/mod.rs`): Updated `parse_lspci_audio_output()` to recognize all variants:
  - `Audio device:`
  - `Multimedia audio controller:`
  - `Audio controller:`
  - `Multimedia controller:` (if line contains "audio")
- Added `is_lspci_audio_command()` helper to centralize probe detection.
- Added `extract_pci_slot()` helper for consistent PCI slot parsing.

**2. Correct grep exit code handling**
- **Issue**: grep exit code 1 (no matches) was treated inconsistently - sometimes as error, sometimes as empty evidence.
- **Fix**: Now correctly treats:
  - exit 0 = matches found (devices present)
  - exit 1 = no matches (valid empty evidence)
  - exit 2+ = grep error

**3. Improved pactl cards parsing**
- **Fix** (`parsers/mod.rs`): Updated `parse_pactl_cards_output()` to handle multiple properties:
  - `alsa.card_name`
  - `device.description`
  - `card.name`
  - `device.product.name`
- Now properly tracks Card # blocks for multiple sound cards.

**4. Audio source merging with deduplication**
- **Fix**: When both lspci and pactl return devices, merge them with deduplication:
  - Prefers lspci devices (have PCI slot)
  - Detects overlapping descriptions (e.g., "Intel Corporation..." and "HDA Intel PCH")
  - Source shows "lspci+pactl" when merged

**5. Improved deterministic answer**
- **Fix** (`deterministic.rs`): Updated `answer_hardware_audio()`:
  - Counts all audio evidence sources for `parsed_data_count`
  - Message indicates which sources were checked: "No audio hardware detected by lspci/pactl."
  - Single device: `**Audio device:** Intel Corporation Cannon Lake PCH cAVS (PCI 00:1f.3)`
  - Multiple devices: Numbered list with PCI slots

### Fixed - ConfigureEditor Grounded Selection

**6. ConfigureEditor now selects editors only from probe evidence**
- **Issue**: ConfigureEditor would suggest "code, vim" even when "code" probe returned exit_code=1 (not installed). The editor list was not grounded in actual probe results.
- **Root Cause**: `reduce_probes()` capped probes to 3 by default, but ConfigureEditor needs 10 probes (all editors).
- **Fix** (`probe_spine.rs`): `reduce_probes()` now allows 10 probes for `configure_editor` route class.
- **Fix** (`rpc_handler.rs`): Added matching probe cap exception for ConfigureEditor in fallback path.

**7. Existing helper already correct**
- `installed_editors_from_parsed()` correctly filters `exists == true` only
- Now all 10 editor probes run, so the selection is truly grounded

### Tests Added

- `golden_v060_lspci_multimedia_audio_controller_parses_positive`
- `golden_v060_pactl_cards_parses_to_audio_devices`
- `golden_v060_audio_dedupe_merges_sources`
- `golden_v060_grep_exit_1_is_valid_empty_evidence`
- `golden_v060_audio_controller_variant_parses`
- `golden_v060_configure_editor_never_invents_code`
- `golden_v060_configure_editor_single_editor_autopicks`
- `golden_v060_probe_spine_allows_10_editor_probes`
- `golden_v060_probe_spine_other_routes_cap_at_3`
- `golden_v060_no_editors_grounded_negative_evidence`

## [0.0.59] - 2025-12-06

### Fixed - ConfigureEditor Evidence-Grounded Flow

**1. ConfigureEditor now grounded in live probe evidence only**
- **Issue**: Response showed "probes: 0" and "grounded: ✗" even when probes ran; editor list included editors not actually probed.
- **Fix** (`rpc_handler.rs`): ConfigureEditor detection uses ONLY `ParsedProbeData::Tool(ToolExists)` from current request's `probe_results`. No inventory cache, no stale data.
- **New** (`parsers/mod.rs`): Added `installed_editors_from_parsed()` helper for consistent editor extraction from probe evidence.
- **New** (`probe_spine.rs`): Added `hx` probe (Helix binary name) to ensure both `helix` and `hx` are detected.

**2. Proper ClarifyRequest structure for multiple editors**
- **Issue**: Multi-editor scenario returned plain text question in answer field instead of structured clarification.
- **Fix** (`service_desk.rs`): Added `create_clarification_with_options()` that builds proper `ClarifyRequest` with structured `ClarifyOption` items.
- Multi-editor response now uses `clarification_request` field (v0.0.47+ format) with numeric menu options.

**3. Reliability signals correctly reflect probe state**
- Clarification responses now have `probe_coverage: true` when probes ran.
- `evidence.probes_executed` contains actual probe results (not empty).
- `grounded: true` when options derived from current probe evidence.

**4. Single-editor answer is deterministic with no questions**
- When exactly one editor detected: returns editor-specific instructions.
- No "Would you like...?" or "Do you want...?" in answer text.
- Questions only appear in proper ClarificationRequired flow.

**5. No-editors-found response lists what was checked**
- When no supported editors found: lists which editors were probed.
- Still grounded (`grounded: true`) because we have valid negative evidence.

### Tests Added

- `golden_v059_editor_probes_include_code`
- `golden_v059_installed_editors_from_tool_evidence`
- `golden_v059_multi_editor_clarification_is_grounded`
- `golden_v059_single_editor_answer_no_questions`
- `golden_v059_no_editors_found_lists_checked`

## [0.0.58] - 2025-12-06

### Fixed - HardwareAudio Parsing and Deterministic Answers

**1. Expanded lspci audio parsing**
- **Issue**: Parser only recognized "Audio device:" but not "Multimedia audio controller:" lines from lspci, causing false "No audio devices detected" responses.
- **Fix** (`parsers/mod.rs`): Updated `parse_lspci_audio_output()` to recognize:
  - `Audio device:`
  - `Multimedia audio controller:`
  - `Multimedia controller:` (if line contains "audio")
- Added `extract_lspci_description()` helper to properly extract device description after device class marker.

**2. Improved PCI slot and vendor extraction**
- PCI slot (e.g., `00:1f.3`) now correctly extracted from first token.
- Description no longer includes device class prefix ("Multimedia audio controller:").
- Vendor extracted from known vendor list (Intel, NVIDIA, AMD, Realtek, etc.).

**3. Valid negative evidence for audio**
- Empty lspci output with exit_code 0 = valid negative evidence (no devices).
- grep exit_code 1 (no match) = valid negative evidence (no devices).
- Both cases parse to `AudioDevices { devices: [], source: "lspci" }` with `is_valid_evidence() = true`.

**4. Improved deterministic audio answer**
- **Fix** (`deterministic.rs`): Updated `answer_hardware_audio()`:
  - Single device: `**Audio device:** Intel Corporation Cannon Lake PCH cAVS (PCI 00:1f.3)`
  - Multiple devices: Numbered list with PCI slots
  - No devices: `No audio devices detected from lspci.`
- Answer now includes PCI slot for hardware identification.

### Tests Added

- `golden_v058_lspci_empty_output_is_valid_negative_evidence`
- `golden_v058_lspci_grep_exit_1_is_valid_negative_evidence`
- `golden_v058_lspci_extracts_pci_slot`
- `golden_v058_lspci_description_no_device_class_prefix`
- `golden_v058_sound_card_query_uses_lspci_audio_probe`

## [0.0.57] - 2025-12-06

### Fixed - ConfigureEditor Flow: No Inventory, No Questions

**1. ConfigureEditor uses ONLY current probe evidence**
- **Issue**: ConfigureEditor could potentially use stale inventory data.
- **Fix** (`rpc_handler.rs`): Removed any inventory dependency. Editor detection now exclusively uses current request's `probe_results` via `get_installed_tools()`.

**2. Expanded supported editors list**
- Added `kate` and `gedit` to supported editors (now 9 total: code, vim, nvim, nano, emacs, micro, helix, kate, gedit).
- Updated `router.rs`, `translator.rs`, and `probe_spine.rs` with new editor probes.

**3. Single-editor answers are editor-specific, no questions**
- **Issue**: Single-editor answer showed generic multi-editor instructions.
- **Fix** (`rpc_handler.rs`): Added `build_editor_config_answer()` that returns editor-specific steps:
  - **vim/vi**: `~/.vimrc` with `syntax on`
  - **nvim**: `~/.config/nvim/init.vim` or `init.lua`
  - **nano**: `~/.nanorc` with `include "/usr/share/nano/*.nanorc"`
  - **emacs**: `~/.emacs` with `(global-font-lock-mode t)`
  - **helix**: Syntax on by default; themes via `~/.config/helix/config.toml`
  - **micro**: Syntax on by default; settings via `~/.config/micro/settings.json`
  - **code**: Language mode based; mentions extensions and Color Theme
  - **kate**: Fonts & Colors in Settings > Configure Kate
  - **gedit**: Preferences > Font & Colors
- No question marks in any answer (no "Would you like...?").

**4. Multi-editor clarification is grounded**
- Clarification question format: `"Which editor would you like to configure? Detected: vim, code"`
- Does not leak raw probe output (no `/usr/bin/vim` paths in message).
- `grounded=true` because options derived from current probe evidence.

### Tests Added

- `golden_v057_single_editor_answer_vim_only`
- `golden_v057_multi_editor_clarification_format`
- `golden_v057_configure_editor_route_includes_all_editors`
- `golden_v057_grounded_clarification_has_probe_coverage`
- `golden_v057_editor_specific_answer_formats`
- `test_vim_answer_no_questions` (in annad)
- `test_nvim_answer_correct_paths` (in annad)
- `test_editor_answers_are_specific` (in annad)
- `test_vi_uses_vim_config` (in annad)

## [0.0.56] - 2025-12-06

### Fixed - UX, Routing, and Grounding Hardening

**1. Clarification responses now attach probes and transcript**
- **Issue**: ClarificationRequired responses (e.g., ConfigureEditor multi-choice) showed `probes: 0` and `grounded=✗` even though probes ran.
- **Fix** (`service_desk.rs`): Added `create_clarification_response_grounded()` helper that attaches probe evidence and sets `grounded=true` when options are derived from current probe evidence.
- **Fix** (`rpc_handler.rs`): ConfigureEditor multi-editor path now uses grounded clarification.

**2. ConfigureEditor routing made explicit and deterministic**
- **Issue**: "enable syntax highlighting" relied on rpc_handler intercept with no probes.
- **Fix** (`router.rs`): ConfigureEditor route now:
  - `can_answer_deterministically: true`
  - `evidence_required: true`
  - `probes: ["command_v_vim", "command_v_nvim", "command_v_nano", "command_v_emacs", "command_v_micro", "command_v_helix", "command_v_code"]`
- **Fix** (`translator.rs`): Added probe ID mappings for all editor command_v variants.

**3. Probe spine expanded for editor config phrases**
- **Issue**: Phrases like "turn on syntax highlighting" and "set vim to show line numbers" didn't trigger editor probes.
- **Fix** (`probe_spine.rs`): Expanded Rule 6 to match:
  - Verbs: `enable`, `turn on`, `activate`, `set up`, `configure`, `show`, `set `
  - Features: `syntax`, `highlight`, `line number`, `word wrap`, `auto indent`, `theme`, `colorscheme`, `color scheme`
  - Named editors: ` vim`, ` nvim`, ` nano`, ` emacs`, ` micro`, ` helix`, ` code`, `vscode`

**4. Audio dual evidence source merging**
- **Issue**: When both lspci and pactl probes ran, only first was used.
- **Fix** (`parsers/mod.rs`): `find_audio_evidence()` now merges sources:
  - If both have devices, prefer lspci (hardware identity)
  - If only one has devices, use that source
  - If both empty, return grounded negative evidence
  - Never say "No audio devices" if either source has devices

**5. Output policy: No follow-up questions**
- **Fix** (`rpc_handler.rs`): Removed "Would you like me to help configure {}?" from single-editor answer.

### Tests Added

- `golden_clarification_attaches_probes_and_transcript`
- `golden_clarification_is_grounded_when_options_come_from_evidence`
- `test_configure_editor_route_is_deterministic_and_requires_evidence`
- `test_configure_editor_route_adds_editor_probes`
- `golden_editor_config_probe_spine_matches_common_phrasings`
- `golden_editor_config_probe_spine_does_not_trigger_on_unrelated_enable`
- `golden_audio_merges_lspci_and_pactl_when_both_present`
- `golden_audio_negative_evidence_is_grounded`
- `golden_audio_uses_non_empty_source`

## [0.0.55] - 2025-12-06

### Fixed - v0.45.8 Regression Fixes

**Bug A: Audio probes not resolving**
- **Root Cause**: Router specified `probes: ["lspci_audio", "pactl_cards"]` but translator
  didn't have mappings for these IDs, causing probes to be silently skipped.

- **Fix** (`translator.rs`):
  - Added `"lspci_audio" => Some("lspci | grep -i audio")` mapping
  - Added `"pactl_cards" => Some("pactl list cards")` mapping
  - Audio probes now execute correctly and produce typed evidence

- **Parser confirmed working**: Both "Audio device:" and "Multimedia audio controller:"
  lspci formats parse correctly to AudioDevices variant.

**Bug B: Editor config using stale InventoryCache**
- **Root Cause**: ConfigureEditor used `load_or_create_inventory().installed_editors()`
  which reads from disk cache instead of current probe evidence.

- **Fix** (`rpc_handler.rs`):
  - Changed to parse `probe_results` into `ParsedProbeData`
  - Extract installed editors from `ToolExists { exists: true }` evidence
  - Only offer editors actually probed and found in current request
  - Results go through `build_result_with_flags` which attaches probe evidence

### Acceptance Criteria

- `"what is my sound card?"` → Lists audio device from lspci (not "No audio devices")
- `"enable syntax highlighting"` → Shows only editors found in probe evidence
- Both fixes ground answers in current request evidence, not stale cache

## [0.0.54] - 2025-12-06

### Fixed - v0.45.8 Audio Evidence + Editor Config Flow

**Bug A: Audio evidence parsing**
- **Root Cause**: `parse_probe_result()` didn't recognize `lspci | grep -i audio` as Audio evidence.
  Sound card queries showed probe output but said "missing evidence: audio".

- **Evidence Type System** (`parsers/mod.rs`):
  - New `AudioDevice` struct: `description`, `pci_slot`, `vendor`
  - New `AudioDevices` struct: `devices: Vec<AudioDevice>`, `source: String`
  - Added `ParsedProbeData::Audio(AudioDevices)` variant
  - `try_parse_audio_devices()` handles `lspci | grep -i audio` and `pactl list cards`
  - Exit code 1 (no audio devices) creates valid empty evidence

- **Evidence Helpers** (`parsers/mod.rs`):
  - `find_audio_evidence(parsed)` - find audio devices from parsed probes
  - `get_installed_tools(parsed)` - get all ToolExists entries

- **Deterministic Answers** (`deterministic.rs`):
  - `answer_hardware_audio()` uses typed `ParsedProbeData::Audio` evidence
  - Formats answer listing audio devices with vendor info

- **Route Updates** (`router.rs`):
  - `HardwareAudio`: Now `can_answer_deterministically: true` (v0.45.8)
  - Sound card queries answered from typed evidence, no specialist needed

**Bug B: Editor config flow**
- **Root Cause**: "enable syntax highlighting" went to specialist LLM which dumped raw probe output.

- **Clarification Flow** (`rpc_handler.rs`):
  - ConfigureEditor now intercepts BEFORE specialist stage
  - Uses `InventoryCache::installed_editors()` to find installed editors
  - If exactly one editor: auto-pick and return deterministic answer
  - If multiple editors: return `ClarificationRequired` with editor choices
  - Never goes to specialist LLM for editor config

### Acceptance Criteria

- `"what is my sound card?"` → **Deterministic answer** from Audio evidence (no "missing evidence")
- `"enable syntax highlighting"` → **ClarificationRequired** or auto-pick (never raw probe output)
- HardwareAudio route is deterministic when Audio evidence exists

### Tests

- **v0.45.8 Golden Tests** (`stabilization_tests.rs`):
  - `golden_v458_lspci_audio_parses_to_audio_devices`
  - `golden_v458_lspci_audio_empty_is_valid_evidence`
  - `golden_v458_find_audio_evidence`
  - `golden_v458_hardware_audio_is_deterministic`
  - `golden_v458_configure_editor_classifies_correctly`

## [0.0.53] - 2025-12-06

### Fixed - v0.45.7 Negative Evidence Binding

- **Root Cause**: Exit code 1 from `command -v` or `pacman -Q` was treated as an ERROR,
  but it's actually VALID NEGATIVE EVIDENCE (tool/package not installed).

- **Evidence Type System** (`parsers/mod.rs`):
  - New `ToolExists` struct: `name`, `exists: bool`, `method`, `path`
  - New `PackageInstalled` struct: `name`, `installed: bool`, `version`
  - New `ToolExistsMethod` enum: `CommandV`, `Which`, `Type`
  - `parse_probe_result()` now handles tool/package probes BEFORE exit code check
  - Exit code 1 creates valid evidence with `exists=false` or `installed=false`

- **Evidence Helpers** (`parsers/mod.rs`):
  - `find_tool_evidence(parsed, name)` - find tool evidence by name
  - `find_package_evidence(parsed, name)` - find package evidence by name
  - `has_evidence_for(parsed, name)` - check if any evidence exists
  - `is_valid_evidence()` - returns true for Tool/Package (even negative)

- **Deterministic Answers** (`deterministic.rs`):
  - `answer_installed_tool_check()` now uses typed evidence parsing
  - Generates answers from `ToolExists` and `PackageInstalled` evidence
  - Example: "**nano** is not found in your PATH" (from negative evidence)

- **Route Updates** (`router.rs`):
  - `InstalledToolCheck`: Now `can_answer_deterministically: true`
  - `CpuCores`: Now `can_answer_deterministically: true`
  - These routes skip specialist LLM when evidence is available

- **Evidence Enforcement** (`rpc_handler.rs`):
  - Updated evidence check to use `is_valid_evidence()` instead of `exit_code == 0`
  - Negative evidence counts as valid evidence for reliability scoring

- **Probe Spine** (`probe_spine.rs`):
  - Rule 6: Editor configuration queries enforce tool probes for common editors
  - "enable syntax highlighting" now probes vim, nvim, nano, emacs, etc.

### Acceptance Criteria

- `"do I have nano?"` with exit 1 → **"nano is not found in your PATH"** (deterministic, high reliability)
- `"how many cores?"` → Deterministic answer from lscpu (no specialist timeout)
- `"enable syntax highlighting"` → Probes for common editors enforced

### Tests

- **Parser Tests** (`parsers/mod.rs`):
  - `test_tool_exists_positive_evidence`: exit 0 = tool exists
  - `test_tool_exists_negative_evidence`: exit 1 = tool NOT exists (valid evidence!)
  - `test_package_installed_positive_evidence`: exit 0 = package installed
  - `test_package_installed_negative_evidence`: exit 1 = package NOT installed (valid evidence!)
  - `test_find_tool_evidence`: Helper function works correctly
  - `test_find_package_evidence`: Helper function works correctly
  - `test_has_evidence_for`: Checks both positive and negative evidence

- **v0.45.7 Golden Tests** (`stabilization_tests.rs`):
  - `golden_v457_tool_not_found_is_valid_evidence`
  - `golden_v457_tool_found_is_valid_evidence`
  - `golden_v457_package_not_installed_is_valid_evidence`
  - `golden_v457_package_installed_is_valid_evidence`
  - `golden_v457_find_tool_evidence`
  - `golden_v457_editor_config_enforces_tool_probes`

## [0.0.52] - 2025-12-06

### Fixed - v0.45.6 Probe Contract: No Silent No-Op

- **Root Cause**: Probes were being "planned" but not executed due to a disconnect between:
  - `probe_spine.rs` generating shell commands like `"sh -lc 'command -v nano'"`
  - `probe_runner.rs` expecting translator probe IDs like `"free"` or `"cpu_info"`
  - Unknown probe specs silently skipped (returned `None`, no log, 0 executed)

- **Fixed Probe Resolution** (`probe_runner.rs`):
  - New `resolve_probe_command()` function handles THREE formats:
    1. Translator probe IDs: `"free"`, `"cpu_info"` → mapped to commands
    2. Direct shell commands: `"lscpu"`, `"free -b"`, `"sh -lc '...'"` → executed as-is
    3. Unknown: returns `None` and logs warning
  - Unknown probes now create failed `ProbeResult` with `exit_code=-2` instead of silent skip
  - Added logging: `"v0.45.6: Running N planned probes"`, execution summary

- **Acceptance Criteria Met**:
  - `"do I have nano?"` → `CommandV` probe executes (`sh -lc 'command -v nano'`)
  - `"how many cores has my cpu?"` → `Lscpu` probe executes (`lscpu`)
  - `"what is my sound card?"` → `LspciAudio` probe executes (`lspci | grep -i audio`)
  - No more `[probes] ok` with 0 probes executed

### Tests

- **Probe Runner Unit Tests** (`probe_runner.rs`):
  - `test_resolve_translator_probe_id`: `"free"` → `"free -h"`
  - `test_resolve_direct_shell_command`: Shell commands executed as-is
  - `test_resolve_unknown_probe`: Unknown probes return `None`
  - `test_resolve_probe_spine_commands`: All probe_spine commands resolvable

- **v0.45.6 Golden Tests** (`stabilization_tests.rs`):
  - `golden_v456_tool_check_enforces_command_v`: Tool check includes CommandV
  - `golden_v456_cpu_cores_enforces_lscpu`: CPU cores includes Lscpu
  - `golden_v456_sound_card_enforces_audio_probes`: Sound card includes LspciAudio
  - `golden_v456_probe_spine_commands_resolvable`: All probes start with known executables
  - `golden_v456_evidence_binding`: Evidence kinds map to probes

## [0.0.51] - 2025-12-05

### Added - v0.45.5 Clarification System

- **ClarificationRequired StageOutcome** (`transcript.rs`):
  - New `StageOutcome::ClarificationRequired { question, choices }` variant
  - `clarification_required()` constructor for building clarification outcomes
  - `is_clarification_required()` and `can_proceed()` helper methods
  - Display format shows question and choice count

- **ClarifyPrereq for Recipes** (`recipe.rs`):
  - `ClarifyPrereq` struct for prerequisite facts before recipe execution
  - Fields: `fact_key`, `question_id`, `evidence_only`, `verify_template`
  - `ClarifyPrereq::editor()` factory for preferred editor prereqs
  - `clarify_prereqs` field added to `Recipe` struct
  - `needs_clarification()` and `get_clarify_prereqs()` methods

- **ConfigureEditor Query Class** (`router.rs`, `query_classify.rs`):
  - New `QueryClass::ConfigureEditor` for editor configuration queries
  - Pattern matching: "enable syntax highlighting", "turn on line numbers"
  - Pattern matching: "how do I enable X in vim/nano/etc"
  - Route returns `evidence_required: true` with `ToolExists` evidence kind
  - `needs_clarification()` method on `QueryClass`
  - `clarification_fact_key()` method returns required fact key

### Changed

- **Transcript Rendering** (`transcript_render.rs`):
  - `format_outcome()` now handles `ClarificationRequired` with yellow CLARIFY label

### Tests

- **v0.45.5 Golden Tests** (`stabilization_tests.rs`):
  - `golden_v455_stage_outcome_clarification_required`: StageOutcome structure
  - `golden_v455_clarify_prereq_editor`: ClarifyPrereq::editor() factory
  - `golden_v455_recipe_needs_clarification`: Recipe with prereqs
  - `golden_v455_recipe_no_clarification_needed`: Recipe without prereqs

## [0.0.50] - 2025-12-05

### Added - v0.45.4 Claim Grounding Integration

- **Full ANCHOR Integration** (`service_desk.rs`):
  - Integrated `extract_claims()` to parse numeric, percent, and status claims from answers
  - Integrated `compute_grounding()` to verify claims against parsed probe evidence
  - `grounding_ratio` and `total_claims` now populated from actual claim verification
  - Uses `ParsedEvidence::from_probes()` to build evidence from probe results

- **GUARD-based Invention Detection**:
  - Replaced `check_no_invention()` heuristic with `check_no_invention_guard()`
  - Uses claim extraction + evidence verification for accurate invention detection
  - Detects contradictions between claims and evidence (e.g., "nginx is running" when nginx is failed)

- **Claim Types Supported**:
  - Numeric: `<subject> uses <size>` (e.g., "firefox uses 4.2GB")
  - Percent: `<mount> is <N>% full` (e.g., "root is 85% full")
  - Status: `<service> is <state>` (e.g., "nginx is running")

### Changed

- **Reliability Scoring**:
  - `answer_grounded` now derived from claim grounding ratio when claims present
  - Falls back to heuristic only when no auditable claims extracted
  - `no_invention` uses GUARD verification with evidence context

## [0.0.49] - 2025-12-05

### Added - v0.45.4 Truth Enforcement

- **No Evidence, No Claims Rule** (`rpc_handler.rs`):
  - Evidence enforcement check: if `evidence_required=true` AND `probe_stats.succeeded==0`, returns deterministic failure
  - `create_no_evidence_response()` in `service_desk.rs` for clear failure message
  - `NO_EVIDENCE_RELIABILITY_CAP` constant (40) in `reliability.rs`

- **Extended Evidence Detection** (`trace.rs`):
  - `EvidenceKind` enum extended with: CpuTemperature, Audio, Network, Processes, Packages, ToolExists, BootTime, System
  - `evidence_kinds_from_probes()` now detects sensors, pactl, ip addr, ps aux, pacman, command -v, systemd-analyze, uname

- **Improved Journal Parser** (`parsers/journalctl.rs`):
  - `parse_journalctl_json()` function for JSON format parsing
  - Proper SYSLOG_IDENTIFIER attribution (priority: SYSLOG_IDENTIFIER > _SYSTEMD_UNIT > _COMM > "unattributed")
  - Auto-detect JSON vs text format in `parse_journalctl_priority()`
  - Probe commands updated to use `-o json` format

- **Enhanced Probe Commands** (`probe_spine.rs`):
  - `CommandV` probe now uses `sh -lc 'command -v <name>'` for login shell PATH
  - HardwareAudio route includes both LspciAudio and PactlCards probes

- **Query Classification** (`query_classify.rs`):
  - Generic tool check pattern: any "do I have <word>" triggers InstalledToolCheck
  - Added "have I got" and "do you have" patterns

### Changed

- **Step Numbering** (`rpc_handler.rs`):
  - New Step 5 for evidence enforcement, subsequent steps renumbered

### Tests

- **v0.45.4 Golden Tests** (`stabilization_tests.rs`):
  - `golden_v454_no_evidence_cap_value`: NO_EVIDENCE_RELIABILITY_CAP == 40
  - `golden_v454_evidence_missing_when_no_probes_succeed`: reliability penalized
  - `golden_v454_query_classify_tool_check`: "do I have nano" enforces CommandV
  - `golden_v454_query_classify_audio`: "sound card" enforces LspciAudio + PactlCards
  - `golden_v454_query_classify_cores`: "how many cores" enforces Lscpu
  - `golden_v454_query_classify_system_triage`: "how is my computer doing" enforces journal probes

- **Journal JSON Tests** (`parsers/journalctl.rs`):
  - JSON parsing with SYSLOG_IDENTIFIER
  - Fallback to _SYSTEMD_UNIT
  - Unattributed entries
  - Auto-detect JSON format

## [0.0.48] - 2025-12-05

### Added - v0.45.3 Smart Clarifications & Minimal Probes

- **Minimal Probe Policy** (`probe_spine.rs`):
  - `reduce_probes()` function limits probes to max 3 (default) or 4 (system health)
  - `Urgency` enum: Normal (max 3), Quick (max 2), Detailed (max 5)
  - `query_wants_warnings()` / `query_wants_errors()` detect explicit queries
  - Never runs both JournalErrors AND JournalWarnings unless Detailed urgency

- **Enhanced Clarification Request** (`clarify_v2.rs`):
  - `ClarifyRequest.ttl_seconds` field (default 300 = 5 minutes)
  - `ClarifyRequest.allow_custom` field to control free-text input
  - `with_ttl()` and `no_custom()` builder methods
  - Menu hides "Something else" option when allow_custom=false

- **REPL Clarification State** (`commands.rs`):
  - `PendingClarification` struct tracks pending clarification requests
  - REPL prompt changes to `[choice]>` when clarification pending
  - TTL enforcement: clarification expires after ttl_seconds
  - Parses user input as choice number, custom text, or cancel

- **ServiceDeskResult Extension** (`rpc.rs`):
  - `clarification_request: Option<ClarifyRequest>` field for rich clarifications
  - All ServiceDeskResult constructors updated with new field

### Changed

- **RPC Handler** (`rpc_handler.rs`):
  - `reduce_probes()` called after spine enforcement
  - Probe cap applied even without spine enforcement (max 3 or 4)

- **Golden Tests**:
  - `probe_spine_tests.rs`: Added minimal probe policy tests
  - `clarify_v2.rs`: Added v0.45.3 TTL and allow_custom tests

## [0.0.47] - 2025-12-05

### Fixed - v0.45.2 Stabilization
- **Probe Spine Enforcement** (`probe_spine.rs`):
  - NEW `enforce_minimum_probes()` function uses USER TEXT keyword matching
  - Last line of defense: "do I have nano?" now forces pacman_q+command_v probes
  - "sound card" queries force lspci_audio probe
  - "temperature" queries force sensors probe
  - "cores" queries force lscpu probe
  - "how is my computer" queries force journal+failed_units probes

- **UX Truthfulness** (`trace.rs`, `rpc_handler.rs`):
  - NEW `evidence_kinds_from_probes()` derives evidence from ACTUAL probe results
  - No longer claims evidence kinds from route class alone
  - ExecutionTrace.evidence_kinds now truthfully reflects gathered data

- **Golden Regression Tests** (`probe_spine_tests.rs`):
  - Tests for all 6 failure scenarios that triggered v0.45.2 work
  - "Do I have nano?" test ensures pacman_q probe enforced
  - "What is my sound card?" test ensures lspci_audio enforced
  - "CPU temperature?" test ensures sensors enforced
  - "How many cores?" test ensures lscpu enforced
  - "How is my computer doing?" test ensures journal probes enforced

### Changed
- Moved probe_spine tests to separate test file (under 400 line limit)

## [0.0.46] - 2025-12-05

### Added
- **Probe Spine** (`probe_spine.rs`):
  - `EvidenceKind` enum for categorizing system evidence types
  - `ProbeId` enum for probe identifiers with command mappings
  - `RouteCapability` struct with evidence requirements and spine probes
  - `enforce_spine_probes()` ensures minimum probes when evidence required
  - `probes_for_evidence()` maps evidence kinds to probe IDs
  - `probe_to_command()` maps probe IDs to shell commands

- **UX Regression Tests** (`ux_regression_tests.rs`):
  - Actor label format tests ([you]/[anna])
  - Probe spine enforcement tests
  - Recipe persistence gate tests
  - Timeout response tests
  - Deterministic answer gating tests

### Changed
- **Router** (`router.rs`):
  - Uses `RouteCapability` for deterministic answer gating
  - `can_answer_deterministically()` now a method, not field
  - Many query classes correctly marked non-deterministic (CpuTemp, HardwareAudio, etc.)
  - SystemHealthSummary requires LLM interpretation

- **Timeout Responses** (`service_desk.rs`):
  - `create_timeout_response()` now provides evidence summary
  - Never asks to rephrase - always provides factual status
  - Higher reliability score when partial evidence available

- **Reliability Scoring** (`service_desk.rs`):
  - `evidence_required` passed from route capability
  - `FallbackContext.evidence_required` field added

- **Clarifications** (`clarify_v2.rs`):
  - `editor_request()` uses friendly labels (Vim, Neovim, etc.)
  - Reason updated to "Options shown are installed on your system"

- **Recipe Persistence** (`recipe.rs`):
  - `RECIPE_PERSIST_THRESHOLD` constant (80)
  - `should_persist_recipe()` documented as ONLY gate

- **Transcript Labels** (`transcript_render.rs`):
  - Clean mode uses [you]/[anna] bracketed format
  - Consistent with debug mode labels

### Fixed
- Deterministic answers no longer generated for queries requiring LLM interpretation
- Timeout responses provide useful information instead of empty answers
- Evidence requirement properly flows through reliability scoring pipeline

## [0.0.45] - 2025-12-05

### Added
- Probe planning enhancements
- Query classification improvements

## [0.0.44] - 2025-12-05

### Added
- **Clarification Engine v2** (`clarify_v2.rs`):
  - `ClarifyRequest` with id, question, options, allow_cancel, reason
  - `ClarifyOption` with numeric key (1-8), label, value, verify expectation
  - `ClarifyResponse` with parse() for numeric and free text input
  - `ClarifyResult` enum: Verified, AutoSelected, NeedsVerification, VerificationFailed, Cancelled
  - Auto-select when only one option installed
  - `VerifyFailureTracker` for re-clarification after 2+ failures

- **Verification Engine** (`verify.rs`):
  - `VerifyExpectation` enum: CommandExists, ExitCode, FileExists, FileContainsLine, PackageInstalled, ServiceState, OutputContains
  - `VerificationStep` with mandatory/optional flag
  - `PreActionVerify` batch for pre-change checks
  - `PostActionVerify` batch for outcome confirmation
  - Helper constructors: `editor_installed()`, `file_has_line()`, `service_is()`

- **Facts Lifecycle Enhancements**:
  - `invalidate_on_uninstall()` marks facts stale when tools are removed
  - `should_skip()` checks if clarification can be bypassed (fresh fact or single option)
  - `store_fact()` stores verified clarification with UserConfirmed source

### Changed
- Refactored clarify.rs to re-export v2 types from clarify_v2.rs
- Installed-only menus: options only show actually installed tools
- Verification before action: checks tool exists before proceeding
- Facts automatically archived when referenced tool is uninstalled

## [0.0.43] - 2025-12-05

### Added
- **Spinner Animation v0.0.43** (`ui.rs`):
  - `Spinner` struct for animated stage progress display
  - `tick()` advances frame, `render()` shows current state with elapsed time
  - `success()`, `error()`, `skip()` for completion states with timing
  - `is_running()` and `stop()` for control flow
  - ANSI spinner characters: ⠋ ⠙ ⠹ ⠸

- **StageProgress Tracker v0.0.43** (`ui.rs`):
  - `StageProgress` for pipeline visualization (translator → probes → specialist)
  - `StageStatus` enum: Pending, Running, Complete, Skipped, Error
  - `start()`, `complete()`, `skip()`, `error()` for stage transitions
  - `render_line()` shows ○ ◉ ● - indicators with team colors
  - `summary()` returns "N/M stages (Xms)" format

### Changed
- Compacted struct definitions in ui.rs to stay under 400 lines

## [0.0.42] - 2025-12-05

### Added
- **Clarification Engine v0.0.42** (`clarify.rs`):
  - Menu-based prompts with numeric keys (1-8 for options, 0=cancel, 9=other)
  - `ClarifyPrompt` struct with title, question, options, default_key, reason
  - `MenuOption` with fact_key, fact_value, verify_cmd for verification
  - `ClarifyOutcome` enum: Answered, Cancelled, Other, VerificationFailed
  - `editor_menu_prompt()` generates menus from installed editors
  - `find_installed_alternative()` for fallback suggestions
  - Escape options (cancel/other) always present in menus

- **Named IT Department Roster v0.0.42** (`roster.rs`, `teams.rs`):
  - Pinned staff names per team for deterministic display
  - Network: Michael/Ana, Desktop: Sofia/Erik, Hardware: Nora/Jon
  - Storage: Lars/Ines, Performance: Kari/Mateo, Security: Priya/Oskar
  - Services: Hugo/Mina, Logs: Daniel/Lea, General: Tomas/Sara
  - New `Team::Logs` variant for log analysis routing
  - Team-aware transcript events use named staff

- **IT-Style Health Output v0.0.42** (`health_delta.rs`):
  - `format_it_style()` shows only warnings/issues (minimal noise)
  - `one_liner()` for quick status display
  - `issue_count()` and `warning_count()` methods
  - "All systems operational" when healthy, detailed list when issues exist
  - Thresholds: memory >=80% = high, disk >=80% = high, disk >=95% = critical

### Changed
- Updated roster.rs golden tests for new pinned names
- Added Logs team prompts to review_prompts module
- Updated narrator.rs for Logs team display names

## [0.0.41] - 2025-12-05

### Added
- **Facts Ledger v1** (`facts.rs`, `facts_types.rs`):
  - `FactSource` enum: ObservedProbe, UserConfirmed, Derived, Legacy
  - `FactValue` enum: String, Number, Bool, List for typed fact storage
  - Pinned TTL rules in `ttl` module: packages 7d, editor 90d, boot 30d, network 1d
  - New fact keys: WallpaperFolder, BootTimeBaseline, InstalledPackage, Desktop, GpuPresent, Hostname, Kernel
  - `get_fresh()` returns None if stale (enforces TTL at query time)
  - `upsert_verified()` with typed source and confidence
  - Fact `confidence` score (0-100)
  - Extracted types to `facts_types.rs` for modularity

- **System Inventory Snapshot** (`inventory.rs`):
  - `SystemInfo` struct: hostname, user, arch, kernel, package_count, desktops, gpu_present, gpu_vendor
  - `INVENTORY_TTL_SECS` = 600 (10 minutes for faster updates)
  - `detect_desktops()` from XDG_CURRENT_DESKTOP and DE packages
  - `detect_gpu()` using lspci for GPU vendor detection
  - `refresh_system_info()`, `full_refresh()`, `get_system_info()` methods
  - `is_inventory_fresh()` for cache validity checks

- **RAG-lite Recipe Index** (`recipe_index.rs`):
  - In-memory token index using BTreeMap for determinism
  - `tokenize()` function: lowercase, alphanumeric, 2+ char tokens
  - `RecipeIndex::search()` with score-based ranking
  - `exact_match()` for zero-LLM queries (tokens fully match recipe)
  - Scoring: TARGET_BOOST (3x), INTENT_TAG_BOOST (2x), BASE_MATCH (1x)
  - Deterministic tie-breaker by recipe_id

- **Recipe Retrieval Keys** (`recipe.rs`):
  - `intent_tags: Vec<String>` for matching
  - `targets: Vec<String>` for boosted matching
  - `preconditions: Vec<String>` for required facts
  - Builder methods: `with_intent_tags()`, `with_targets()`, `with_preconditions()`

- **Health Deltas** (`health_delta.rs`):
  - `HealthDelta` struct: changed_fields, prev_values, new_values, summary
  - `SnapshotHistory` stores last 5 snapshots in memory
  - `latest_delta()` compares current to previous snapshot
  - `HealthSummary` for "how is my computer" queries (<1s response)
  - `format_brief()` for compact status display
  - `is_healthy()` and `status_emoji()` helpers

- **LLM Budget Control** (`budget.rs`):
  - Token constants: `LLM_MAX_DRAFT_TOKENS` (800), `LLM_MAX_SPECIALIST_TOKENS` (1200), `LLM_MAX_CONTEXT_TOKENS` (6000)
  - Timeout constants: `TRANSLATOR_TIMEOUT_SECS` (30), `SPECIALIST_TIMEOUT_SECS` (45)
  - `LlmBudget` struct with fast_path/standard/extended presets
  - `LlmFallback` enum: Continue, TranslatorTimeout, SpecialistTimeout
  - `check_llm_fallback()` for timeout-based fallback decisions
  - `fallback_message()` for user-facing timeout explanations

- **Timeout Fallback Events** (`transcript.rs`):
  - `LlmTimeoutFallback` event: stage, timeout_secs, elapsed_secs, fallback_action
  - `GracefulDegradation` event: reason, original_type, fallback_type
  - Helper methods: `llm_timeout_fallback()`, `graceful_degradation()`

### Changed
- PreferredEditor TTL reduced from >100 days to 90 days (per spec)
- InventoryItem TTL changed from 3600s to INVENTORY_TTL_SECS (600s)

## [0.0.40] - 2025-12-05

### Added
- **Relevant Health View** (`health_view.rs`):
  - `RelevantHealthSummary` produces minimal, actionable health output
  - `HealthItem` with severity (Critical, Warning, Note) and category (Disk, Memory, Services)
  - `HealthChange` for tracking changes since last snapshot
  - `build_health_summary()` produces actionable-only output
  - When healthy: "No critical issues detected. No warnings detected." (short!)
  - Only shows disk/memory/service issues when thresholds exceeded

- **Clarity Counters** (`stats.rs`):
  - `clarifications_asked` - total clarification questions asked
  - `clarifications_verified` - answers that passed verification
  - `clarifications_failed` - answers that failed verification
  - `facts_learned` - facts stored after verification
  - `clarifications_cancelled` - user cancelled clarifications
  - `clarification_verify_rate()` helper method

- **Packet Size Limit** (`ticket_packet.rs`):
  - `MAX_PACKET_BYTES` constant (8KB)
  - `estimated_size()` and `exceeds_limit()` methods
  - `truncate_to_limit()` automatically truncates probe outputs
  - Builder enforces 8KB limit on build

- **Timeout Fallback for Health Queries** (`fast_path_handler.rs`):
  - `is_health_query()` checks if query is health-related
  - `force_fast_path_fallback()` returns health status even with stale snapshot
  - Health queries never produce "rephrase" on timeout - always fall back to cached data

### Changed
- `answer_system_health()` now uses `RelevantHealthSummary` for minimal output
- Extracted `PersonStats` and `PersonStatsTracker` to `person_stats.rs` (keeps stats.rs under 400 lines)
- `TicketPacketBuilder::build()` now enforces MAX_PACKET_BYTES limit

### Fixed
- Health queries no longer show verbose system info when healthy
- Timeout responses for health queries now use fast path fallback instead of error message

## [0.0.39] - 2025-12-05

### Added
- **Fast Path Engine** (`fastpath.rs`):
  - Answers health/status queries without LLM calls
  - `FastPathClass` enum: SystemHealth, DiskUsage, MemoryUsage, FailedServices, WhatChanged
  - `FastPathPolicy` struct for configuration (snapshot_max_age, min_reliability)
  - `classify_fast_path()` for deterministic query classification
  - `try_fast_path()` produces answers from cached snapshot data
  - Strips common greetings for better classification

- **Inventory Cache** (`inventory.rs`):
  - `InventoryCache` caches installed tools to avoid repeated checks
  - `VIP_TOOLS` list: common editors, package managers, network tools
  - `check_tool_installed()` uses `command -v` for detection
  - `filter_installed_options()` for clarification filtering
  - Integration with `clarify.rs` for installed-only editor options

- **Knowledge Pack** (`knowledge/pack.rs`):
  - Built-in Arch Linux knowledge for common questions
  - 20 entries covering: package management, services, disk, network, troubleshooting
  - `search_builtin_pack()` keyword-based retrieval
  - `try_builtin_answer()` for high-confidence matches
  - `KnowledgeSource::BuiltIn` for static knowledge that never expires

- **Performance Statistics** (`stats.rs`):
  - `fast_path_hits` counter for LLM-free answers
  - `snapshot_cache_hits` / `snapshot_cache_misses` for cache effectiveness
  - `knowledge_pack_hits` and `recipe_hits` for RAG tracking
  - `translator_timeouts` and `specialist_timeouts` for timeout monitoring
  - Helper methods: `fast_path_percentage()`, `snapshot_cache_hit_rate()`, `timeout_rate()`

- **Fast Path Handler** (`fast_path_handler.rs`):
  - Extracted from rpc_handler.rs for modularity
  - `try_fast_path_answer()` checks cache and returns if handled
  - `build_fast_path_result()` constructs ServiceDeskResult

- **Transcript FastPath Event**:
  - `TranscriptEventKind::FastPath` for debug mode visibility
  - Shows class, cache status, and probes_needed flag

### Changed
- `generate_editor_options_sync()` now uses InventoryCache instead of running commands
- Added `generate_editor_options_with_cache()` for testability
- Fixed `parse_failed_services_into_snapshot()` to handle bullet point (●) prefix
- Moved specialist handling to `specialist_handler.rs` (keeps rpc_handler.rs under 400 lines)

### Fixed
- Snapshot parsing now correctly extracts service names from systemctl --failed output

## [0.0.36] - 2025-12-05

### Added
- **SystemSnapshot (Preventive Anna)** (`snapshot.rs`):
  - `SystemSnapshot` struct captures minimal deterministic system state
  - Tracks: disk usage per mount, failed services, memory (total/used)
  - `capture_snapshot()` parses df, free, and systemctl --failed output
  - `diff_snapshots()` detects meaningful changes with anti-spam thresholds
  - `DeltaItem` enum: DiskWarning, DiskCritical, NewFailedService, MemoryHigh, etc.
  - Thresholds: DISK_WARN=85%, DISK_CRITICAL=95%, MEMORY_HIGH=85%
  - Persistence: `save_snapshot()`, `load_last_snapshot()`, `clear_snapshots()`
  - `is_fresh()` checks if snapshot is within `snapshot_max_age_secs` (default 300s)

- **PendingClarification** (`pending.rs`):
  - `PendingClarification` struct for REPL session continuity
  - Persists pending questions to `~/.anna/pending.json`
  - `ParseResult` enum: Selected, Custom, Cancelled, Invalid
  - `VerifyResult` enum for answer verification (vim vs vi fallback)
  - `format_prompt()` generates numbered option list
  - `parse_input()` handles number, name, or custom input
  - Stale detection: clarifications expire after 1 hour

- **PacketPolicy per Team** (`ticket_packet.rs`):
  - `PacketPolicy` struct: max_summary_lines, allowed_facts, required_probes, max_probes
  - `for_team()` returns team-specific policy (Desktop, Storage, Network, etc.)
  - `truncate_summary()` for deterministic truncation with "(n more lines omitted)"
  - `is_fact_allowed()` validates fact access per team
  - Desktop: max 10 lines, PreferredEditor fact allowed
  - Storage: disk_usage + block_devices required
  - Performance: max 5 probes, memory_info + cpu_info + top_cpu required

- **ProbeBudget** (`budget.rs`):
  - New `ProbeBudget` struct for controlling probe resource usage
  - Methods: `fast_path()`, `standard()`, `extended()` presets
  - `max_probes`, `max_output_bytes`, `per_probe_cap_bytes` limits
  - `would_exceed()` and `cap_output()` for budget enforcement
  - `ProbeBudgetCheck` enum for budget validation results

- **Clarification Cancel option** (`clarify.rs`):
  - `CLARIFY_CANCEL_KEY` and `CLARIFY_OTHER_KEY` constants
  - `is_cancel_selection()` and `is_other_selection()` helpers
  - Editor options now always include Cancel and Other options
  - Cancel allows user to skip clarification without answering

- **Enhanced latency tracking** (`state.rs`, `status.rs`):
  - Added `p50_ms()` and `p90_ms()` percentile methods to LatencyStats
  - Added `min_ms()` and `max_ms()` methods
  - Updated `LatencyStatus` struct with p50, p90 fields for all stages
  - Helper `percentile_ms()` method for flexible percentile calculation

- **TicketPacket** (`ticket_packet.rs`):
  - `TicketPacket` struct for domain-relevant evidence collection
  - `PacketBudget` tracks probe execution stats
  - `TicketPacketBuilder` with fluent API for packet construction
  - `recommended_probes_for_domain()` returns domain-specific probes
  - `evidence_kinds_for_domain()` returns required evidence kinds
  - Methods: `find_probe()`, `successful_probes()`, `probe_success_rate()`

### Changed
- Latency status now reports p50, p90, p95 percentiles (was only p95)
- Editor clarification always shows installed editors + Other + Cancel
- `annactl reset` now clears snapshots and pending clarifications
- Config: Added `snapshot_max_age_secs` (default 300s = 5 minutes)

## [0.0.35] - 2025-12-05

### Added
- **SystemTriage patterns extended** (v0.0.35):
  - "health", "status" now route to SystemTriage (fast path), not full report
  - Added `boot_time` probe (systemd-analyze) to triage probes
  - Patterns: "how is my computer", "any errors", "any problems", "health", "status"

- **Journalctl parser module** (`parsers/journalctl.rs`):
  - `JournalSummary` with `count_total` and `top: Vec<JournalTopItem>`
  - Deterministic grouping by unit name (case-insensitive)
  - Stable ordering: count desc, then key asc
  - `BootTimeInfo` with millisecond precision (`total_ms`, `kernel_ms`, `userspace_ms`)
  - `parse_boot_time()` extracts timing from systemd-analyze output

- **ParsedProbeData variants** (parsers/mod.rs):
  - `JournalErrors(JournalSummary)` for journalctl -p 3
  - `JournalWarnings(JournalSummary)` for journalctl -p 4
  - `BootTime(BootTimeInfo)` for systemd-analyze

- **Editor clarification** (clarify.rs):
  - `KNOWN_EDITORS` constant with vim, vi, nvim, nano, emacs, code, micro, hx
  - `generate_editor_options_sync()` probes `which <editor>` for installed editors
  - `verify_editor_installed()` checks if user's choice is available
  - `generate_editor_clarification()` returns question + detected options

- **RAG-lite keyword search** (recipe.rs):
  - `RecipeMatch` struct with score and matched keywords
  - `search_recipes_by_keywords()` returns top N matches deterministically
  - Scoring: keyword matches + route_class/domain bonuses + reliability/maturity
  - `find_config_edit_recipes()` for use before junior escalation

### Changed
- **SystemHealthSummary narrowed**: Only triggers on explicit "summary", "report", "overview"
- **Probe mappings** (translator.rs): Added journal_errors, journal_warnings, failed_units, boot_time
- **Triage answer format**: Shows boot time, evidence sources, top 3 error/warning keys

## [0.0.34] - 2025-12-05

### Added
- **FAST PATH routing (SystemTriage)**: Zero-timeout path for "how is my computer?" queries
  - New `QueryClass::SystemTriage` routes error-focused queries before SystemHealthSummary
  - Probes: `journal_errors`, `journal_warnings`, `failed_units` only (no disk/memory unless needed)
  - Matches: "any errors", "any problems", "is everything ok", "how is my computer"

- **Journalctl parser** (`parsers.rs`):
  - `parse_journalctl()`: Parses error/warning output with unit grouping
  - `parse_failed_units()`: Extracts failed systemd units
  - `JournalSummary` and `FailedUnit` structs for deterministic processing

- **Deterministic triage answer generator** (`triage_answer.rs`):
  - `generate_triage_answer()`: Produces deterministic answers from journal/systemctl evidence
  - Rules: "No critical issues" + warnings, or list errors/failed units
  - Always includes evidence summary for auditability

- **CLARIFY loop enhancements** (`clarify.rs`):
  - `ClarifyOption` with evidence strings (e.g., "installed: true")
  - `ClarifyAnswer` for structured user responses
  - Verification probes for all clarification types

- **Evidence kinds** (`trace.rs`):
  - New `EvidenceKind::Journal` and `EvidenceKind::FailedUnits`
  - `evidence_kinds_from_route("system_triage")` returns Journal + FailedUnits

- **REPL greeting UX** (`display.rs`):
  - Shows only relevant deltas on startup: failed units, journal errors, boot delta
  - No full report unless user asks `annactl report`

### Changed
- **RESCUE hardening (Phase D)**:
  - Global timeout responses no longer say "please rephrase"
  - Provides deterministic status answer with actionable suggestions
  - New `ExecutionTrace::global_timeout()` for tracing
  - All timeout responses set `needs_clarification: false`

## [0.0.33] - 2025-12-05

### Added
- **Knowledge Store (RAG-first)**: Local retrieval system for fast, deterministic answers
  - `knowledge/sources.rs`: KnowledgeDoc, KnowledgeSource enum (Recipe, SystemFact, PackageFact, ArchWiki, AUR, Journal, Usage)
  - `knowledge/index.rs`: BM25-lite keyword index for sub-50ms retrieval
  - `knowledge/retrieval.rs`: RetrievalQuery, RetrievalHit with source filtering
  - `knowledge/store.rs`: KnowledgeStore with JSONL persistence at ~/.anna/knowledge/
  - `knowledge/conversion.rs`: Recipe-to-KnowledgeDoc conversion for learning

- **System Collectors**: On-demand knowledge gathering from system state
  - `collectors.rs`: collect_boot_time() from systemd-analyze
  - `collectors.rs`: collect_packages() from pacman -Q or dpkg
  - `collectors.rs`: collect_journal_errors() from journalctl -p 3 -b
  - Full provenance tracking for auditability

- **RAG-first Query Classes**: Direct answers from knowledge store, skip LLM
  - `QueryClass::BootTimeStatus`: "boot time", "how long to boot"
  - `QueryClass::InstalledPackagesOverview`: "how many packages", "what's installed"
  - `QueryClass::AppAlternatives`: "alternative to vim", "instead of firefox"
  - `rag_answerer.rs`: try_rag_answer() routes queries through knowledge store

### Changed
- Knowledge answers use collectors on-demand if store is empty (collect-then-answer pattern)
- App alternatives suggest importing Arch Wiki/AUR data when knowledge is missing

### Fixed
- BriefSeverity now implements Default (required for HealthBrief)
- Integer overflow in health_brief_builder for terabyte sizes

## [0.0.32] - 2025-12-05

### Added
- **Humanized IT Department Roster**: Stable person profiles for service desk narration
  - `roster.rs` module with PersonProfile struct (person_id, display_name, role_title, team, tier)
  - Deterministic `person_for(team, tier)` mapping - same inputs always return same person
  - 16 named specialists: Alex, Morgan, Jordan, Taylor, Riley, Casey, Drew, Quinn, etc.

- **Fact Lifecycle Management**: Facts with TTL, staleness, and automatic expiration
  - `StalenessPolicy` enum: Never, TTLSeconds(u64), SessionOnly
  - `FactLifecycle` enum: Active, Stale, Archived
  - `apply_lifecycle()` transitions facts based on current time

- **Health Brief (NEW)**: Relevant-only health status for "how is my computer" queries
  - `health_brief.rs` module with BriefSeverity (Ok, Warning, Error) and BriefItem
  - Only shows actionable items: disk warnings (>85%), memory pressure (>90%), failed services
  - `HealthBrief.format_answer()` returns "Your system is healthy" when nothing needs attention
  - Replaces full system reports for health queries

- **Clarify Module (NEW)**: Clarification questions with verification probes
  - `clarify.rs` module with ClarifyKind enum (PreferredEditor, ServiceName, MountPoint, etc.)
  - `ClarifyQuestion` struct with verification probe template
  - `generate_question()` creates questions with defaults from facts
  - `needs_clarification()` checks if clarification is needed based on query

- **Per-Person Statistics**: Track individual specialist performance
  - `PersonStats` struct with tickets_closed, escalations_sent/received, avg_loops, avg_score
  - `PersonStatsTracker` tracks all 16 roster entries

### Changed
- **Fast Translator Model**: Use smaller, faster model to eliminate timeouts
  - Changed default translator model from qwen2.5:1.5b-instruct to qwen2.5:0.5b-instruct
  - Changed default supervisor model from qwen2.5:1.5b-instruct to qwen2.5:0.5b-instruct
  - Reduced translator timeout from 4s to 2s
  - Reduced specialist timeout from 8s to 6s

- **Faster Budget Defaults**: Bias toward deterministic answers
  - Translator budget: 5s → 1.5s
  - Probes budget: 12s → 8s
  - Specialist budget: 15s → 6s
  - Supervisor budget: 8s → 4s
  - Total budget: 25s → 18s

- **Health Query Routing**: SystemHealthSummary now uses HealthBrief
  - Routes to health_brief_builder instead of full system summary
  - Uses disk_usage, memory_info, failed_services, top_cpu probes
  - Returns "healthy" status when no issues detected

- **Always-Answer Behavior**: Removed "Could you rephrase" failure mode
  - `create_no_data_response()` now builds best-effort answer from available probe data
  - Never asks for rephrase - always provides actionable information
  - Timeout responses no longer ask user to "try again"

### Technical
- Tests updated for new default values
- Golden tests for deterministic health brief output
- All files under 400 lines

## [0.0.31] - 2025-12-05

### Added
- **Facts Store (Phase 1)**: Persistent store for verified user/system facts
  - `facts.rs` module with FactKey enum for typed fact identification
  - `Fact` struct with key, value, verified flag, source, and timestamp
  - `FactsStore` with save/load, deterministic JSON serialization
  - Facts persisted to `~/.anna/facts.json` only when verified
  - `FactStatus` enum: Known, Unknown, Stale for fact querying

- **Intake with Verification Plans (Phase 2)**: Clarification questions with verification
  - `intake.rs` module for query analysis and clarification planning
  - `VerifyPlan` enum: BinaryExists, UnitExists, MountExists, InterfaceExists, etc.
  - `ClarificationQuestion` with question ID, prompt, choices, verify plan
  - `IntakeResult` for intake analysis with clarifications and facts used
  - `ClarificationSlot` enum: EditorName, ConfigPath, NetworkInterface, etc.
  - `analyze_intake()` checks known facts before asking clarifications

- **Verification Probes (Phase 3)**: Safe probes for clarification verification
  - `verify_probes.rs` module with safe read-only verification commands
  - `run_verify_probe()` executes verification based on VerifyPlan
  - `verify_and_store()` verifies and stores fact if valid
  - `VerificationResult` with verified flag, value, alternatives

- **Clarification Ticket States (Phase 4)**: Ticket pause/resume for clarification
  - `AwaitingClarification` and `VerifyingClarification` ticket statuses
  - Clarification fields in Ticket: pending_clarification_id, answer, rounds
  - `set_pending_clarification()`, `set_clarification_answer()`, `complete_clarification()`
  - Transcript events: ClarificationAsked, ClarificationAnswered, ClarificationVerified, FactStored

- **Clarification Templates (Phase 5)**: Learned clarification patterns in recipes
  - `RecipeKind::ClarificationTemplate` for storing learned patterns
  - `RecipeSlot` struct with name, question_id, required, verify_type
  - `Recipe::clarification_template()` constructor for template recipes
  - Templates store which clarifications to ask for an intent

### Changed
- Recipe now includes clarification_slots, default_question_id, populates_facts fields
- Transcript renderer handles new clarification events in debug mode
- Test coverage updated for new event types

### Technical
- All tests passing
- Files remain under 400 lines
- No breaking CLI changes

## [0.0.30] - 2025-12-05

### Fixed
- **Specialist Timeout Fallback (Phase 1-2)**: Fixed "specialist TIMEOUT → useless rephrase" failure mode
  - Health/status queries ("how is my computer", "any errors") now route deterministically before translator
  - Added `strip_greetings()` to ignore "hello", "hi anna", emoticons in query classification
  - Expanded `SystemHealthSummary` patterns to catch conversational health queries
  - `generate_best_effort_summary()` produces useful answers from any available probe evidence
  - When specialist times out but evidence exists, returns parsed summary instead of rephrase request

- **Translator Hardening (Phase 3)**: Prevent greeting + health query misrouting
  - Updated translator prompt with explicit instructions to ignore greetings
  - Added health query examples to guide correct classification (system domain, not network)
  - `translate_fallback()` now detects health queries before domain classification
  - Health fallback returns comprehensive probe set: memory_info, disk_usage, cpu_info, failed_services

### Added
- **Latency Guardrails (Phase 4)**: Protect against slow specialist responses
  - `max_specialist_prompt_bytes` config option (default 16KB) caps prompt size
  - Prompts exceeding cap skip to deterministic fallback immediately
  - Reduced default specialist timeout from 12s to 8s (deterministic fallback handles gaps)
  - Early budget enforcement prevents wasted time on oversized prompts

### Changed
- Default `specialist_timeout_secs` reduced from 12 to 8 (fallback covers timeouts reliably)
- New config option: `llm.max_specialist_prompt_bytes` (default 16384)
- Router patterns expanded for health/status queries
- Translator fallback now health-query-aware

### Technical
- All tests passing
- Files remain under 400 lines
- No breaking API changes

## [0.0.29] - 2025-12-05

### Added
- **StatusSnapshot (Phase 1)**: Comprehensive system state snapshot
  - `status_snapshot.rs` module with complete system state capture
  - `StatusSnapshot` struct: versions, daemon, permissions, update, helpers, models, config
  - `VersionInfo`, `DaemonInfo`, `PermissionsInfo` for granular health data
  - `UpdateInfo`, `UpdateResult` for update subsystem tracking
  - `HelpersInfo`, `ModelsInfo`, `ConfigInfo` for component state
  - `StatusSnapshot` RPC method for detailed status queries
  - `health_status()` returns "OK", "DAEMON_DOWN", "OLLAMA_MISSING", etc.

- **Update Ledger (Phase 2)**: Auto-update transparency and auditability
  - `update_ledger.rs` module for tracking update checks
  - `UpdateCheckEntry`: timestamp, local_version, remote_tag, result, duration
  - `UpdateCheckResult` enum: UpToDate, UpdateAvailable, Downloaded, Installed, Failed
  - `UpdateLedger` with max 20 entries, persisted to `~/.anna/update_ledger.json`
  - Daemon update loop now records all check results with timing

- **Model Registry (Phase 3)**: Hardware-aware model selection
  - `model_registry.rs` module for role-model bindings
  - `ModelSpec`: name, size_hint_gb, quantization
  - `RoleBinding`: team + role to model mapping
  - `HardwareTier` enum: Low/Medium/High/VeryHigh based on RAM/CPU/GPU
  - `recommended_model_for_tier()` returns appropriate model spec
  - `ModelRegistry` tracks bindings and model states
  - `parse_ollama_list()` for model state detection

- **Telemetry Snapshots (Phase 4)**: Measured system deltas
  - `telemetry/mod.rs`, `telemetry/boot.rs`, `telemetry/pacman.rs` modules
  - `BootSnapshot`: tracks boot time changes via systemd-analyze
  - `parse_systemd_analyze_time()` parses various boot time formats
  - `PacmanSnapshot`: tracks package events from /var/log/pacman.log
  - `PackageEvent`: timestamp, kind (installed/upgraded/removed), package, version
  - Checkpoint-based incremental log reading for efficiency
  - REPL greeting now shows measured telemetry when available
  - Shows "boot X.Xs faster/slower" and "N pkg changes" in greeting

### Changed
- `update_check_loop` now records all results to UpdateLedger
- `DaemonStateInner.to_status_snapshot()` builds comprehensive snapshot
- `print_repl_header()` shows telemetry data when available

### Technical
- All new modules under 400 lines per project guidelines
- All tests passing (257+ tests)
- StatusSnapshot RPC wired into daemon handlers

## [0.0.28] - 2025-12-05

### Added
- **Team-Specialized Junior/Senior Execution (Phase 1)**
  - Extended `SpecialistsRegistry` with prompt accessors
  - `SpecialistProfile.prompt()` returns team-specific prompt
  - `SpecialistsRegistry.junior_prompt(team)` and `senior_prompt(team)`
  - `SpecialistsRegistry.junior_model(team)` and `senior_model(team)`
  - `SpecialistsRegistry.escalation_threshold(team)`

- **Helpers Management (Phase 2)**: Track external dependencies
  - `helpers.rs` module for helper package tracking
  - `HelperPackage` struct: id, name, version, install_source, available
  - `InstallSource` enum: Anna, User, Bundled, Unknown
  - `HelpersRegistry` for managing tracked packages
  - `known_helpers()` returns default helper definitions (ollama)
  - `detect_helper()` for system package detection
  - Persistence to `~/.anna/helpers.json`

- **True Reset (Phase 3)**: `annactl reset` now wipes all learned data
  - Clears ledger (existing behavior)
  - Clears recipes (`~/.anna/recipes/`)
  - Clears helpers store (`~/.anna/helpers.json`)
  - Enhanced reset confirmation dialog showing what will be cleared
  - Returns list of cleared stores in response

- **IT Department Dialog Style (Phase 4)**: Polish for non-debug mode
  - `it_greeting(domain)` - contextual greeting based on query type
  - `it_confidence(score)` - reliability as IT confidence statement
  - `it_domain_context(domain)` - domain as IT department context
  - Clean mode output now uses IT department style formatting
  - Footer shows: Domain Context | Confidence Note | Score

### Changed
- Moved specialists tests to `tests/specialists_tests.rs` (file now 232 lines)
- Enhanced `handle_reset` handler to clear recipes and helpers
- Updated `handle_reset` command with better user feedback

### Tests
- 6 new specialists registry tests for v0.0.28 features
- 10 new helpers module tests
- 3 new narrator IT department style tests

## [0.0.27] - 2025-12-05

### Added
- **Recipe Learning Loop**: Team-tagged recipes for learning from successful patterns
  - `RecipeKind` enum: Query, ConfigEnsureLine, ConfigEditLineAppend
  - `RecipeTarget` struct with app_id and config_path_template
  - `RecipeAction` enum: EnsureLine, AppendLine, None
  - `RollbackInfo` struct for reversible changes
  - Recipe persistence to `~/.anna/recipes/` with deterministic naming
  - Only persists when: Ticket status = Verified, reliability score >= 80

- **Safe Change Engine**: Backup-first, idempotent config edits
  - `ChangePlan` struct describing what will change before execution
  - `ChangeResult` struct with applied, was_noop, backup_path, diagnostics
  - `ChangeOperation` enum: EnsureLine, AppendLine
  - `ChangeRisk` levels: Low, Medium, High
  - `plan_ensure_line()` function for planning changes
  - `apply_change()` function with automatic backup
  - `rollback()` function to restore from backup
  - Deterministic backup naming using path hash

- **Config Intent Detection**: Pattern-based detection for config edit requests
  - `detect_vim_config_intent()` for vim config patterns
  - `detect_config_intent()` for general config detection
  - Supports: syntax on, line numbers, autoindent, mouse, tabs
  - Bridges query classification to change engine

- **Stats Command**: Per-team statistics via `annactl stats`
  - Total requests, success rate, avg reliability
  - Per-team breakdown: total, success, failed, avg rounds, avg score
  - Most consulted team indicator

- **Enhanced Team Routing**: Desktop team routes for editor configs
  - Added vim, nano, emacs, syntax, config_edit route classes

### Changed
- Extracted change.rs tests to tests/change_tests.rs (under 400 lines)
- Added `Stats` RPC method for statistics retrieval

### Tests
- 8 new change engine tests (tempdir-based)
- 9 new config intent detection tests
- 18 recipe tests including v0.0.27 config edit recipes

## [0.0.26] - 2025-12-05

### Added
- **SPECIALISTS Registry**: Team-scoped specialist system
  - `SpecialistRole` enum: Translator, Junior, Senior
  - `SpecialistProfile` struct with team, role, model_id, max_rounds, escalation_threshold
  - `SpecialistsRegistry` with 24 default profiles (8 teams × 3 roles)
  - Teams: Desktop, Storage, Network, Performance, Services, Security, Hardware, General

- **Deterministic Review Gate**: Hybrid review that minimizes LLM calls
  - `ReviewContext` struct with all deterministic signals
  - `GateOutcome` with decision, reasons, requires_llm_review, confidence
  - Pure `deterministic_review_gate()` function - no I/O
  - Rules: Invention → Escalate, No claims → Revise, Low grounding → Revise, High score → Accept
  - Medium scores (50-79) trigger LLM review only when needed

- **Team-Specific Review Prompts**: Customized junior/senior prompts per team
  - Each team has domain-specific verification rules
  - Storage: verify df/lsblk output exactly
  - Network: verify ip/ss output
  - Performance: verify free/top output
  - Security: flag risky operations

- **Review Gate Transcript Events**:
  - `ReviewGateDecision { decision, score, requires_llm }`
  - `TeamReview { team, reviewer, decision, issues_count }`
  - Full visibility into review decisions

- **Trace Enhancements**:
  - `ReviewerOutcome` enum for audit trail
  - `FallbackUsed::Timeout` variant for timeout fallback tracking

- **Ticket Service Integration**:
  - `run_review_gate()` function wired into ticket verification
  - Transcript events emitted for all gate decisions

### Changed
- Refactored `transcript.rs` (495→368 lines) with `transcript_ext.rs` extension module
- Split `review_prompts.rs` into modular directory structure
- All files now under 400 line limit per project standards

### Tests
- 550+ tests passing
- Golden tests for specialists registry serialization
- Golden tests for review gate decisions
- Tests for transcript event creation

## [0.0.23] - 2025-12-05

### Added
- **TRACE Phase**: Structured execution trace for debugging degraded paths
  - `ExecutionTrace` in ServiceDeskResult (wire-compatible, optional)
  - `SpecialistOutcome` enum: Ok | Timeout | BudgetExceeded | Skipped | Error
  - `FallbackUsed` enum: None | Deterministic { route_class }
  - `ProbeStats`: planned/succeeded/failed/timed_out counts
  - `EvidenceKind` enum: Memory | Disk | BlockDevices | Cpu | Services
  - Trace rendering in `annactl` debug mode
  - 12 golden tests for trace scenarios

- **TRUST+ Phase**: Enhanced reliability explanations with fallback context
  - `ReasonContext` extended with fallback fields
  - ProbeTimeout explanation now mentions fallback source and evidence kinds
  - Example: "2 of 3 probes timed out; used deterministic fallback from memory evidence"

- **RESCUE Hardening**: Explicit threshold constants for reliability scoring
  - `INVENTION_CEILING = 40`
  - `PENALTY_NOT_GROUNDED = -30`
  - `PENALTY_BUDGET_EXCEEDED = -15`
  - `PENALTY_PROBE_TIMEOUT = -10`
  - `PENALTY_EVIDENCE_MISSING = -25`
  - `MAX_PROBE_COVERAGE_PENALTY = 30.0`
  - Confidence thresholds: `CONFIDENCE_LOW_THRESHOLD = 0.70`, `CONFIDENCE_MEDIUM_THRESHOLD = 0.85`
  - All magic numbers in `compute_reliability()` replaced with named constants

- **New Parsers**: `lsblk.rs` and `lscpu.rs` for block device and CPU info
- **Probe Answers Module**: Centralized deterministic answer generation

### Changed
- `DeterministicResult` now includes `route_class` field for trace auditing
- All deterministic answers include route classification

## [0.0.18] - 2025-12-05

### Fixed
- **Duplicate `[anna]` block**: Debug mode no longer prints final answer twice
  - Transcript renderer tracks if Anna's answer was already printed in event stream
  - Only prints fallback `[anna]` block if no Anna message was rendered
- **CLI `help` command conflict**: `annactl help` now sends "help" as a request to Anna
  - Disabled clap's implicit help subcommand (`disable_help_subcommand = true`)
  - `annactl --help` still shows CLI usage
- **Misleading specialist output**: Deterministic path shows correct stage status
  - New `StageOutcome::Deterministic` variant
  - Shows `[specialist] skipped (deterministic)` instead of `ok`

### Added
- CLI integration tests for argument parsing regressions
- `ProgressTracker::skip_stage_deterministic()` for cleaner stage handling

## [0.0.17] - 2025-12-05

### Added
- **docs/VERIFICATION.md**: Comprehensive verification guide with exact commands
  - Binary verification, release asset checks, smoke tests
  - Per-feature validation commands for deterministic outputs

### Changed
- **SPEC.md updated to v0.0.16**: Full specification refresh
  - Documents all features from v0.0.13-v0.0.16
  - Pipeline flow diagram, configuration reference
  - Latency stats, timeout handling, probe allowlist

### Fixed
- Cleaned up dead code warnings with `#[allow(dead_code)]` annotations
- Removed unused imports in test files and commands.rs

## [0.0.16] - 2025-12-05

### Added
- **Global Request Timeout**: Configurable `request_timeout_secs` in config.toml (default 20s)
  - Entire pipeline wrapped in global timeout
  - Graceful timeout response with clarification message
- **Per-Stage Latency Stats**: Track avg and p95 latency for last 20 requests
  - Exposed via `annactl status --debug` flag
  - Tracks translator, probes, specialist, and total latency
- **`annactl status --debug`**: Extended status output showing latency statistics
- **v0.0.16 Golden Tests**: Tests for PID column, CRITICAL warnings, state display

### Changed
- **Deterministic Outputs Improved**:
  - top_memory: Shows 10 processes with PID, COMMAND, %MEM, RSS, USER
  - network_addrs: Shows active connection at top ("Active: Wi-Fi (wlan0)...")
  - RSS values formatted human-readable (12M, 1.2G)
- **Translator JSON Parser**: Fully tolerant of malformed JSON
  - Parse errors fallback to defaults instead of failing
  - Missing confidence defaults to 0.0
  - Null arrays become empty Vec
- **Strict Translator Prompt**: Forces exact enum values (intent, domain)
- **Parser Struct Updates**: ProcessInfo now includes `pid` and `rss` fields

### Fixed
- All source files kept under 400 line limit
- Removed unused `extract_pid_from_process` function

## [0.0.15] - 2025-12-05

### Added
- **Triage Router**: Handles ambiguous queries with LLM translator and confidence thresholds
  - Confidence < 0.7 triggers immediate clarification (reliability capped at 40%)
  - Probe cap at 3 maximum per query, warning in evidence if exceeded
  - Deterministic clarification generator fallback if translator fails
- **Probe Summarizer**: Compacts probe outputs to <= 15 lines for specialist
  - Raw output only sent in debug mode with explicit "show raw" request
- **Evidence Redaction**: Automatic removal of sensitive patterns
  - Private keys, password hashes, AWS keys, API tokens
  - Applied even in debug mode for security
- **Two Display Modes**:
  - debug OFF: Clean fly-on-the-wall format (`anna vX.Y.Z`, `[you]`, `[anna]`, reliability/domain footer)
  - debug ON: Full stages with consistent speaker tags on separate lines
- **REPL Polish**:
  - Spinner only in debug OFF mode while waiting
  - Stage transitions shown in debug ON mode
  - Improved help command with examples
- **Config-based debug mode**: `daemon.debug_mode` in config.toml

### Changed
- **Specialist receives summarized probes**: Never raw stdout unless debug + "show raw"
- **Scoring refinement**: Triage path grounded=true only if answer references probe/snapshot facts
- **Clarification max reliability**: Capped at 40% when clarification returned
- **Transcript format**: Content starts on line after speaker tag, no inline arrows

### Fixed
- Display consistency between REPL and one-shot modes
- Redundant separators and spacing in output
- Final [anna] block always present with answer (never empty)

## [0.0.14] - 2025-12-04

### Added
- **Deterministic Router**: Overrides LLM translator for known query classes
  - CPU/RAM/GPU queries: Use hardware snapshot, no probes needed
  - Memory processes: Automatically runs `top_memory` probe
  - CPU processes: Automatically runs `top_cpu` probe
  - Disk queries: Routes to Storage domain with `disk_usage` probe
  - Network queries: Routes to Network domain with `network_addrs` probe
  - "help": Returns deterministic help response
  - "slow/sluggish": Runs multi-probe diagnostic (CPU, memory, disk)
- **Help command**: "help" now returns comprehensive usage guide
- **Interface type detection**: WiFi vs Ethernet heuristics (wlan*/wlp* = WiFi)
- **Golden tests**: Router, translator robustness, scoring validation

### Changed
- **Translator JSON parsing tolerant**: Missing fields use sensible defaults
  - Missing `confidence` → 0.0
  - Null arrays → empty Vec
  - Missing `intent`/`domain` → fallback to deterministic router
- **Specialist skipped for known classes**: Deterministic answers bypass LLM
- **Scoring reflects reality**:
  - `grounded=true` only if parsed data count > 0
  - Empty parser result = clarification needed, not 100% score
  - Coverage based on actual probe success, not request count
- **Improved deterministic outputs**:
  - Process tables include PID column
  - Disk usage shows critical (>=95%) and warning (>=85%) status
  - Network interfaces show type (WiFi/Ethernet/Loopback)

### Fixed
- Known query classes can't be misrouted by LLM translator
- Translator errors don't block deterministic answering
- Empty parser results don't claim 100% reliability

## [0.0.13] - 2025-12-04

### Added
- **Per-stage model selection**: Configure different models for each pipeline stage
  - `translator_model`: Fast small model for query classification (default: qwen2.5:1.5b-instruct)
  - `specialist_model`: Capable model for domain expert answers (default: qwen2.5:7b-instruct)
  - `supervisor_model`: Validation model (default: qwen2.5:1.5b-instruct)
- **Config file support**: `/etc/anna/config.toml` with LLM section
- **Configurable timeouts**: Per-stage timeouts in config file
  - `translator_timeout_secs`: 4s (default)
  - `specialist_timeout_secs`: 12s (default)
  - `supervisor_timeout_secs`: 6s (default)
  - `probe_timeout_secs`: 4s (default)

### Changed
- **Translator payload minimized**: < 2KB for typical requests
  - Inputs: user query, one-line hardware summary, probe ID list
  - NO probe stdout/stderr, NO evidence blocks, NO long policy text
- **Daemon pulls all required models on startup/healthcheck**
- **Status shows all models with roles** (translator, specialist, supervisor)
- **Models pulled based on config**, not hardware detection

### Fixed
- Translator no longer receives large probe outputs
- Consistent timeout values across pipeline stages

## [0.0.12] - 2025-12-04

### Added
- **Deterministic Answerer**: Fallback module that answers common queries without LLM
  - CPU info: From hardware snapshot or lscpu probe
  - RAM info: From hardware snapshot or free -h probe
  - GPU info: From hardware snapshot
  - Top memory processes: Parsed from ps aux --sort=-%mem
  - Disk space: Parsed from df -h with critical/warning flags
  - Network interfaces: Parsed from ip addr show
  - Rules: Never invents facts, always produces grounded answers

### Changed
- **Specialist timeout behavior**: Now tries deterministic answerer instead of asking for clarification
- **Scoring improvements**:
  - Deterministic answers get `answer_grounded=true` and `no_invention=true` automatically
  - `translator_confident` is false if translator timed out
  - Score no longer capped at 20 when probes succeed with deterministic answer
- **Domain consistency**: ServiceDeskResult.domain now matches the classified domain
- **Update check**: Verifies release assets exist before showing update available

### Fixed
- Anna now produces answers even when specialist LLM times out (reliability > 20)
- Domain in summary now matches dispatcher routing
- Clarification no longer shown when probe data is available

## [0.0.11] - 2024-12-04

### Added
- **Transcript event model**
  - Single `TranscriptEvent` type for pipeline visibility
  - Events: Message, StageStart, StageEnd, ProbeStart, ProbeEnd, Note
  - Actors: You, Anna, Translator, Dispatcher, Probe, Specialist, Supervisor, System
  - Full request tracing with elapsed timestamps

- **Two render modes**
  - debug OFF: Human-readable fly-on-the-wall format
  - debug ON: Full troubleshooting view with stage timings

- **REPL improvements**
  - Prompt changed to `anna> `
  - Ctrl-D (EOF) now exits cleanly
  - Empty lines after answers for readability

- **CI improvements**
  - Release artifact naming check
  - Test files excluded from 400-line limit

### Changed
- ServiceDeskResult now includes `request_id` and `transcript`
- Transcript events generated during pipeline execution
- Refactored rpc_handler.rs to stay under 400 lines
  - Extracted utility handlers to handlers.rs
  - Extracted ProgressTracker to progress_tracker.rs

### Fixed
- Release script already had correct artifact naming (annad-linux-x86_64, annactl-linux-x86_64)
- CI now verifies release script uses correct names

## [0.0.7] - 2024-12-04

### Added
- **Service desk architecture**
  - Internal roles: translator, dispatcher, specialist, supervisor
  - Specialist domains: system, network, storage, security, packages
  - Automatic domain classification from query
- **Reliability scores**
  - Every response includes 0-100 reliability score
  - Score increases with successful probes
  - Color-coded display (green >80%, yellow 50-80%, red <50%)
- **Unified output format**
  - One-shot and REPL use identical formatting
  - Shows version, specialist domain, reliability, probes used
  - Consistent `[you]`/`[anna]` transcript blocks
- **Probe allowlist**
  - Only 11 read-only commands allowed
  - Dangerous commands are explicitly denied
  - Security tests verify allowlist safety
- **Clarification rules**
  - Short/ambiguous queries ask for more details
  - "help" without context triggers clarification
- **Golden tests**
  - 16 new tests for service desk behavior
  - Domain routing tests
  - Probe security tests
  - Output format consistency tests

### Changed
- **Request pipeline now uses service desk**
  - translate → dispatch → specialist → supervisor
  - All responses include ServiceDeskResult metadata
- **Response format includes domain and reliability**
  - No longer just raw text response
  - Full metadata for transparency

### Fixed
- REPL and one-shot now produce identical output format
- Commands.rs uses single send_request function for both modes

## [0.0.6] - 2024-12-04

### Added
- **Grounded LLM responses**
  - RuntimeContext injected into every LLM request
  - Hardware snapshot (CPU, RAM, GPU) always available to LLM
  - Capability flags prevent claiming abilities Anna doesn't have
- **Auto-probes for queries**
  - Memory/process queries auto-run `ps aux --sort=-%mem`
  - Disk queries auto-run `df -h`
  - Network queries auto-run `ip addr show`
- **Probe RPC method**
  - `top_memory` - Top processes by memory
  - `top_cpu` - Top processes by CPU
  - `disk_usage` - Filesystem usage
  - `network_interfaces` - Network info
- **Integration tests for grounding**
  - Version consistency tests
  - Hardware context tests
  - Capability safety tests

### Changed
- **System prompt completely rewritten**
  - Strict grounding rules enforced
  - Never invents facts not in context
  - Answers hardware questions from snapshot
  - Never suggests manual commands when data available

### Fixed
- Anna no longer claims to be "v0.0.1" or wrong versions
- Anna no longer suggests `lscpu` when CPU info is in context
- Anna answers memory questions with actual process data

### Documentation
- SPEC.md updated to v0.0.6 with grounding policy
- README.md updated with features
- TRUTH_REPORT.md documents what was broken and how it was fixed

## [0.0.5] - 2024-12-04

### Added
- **Enhanced status display**
  - CPU model and core count
  - RAM total in GB
  - GPU model and VRAM
- **Improved REPL exit commands**
  - Added: bye, q, :q, :wq (for vim users!)

### Changed
- **Smarter model selection**
  - With 8GB VRAM: llama3.1:8b (was llama3.2:3b)
  - With 12GB+ VRAM: qwen2.5:14b
  - Better tiered selection based on GPU/RAM

### Fixed
- Friendlier goodbye message

## [0.0.4] - 2024-12-04

### Added
- **Auto-update system**
  - GitHub release version checking every 60 seconds
  - Automatic download and verification of new releases
  - Zero-downtime updates via atomic binary replacement
  - SHA256 checksum verification for security
- **Enhanced status display**
  - Current version and available version from GitHub
  - Update check pace (every 60s)
  - Countdown to next update check
  - Auto-update enabled/disabled status
  - "update available" indicator when new version exists
- **Security and permissions**
  - Dedicated `anna` group for socket access
  - Installer automatically creates group and adds user
  - Health check auto-adds new users to anna group
  - No reboot needed - `newgrp anna` activates immediately
  - Fallback to permissive mode if group unavailable

### Changed
- Update check interval reduced from 600s to 60s
- Status output now shows comprehensive version/update information
- Socket permissions now use group-based access (more secure)

## [0.0.3] - 2024-12-04

### Added
- **Self-healing health checks**
  - Periodic health check loop (every 30 seconds)
  - Automatic detection of missing Ollama or models
  - Auto-repair sequence when issues detected
- **Package manager support**
  - Ollama installation via pacman on Arch Linux
  - Fallback to official installer for other distros
- **Friendly bootstrap UI**
  - Live progress display when environment not ready
  - "Hello! I'm setting up my environment. Come back soon! ;)"
  - Spinner with phase and progress bar
  - Auto-continues when ready

### Changed
- annactl now waits and shows progress if LLM not ready
- REPL shows bootstrap progress before accepting input
- Requests wait for bootstrap completion automatically
- Split display code into separate module for maintainability

### Fixed
- Socket permissions allow regular users to connect
- Installer stops existing service before upgrade

## [0.0.2] - 2024-12-04

### Added
- **Beautiful terminal UI**
  - Colored output with ANSI true color (24-bit)
  - Progress bars for downloads
  - Formatted byte sizes (1.2 GB, 45 MB, etc.)
  - Formatted durations (2h 30m 15s)
  - Consistent styling across all commands
- **Enhanced status display**
  - LLM state indicators (Bootstrapping, Ready, Error)
  - Benchmark results display (CPU, RAM, GPU status)
  - Model information with roles
  - Download progress with ETA
  - Uptime and update check timing
- **Improved installer**
  - Beautiful step-by-step output
  - Clear sudo explanations
  - Checksum verification display

### Changed
- Refactored status types for richer UI
- Moved UI helpers to anna-shared for consistency

## [0.0.1] - 2024-12-04

### Added
- Initial release with complete repository rebuild
- **annad**: Root-level systemd daemon
  - Automatic Ollama installation and management
  - Hardware probing (CPU, RAM, GPU detection)
  - Model selection based on system resources
  - Installation ledger for safe uninstall
  - Update check ticker (every 600 seconds)
  - Unix socket RPC server (JSON-RPC 2.0)
- **annactl**: User CLI
  - `annactl <request>` - Send natural language request
  - `annactl` - Interactive REPL mode
  - `annactl status` - Show system status
  - `annactl reset` - Reset learned data
  - `annactl uninstall` - Safe uninstall via ledger
  - `annactl -V/--version` - Show version
- Installer script (`scripts/install.sh`)
- Uninstaller script (`scripts/uninstall.sh`)
- CI workflow with enforcement checks:
  - 400-line file limit
  - CLI surface verification
  - Build and test verification

### Security
- annad runs as root systemd service
- annactl communicates via Unix socket
- No remote network access except for Ollama API and model downloads

### Known Limitations
- v0.0.1 supports read-only operations only
- Full LLM pipeline planned for future versions
- Single model support only

[Unreleased]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.18...HEAD
[0.0.18]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.17...v0.0.18
[0.0.17]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.16...v0.0.17
[0.0.16]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.15...v0.0.16
[0.0.15]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.14...v0.0.15
[0.0.14]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.13...v0.0.14
[0.0.13]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.12...v0.0.13
[0.0.12]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.11...v0.0.12
[0.0.11]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.7...v0.0.11
[0.0.7]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.0.1
