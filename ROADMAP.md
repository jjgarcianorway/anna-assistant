# Anna Roadmap

## Current Focus (v0.0.369+)

**Theme**: UI Consistency + Code Quality + Reliability

Anna v0.0.369 focuses on:
- Centralized UI system for consistent terminal output
- Zero compiler warnings for cleaner builds (library + tests)
- Modular code design (all files under 400 lines)
- Enhanced user experience across all displays
- Natural language guidance (no non-existent CLI references)
- Unified symbols across all UI components

### Recent Completions (v0.0.346-369)
- [x] Unified bullet symbol to symbols::BULLET (v0.0.369)
- [x] Documentation updates (v0.0.368)
- [x] Fixed stale CLI command references in 7 files (v0.0.365)
- [x] UI consistency in REPL/uninstall messages (v0.0.366)
- [x] Fixed undo command reference → natural language (v0.0.367)
- [x] Documentation cleanup after revert (v0.0.364)
- [x] UI consistency in errors.rs - print_step() for recovery suggestions (v0.0.356)
- [x] UI consistency in handlers.rs - print_ok/warn for uninstall flow (v0.0.356)
- [x] Zero test warnings - cleaned unused imports/functions (v0.0.357)
- [x] Documentation updates - FEATURES.md, README.md, ROADMAP.md (v0.0.359-360)

---

## Completed

### v0.0.337-363 - Centralized UI System (Phase 28) ✓
- [x] `anna_shared::ui::printing` module with consistent helpers
- [x] print_header, print_title, print_footer, print_hr
- [x] print_section_header, print_label, print_hint, print_step
- [x] print_ok, print_err, print_warn with symbols
- [x] kv() and kv_colored() for key-value display
- [x] colors and symbols constants centralized
- [x] Updated greeting/status.rs, greeting/personal.rs
- [x] Updated errors.rs, handlers.rs, progress_display.rs
- [x] Updated change_commands.rs, learning.rs
- [x] Zero compiler warnings in release build + tests (v0.0.357)
- [x] Removed unused DecayResult import
- [x] Removed unused VerificationInput.id field
- [x] Removed unused VerificationResult.score field
- [x] Removed unused test imports and functions (v0.0.357)
- [x] Removed dead code files (v0.0.363)

### v0.0.103 - Recipe Feedback System (Phase 23) ✓
- [x] `FeedbackRequest` struct - Anna asks user for feedback when uncertain
- [x] `feedback_request` field in ServiceDeskResult
- [x] Anna asks for feedback on borderline confidence (60-75) or new recipes (<3 uses)
- [x] Interactive feedback handling in REPL and one-shot modes
- [x] Feedback adjusts recipe reliability_score (+1 helpful, -5 not-helpful)
- [x] Feedback history logged to ~/.anna/feedback_history.jsonl

### v0.0.102 - Recipe Direct Answers (Phase 22) ✓
- [x] Direct answer from recipe template (skip probes too)
- [x] `build_recipe_result()` creates ServiceDeskResult from recipe
- [x] `can_answer_directly()` checks for answer template
- [x] Instant responses for learned patterns

### v0.0.101 - Recipe Fast Path Integration (Phase 21) ✓
- [x] Recipe index built at daemon startup
- [x] Recipe check BEFORE calling LLM translator
- [x] High-confidence recipes skip LLM (score >= 70)
- [x] ConfigureShell and ConfigureGit query classes
- [x] Shell/git config query routing

### v0.0.100 - Recipe Matcher & Config Recipes (Phase 20) ✓
- [x] Recipe matcher for translator fast-path
- [x] Shell configuration recipes (bash, zsh, fish)
- [x] Git configuration recipes
- [x] New RecipeKind variants (ShellConfig, GitConfig)

### v0.0.99 - Natural Language Package & Service Management (Phase 19) ✓
- [x] Package install via natural language ("install htop")
- [x] Service management via natural language ("restart docker")
- [x] Cross-distro package name mapping
- [x] Protected service detection
- [x] QueryClass: InstallPackage, ManageService

### v0.0.98 - Multi-file Transactions & Recipe Systems (Phase 18) ✓
- [x] ChangeTransaction for atomic multi-file changes
- [x] Automatic rollback on failure
- [x] Package recipes with multi-manager support (pacman, apt, dnf, flatpak, snap)
- [x] Service recipes with risk levels and protected services
- [x] Cross-distro package name mapping

### v0.0.97 - Change History and Undo (Phase 17) ✓
- [x] Change history tracking in ~/.anna/change_history.jsonl
- [x] `annactl history` command
- [x] `annactl undo <id>` command
- [x] Backup-based restoration

### v0.0.96 - Desktop Team Editor Config Flow (Phase 16) ✓
- [x] Natural language editor configuration ("enable syntax highlighting")
- [x] proposed_change field in ServiceDeskResult
- [x] CLI confirmation flow for config changes
- [x] Integration with Safe Change Engine

### v0.0.95 - Safe Change Engine (Phase 15) ✓
- [x] PlanChange, ApplyChange, RollbackChange RPC methods
- [x] Backup-first, idempotent config modifications
- [x] Extracted editor_recipe_data.rs module
- [x] All files under 400 lines

### v0.0.94 - Recipe Learning System (Phase 14) ✓
- [x] Automatic recipe learning from successful queries
- [x] Learning criteria: verified=true, reliability >= 80
- [x] Recipe persistence in ~/.anna/recipes/
- [x] Team assignment from domain

### v0.0.93 - Documentation Update (Phase 13) ✓
- [x] Updated README, ROADMAP, FEATURES for current version
- [x] Hollywood IT aesthetic documentation

### v0.0.92 - Codebase Hygiene (Phase 12) ✓
- [x] Zero compiler warnings across entire workspace
- [x] Fixed unused methods, variables, and imports
- [x] Applied cargo fix to test files

### v0.0.91 - ASCII-Style Achievement Badges (Phase 11) ✓
- [x] Replaced emoji badges with ASCII art symbols
- [x] Badge styles: `[1]` `<3d>` `(90+)` `{*}` `~00~` `|7d|`
- [x] Hollywood IT aesthetic consistency

### v0.0.90 - Achievement Badges (Phase 10) ✓
- [x] 22 unique achievements across 6 categories
- [x] Milestones, Streaks, Quality, Teams, Special, Tenure
- [x] Integration with stats display

### v0.0.89 - Personalized Greetings (Phase 9) ✓
- [x] Time-of-day awareness (Morning, Afternoon, Evening, Night)
- [x] User personalization from $USER
- [x] Domain-specific follow-up prompts
- [x] New greetings.rs module

### v0.0.88 - Warning Cleanup (Phase 8) ✓
- [x] Removed all compiler warnings
- [x] Fixed unused imports across workspace

### v0.0.87 - Dialogue Variety (Phase 7) ✓
- [x] Varied junior approval phrases
- [x] Varied escalation requests
- [x] Varied senior responses
- [x] Seed-based deterministic variety

### v0.0.81-86 - Service Desk Theatre ✓
- [x] Named IT personas with roles
- [x] Cinematic narrative rendering
- [x] Internal communications mode (-i flag)
- [x] Streak tracking and XP system

### v0.0.75 - RPG Stats System ✓
- [x] Event logging with JSONL store
- [x] XP calculation and level progression
- [x] Titles from Trainee to Principal Engineer
- [x] Stats display with progress bars

### v0.0.71 - Version Truth ✓
- [x] Single source of truth: workspace Cargo.toml version only
- [x] Unified version display: annactl/annad --version format consistent
- [x] Status shows: installed (annactl), daemon_ver (annad), available, last_check, next_check, auto_update
- [x] Hard gate tests: CI fails if annactl/annad version != workspace version
- [x] No hardcoded version strings in tests (compare against VERSION constant)
- [x] Auto-update semantic comparison with no-downgrade guarantee

### v0.0.70 - Version Unification + Release Hygiene ✓
- [x] Single source of truth: workspace Cargo.toml version is authoritative
- [x] All crates use version.workspace = true
- [x] anna_shared::VERSION uses env!("CARGO_PKG_VERSION")
- [x] install.sh fetches version from GitHub releases API (no hardcoding)
- [x] Version consistency tests validate all sources
- [x] Status output shows: installed, available, last_check, next_check, auto_update
- [x] Auto-update uses semantic version comparison (no string comparison)
- [x] No downgrade guarantee: newer installed version is never replaced

### v0.0.69 - Unified Versioning + REPL Enhancements ✓
- [x] Single source of truth for version (workspace Cargo.toml)
- [x] REPL "since last time" summary with snapshot comparison
- [x] Delta tracking for failed services, disk, memory changes
- [x] Version consistency tests
- [x] Documentation updates (CHANGELOG, FEATURES, README)

### v0.0.68 - Audio Parse Correctness + ConfigureEditor Grounding ✓
- [x] Audio deterministic answer handles "Multimedia audio controller"
- [x] ConfigureEditor uses full router probe list (skip spine override)
- [x] Clarification prompts end with period, not question mark

### v0.0.67 - Service Desk Theatre UX ✓
- [x] Service desk narrative renderer (render.rs)
- [x] REPL narrative header with boot status, critical issues
- [x] Stats RPG system with XP calculation
- [x] Local citations system (citations.rs)

### v0.0.66 - Version Normalization + Regressions ✓
- [x] Version consolidation across all sources
- [x] Audio evidence parsing for lspci PCI class codes
- [x] ConfigureEditor numbered options without question marks

### v0.0.63 - Service Desk Theatre Renderer ✓
- [x] Narrative flow in normal mode ("Checking X...")
- [x] Evidence source in footer when grounded
- [x] Clarification options numbered display
- [x] New transcript events (EvidenceSummary, DeterministicPath, ProposedAction, ActionConfirmationRequest)
- [x] Debug mode rendering for all new events

### v0.0.62 - ConfigureEditor Grounding ✓
- [x] Proper probe accounting with valid_evidence_count
- [x] Execution trace for all ConfigureEditor paths
- [x] Grounding signals based on valid evidence

### v0.0.61 - HardwareAudio Parser ✓
- [x] Content-based audio detection (not just command pattern)
- [x] pactl detection by "Card #" blocks
- [x] Evidence merge from lspci + pactl

### v0.0.45 - Query Classification & Probe Planning ✓
- [x] New QueryClass variants: InstalledToolCheck, HardwareAudio, CpuTemp, CpuCores, PackageCount, MemoryFree
- [x] Modularized router.rs + query_classify.rs
- [x] Stabilization golden tests
- [x] ReliabilityInput builder methods

### v0.0.26 - Team-Scoped Review System ✓
- [x] SPECIALISTS Registry: Team-scoped roles (Translator, Junior, Senior)
- [x] 8 Teams: Desktop, Storage, Network, Performance, Services, Security, Hardware, General
- [x] Deterministic Review Gate: Hybrid logic that minimizes LLM calls
- [x] Team-specific junior/senior review prompts
- [x] Review gate transcript events
- [x] Trace enhancements (ReviewerOutcome, FallbackUsed::Timeout)

### v0.0.23 - TRACE + TRUST+ + RESCUE ✓
- [x] Execution trace for debugging degraded paths
- [x] Enhanced reliability explanations
- [x] Explicit threshold constants for scoring

### v0.0.18 and earlier ✓
- [x] Core pipeline with grounded responses
- [x] Deterministic probe routing
- [x] Auto-update mechanism
- [x] Per-stage latency tracking
- [x] Hardware-aware model selection

### v0.0.235 - Docker Compose Recipes (Phase 27) ✓
- [x] DockerFeature enum (CreateCompose, StartServices, StopServices, ViewLogs, etc.)
- [x] DockerRecipe with answer templates
- [x] 10 built-in recipes for Docker Compose workflows
- [x] Query matcher integration with recipe_fast_path
- [x] RecipeKind::DockerCompose variant

### v0.0.234 - Cron Job Recipes (Phase 26) ✓
- [x] CronFeature enum (AddJob, ListJobs, EditCrontab, RemoveJob, etc.)
- [x] CronPreset enum (Hourly, Daily, Weekly, Monthly, etc.)
- [x] CronRecipe with syntax help and examples
- [x] 8 built-in recipes for cron management
- [x] Query matcher integration with recipe_fast_path
- [x] RecipeKind::CronJob variant

### v0.0.233 - Systemd Unit File Recipes (Phase 25) ✓
- [x] SystemdFeature enum (CreateService, CreateTimer, EnableService, ViewLogs, etc.)
- [x] UnitType, RestartPolicy enums
- [x] SystemdRecipe with unit file templates
- [x] 8 built-in recipes for systemd management
- [x] Query matcher integration with recipe_fast_path
- [x] RecipeKind::SystemdUnit variant

### v0.0.104 - SSH Key Management Recipes (Phase 24) ✓
- [x] SshFeature enum (GenerateKey, CopyKey, Config, Agent, GitHub, etc.)
- [x] SshRecipe with answer templates
- [x] 7 built-in recipes for SSH key management
- [x] Query matcher for SSH-related queries
- [x] RecipeKind::SshConfig variant

## Planned

### Phase 29 - Enhanced Status Display ✓ (v0.0.463)
- [x] Show installed vs available GitHub version with asset verification
- [x] Display update check pace, last check, next scheduled check
- [x] Show all user groups and folder permissions (FolderPermission struct)
- [x] Show Ollama status separately from daemon status
- [x] List helpers with "installed by Anna" vs "installed by user" labels
- [x] Display which specialist uses which LLM
- [x] Show all config settings in status (not just debug mode)
- [x] REPL greeting announces errors (SystemError struct, add_error(), render())

### Phase 30 - Enhanced Stats Display ✓ (v0.0.464)
- [x] Non-linear XP progression (0-100 RPG style, logistic curve)
- [x] Funny titles based on level (Trainee -> Linus's Chosen One)
- [x] Recipes categorized by type (in by_handler)
- [x] Average interaction count between Anna and specialists
- [x] Number of times Anna managed on her own (anna_solo_count)
- [x] Number of recipes learned (recipes_learned)
- [x] Longest/shortest resolution times (min/max_duration_ms)
- [x] Most consulted team (most_consulted_team)
- [x] Repeated questions tracking (repeated_queries)
- [x] Topic most asked about (top_topic)
- [x] Longest/shortest reply (tracked via duration proxy)
- [x] Installation date display (first_event_ts)
- [x] `annactl stats <category>` for detailed views (rpg, learning, outcomes, topics, repeated)

### Phase 31 - Natural Language Dialog ✓ (v0.0.465)
- [x] Anna-to-Specialist dialog rendering in normal mode (TranscriptSegment system)
- [x] Case number display (CN-XXXX-DDMMYYYY format via format_case_with_date)
- [x] Specialist responses with reliability assessments (junior_approval, senior_response)
- [x] Anna confirming recipe storage after high-reliability answers (anna_recipe_storage_confirmation)
- [x] Real-time streaming word-by-word from LLM (Progress segment kind)
- [x] Screen updates after each LLM call (transcript segments streamed)

### Phase 32 - Smart Helper Management ✓ (v0.0.466)
- [x] Auto-install helpers learned from specialists (register_anna_installed)
- [x] Track helper source (Anna vs User) (InstallSource enum)
- [x] Remove only Anna-installed helpers on uninstall (remove_anna_installed)
- [x] Skip useless helpers (no ethtool without ethernet) (hardware_requirement, is_useful)
- [x] Display helper last_used timestamp (last_used, last_used_display)

### Phase 33 - Dynamic Team Availability ✓ (v0.0.454)
- [x] Detect hardware capabilities (sound, network, etc.)
- [x] Hide teams for missing hardware (no Sound team if no audio)
- [x] Show available team count in status

### Phase 34 - Long-Running Task Handling ✓ (v0.0.455)
- [x] Detect tasks taking > X minutes
- [x] Ask for email (store for reuse)
- [x] Move to background with notification
- [ ] Investigate during idle time (deferred to Phase 37)
- [ ] Email with chain of thoughts and conclusion (deferred to Phase 37)
- [ ] Second email if internet research needed (deferred to Phase 37)

### Phase 35 - User Notifications ✓ (v0.0.456)
- [x] Email notifications for long tasks
- [x] libnotify integration for desktop alerts
- [x] wall for terminal broadcasts (optional)
- [x] Custom alarms via natural language ("notify me every Monday at 9 about storage")

### Phase 36 - Knowledge & Citations ✓ (v0.0.457)
- [x] Citations from Arch Wiki, man pages, --help commands
- [x] Local cache of Arch Wiki pages linked from official docs
- [x] Learning mode explains why commands are run
- [x] Learning mode explains how commands work

### Phase 37 - Idle Time Learning ✓ (v0.0.458)
- [x] Senior specialists think strategically during idle time
- [x] Resumable tasks if interrupted
- [x] Email notification when idle task completes

### Phase 38 - Kubernetes Recipes ✓ (v0.0.459)
- [x] K8sFeature enum (19 features for pods, deployments, services, etc.)
- [x] K8sRecipe with answer templates and commands
- [x] Query matcher with longest-keyword matching
- [x] Builtin recipes for all common kubectl operations
- [x] Debugging recipes for CrashLoopBackOff, ImagePullBackOff

### Phase 39 - Web Server Recipes ✓ (v0.0.460)
- [x] WebServerFeature enum (15 features for Nginx/Apache)
- [x] WebServerRecipe with config examples and commands
- [x] Installation recipes for Arch, Debian, Fedora
- [x] SSL/TLS with Let's Encrypt/Certbot
- [x] Reverse proxy, load balancing, performance optimization

### Phase 40 - Database Recipes ✓ (v0.0.461)
- [x] DatabaseFeature enum (15 features for backup, restore, management)
- [x] DatabaseRecipe with multi-DB support
- [x] PostgreSQL, MySQL/MariaDB, SQLite, MongoDB, Redis
- [x] User/permission management recipes
- [x] Import/export data recipes

### Phase 41 - Network Troubleshooting Recipes ✓ (v0.0.462)
- [x] NetworkFeature enum (15 features for diagnostics)
- [x] NetworkRecipe with tool requirements
- [x] Connectivity, DNS, traceroute recipes
- [x] Port scanning, firewall, listening ports
- [x] SSL certificate, HTTP testing
- [x] WiFi diagnostics and VPN status

### Phase 43 - Personality Configuration via Natural Language ✓ (v0.0.467)
- [x] ConfigChange enum for 7 preference categories
- [x] detect_config_change() natural language parser
- [x] is_show_preferences() query detection
- [x] apply_config_change() updates UserProfile
- [x] format_preferences() for display
- [x] Supports: learning mode, verbosity, auto-confirm, internal comms
- [x] Personality: formality, humor, technical depth

### Phase 44 - Tips in Greetings ✓ (v0.0.468)
- [x] greeting_tips.rs module for configuration tips
- [x] Tips based on current user settings
- [x] Randomized selection with variety
- [x] Non-intrusive (1 in 3 greetings probability)
- [x] Categories: Learning, Personality, Safety, Display, Notifications
- [x] get_random_greeting_tip() convenience function

### Phase 45 - Monitoring & Custom Alarms ✓ (v0.0.469)
- [x] system_monitors.rs for proactive monitoring
- [x] CheckType enum for 5 metric types
- [x] Platform-specific monitoring (reads /proc, uses df, systemctl)
- [x] evaluate_condition() for alarm condition checking
- [x] check_conditional_alarms() integrates with AlarmStore
- [x] run_all_checks() for full system health
- [x] format_monitor_results() for display

### Phase 46 - Notification Configuration via Natural Language ✓ (v0.0.470)
- [x] notification_config.rs for notification settings
- [x] NotifyConfigChange enum for 7 setting types
- [x] Email configuration via "set my email to X"
- [x] Desktop notification toggle
- [x] Wall broadcast toggle
- [x] Quiet hours configuration
- [x] format_notification_settings() for display

### Phase 47 - Facts Lifecycle Management ✓ (v0.0.471)
- [x] facts_maintenance.rs for scheduled cleanup
- [x] MaintenanceResult tracks transitions
- [x] FactsHealth with statistics by category
- [x] run_maintenance() for lifecycle transitions
- [x] get_health() for facts statistics
- [x] Health ratio check (< 30% stale = healthy)

### Phase 48 - Arch Wiki Local Caching ✓ (v0.0.472)
- [x] wiki_cache.rs for local caching
- [x] WikiCacheEntry with metadata
- [x] WikiCacheIndex for management
- [x] essential_pages() and missing_essential()
- [x] prune_stale() and prune_to_size()
- [x] Content hash for change detection

### Phase 49 - Debug Configuration via Natural Language ✓ (v0.0.473)
- [x] debug_config.rs for NL debug settings
- [x] DebugConfigChange enum (SetLevel, EnableLogFile, Redact*)
- [x] detect_debug_config() parses natural language
- [x] apply_debug_change() updates DebugConfig
- [x] format_debug_settings() displays settings
- [x] parse_debug_level() parses level strings

### Phase 50 - Risk Level Configuration via Natural Language ✓ (v0.0.474)
- [x] risk_config.rs for NL risk tolerance settings
- [x] RiskTolerance struct with auto-confirm, warnings, protection
- [x] RiskConfigChange enum for config changes
- [x] Presets: cautious, balanced, confident, expert
- [x] detect_risk_config() parses natural language
- [x] should_auto_confirm() checks risk against tolerance

### Phase 51 - Session Summary Display ✓ (v0.0.475)
- [x] session_display.rs for session info display
- [x] SessionStats for current session statistics
- [x] format_current_session() displays stats
- [x] format_session_history() shows past sessions
- [x] is_session_query() detects session queries
- [x] get_since_last_time() for greeting integration

### Phase 52 - Unified Settings Display ✓ (v0.0.476)
- [x] settings_display.rs for unified settings view
- [x] AllSettings struct combining all config types
- [x] format_all_settings() shows all sections
- [x] format_settings_summary() compact summary
- [x] is_all_settings_query() query detection
- [x] get_settings_section() section filtering

### Phase 53 - Learning Mode Explanations Enhancement ✓ (v0.0.477)
- [x] Split into module directory (mod.rs, commands.rs)
- [x] Added 15+ new command explanations
- [x] list_explained_commands() for discovery
- [x] Improved display format (cleaner output)
- [x] Total 29+ explained commands

### Phase 54 - XP/Level Display Enhancement ✓ (v0.0.478)
- [x] xp_display.rs for RPG-style progression
- [x] AnnaXP struct with level, XP, progress, title
- [x] Non-linear XP thresholds (10 levels)
- [x] Funny level titles (Intern → Chief Technology Guru)
- [x] format_xp_display() with progress bar
- [x] format_xp_compact() for greetings

### Phase 55 - Fun Statistics Display ✓ (v0.0.479)
- [x] fun_stats_display.rs for VISION.md Fun Statistics
- [x] FunStats struct from AggregatedEvents
- [x] Installation date and days active
- [x] Most consulted team with count
- [x] Lucky team with success rate
- [x] Anna solo count and percentage
- [x] Longest/shortest reply times
- [x] Current and best streak
- [x] format_fun_stats() full display
- [x] format_fun_stats_compact() for greetings
- [x] generate_fun_fact() random facts
- [x] is_fun_stats_query() query detection

### Phase 56 - Capabilities Display ✓ (v0.0.480)
- [x] capabilities_display.rs for "what can you do?" queries
- [x] CapabilityCategory enum (9 categories)
- [x] Examples and descriptions for each category
- [x] format_capabilities() full display
- [x] format_capabilities_compact() one-line
- [x] format_capability_category() single category
- [x] format_capabilities_with_teams() with team count
- [x] is_capabilities_query() query detection
- [x] parse_capability_category() parse from query
- [x] capability_facts() and random_capability_fact()

### Phase 57 - Query Type Router ✓ (v0.0.481)
- [x] query_type_router.rs consolidates query detection
- [x] QueryType enum for all informational queries
- [x] QueryType::detect() from natural language
- [x] route_query() central routing function
- [x] should_handle_locally() skip specialist check
- [x] suggest_display() display function suggestion
- [x] is_status_query() status detection
- [x] extract_help_context() for "help with X"
- [x] is_informational() pure info check

### Phase 58 - Contextual Tips System ✓ (v0.0.482)
- [x] contextual_tips.rs for context-aware tips
- [x] TipContext for topic/command tracking
- [x] Topic detection from queries (8 categories)
- [x] get_contextual_tips() returns relevant tips
- [x] Tips with related actions
- [x] Learning mode tips
- [x] General fallback tips
- [x] format_tip() with action hints

### Phase 59 - Command Shortcuts ✓ (v0.0.483)
- [x] command_shortcuts.rs for quick aliases
- [x] 25+ built-in shortcuts (8 categories)
- [x] expand_shortcut() expands short to full
- [x] is_shortcut() detection
- [x] shortcuts_by_category() filtering
- [x] format_shortcuts() display
- [x] is_shortcuts_query() query detection
- [x] Case-insensitive matching

### Phase 60 - Quick Status Summary ✓ (v0.0.484)
- [x] quick_status.rs for at-a-glance status
- [x] HealthLevel enum (Good, Warning, Critical, Unknown)
- [x] StatusItem struct with name, health, message, value
- [x] QuickStatus struct with items, overall health, summary
- [x] memory_status() - Memory health from percentage
- [x] disk_status() - Disk health from percentage
- [x] cpu_status() - CPU health from load/cores
- [x] service_status() - Service health from state
- [x] format_quick_status_oneline() - One-line summary
- [x] format_quick_status_compact() - Issues only
- [x] format_quick_status_full() - Full display
- [x] is_quick_status_query() - Query detection

### Phase 61 - Repeated Questions Detection ✓ (v0.0.485)
- [x] repeated_questions.rs for tracking similar questions
- [x] RecordedQuestion struct with variants, count, timestamps
- [x] QuestionHistory for storing and querying
- [x] normalize_question() - Remove filler words
- [x] calculate_similarity() - Jaccard-like similarity
- [x] detect_category() - Topic categorization
- [x] record() with similarity grouping
- [x] get_repeated(), top_repeated(), by_category()
- [x] unresolved_repeated() - Pending questions
- [x] format_repeated_questions/compact() - Display
- [x] is_repeated_questions_query() - Query detection

### Phase 62 - Response Length Tracking ✓ (v0.0.486)
- [x] response_length.rs for tracking response lengths
- [x] RecordedResponse struct with char/word/line counts
- [x] ResponseLengthTracker for statistics
- [x] record() / record_with_category() - Track responses
- [x] average_chars() / average_words() - Averages
- [x] Longest/shortest by chars and words
- [x] Recent responses (last 10)
- [x] format_response_lengths() - Full display
- [x] format_response_lengths_compact() - Compact
- [x] response_length_fun_fact() - Fun facts
- [x] is_response_length_query() - Query detection

### Phase 63 - Resolution Time Tracking ✓ (v0.0.487)
- [x] resolution_time.rs for tracking resolution times
- [x] ResolutionRecord struct with timing and metadata
- [x] ResolutionTimeTracker for comprehensive stats
- [x] record() / record_simple() - Track resolutions
- [x] average_ms(), success_rate(), escalation_rate()
- [x] Fastest/slowest resolutions
- [x] Per-category stats (count, total, avg, range)
- [x] Recent resolutions (last 20)
- [x] format_resolution_times() - Full display
- [x] format_resolution_times_compact() - Compact
- [x] format_duration_ms() - Human-readable
- [x] resolution_time_fun_fact() - Fun facts

### Future
- All VISION.md features implemented!
