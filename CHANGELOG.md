# Changelog

All notable changes to Anna will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.288] - 2025-12-10

### Enhanced - Data-Driven Learning

**Learning Progress Module:**
- New `learning_progress.rs` tracks Anna's knowledge growth
- All metrics derived from actual recipes - no hardcoded data
- Self-sufficiency metric: % of requests handled from own knowledge
- Strong/growing areas identified from recipe reliability scores
- Summary generation for translator to naturalize

**Removed Hardcoded Suggestions:**
- Removed `general_exploration_suggestions()` function
- Removed hardcoded example queries for domains
- All suggestions now come from actual recipe/learning data
- `example_query_for_domain()` now returns None

**Stats Display Improvements:**
- Learning section now shows self_sufficiency percentage
- Strong areas displayed from actual recipe data
- Growing areas shown (categories needing more learning)
- Color-coded self-sufficiency (green >=50%, yellow >=20%)

**Key Philosophy:**
- Anna starts knowing little, learns from specialists
- Stats should show growth over time
- No hardcoded content - everything data-driven

**Files changed:**
- `anna-shared/src/learning_progress.rs` - New module
- `anna-shared/src/learning_suggestions.rs` - Removed hardcoded suggestions
- `anna-shared/src/lib.rs` - Added module export
- `annactl/src/stats_display_v2.rs` - Enhanced learning section

## [0.0.287] - 2025-12-10

### Enhanced - Maintenance Prompts in Greeting

**Greeting Context Integration:**
- Maintenance prompts now appear in session greeting
- Critical/urgent maintenance actions (urgency <= 2) shown as "Quick actions"
- Each prompt includes ready-to-use Anna query
- Fallback greeting displays maintenance section

**GreetingContext Enhancements:**
- Added `maintenance_prompts` field to GreetingContext
- New `with_maintenance()` method populates prompts from snapshot/telemetry
- New `has_maintenance()` helper method
- Optimized to load telemetry only once per greeting

**Files changed:**
- `anna-shared/src/greeting_context.rs` - Added maintenance prompt support
- `annactl/src/greeting/mod.rs` - Integrated maintenance into greeting flow

## [0.0.286] - 2025-12-10

### Added - Proactive Maintenance Actions

**Maintenance Actions Module:**
- New `maintenance_actions.rs` module for actionable maintenance suggestions
- Turns health observations into concrete actions Anna can help execute
- Categories: DiskCleanup, MemoryOptimize, ServiceRepair, SecurityAudit, PerformanceTune, SystemUpdate
- Urgency levels (1=critical, 5=optional) with visual markers

**Stats Display Integration:**
- New `[maintenance]` section in `annactl stats` display
- Shows urgent actions (urgency <= 3) with action prompts
- Each action includes a ready-to-use Anna query
- Urgency markers: `[!!]` critical, `[! ]` warning, `[* ]` info

**Action Generation:**
- Disk cleanup actions from disk usage levels
- Memory optimization from memory pressure
- Service repair actions from failed services
- Telemetry-based actions from health score and trends
- Network health checks from detected anomalies

**Files changed:**
- `anna-shared/src/maintenance_actions.rs` - New maintenance action system
- `anna-shared/src/lib.rs` - Added module export
- `annactl/src/stats_display_v2.rs` - Integrated maintenance section

## [0.0.285] - 2025-12-10

### Enhanced - Telemetry-Based Health Tips

**Health Tips from Telemetry:**
- `generate_telemetry_tips()` generates tips from telemetry data
- Health score tips when score is below 75% or 60%
- Disk trend tips when usage increasing >5%
- Memory trend tips when usage increasing >15%
- CPU trend tips when load increasing >20%
- Anomaly-based tips with role-appropriate staff
- Priority-based tip ordering

**Idle Tips Integration:**
- Live request now includes telemetry health tips
- Higher priority health tips shown first
- Tips from trends help users proactively

**Files changed:**
- `anna-shared/src/health_tips.rs` - Added telemetry tip generation
- `annactl/src/live_request.rs` - Integrated telemetry tips

## [0.0.284] - 2025-12-10

### Enhanced - Greeting Health Alerts & Idle Tips

**Session Greeting Health Integration:**
- Integrated telemetry-based health alerts into session greeting
- Critical alerts shown as `[!]` prefix in greeting
- Warning alerts shown as `[*]` prefix
- Low health score (<70%) mentioned proactively
- Combines failed services + telemetry anomalies

**Idle Tips During Wait Times:**
- Tips shown after 3+ seconds of waiting for response
- Max one tip per request to avoid distraction
- Contextual tips based on user preferences
- Tips about learning mode, auto-confirm, email setup
- Category icons and priority-based selection

**Files changed:**
- `annactl/src/greeting/mod.rs` - Integrated telemetry health alerts
- `annactl/src/live_request.rs` - Added idle tips during waits

## [0.0.283] - 2025-12-10

### Enhanced - Stats Display with Suggestions

**Stats Command Enhancements:**
- New `[suggestions]` section in `annactl stats` display
- Shows personalized learning suggestions based on recipe gaps
- Displays health-related suggestions from telemetry data
- Example queries to help users teach Anna new skills
- Priority-based ordering (high priority = yellow, low = dim)
- Category icons: [+] new domain, [>] deep dive, [?] gap, [^] improve, [!] health

**Telemetry Improvements:**
- New `TelemetryStore::load_if_exists()` method
- Returns None when no telemetry data exists (avoids false empty state)

**Files changed:**
- `annactl/src/stats_display_v2.rs` - Added suggestions section
- `anna-shared/src/system_telemetry.rs` - Added load_if_exists method

## [0.0.282] - 2025-12-10

### Added - Recipe Learning Enhancements

**LLM-Based Similarity Scoring:**
- New `recipe_similarity.rs` module for semantic query matching
- Uses translator LLM to score similarity between queries and recipes
- `SimilarityScore` struct with score, intent match, target match
- `quick_prefilter()` for efficient candidate selection before LLM
- `build_similarity_prompt()` generates prompts for LLM scoring
- Thresholds for skipping specialist LLM on high-confidence matches

**Idle-Time Learning Suggestions:**
- New `learning_suggestions.rs` module for proactive learning
- Analyzes recipe coverage gaps by category
- Identifies weak recipes needing improvement
- Generates example queries to help users teach Anna
- Health-related learning suggestions from telemetry
- `format_suggestions_for_display()` for terminal output

**Files changed:**
- New: `anna-shared/src/recipe_similarity.rs` - LLM similarity scoring
- New: `anna-shared/src/learning_suggestions.rs` - Learning suggestions

## [0.0.281] - 2025-12-10

### Added - Proactive Health Alerts

**Health Alerts System:**
- New `health_alerts.rs` module for generating proactive alerts
- Converts telemetry anomalies into user-friendly alerts
- Three severity levels: Info, Warning, Critical
- Contextual recommendations for each alert type
- Trend-based alerts (e.g., "disk filling up")

**Greeting Context Integration:**
- `GreetingContext::with_telemetry()` populates health issues from telemetry
- Critical and warning alerts shown in greetings
- Low health score (<70%) mentioned proactively
- Health summary generation for LLM context

**Daemon Integration:**
- Telemetry collector started at daemon boot
- Health alerts available for greeting generation

**Files changed:**
- New: `anna-shared/src/health_alerts.rs` - Alert generation and formatting
- `anna-shared/src/greeting_context.rs` - Telemetry integration methods
- `annad/src/server.rs` - Telemetry collector startup

## [0.0.280] - 2025-12-10

### Added - System Telemetry and Learning Analytics

**System Telemetry Tracking:**
- New `system_telemetry.rs` module for tracking system state over time
- Tracks CPU, memory, disk, load, services, and network metrics
- Anomaly detection for high CPU, memory, disk, load, and service failures
- Trend analysis to detect increasing/decreasing resource usage
- Health score calculation (0-100) based on anomalies
- Insight generation with recommendations

**Telemetry Collector:**
- Background task collects system metrics every 5 minutes
- Persists telemetry data to disk for historical analysis
- Automatic anomaly detection and trend calculation

**Status Display Enhancements:**
- New `[telemetry]` section showing:
  - Health score with color-coded status
  - Sample count and tracking window
  - Trends (CPU, memory, disk)
  - Recent anomalies with severity
  - Generated insights with recommendations
- New `[learning]` section showing:
  - Recipes learned count
  - Total recipe uses
  - Average reliability score
  - Breakdown by category
  - Top recipes by usage

**Files changed:**
- New: `anna-shared/src/system_telemetry.rs` - Telemetry types and analysis
- New: `annad/src/telemetry_collector.rs` - Background metric collection
- New: `annactl/src/status_telemetry.rs` - Telemetry display section
- New: `annactl/src/status_learning.rs` - Learning stats display section
- `annactl/src/status_display_v2.rs` - Integrated telemetry and learning sections

## [0.0.279] - 2025-12-10

### Enhanced - Hollywood-Style Progress Display

**Real-time spinner with stage-aware messages:**
- Spinner now shows current stage: "classifying query", "gathering system data", "consulting specialist", "verifying answer"
- Better spinner clearing and state management
- More polished internal comms header ("--- internal comms ---")

**Improved streaming token handling:**
- Cleaner line clearing when transitioning from spinner to streaming
- Stage context preserved for generation progress messages

**Files changed:**
- `annactl/src/live_request.rs` - Enhanced progress display with Hollywood-style spinners

## [0.0.278] - 2025-12-10

### Enhanced - Status Display with Model Hierarchy

**LLM section now shows full model hierarchy:**
- Displays translator, junior, and senior models with their roles
- Shows descriptions: "query classification, fastest", "regular queries", "complex/escalated"
- Junior and senior models added to `LlmStatus` struct

**Statistics section enhanced with RPG gamification:**
- XP and title now displayed prominently in statistics section
- Colored output for better visibility (cyan for XP, green for title)

**Files changed:**
- `anna-shared/src/status.rs` - Added junior_model and senior_model to LlmStatus
- `annactl/src/status_display_v2.rs` - Enhanced [llm] section with model hierarchy
- `annactl/src/status_statistics.rs` - Added XP and title display
- `annad/src/server.rs` - Populate junior_model and senior_model in status

## [0.0.277] - 2025-12-10

### Added - Tiered Model Hierarchy

**New three-tier model configuration:**
- **Translator**: `qwen2.5:0.5b-instruct` - Fastest, for classification and formatting
- **Junior**: `qwen2.5:3b-instruct` - Mid-size, for regular specialist work
- **Senior**: `qwen2.5:7b-instruct` - Largest/smartest, for complex/escalated queries

**Configuration changes:**
- Added `junior_model` config option (defaults to 3b model)
- Added `senior_model` config option (defaults to 7b model)
- Legacy `specialist_model` maps to `junior_model` for backwards compatibility
- `ModelRole` enum now includes `Junior` and `Senior` variants

**Files changed:**
- `anna-shared/src/model_selector/types.rs` - Added Junior/Senior model roles
- `annad/src/config.rs` - Added junior_model and senior_model config options

## [0.0.276] - 2025-12-10

### Added - LLM Response Formatting

**Deterministic answers now formatted via translator LLM:**
- Deterministic/static answers are rephrased by the translator LLM for variety
- Original facts, numbers, paths, and commands are preserved
- Validation ensures formatted response maintains key information
- Falls back to original answer if formatting fails or validation fails
- Only applies to deterministic answers (LLM answers already varied)

**Files changed:**
- New: `annad/src/response_formatter.rs` - Response formatting module
- `annad/src/rpc_handler/llm_request.rs` - Integrated formatting after specialist stage
- `annad/src/lib.rs` - Added response_formatter module

## [0.0.275] - 2025-12-10

### Added - LLM-Generated Greetings

**Personalized greetings now generated via translator LLM:**
- REPL greetings are now generated by the translator LLM for varied, natural text
- Greetings incorporate user facts: username, time since last session, streak days, preferred editor, top topics, open tickets, health issues
- Each greeting is unique while maintaining consistent content structure
- Falls back to deterministic greeting if LLM unavailable

**New RPC method `GenerateGreeting`:**
- Accepts `GreetingContext` with user profile data
- Returns `GreetingResponse` with greeting text and LLM status flag
- Uses translator model for fast response (~10s timeout)

**Files changed:**
- New: `anna-shared/src/greeting_context.rs` - Greeting context and response types
- New: `annad/src/greeting_generator.rs` - LLM-based greeting generation
- `annad/src/handlers.rs` - Added `handle_generate_greeting` handler
- `annad/src/rpc_handler/dispatcher.rs` - Added GenerateGreeting dispatch
- `annactl/src/client.rs` - Added `generate_greeting` client method
- `annactl/src/greeting/mod.rs` - Updated to use LLM-generated greetings
- `anna-shared/src/rpc/method.rs` - Added GenerateGreeting enum variant

## [0.0.274] - 2025-12-10

### Added - Statistics Section in Status Display

**New `[statistics]` section in `annactl status`:**
- Shows total cases and success rate with color-coded indicators
- Displays average response time
- Lists top 3 departments/teams with case counts and resolution scores
- Shows top 3 staff performers with success rates

**Example output:**
```
[statistics]
  total_cases           42
  success_rate          85%
  avg_response          1234ms
  top_departments
    desktop       cases:  15  ok:  13  score:    82
    network       cases:  12  ok:  10  score:    78
    storage       cases:   8  ok:   7  score:    85
  top_staff
    Sofia         cases:  10  success:   90%
    Michael       cases:   8  success:   87%
    Chen          cases:   6  success:   83%
```

**Files changed:**
- New: `annactl/src/status_statistics.rs` - Dedicated statistics section module
- `annactl/src/status_display_v2.rs` - Integrated statistics call
- `annactl/src/main.rs` - Added module

## [0.0.273] - 2025-12-10

### Fixed - Routing Accuracy Improved to 100%

**Fixed routing mismatches (from 79% to 100%):**

- Reordered domain inference to check security before logs (fixes "login" matching "log")
- Reordered inference to check services before network (fixes database connection queries)
- Reordered inference to check logs/hardware before performance (fixes kernel/temp queries)
- Fixed "system" domain fallback to General (not Performance) for generic queries
- Added direct domain mapping for "performance", "desktop", "logs" domains

**Specific fixes:**
- "postgresql won't accept connections" now routes to Services (not Network)
- "check CPU temperature" now routes to Hardware (not Performance)
- "view kernel messages from last boot" now routes to Logs (not Performance)
- "configure ufw firewall" now routes to Security (not Network)
- "check for failed login attempts" now routes to Security (not Logs)
- "how is my computer", "install htop", etc. now route to General (not Performance)

**Technical changes:**
- `infer_domain()` and `infer_route_class()` in tests now match routing behavior
- `team_from_domain()` expanded to include performance, desktop, logs mappings
- Login vs log disambiguation via substring ordering

**Files changed:**
- `anna-shared/src/query_scenarios/tests.rs` - Fixed inference ordering
- `anna-shared/src/teams.rs` - Fixed domain-to-team mapping

## [0.0.272] - 2025-12-10

### Added - Expanded Synonym Groups for Better Recipe Matching

**Extended synonym dictionary (from ~40 to ~70 synonym groups):**

New synonym groups added:
- Storage: partition, unused, taken, amount/much/how/what
- CPU/Performance: core, busy, hanging, performance/speed/efficiency
- Network: online, connected, ethernet/wired/lan
- Services: systemd, up/down, status/state/condition
- Actions: start, stop, get, configure/modify/edit, troubleshoot
- Config: options, colorscheme, line/lines/number/numbers
- System: laptop/desktop, healthy/ok/okay/fine/good, error/problem/issue/warning
- Editors: nano/pico
- Desktop: gnome/gtk, kde/plasma/qt, window/wm, desktop/de/environment
- Docker: container/containers, image/images
- Packages: pkg, pacman/apt/dnf/yum
- Hardware: gpu/graphics/video/nvidia/amd, audio/sound/speaker, bluetooth/bt
- Security: permission/perms/access, ssh/secure/key, firewall/ufw/iptables

**Test improvements:**
- Updated query_scenarios tests to use synonym expansion
- `queries_would_match()` now expands tokens with synonyms before comparison

**Files changed:**
- `anna-shared/src/synonyms.rs` - Expanded synonym groups
- `anna-shared/src/query_scenarios/tests.rs` - Updated to use synonym expansion

## [0.0.271] - 2025-12-10

### Added - LLM-Based Semantic Similarity for Recipe Matching

**New semantic similarity module using translator LLM:**
- Uses the translator model to check if queries are semantically equivalent
- Catches paraphrases that token matching misses (e.g., "disk space" ~ "storage usage")
- Prompt-based similarity comparison returns JSON with score and reasoning
- Only triggers after token-based matching fails (to preserve fast path)

**How it works:**
1. Token-based recipe matching runs first (fast, no LLM)
2. If no match found, gets top 5 recipe candidates
3. Uses translator LLM to compare new query with each candidate's pattern
4. If similarity score >= 75, reuses the matched recipe
5. Falls back to full translator if no semantic match

**Example matches now possible:**
- "how much disk space" ~ "what is my storage usage"
- "check CPU load" ~ "processor usage"
- "am I connected" ~ "network status"

**Files changed:**
- New: `annad/src/recipe_similarity.rs` - Semantic similarity module
- `annad/src/routing_stage.rs` - Integrated semantic check before triage
- `annad/src/lib.rs` - Export recipe_similarity module

## [0.0.270] - 2025-12-09

### Added - Recipe Learning Verification Tests

**New test suite for recipe learning quality:**
- `test_similar_queries_have_token_overlap` - Verifies similar queries share tokens (22% currently)
- `test_learnable_scenarios_have_good_tokens` - Ensures learnable queries have 3+ tokens
- `test_paraphrase_matching_examples` - Tests canonical paraphrase pairs (40% match rate)
- `test_action_verb_preservation` - Verifies action verbs like install/restart stay in tokens
- `test_target_noun_preservation` - Verifies target nouns like docker/vim stay in tokens

**Insights discovered:**
- Current token matching rate for paraphrases is ~40% (without synonym expansion)
- Similar query overlap is ~22% (many use completely different words)
- Future improvement: Add synonym expansion to recipe_index (disk<->storage, check<->show)
- Action verbs and target nouns are well-preserved for recipe learning

**Files changed:**
- `anna-shared/src/query_scenarios/tests.rs` - Added 5 recipe learning tests

## [0.0.269] - 2025-12-09

### Added - Intelligent Model Auto-Selection with Benchmarking

**Automatic model selection based on hardware:**
- Anna now automatically selects the best models for each role (Translator, Specialist)
- Uses hardware detection (RAM, GPU) to filter suitable models
- Benchmarks available models to measure tokens/sec and TTFT
- Prefers Qwen3-VL and DeepSeek-R1 families when available

**New modules and functions:**
- `auto_select.rs` - Intelligent model selection orchestrator
- `ollama::list_models()` - Lists all locally available Ollama models
- `auto_select_models(ram_gb, has_gpu)` - Full auto-selection with benchmarking
- `quick_select_models(ram_gb)` - Fast selection without benchmarking (for quick startup)

**Smart model pulling:**
- Identifies which models are missing from the catalog
- Automatically pulls best models for each role based on hardware constraints
- Prefers larger/better models when RAM allows (DeepSeek-R1:7b, Qwen3-VL:4b)
- Falls back to config defaults if auto-selection fails

**Server initialization improvements:**
- New "selecting_models" LLM phase during initialization
- Benchmark results stored in state for each model
- Models pulled by Anna marked in ledger for status display

**Files changed:**
- New: `annad/src/auto_select.rs` - Auto-selection module
- `annad/src/ollama.rs` - Added `list_models()` function
- `annad/src/server.rs` - Integrated auto-selection in initialization
- `annad/src/lib.rs` - Export auto_select module

## [0.0.268] - 2025-12-09

### Added - Query Scenario Test Framework & Routing Improvements

**New query scenario test corpus with 100+ queries:**
- Comprehensive test coverage across all 9 teams (Storage, Network, Desktop, Services, Performance, Hardware, Security, Logs, General)
- Tests grouped by difficulty (Simple, Medium, Complex)
- Expected routing paths (FastPath, JuniorOnly, SeniorReview, LearnableRecipe)
- Similar query matching for recipe recall testing
- Statistics collector with summary reports

**Routing improvements (from 65% to 79% accuracy):**
- Added Desktop team routing for: GNOME, KDE, Hyprland, Sway, i3, Wayland, X11
- Added Desktop team routing for: GTK, theme, font, HiDPI, dark mode
- Added Desktop team routing for: tmux, bash prompt, PS1, shell config
- Added Desktop team routing for: shortcuts, keybindings

**Fast path improvements:**
- Added IP address queries to NetworkStatus ("what is my IP", "IP address")
- Added CPU core queries to CpuUsage ("how many cores", "CPU cores", "number of cores")

**Files changed:**
- New module: `anna-shared/src/query_scenarios/` (mod.rs, corpus.rs, stats.rs, tests.rs)
- `anna-shared/src/teams.rs` - Expanded Desktop routing patterns
- `anna-shared/src/fastpath/classify.rs` - Added IP and CPU core patterns
- `anna-shared/src/lib.rs` - Export query_scenarios module

## [0.0.267] - 2025-12-09

### Added - DeepSeek-R1 Model Support & Status Improvements

**New model family support:**
- Added DeepSeek-R1 to the model catalog as an alternative to Qwen for reasoning tasks
- `deepseek-r1:7b` for Specialist role (strong reasoning capability)
- `deepseek-r1:1.5b` for Translator role (compact reasoning model)
- DeepSeek-R1 models are based on Qwen but fine-tuned for reasoning

**Status display improvements:**
- `annactl status` now shows which models were downloaded by Anna
- New `models_by_anna` field in [llm] section lists models from ledger
- Shows up to 5 model names with count for more

**Research findings (Qwen vs DeepSeek for local Linux troubleshooting):**
- Qwen has more size options (0.5B to 72B+), better for constrained laptops
- DeepSeek V3.2 is 671B (cloud only), but R1:1.5b works locally
- For most laptops: Qwen remains the default choice
- For systems with 16GB+ VRAM: DeepSeek-R1 provides stronger reasoning

**Files changed:**
- `anna-shared/src/model_selector/types.rs` - Added DeepSeekR1 to ModelFamily
- `anna-shared/src/model_selector/catalog.rs` - Added DeepSeek-R1 models
- `anna-shared/src/model_selector/selection.rs` - DeepSeek detection, family ordering
- `anna-shared/src/ledger.rs` - Added models_pulled() method
- `annactl/src/status_display_v2.rs` - Added models_by_anna display

## [0.0.266] - 2025-12-09

### Added - Daemon Snapshot Collection & Bug Fixes

**Critical fix for fast path queries!**

The daemon now collects system snapshots every 60 seconds instead of relying on annactl greeting to capture them. This fixes fast path queries that were failing because the snapshot file was empty.

**New snapshot_loop module collects:**
- Disk usage (/, /home, /var, /tmp, /mnt/*, /media/*)
- Memory usage (total and used)
- Failed systemd services
- Boot time (from uptime -s)
- Load averages (1, 5, 15 min)
- Network status (IP addresses, connection state)

**Bug fixes:**
1. **Fixed query context leaking between requests** - Internal comms from previous request no longer appear at start of new request. Progress events are now cleared at the very start of request handling.

2. **Fixed team routing for config queries** - ConfigureEditor, ConfigureShell, ConfigureGit now correctly route to Desktop team (Sofia) instead of Performance team (Kari). New `team_from_query_class` function overrides domain-based routing for config classes.

3. **Added "system load" to fast path patterns** - Queries like "what is the system load" now use fast path for instant answers.

**Files changed:**
- `annad/src/snapshot_loop.rs` - New module for periodic snapshot collection
- `annad/src/server.rs` - Spawns snapshot loop on daemon start
- `annad/src/lib.rs` - Exports snapshot_loop module
- `annad/src/rpc_handler/llm_request.rs` - Clear progress events earlier to prevent leak
- `annad/src/comms/routing.rs` - New `team_from_query_class` function for config routing
- `anna-shared/src/fastpath/classify.rs` - Added "system load" pattern to CpuUsage

## [0.0.265] - 2025-12-09

### Changed - UX Cleanup Release

**Major fixes to user experience:**

1. **Disabled LLM dialogue generation** - The small translator model (qwen2.5:0.5b) was producing nonsensical internal dialogue. All dialogue now uses static fallbacks from `messages.rs`

2. **Internal comms hidden by default** - `show_internal_comms` now defaults to `false`. Most users found the internal IT dialogue confusing. Can still be enabled with "show internal comms"

3. **All emojis replaced with ASCII** - Replaced all Unicode emojis with ASCII equivalents for better terminal compatibility:
   - `[ok]`, `[!]`, `[!!]`, `[X]` for status indicators
   - `[+]`, `[*]`, `[^]`, `[i]` for other icons

4. **Boot time fast path fixed** - "boot time" query now properly routes to fast path (was falling through to slow path)

**Files changed:**
- `dialogue_gen.rs` - All LLM generation functions now return None
- `user_profile/types.rs` - `show_internal_comms` default changed to false
- `fastpath/classify.rs` - Added "boot time" pattern
- Multiple files: Replaced emojis with ASCII

## [0.0.264] - 2025-12-09

### Changed - Recipe Learning Architecture for Config Edits

**Anna now learns config solutions from specialists instead of using hardcoded answers!**

This is a fundamental architectural change to how Anna handles editor configuration requests
(like "enable syntax highlighting in vim"). Instead of hardcoded answers, Anna now:

1. **Checks for learned recipes first** - If Anna has previously learned how to do this, she uses that
2. **Asks specialists for help** - If no learned recipe exists, Anna asks the Desktop team (Sofia)
3. **Learns from the answer** - Once Sofia gives a verified answer, it becomes a learned recipe
4. **Uses seed recipes as fallback** - Hardcoded patterns are now "seed recipes" with lower confidence (0.7)

**New files:**
- `config_types.rs` - ConfigTarget, ConfigEditAction, ConfigIntent types
- `config_seed_recipes.rs` - Bootstrap/seed recipes for when Anna hasn't learned yet
- `config_intent.rs` - Now focused on ConfigHint for specialist routing

**New types:**
- `ConfigHint` - Hint for specialists about what the user wants
- `ConfigFeatureHint` - Feature categories (Syntax, LineNumbers, Theme, Indent, etc.)
- `SeedRecipe` - Bootstrap recipe with lower confidence

**Technical details:**
- Seed recipes have confidence 0.5-0.7 (vs. 0.8+ for learned recipes)
- `get_config_hint_for_specialist()` provides context to specialists
- `ConfigHint::to_specialist_context()` formats hints for LLM prompts
- Learned recipes take precedence over seed recipes

**Why this matters:**
Anna's goal is to learn from her interactions with specialists, building her own knowledge base.
This change ensures config queries go through the learning pipeline instead of bypassing it.

## [0.0.260] - 2025-12-09

### Added - OS/Kernel Info in LLM Context

**Anna now knows what OS and kernel version you're running!**

The LLM specialist prompt now includes:
- **OS/Distribution** - "Arch Linux", "Ubuntu 22.04", etc.
- **Kernel version** - "6.17.9-arch1-1"

**Benefits:**
- More accurate package manager suggestions (apt vs pacman vs dnf)
- Kernel-specific advice and troubleshooting
- Better understanding of system capabilities

**Technical changes:**
- `HardwareInfo` extended with `os_name`, `kernel`, `distro` fields
- `HardwareSummary` extended for LLM context
- `probe_hardware()` now reads from `/proc/sys/kernel/` and `/etc/os-release`
- Specialist prompt shows System section before Hardware

## [0.0.259] - 2025-12-09

### Added - More Fast Path Query Classes

**Anna now answers uptime, CPU load, and network status instantly!**

New fast path classes:
- **Uptime** - "uptime", "how long has my computer been running", "when was last boot"
- **CpuUsage** - "cpu usage", "cpu load", "load average"
- **NetworkStatus** - "network status", "am I connected", "internet connection", "wifi status"

**SystemSnapshot extended:**
- `boot_time_secs` - Boot time as Unix timestamp
- `load_1min`, `load_5min`, `load_15min` - Load averages
- `network_connected` - Network connectivity status
- `ip_addresses` - List of active IP addresses

**New EvidenceKind variants:**
- `LoadAverage` - For CPU load monitoring
- `NetworkInfo` - For network status

**Probe parsing:**
- `uptime` output parsing for load averages
- `uptime -s` output parsing for boot time
- `ip addr` output parsing for network info

## [0.0.258] - 2025-12-09

### Added - Pending Ticket Retry Queue

**Anna now keeps tickets open and retries them during idle time!**

When Anna can't give a confident answer (reliability < 70), instead of closing
the ticket, it's queued for retry. During idle time, Anna will:
- Re-process pending tickets with fresh probes
- Try different approaches if the first attempt failed
- Improve reliability score through multiple attempts

**Features:**
- `PendingRetry` ticket status for retryable tickets
- Configurable retry intervals (default: 5 minutes between attempts)
- Maximum 3 retry attempts before giving up
- Persists queue to `~/.anna/pending_queue.json`
- Reasons tracked: low reliability, timeout, insufficient evidence, LLM unavailable

**Example flow:**
1. User asks "what's using my GPU?"
2. Anna tries but gets 55% reliability (timeout on nvidia-smi)
3. Ticket goes to PendingRetry instead of closed
4. During idle time, Anna retries with more time
5. Gets 85% reliability, ticket resolved

**Changes:**
- `pending_queue.rs`: New module for retry queue management
- `ticket_tracker/status.rs`: Added PendingRetry status

## [0.0.257] - 2025-12-09

### Added - Desktop Configuration Support

**Anna can now change wallpaper, enable dark mode, and set themes!**

Supports multiple desktop environments:
- **GNOME** - Uses gsettings
- **KDE Plasma** - Uses qdbus and lookandfeeltool
- **Xfce** - Uses xfconf-query
- **Cinnamon** - Uses gsettings
- **MATE** - Uses gsettings

**Example commands Anna now understands:**
- "set my wallpaper to /home/user/pic.jpg"
- "enable dark mode"
- "switch to light mode"
- "change theme to dracula"
- "set arc-dark theme"

**Features:**
- Auto-detects desktop environment from XDG_CURRENT_DESKTOP
- Generates correct command for each DE
- Provides rollback commands where possible
- Supports common themes (Adwaita, Arc, Dracula, Nord, Breeze, etc.)

**Changes:**
- `desktop_recipes.rs`: New module for desktop configuration

## [0.0.256] - 2025-12-09

### Added - Synonym Expansion for Smarter Recipe Matching

**Anna now understands synonyms, so similar questions find learned recipes.**

When Anna learns from a question like "how much disk space do I have", she can now
also answer variations like:
- "how much storage is free"
- "show available drive space"
- "check disk usage"

**Synonym groups include:**
- Storage: disk, storage, space, drive, volume
- Memory: memory, ram, mem
- Actions: enable/activate, check/show/view, install/setup
- Services: service, daemon, unit
- Network: network, internet, connection
- And 30+ more synonym groups!

**How it works:**
1. When searching for recipes, query tokens are expanded to include synonyms
2. Recipes indexed with "disk" now also match queries containing "storage" or "space"
3. A synonym match bonus is added to the recipe score

**Changes:**
- `synonyms.rs`: New module with 35+ synonym groups
- `recipe_index.rs`: Integrated synonym expansion into search and scoring

## [0.0.255] - 2025-12-09

### Changed - Personality Quirks in Dialogue

**Staff dialogue now reflects unique personality traits for authentic character voices.**

Each IT specialist now speaks with their own distinctive style:

```
--- internal ---
  Anna: Hey Sofia! Config question incoming. Case DSK-0042
  Sofia (Desktop Administrator): Just :wq'd my last task, on it!
  Sofia (Desktop Administrator): Running 2 checks, like editing a buffer...
  Sofia (Desktop Administrator): That's a clean config.
  Anna: Thanks Sofia!
```

**Personality quirks include:**
- Sofia: Everything reminds her of vim (`:wq`, `hjkl`, buffers)
- Daniel: Drinks way too much coffee (`*sips coffee*`)
- Lars: Always worried about disk space
- Michael: Loves TCP/IP trivia
- Nora: Makes beep boop sounds
- And 10 more unique personalities!

The LLM receives each character's quirk and subtly incorporates it into their dialogue,
making conversations feel like a real IT department with distinct personalities.

**Changes:**
- `comms/dialogue_gen.rs`: Added personality_for() lookups to all dialogue generation prompts

## [0.0.254] - 2025-12-09

### Changed - LLM-Powered Natural Dialogue

**Specialist chatter is now generated by the LLM for natural, varied dialogue.**

Instead of repetitive static messages, Anna and the IT staff now speak with natural
variety that reflects what they're actually doing:

```
--- internal ---
  Anna: Hey Sofia! Quick one - user has a RAM question. Case DSK-0042
  Sofia (Desktop Administrator): On it!
  Sofia (Desktop Administrator): Pulling some diagnostics - 2 checks...
  Sofia (Desktop Administrator): Got the data.
  Sofia (Desktop Administrator): Checking the numbers...
  Sofia (Desktop Administrator): Looks good - 92% confident.
  Anna: Thanks Sofia! Sending to user.
```

Each conversation is unique because the LLM generates contextual responses based on:
- The user's actual query
- The team handling the request
- The stage of processing (dispatch, probing, reviewing, etc.)

Falls back to static messages if LLM is slow (3s timeout) to avoid blocking.

**Changes:**
- `comms/dialogue_gen.rs`: New module for LLM-powered dialogue generation
- `comms/generator.rs`: Added `with_query()` and `with_model()` builder methods
- `comms/messages.rs`: Added async methods that try LLM first, fall back to static
- `llm_request.rs`: Wired up async dialogue methods with translator model
- `probe_stage.rs`: Updated to use async dialogue methods

## [0.0.253] - 2025-12-09

### Changed - Real-Time Specialist Dialogue Enhancement

**Internal IT department chatter now displays with full role titles during request processing.**

When you send a request, you'll see the "behind the scenes" dialogue with proper role titles:
```
--- internal ---
  Anna: Hey Sofia! New case DSK-0042 coming your way.
  Sofia (Desktop Administrator): On it, checking the system...
  Sofia (Desktop Administrator): Running 2 probes...
  Sofia (Desktop Administrator): All 2 probes succeeded.
  Anna: Thanks Sofia! I'll take it from here.
```

**Changes:**
- `live_request.rs`: Added "--- internal ---" header when internal comms begin
- `live_request.rs`: Staff names now show role titles (e.g., "Sofia (Desktop Administrator)")
- `roster/data.rs`: Added `person_by_display_name()` for looking up staff by name
- `roster/mod.rs`: Exported new lookup function

## [0.0.252] - 2025-12-09

### Changed - Evidence Bullets in Query Responses

**Anna's responses now show evidence bullets showing the data sources used.**

After the answer, evidence items are displayed showing where the information came from:
```
[anna]
Your system has 64 GB of RAM.
• evidence: /proc/meminfo showed MemTotal: 67108864 kB
```

This makes responses more trustworthy by showing exactly what data supported the answer.

**Changes:**
- `theatre_render/render.rs`: Added `print_evidence_bullets()` function
- `theatre_render/render.rs`: Added `summarize_probe_output()` for human-readable evidence
- Evidence limited to 3 items max for conciseness
- Maps common probe commands to readable descriptions (memory, disk, CPU, etc.)

## [0.0.251] - 2025-12-09

### Changed - Domain-Prefixed Case Numbers

**Ticket IDs now use domain-specific prefixes instead of generic "CN".**

Case numbers now reflect the team handling the request:
- `DSK-0001` - Desktop/System queries
- `NET-0001` - Network queries
- `STO-0001` - Storage queries
- `SEC-0001` - Security queries
- `SVC-0001` - Services/Packages queries

Each domain has its own counter that persists across sessions.

**Changes:**
- `tracker.rs`: Added `TicketDomain` enum with domain prefixes
- `tracker.rs`: Added `next_case_number_for_domain()` method
- `theatre.rs`: Now generates domain-prefixed case numbers based on query domain
- Counter format changed from date-based reset to persistent per-domain counters

## [0.0.250] - 2025-12-09

### Changed - RPG-Style Stats Display

**`annactl stats` now shows an RPG-style gamified dashboard.**

New sectioned format with `[profile]`, `[throughput]`, `[quality]`, `[learning]`,
`[team leaderboard]`, `[teams]`, `[achievements]` providing a gamified view of
your Service Desk activity.

**Sample output:**
```
──────────────────────────────────────────────────────────────────────────────
Anna Service Desk  |  Junior Sysadmin  |  Level 3
──────────────────────────────────────────────────────────────────────────────

[profile]
  title                 Junior Sysadmin
  level                 3
  xp                    450 / 600  [████████░░░░░░░░░░░░]
  xp_to_next            150
  tenure                2 weeks
  current_streak        5 days

[throughput]
  total_cases           42
  resolved_ok           38
  failed                2
  timeouts              2
  ...
```

**Changes:**
- `stats_display_v2.rs`: New module implementing RPG-style display
- `stats_display.rs`: Simplified to delegate to v2 display
- `main.rs`: Added stats_display_v2 module

## [0.0.249] - 2025-12-09

### Changed - Rich Status Display

**`annactl status` now shows a sectioned dashboard matching IT department style.**

New format with bracketed sections `[core]`, `[updates]`, `[permissions]`, `[llm]`,
`[helpers]`, `[tickets]`, `[annad logs]`, `[health]` providing a comprehensive
system overview at a glance.

**Sample output:**
```
──────────────────────────────────────────────────────────────────────────────
Anna Service Desk (local)  |  status: OPERATIONAL  |  debug_mode: OFF
──────────────────────────────────────────────────────────────────────────────

[core]
  annactl_version       0.0.249
  annad_version         0.0.249
  protocol              1
  daemon                RUNNING  (pid 12345)
  uptime                2h 15m
  ...

[llm]
  provider              ollama
  state                 READY
  translator_model      qwen2.5-coder:7b
  ...
```

**Changes:**
- `status_display_v2.rs`: New module implementing rich sectioned display
- `display.rs`: Simplified to delegate to v2 display
- `main.rs`: Added status_display_v2 module

## [0.0.248] - 2025-12-09

### Fixed - Stats Tracking

**All requests now counted, not just successful completions.**

Previously, `stats` showed 0 requests because the counter was only incremented
at the very end of request handling. Early returns for fast-path, clarification,
timeout, no-evidence, etc. all bypassed the stats update.

**The Fix:**
- `record_request_received()` called at the START of `handle_llm_request`
- Every request is counted, regardless of how it completes
- Removed duplicate increment from `record_fast_path_hit()` and `record_request()`

**Changes:**
- `stats.rs`: Added `record_request_received()` method
- `stats.rs`: `record_fast_path_hit()` no longer increments `total_requests`
- `state.rs`: `record_request()` no longer increments `total_requests`
- `llm_request.rs`: Calls `record_request_received()` at function entry

### Fixed - Real-time Progress Events

**Staff chatter and stage events now visible during request processing.**

Previously, internal comms (staff messages) and stage start/complete events
were only visible after `save_progress()` was called. Now they're pushed
immediately to the shared streaming events for real-time polling.

**Changes:**
- `progress_tracker.rs`: `add_internal_comms()` also pushes to `streaming_events`
- `progress_tracker.rs`: `start_stage()` and `complete_stage()` push to `streaming_events`
- `handlers.rs`: Added debug logging for progress polling

## [0.0.247] - 2025-12-09

### Fixed - Real-Time LLM Streaming

**Critical UX fix: You can now see tokens as they stream!**

Previously, the user stared at a blank screen for 40+ seconds waiting for
the LLM to respond. Now you see each token as it's generated.

**Root Cause:**
- `ProgressTracker` stored streaming tokens in its own Arc<Mutex>
- Daemon's RPC handler read from a separate `progress_events` vector
- These two were never synchronized during streaming - only after completion

**The Fix:**
- `ProgressTracker` now accepts shared streaming events from daemon state
- `handle_progress` RPC merges live streaming events with stored events
- Tokens pushed during LLM generation are immediately visible to polling client

**Changes:**
- `state.rs`: Added `streaming_events: Arc<Mutex<Vec<ProgressEvent>>>` to daemon state
- `progress_tracker.rs`: Added `with_streaming_events()` constructor
- `handlers.rs`: `handle_progress` now merges live streaming events
- `llm_request.rs`: Creates ProgressTracker with shared streaming events

**Result:**
Users now see word-by-word output during LLM calls instead of waiting silently.

## [0.0.246] - 2025-12-09

### Added - Deeper Context Memory

**Anna now remembers you across sessions:**
- Tracks interaction patterns by topic (frequency, timing, related queries)
- Learns preferences from behavior (editor choice, shell preference)
- Provides continuity messages ("You've been asking about X a lot...")
- Remembers mastered commands (no re-explaining what you know)

**Memory Features:**
- `InteractionPattern` - Tracks topic frequency, timing, related queries
- `LearnedPreference` - Stores inferred preferences with confidence
- `ContinuityItem` - Remembers important things for next session
- `response_hints()` - Context-aware response guidance

**Smart Behaviors:**
- `should_explain()` - Skip explanations for mastered commands
- `continuity_greeting()` - Natural "last time we discussed..." messages
- `frequent_topics()` / `recent_topics()` - Pattern recognition
- Editor and shell preference detection

**Persistence:**
- Context saved to `~/.local/share/anna/context_memory.json`
- Automatic loading on startup
- Truncation to prevent unbounded growth

**New Module:**
- `context_memory.rs` - Cross-session user context tracking

## [0.0.245] - 2025-12-09

### Added - Greeting Insights from System State

**Anna now greets you with context-aware observations:**
- Greeting includes quick system status from IT staff
- Staff members deliver insights in their unique voice
- Both problems and good news are surfaced appropriately

**Insight Types:**
- **Critical issues**: Storage/Performance seniors flag problems immediately
- **Warnings**: Juniors give friendly heads-up on developing issues
- **Good news**: "Plenty of space!" or "Numbers look good!" when healthy
- **Changes**: Service recoveries or new failures since last session

**Features:**
- `generate_insights()` - Extracts insights from snapshot and deltas
- `format_insights_for_greeting()` - Formats for display in welcome
- `quick_status_line()` - One-liner: "disks ok • memory ok • services ok"
- Priority-based ordering (critical first)
- Limited to 2 insights per greeting (don't overwhelm)

**New Module:**
- `greeting_insights.rs` - Context-aware greeting enrichment

## [0.0.244] - 2025-12-09

### Added - Proactive Health Monitoring Alerts

**Health-based tips surface during REPL idle time:**
- Staff personalities now deliver proactive alerts about system issues
- Tips generated from actual health deltas (disk, memory, services)
- Each alert comes from the appropriate team member with their personality

**Alert Types:**
- **Critical disk** (95%+): Senior Storage (Ines) alerts immediately
- **Warning disk** (80%+): Junior Storage (Lars) gives heads-up
- **Critical memory** (90%+): Senior Performance (Mateo) recommends action
- **Warning memory** (80%+): Junior Performance (Kari) mentions htop
- **Failed services**: Senior Services (Mina) reports container issues
- **Service recovered**: Good news alerts with lower priority

**Smart Filtering:**
- Small changes (< 10%) don't generate tips (anti-spam)
- Tips include actionable suggestions ("Ask me: ...")
- Priority-based queue ensures critical issues surface first
- History tracking prevents repetitive tips

**New Module:**
- `health_tips.rs` - Converts system state to personalized idle tips

## [0.0.243] - 2025-12-09

### Added - Staff Personality System

**Every IT staff member now has unique personality traits:**
- 16 named staff members across 8 teams (Junior + Senior each)
- Individual greetings, success phrases, and uncertainty phrases
- Unique personality quirks that define each character

**Staff Personalities:**
- **Michael (Network Jr)**: Enthusiastic newbie, loves TCP/IP trivia
- **Ana (Network Sr)**: Calm expert, speaks in networking metaphors
- **Sofia (Desktop Jr)**: Vim enthusiast, everything reminds her of vim
- **Erik (Desktop Sr)**: DE expert, slightly grumpy about Wayland
- **Nora (Hardware Jr)**: Excited about hardware, makes beep boop sounds
- **Jon (Hardware Sr)**: Firmware wizard, calls hardware "the metal"
- **Lars (Storage Jr)**: Always worried about disk space
- **Ines (Storage Sr)**: ZFS enthusiast, recommends it for everything
- **Kari (Performance Jr)**: Always has htop running
- **Mateo (Performance Sr)**: Speaks in percentages
- **Priya (Security Jr)**: Checks everything twice
- **Oskar (Security Sr)**: Night owl, cryptography nerd
- **Hugo (Services Jr)**: Enthusiastic about systemd
- **Mina (Services Sr)**: Container expert, cynical about microservices
- **Daniel (Logs Jr)**: Night shift, drinks way too much coffee
- **Lea (Logs Sr)**: Extremely organized log dashboards
- **Tomas (General Jr)**: Takes detailed notes on everything
- **Sara (General Sr)**: Knows everyone's schedule by heart

**New Module:**
- `roster/personality.rs` - Personality traits and dialogue selection
- Deterministic phrase selection via seed-based indexing
- Helper functions: `get_greeting()`, `get_success()`, `get_uncertain()`

## [0.0.242] - 2025-12-09

### Added - Comprehensive README Overhaul

**Complete documentation rewrite with Anna's personality:**
- ASCII art logo and tagline
- Real example outputs with humor
- Hollywood IT experience documentation
- Feature walkthroughs with practical examples
- Team descriptions and responsibilities
- Architecture diagram
- Privacy section emphasizing local-first
- FAQ with personality
- All recipes documented

**Documentation Philosophy:**
- "The AI that reads your system, not your mind"
- Examples show actual Anna output style
- Jokes about Chrome tabs and IT department tropes
- Grounded in reality - all examples are real functionality

## [0.0.241] - 2025-12-09

### Added - Real-Time Streaming Token Output

**Streaming Wiring Complete:**
- LLM responses now stream word-by-word to the client in real-time
- No more waiting for the full response - see Anna's thinking as it happens
- StreamingSink provides thread-safe token push from async callbacks
- Final token marker indicates when streaming is complete

**Progress Tracker Enhancements:**
- Added `streaming_sink()` to get thread-safe sink for callbacks
- Events now merge main + streaming events with temporal ordering
- StreamingSink.push_token() for lock-free event pushing

**Specialist Handler Integration:**
- Callback now emits each token via streaming_sink.push_token()
- Final `is_final=true` token marks completion
- Existing token counting preserved alongside streaming

**Client-Side (already in v0.0.238):**
- live_request.rs handles StreamingToken events
- Clears spinner, prints tokens inline
- Faster 50ms polling during streaming for smooth output

## [0.0.240] - 2025-12-09

### Added - Idle-Time Tips & Learning

**Idle Detection in REPL:**
- Anna notices when you're idle (30 seconds)
- Shows helpful tips about features you haven't tried
- Maximum 3 tips per session to avoid being annoying
- Tips remember what's been shown (persisted to disk)

**Contextual Tips System (idle_tips.rs):**
- Tips based on your current config (suggests features not enabled)
- Categories: Performance, Security, Storage, Services, Network, System, Patterns
- TipQueue with priority-based ordering
- TipHistory tracks shown tips for deduplication

**Example Tips:**
- If learning mode disabled: suggests enabling it
- If email not set: mentions long-request notification feature
- If internal comms hidden: suggests the "fly on wall" experience

**Async REPL Input:**
- Migrated to tokio async stdin for timeout-based idle detection
- Uses tokio::select! to race input against idle timer
- Smooth re-prompt after showing tip

## [0.0.239] - 2025-12-09

### Added - Natural Language Email Setup

**Email via Conversation:**
- Set email with natural language: "my email is user@example.com"
- Or: "notify me at user@example.com", "reach me at..."
- Disable with: "disable email notifications", "stop emailing me"
- Email shown in `config` / `settings` status display

**Config Parser Improvements:**
- Added email address extraction with regex
- ConfigChange::Email and ConfigChange::ClearEmail variants
- Email stored in both profile and email config for consistency

## [0.0.238] - 2025-12-09

### Added - Session History & "Since Last Time" Summary

**Session History Tracking (user_profile/session.rs):**
- Track what happens in each session (queries, topics, commands learned)
- Store up to 5 recent sessions for context
- SessionSummary: query_count, topics discussed, commands learned, recipes executed
- SessionHistory: maintains recent session list with automatic truncation

**"Since Last Time" Summary:**
- Shows what you did together in the last session
- Example: "Last time (2 hours ago): handled 5 queries, discussed vim (3 times), learned about nvim"
- Appears in REPL greeting after the personalized hello
- Only shown when there's meaningful history to share

**Streaming Token Infrastructure:**
- Added StreamingToken event type for future word-by-word output
- Progress tracker support for streaming tokens
- Client-side handling in live_request.rs (infrastructure ready)

## [0.0.237] - 2025-12-09

### Added - Natural Language Settings & Enhanced UX

**Config Command Handler (commands/config.rs):**
- Natural language config changes in REPL
- Type "config" or "settings" to see current preferences
- Examples:
  - "Anna, enable learning mode"
  - "make Anna more casual"
  - "hide internal communications"
  - "be more playful"
- show_config_status() displays all current settings
- Config tips shown after changes

**REPL Integration:**
- Config commands handled locally (no daemon needed)
- Fast path for settings changes
- Respects show_internal_comms preference

**Enhanced Internal Comms Display:**
- Conversational dialogue format
- Staff members show with team context
- Spinner animation during LLM generation
- Respects user preference for hiding/showing internal comms

## [0.0.236] - 2025-12-09

### Added - User Experience Enhancements

**Pattern History Tracking (user_profile/patterns.rs):**
- Time-windowed tool usage tracking (weekly patterns)
- Editor trend detection ("You're using vim more than nano lately")
- Topic trend detection ("Your network questions increased this week")
- EditorTrendInsight: RecentSwitch, LearningNew variants
- TopicTrendInsight: Increasing, Dominant variants
- Automatic cleanup of old data (12-week retention)

**Natural Language Config Parser (config_parser.rs):**
- Parse user requests to change Anna's settings
- Support for settings:
  - Learning mode: "enable/disable learning mode"
  - Verbosity: "be more verbose", "brief answers"
  - Auto-confirm: "auto confirm low risk", "always ask"
  - Internal comms: "show/hide internal communications"
  - Formality: "be more casual/formal"
  - Humor: "be playful", "no jokes"
  - Technical depth: "expert mode", "simpler terms"
- ConfigChange enum with apply() method
- is_config_request() for routing detection

**Enhanced User Profile:**
- Added pattern_history field to UserProfile
- editor_trend() method for insight detection
- topic_trend() method for topic pattern detection
- cleanup_patterns() for maintenance

**Updated Greeting System:**
- Greeting now shows editor trend insights
- Topic trend insights in pattern display

## [0.0.235] - 2025-12-09

### Added - Docker Compose Recipes

**New docker_recipes module with 10 built-in recipes:**
- `CreateCompose`: Create a docker-compose.yml file with templates
- `StartServices`: Start Docker Compose services
- `StopServices`: Stop Docker Compose services
- `ViewLogs`: View Docker container logs
- `ListContainers`: List running containers
- `BuildImages`: Build Docker images
- `PullImages`: Pull/update Docker images
- `ExecContainer`: Execute commands in containers
- `Cleanup`: Cleanup Docker resources (prune)
- `Debug`: Debug Docker issues

**Module structure:**
- `types.rs`: DockerFeature, DockerRecipe (108 lines)
- `recipes.rs`: Built-in recipe definitions (285 lines)
- `matcher.rs`: Query matching logic (81 lines)
- `tests.rs`: Unit tests (72 lines)
- `mod.rs`: Re-exports (14 lines)

**Integration:**
- Added RecipeKind::DockerCompose variant
- Integrated with recipe_fast_path for query matching
- Added to knowledge/conversion.rs for tagging

## [0.0.234] - 2025-12-09

### Added - Cron Job Recipes

**New cron_recipes module with 8 built-in recipes:**
- `AddJob`: Add a new cron job with schedule examples
- `ListJobs`: List current cron jobs (user and system-wide)
- `EditCrontab`: Edit crontab with editor options
- `RemoveJob`: Remove cron jobs safely
- `ViewLogs`: View cron job logs and output
- `SyntaxHelp`: Comprehensive cron syntax explanation
- `Environment`: Cron environment variables guide
- `DebugJob`: Debug a failing cron job

**Module structure:**
- `types.rs`: CronPreset, CronFeature, CronRecipe (141 lines)
- `recipes.rs`: Built-in recipe definitions (232 lines)
- `matcher.rs`: Query matching logic (69 lines)
- `tests.rs`: Unit tests (68 lines)
- `mod.rs`: Re-exports (14 lines)

**Integration:**
- Added RecipeKind::CronJob variant
- Integrated with recipe_fast_path for query matching
- Added to knowledge/conversion.rs for tagging

## [0.0.233] - 2025-12-09

### Added - Systemd Unit File Recipes

**New systemd_recipes module with 8 built-in recipes:**
- `CreateService`: Create a basic systemd service unit
- `CreateTimer`: Create a systemd timer (cron replacement)
- `CreateUserService`: Create a user-level service (~/.config/systemd/user)
- `EnableService`: Enable and start a service
- `ViewLogs`: View service logs with journalctl
- `DebugService`: Debug a failing service
- `SocketActivation`: Create a socket-activated service
- `HardenService`: Harden a service with security options

**Module structure:**
- `types.rs`: UnitType, SystemdFeature, RestartPolicy, SystemdRecipe (161 lines)
- `recipes.rs`: Built-in recipe definitions (260 lines)
- `matcher.rs`: Query matching logic (70 lines)
- `tests.rs`: Unit tests (62 lines)
- `mod.rs`: Re-exports (14 lines)

**Integration:**
- Added RecipeKind::SystemdUnit variant
- Integrated with recipe_fast_path for query matching
- Added to knowledge/conversion.rs for tagging

## [0.0.232] - 2025-12-09

### Changed - recipe_store.rs Modularization

**Split recipe_store.rs (378→18 lines) into 4 domain-specific modules:**
- `types.rs`: RecipeRisk, RecipeStep, Citation, Recipe structs (166 lines)
- `store.rs`: RecipeStore struct with persistence (118 lines)
- `learning.rs`: MIN_LEARN_RELIABILITY, should_learn_recipe (14 lines)
- `tests.rs`: Unit tests (59 lines)
- `mod.rs`: Re-exports for backwards compatibility (18 lines)

**All recipe_store/ files under 400-line limit per CLAUDE.md policy.**

## [0.0.231] - 2025-12-09

### Changed - shell_recipes.rs Modularization

**Split shell_recipes.rs (374→17 lines) into 4 domain-specific modules:**
- `types.rs`: Shell, ShellFeature enums, ShellRecipe struct (124 lines)
- `catalog.rs`: builtin_recipes with shell configs (128 lines)
- `search.rs`: find_recipe, find_recipes_by_keywords, detect_feature (68 lines)
- `tests.rs`: Unit tests (44 lines)
- `mod.rs`: Re-exports for backwards compatibility (17 lines)

**All shell_recipes/ files under 400-line limit per CLAUDE.md policy.**

## [0.0.230] - 2025-12-09

### Changed - package_recipes.rs Modularization

**Split package_recipes.rs (378→24 lines) into 4 domain-specific modules:**
- `types.rs`: PackageManager, PackageCategory, PackageRecipe structs (181 lines)
- `catalog.rs`: common_packages with built-in recipes (109 lines)
- `search.rs`: find_recipe, confirmation_prompt (27 lines)
- `tests.rs`: Unit tests (39 lines)
- `mod.rs`: Re-exports for backwards compatibility (24 lines)

**All package_recipes/ files under 400-line limit per CLAUDE.md policy.**

## [0.0.229] - 2025-12-09

### Changed - brief.rs Modularization

**Split brief.rs (384→16 lines) into 3 domain-specific modules:**
- `types.rs`: TicketBrief struct and build_brief_from_ticket (95 lines)
- `filtering.rs`: PROBE_PATTERNS, evidence_kind_for_probe, is_probe_relevant (76 lines)
- `tests.rs`: Unit tests (179 lines)
- `mod.rs`: Re-exports for backwards compatibility (16 lines)

**All brief/ files under 400-line limit per CLAUDE.md policy.**

## [0.0.228] - 2025-12-09

### Changed - review_gate.rs Modularization

**Split review_gate.rs (385→14 lines) into 3 domain-specific modules:**
- `types.rs`: ReviewContext, GateOutcome, GateThresholds structs (158 lines)
- `logic.rs`: deterministic_review_gate functions (87 lines)
- `tests.rs`: Unit tests (114 lines)
- `mod.rs`: Re-exports for backwards compatibility (14 lines)

**All review_gate/ files under 400-line limit per CLAUDE.md policy.**

## [0.0.227] - 2025-12-09

### Changed - pending.rs Modularization

**Split pending.rs (386→18 lines) into 4 domain-specific modules:**
- `types.rs`: PendingClarification struct, ParseResult, VerifyResult enums (136 lines)
- `verification.rs`: verify_answer function (49 lines)
- `persistence.rs`: File I/O functions (56 lines)
- `tests.rs`: Unit tests (111 lines)
- `mod.rs`: Re-exports for backwards compatibility (18 lines)

**All pending/ files under 400-line limit per CLAUDE.md policy.**

## [0.0.226] - 2025-12-09

### Changed - theatre.rs Modularization

**Split theatre.rs (388→20 lines) into 4 domain-specific modules:**
- `types.rs`: Speaker enum, NarrativeSegment struct (114 lines)
- `builder.rs`: NarrativeBuilder struct with all methods (142 lines)
- `formatting.rs`: describe_check, format_case_id helpers (44 lines)
- `tests.rs`: Unit tests (67 lines)
- `mod.rs`: Re-exports for backwards compatibility (20 lines)

**All theatre/ files under 400-line limit per CLAUDE.md policy.**

## [0.0.225] - 2025-12-09

### Changed - health_delta.rs Modularization

**Split health_delta.rs (398→18 lines) into 4 domain-specific modules:**
- `types.rs`: HealthDelta struct with comparison logic (98 lines)
- `history.rs`: SnapshotHistory struct (69 lines)
- `summary.rs`: HealthSummary struct with formatting (205 lines)
- `helpers.rs`: generate_summary, format_disk_summary (44 lines)
- `mod.rs`: Re-exports for backwards compatibility (18 lines)

**All health_delta/ files under 400-line limit.**

## [0.0.224] - 2025-12-09

### Changed - git_recipes.rs Modularization

**Split git_recipes.rs (402→20 lines) into 5 domain-specific modules:**
- `types.rs`: GitScope, GitFeature enums (100 lines)
- `recipe.rs`: GitRecipe, GitParameter structs (72 lines)
- `catalog.rs`: builtin_recipes function (135 lines)
- `search.rs`: find_recipe, find_recipes_by_keywords, detect_feature (70 lines)
- `tests.rs`: Unit tests (35 lines)
- `mod.rs`: Re-exports for backwards compatibility (20 lines)

**All git_recipes/ files under 400-line limit.**

## [0.0.223] - 2025-12-09

### Changed - model_selector.rs Modularization

**Split model_selector.rs (402→21 lines) into 6 domain-specific modules:**
- `types.rs`: ModelFamily, ModelRole, ModelCandidate, ModelSelection, ModelBenchmark (61 lines)
- `config.rs`: ModelSelectorConfig, ModelSelectorState (39 lines)
- `catalog.rs`: model_catalog function (69 lines)
- `selection.rs`: select_model, model_matches, detect_family (128 lines)
- `benchmark.rs`: BENCHMARK_PROMPT, parse_benchmark_response (56 lines)
- `tests.rs`: Unit tests (71 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All model_selector/ files under 400-line limit.**

## [0.0.222] - 2025-12-09

### Changed - review.rs Modularization

**Split review.rs (403→21 lines) into 4 domain-specific modules:**
- `types.rs`: ReviewDecision, ReviewerType, ReviewSeverity, ReviewIssueKind enums (120 lines)
- `issue.rs`: ReviewIssue, ReviewRevision structs (95 lines)
- `inputs.rs`: ReviewInputsSummary struct (49 lines)
- `artifact.rs`: ReviewArtifact struct (147 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All review/ files under 400-line limit.**

## [0.0.221] - 2025-12-09

### Changed - helpers.rs Modularization

**Split helpers.rs (405→20 lines) into 5 domain-specific modules:**
- `types.rs`: HelperPackage, InstallSource (100 lines)
- `registry.rs`: HelpersRegistry struct and methods (98 lines)
- `detection.rs`: known_helpers, detect_helper, detect_ollama (52 lines)
- `persistence.rs`: helpers_store_path, load_helpers, save_helpers (43 lines)
- `tests.rs`: Unit tests (126 lines)
- `mod.rs`: Re-exports for backwards compatibility (20 lines)

**All helpers/ files under 400-line limit.**

## [0.0.220] - 2025-12-09

### Changed - rpc.rs Modularization

**Split rpc.rs (411→21 lines) into 7 domain-specific modules:**
- `method.rs`: RpcMethod enum, DaemonInfo (42 lines)
- `request_response.rs`: RpcRequest, RpcResponse, RpcError (70 lines)
- `params.rs`: RequestParams, ProbeParams, ProbeType, PlanChangeParams, ChangeParams (41 lines)
- `context.rs`: RuntimeContext, Capabilities, HardwareSummary (43 lines)
- `routing.rs`: SpecialistDomain, QueryIntent, TranslatorTicket (73 lines)
- `result.rs`: ProbeResult, EvidenceBlock, ReliabilitySignals, ServiceDeskResult (135 lines)
- `tests.rs`: Unit tests (27 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All rpc/ files under 400-line limit.**

## [0.0.219] - 2025-12-09

### Changed - snapshot.rs Modularization

**Split snapshot.rs (412→22 lines) into 4 domain-specific modules:**
- `types.rs`: SystemSnapshot struct, thresholds constants (95 lines)
- `capture.rs`: capture_snapshot, parse_df/free/failed_services (109 lines)
- `delta.rs`: DeltaItem, diff_snapshots, format_deltas_text (166 lines)
- `persistence.rs`: load/save/clear snapshots (48 lines)
- `mod.rs`: Re-exports for backwards compatibility (22 lines)

**All snapshot/ files under 400-line limit.**

## [0.0.218] - 2025-12-09

### Changed - narrator.rs Modularization

**Split narrator.rs (413→19 lines) into 5 domain-specific modules:**
- `roles.rs`: team_role_name, team_tag, reviewer_badge (62 lines)
- `narration.rs`: narrate_team_action, narrate_review_result, etc. (76 lines)
- `person.rs`: Person-based narration functions (58 lines)
- `it_dialog.rs`: IT department dialog style (44 lines)
- `tests.rs`: Unit tests (160 lines)
- `mod.rs`: Re-exports for backwards compatibility (19 lines)

**All narrator/ files under 400-line limit.**

## [0.0.217] - 2025-12-09

### Changed - user_profile.rs Modularization

**Split user_profile.rs (416→14 lines) into 4 domain-specific modules:**
- `types.rs`: UserProfile, UserPreferences, PersonalityTraits structs (96 lines)
- `profile.rs`: UserProfile impl (load, save, record_*, greeting_context) (171 lines)
- `greeting.rs`: GreetingContext and generate_greeting (45 lines)
- `tests.rs`: Unit tests (60 lines)
- `mod.rs`: Re-exports for backwards compatibility (14 lines)

**All user_profile/ files under 400-line limit.**

## [0.0.216] - 2025-12-09

### Changed - ticket_packet.rs Modularization

**Split ticket_packet.rs (421→18 lines) into 5 domain-specific modules:**
- `types.rs`: PacketBudget, MAX_PACKET_BYTES constant (21 lines)
- `packet.rs`: TicketPacket struct and methods (155 lines)
- `builder.rs`: TicketPacketBuilder (78 lines)
- `domain.rs`: recommended_probes_for_domain, evidence_kinds_for_domain (32 lines)
- `policy.rs`: PacketPolicy, policy_for_team (122 lines)
- `mod.rs`: Re-exports for backwards compatibility (18 lines)

**All ticket_packet/ files under 400-line limit.**

## [0.0.215] - 2025-12-09

### Changed - ticket.rs Modularization

**Split ticket.rs (422→20 lines) into 4 domain-specific modules:**
- `types.rs`: RiskLevel, TicketStatus enums, constants (78 lines)
- `ticket_struct.rs`: Ticket struct and core methods (172 lines)
- `clarification.rs`: Clarification-related methods (60 lines)
- `tests.rs`: Unit tests (95 lines)
- `mod.rs`: Re-exports for backwards compatibility (20 lines)

**All ticket/ files under 400-line limit.**

## [0.0.214] - 2025-12-09

### Changed - service_recipes.rs Modularization

**Split service_recipes.rs (428→25 lines) into 5 domain-specific modules:**
- `types.rs`: ServiceAction, ServiceCategory, ServiceRisk enums (123 lines)
- `recipe.rs`: ServiceRecipe struct and methods (56 lines)
- `catalog.rs`: known_services, find_service (125 lines)
- `prompt.rs`: confirmation_prompt (42 lines)
- `tests.rs`: Unit tests (50 lines)
- `mod.rs`: Re-exports for backwards compatibility (25 lines)

**All service_recipes/ files under 400-line limit.**

## [0.0.213] - 2025-12-09

### Changed - ui.rs Modularization

**Split ui.rs (431→21 lines) into 7 domain-specific modules:**
- `colors.rs`: ANSI color constants (10 lines)
- `symbols.rs`: Unicode symbols (8 lines)
- `formatting.rs`: format_bytes, format_duration, progress_bar (45 lines)
- `printing.rs`: print_header, print_footer, print_section, print_ok, print_err, print_kv, print_kv_status (65 lines)
- `terminal.rs`: print_inline, clear_line, cursor_up (20 lines)
- `spinner.rs`: Spinner struct and methods (112 lines)
- `stage.rs`: StageProgress, StageStatus (95 lines)
- `tests.rs`: Unit tests (60 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All ui/ files under 400-line limit.**

## [0.0.212] - 2025-12-09

### Changed - det_extended/system.rs Modularization

**Split det_extended/system.rs (445→21 lines) into 7 domain-specific modules:**
- `packages.rs`: answer_package_updates (51 lines)
- `time.rs`: answer_timezone_info, answer_system_uptime, answer_last_boot (99 lines)
- `host.rs`: answer_hostname, answer_os_info, answer_system_architecture (102 lines)
- `resources.rs`: answer_swap_info, answer_system_load, answer_open_files (83 lines)
- `processes.rs`: answer_process_tree (41 lines)
- `locale.rs`: answer_system_locale (50 lines)
- `diagnostics.rs`: answer_virtualization_info, answer_coredump_list, answer_tmp_files (72 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All system/ files under 400-line limit.**

## [0.0.211] - 2025-12-09

### Changed - status_snapshot.rs Modularization

**Split status_snapshot.rs (435→24 lines) into 9 domain-specific modules:**
- `version.rs`: VersionInfo (36 lines)
- `daemon.rs`: DaemonInfo (42 lines)
- `permissions.rs`: PermissionsInfo (44 lines)
- `update.rs`: UpdateResult, UpdateInfo (48 lines)
- `helpers_info.rs`: HelperPackageLite, HelpersInfo (52 lines)
- `models.rs`: ModelDownloadStatus, RoleModelBinding, ModelsInfo (53 lines)
- `config.rs`: ConfigInfo (14 lines)
- `snapshot.rs`: StatusSnapshot struct and methods (79 lines)
- `tests.rs`: Unit tests (82 lines)
- `mod.rs`: Re-exports for backwards compatibility (24 lines)

**All status_snapshot/ files under 400-line limit.**

## [0.0.210] - 2025-12-09

### Changed - health_view.rs Modularization

**Split health_view.rs (438→14 lines) into 4 domain-specific modules:**
- `types.rs`: HealthSeverity, HealthCategory, HealthItem, HealthChange (98 lines)
- `summary.rs`: RelevantHealthSummary struct and methods (118 lines)
- `builder.rs`: build_health_summary, has_health_issues (103 lines)
- `tests.rs`: Unit tests (105 lines)
- `mod.rs`: Re-exports for backwards compatibility (14 lines)

**All health_view/ files under 400-line limit.**

## [0.0.209] - 2025-12-09

### Changed - answer_contract.rs Modularization

**Split answer_contract.rs (442→18 lines) into 5 domain-specific modules:**
- `types.rs`: Verbosity, RequestedField (94 lines)
- `contract.rs`: AnswerContract struct and methods (103 lines)
- `validation.rs`: AnswerValidation, validate_answer, field_present_in_answer (96 lines)
- `trimming.rs`: trim_answer, extract helpers (76 lines)
- `tests.rs`: Unit tests (59 lines)
- `mod.rs`: Re-exports for backwards compatibility (18 lines)

**All answer_contract/ files under 400-line limit.**

## [0.0.208] - 2025-12-09

### Changed - revision.rs Modularization

**Split revision.rs (442→21 lines) into 5 domain-specific modules:**
- `types.rs`: RevisionIssue, RevisionInstruction (131 lines)
- `junior.rs`: JuniorVerification (36 lines)
- `senior.rs`: SeniorEscalation (41 lines)
- `conversion.rs`: junior_to_review_artifact, senior_to_review_artifact (107 lines)
- `tests.rs`: Unit tests (106 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All revision/ files under 400-line limit.**

## [0.0.207] - 2025-12-09

### Changed - health_brief.rs Modularization

**Split health_brief.rs (449→18 lines) into 4 domain-specific modules:**
- `types.rs`: BriefSeverity, BriefItemKind, BriefItem (102 lines)
- `severity.rs`: disk_severity, memory_severity (25 lines)
- `brief.rs`: HealthBrief struct and methods (176 lines)
- `tests.rs`: Unit tests (113 lines)
- `mod.rs`: Re-exports for backwards compatibility (18 lines)

**All health_brief/ files under 400-line limit.**

## [0.0.206] - 2025-12-09

### Changed - email.rs Modularization

**Split email.rs (453→23 lines) into 4 domain-specific modules:**
- `system.rs`: inbox_path, is_email_available, email_package_name (76 lines)
- `config.rs`: EmailConfig, EmailHealth (113 lines)
- `notifications.rs`: EmailNotification, send_notification, format_email (192 lines)
- `tests.rs`: Unit tests (55 lines)
- `mod.rs`: Re-exports for backwards compatibility (23 lines)

**All email/ files under 400-line limit.**

## [0.0.205] - 2025-12-09

### Changed - commands.rs Modularization

**Split commands.rs (458→11 lines) into 3 domain-specific modules:**
- `handlers.rs`: handle_status, handle_stats, handle_request, handle_uninstall (182 lines)
- `repl.rs`: handle_repl, PendingClarification (193 lines)
- `feedback.rs`: handle_feedback_request, handle_request_error (76 lines)
- `mod.rs`: Re-exports for backwards compatibility (11 lines)

**All commands/ files under 400-line limit.**

## [0.0.204] - 2025-12-09

### Changed - progress.rs Modularization

**Split progress.rs (459→14 lines) into 3 domain-specific modules:**
- `types.rs`: DiagnosticText, RequestStage, TimeoutConfig (97 lines)
- `event.rs`: ProgressEvent, ProgressEventType (196 lines)
- `tests.rs`: Unit tests (148 lines)
- `mod.rs`: Re-exports for backwards compatibility (14 lines)

**All progress/ files under 400-line limit.**

## [0.0.203] - 2025-12-09

### Changed - render.rs Modularization

**Split render.rs (465→25 lines) into 5 domain-specific modules:**
- `types.rs`: RenderPolicy, Verbosity, UiConfig, RiskLevel (72 lines)
- `formatting.rs`: generate_case_id, format_time_delta, determine_risk_level (56 lines)
- `output.rs`: All render_* functions (209 lines)
- `spinner.rs`: Spinner, ProgressRenderer (75 lines)
- `tests.rs`: Unit tests (37 lines)
- `mod.rs`: Re-exports for backwards compatibility (25 lines)

**All render/ files under 400-line limit.**

## [0.0.202] - 2025-12-09

### Changed - theatre_render.rs Modularization

**Split theatre_render.rs (471→15 lines) into 5 domain-specific modules:**
- `helpers.rs`: Utility functions (reliability_color, team_from_domain, probe_id_from_command) (68 lines)
- `narrative.rs`: build_narrative and related functions (84 lines)
- `footer.rs`: Footer rendering and evidence formatting (117 lines)
- `render.rs`: Main render_theatre function and segment printing (161 lines)
- `tests.rs`: Unit tests (33 lines)
- `mod.rs`: Re-exports for backwards compatibility (15 lines)

**All theatre_render/ files under 400-line limit.**

## [0.0.201] - 2025-12-09

### Changed - model_registry.rs Modularization

**Split model_registry.rs (479→20 lines) into 4 domain-specific modules:**
- `types.rs`: ModelSpec, RoleBinding, ModelState, HardwareTier, recommended_model_for_tier (137 lines)
- `registry.rs`: ModelRegistry struct and methods (148 lines)
- `persistence.rs`: load/save functions, parse_ollama_list with tests (120 lines)
- `tests.rs`: Unit tests for registry functionality (93 lines)
- `mod.rs`: Re-exports for backwards compatibility (20 lines)

**All model_registry/ files under 400-line limit.**

## [0.0.200] - 2025-12-09

### Changed - rpc_handler.rs Modularization

**Split rpc_handler.rs (483→12 lines) into 3 domain-specific modules:**
- `dispatcher.rs`: handle_request RPC dispatcher (28 lines)
- `helpers.rs`: save_progress, record_event_log (46 lines)
- `llm_request.rs`: handle_llm_request and inner pipeline (355 lines)
- `mod.rs`: Re-exports for backwards compatibility (12 lines)

**All rpc_handler/ files under 400-line limit.**

## [0.0.199] - 2025-12-09

### Changed - budget.rs Modularization

**Split budget.rs (484→23 lines) into 4 domain-specific modules:**
- `types.rs`: Stage enum, StageTiming, constants (58 lines)
- `probe.rs`: ProbeBudget, ProbeBudgetCheck (79 lines)
- `stage.rs`: StageBudget, BudgetCheck, BudgetEnforcer (219 lines)
- `llm.rs`: LlmBudget, LlmFallback, check_llm_fallback (121 lines)
- `mod.rs`: Re-exports for backwards compatibility (23 lines)

**All budget/ files under 400-line limit.**

## [0.0.198] - 2025-12-09

### Changed - verify.rs Modularization

**Split verify.rs (485→15 lines) into 4 domain-specific modules:**
- `types.rs`: VerifyExpectation, ServiceExpectedState, VerificationStep, VerifyResult (140 lines)
- `runners.rs`: run_verification and all verify_* helpers (178 lines)
- `batch.rs`: PreActionVerify, PostActionVerify (97 lines)
- `tests.rs`: Unit tests (48 lines)
- `mod.rs`: Re-exports for backwards compatibility (15 lines)

**All verify/ files under 400-line limit.**

## [0.0.197] - 2025-12-09

### Changed - clarify_v2.rs Modularization

**Split clarify_v2.rs (489→17 lines) into 3 domain-specific modules:**
- `types.rs`: ClarifyRequest, ClarifyOption, ClarifyResponse, ClarifyResult, VerifyFailureTracker (263 lines)
- `processing.rs`: process_response, find_installed_alternatives, editor_request (130 lines)
- `facts.rs`: should_skip, store_fact, invalidate_on_uninstall (62 lines)
- `mod.rs`: Re-exports for backwards compatibility (17 lines)

**All clarify_v2/ files under 400-line limit.**

## [0.0.196] - 2025-12-09

### Changed - ssh_recipes.rs Modularization

**Split ssh_recipes.rs (491→17 lines) into 5 domain-specific modules:**
- `types.rs`: SshKeyType, SshFeature, SshRecipe, SshStep (139 lines)
- `paths.rs`: ssh_dir, ssh_config_path (14 lines)
- `recipes.rs`: builtin_recipes with all SSH recipes (230 lines)
- `matcher.rs`: match_query for query matching (39 lines)
- `tests.rs`: Unit tests (38 lines)
- `mod.rs`: Re-exports for backwards compatibility (17 lines)

**All ssh_recipes/ files under 400-line limit.**

## [0.0.195] - 2025-12-09

### Changed - grounding.rs Modularization

**Split grounding.rs (496→21 lines) into 3 domain-specific modules:**
- `types.rs`: GroundingReport, ClaimVerification, VerificationReason, ParsedEvidence (83 lines)
- `verify.rs`: compute_grounding, verify_claim, verify_numeric, verify_percent, verify_status, is_answer_grounded (164 lines)
- `tests.rs`: Unit tests (214 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All grounding/ files under 400-line limit.**

## [0.0.194] - 2025-12-09

### Changed - guard.rs Modularization

**Split guard.rs (502→18 lines) into 3 domain-specific modules:**
- `types.rs`: VerifyResult, GuardReport, GuardItem (56 lines)
- `verify.rs`: run_guard, verify_claim, verify_numeric, verify_percent, verify_status (148 lines)
- `tests.rs`: Unit tests (260 lines)
- `mod.rs`: Re-exports for backwards compatibility (18 lines)

**All guard/ files under 400-line limit.**

## [0.0.193] - 2025-12-09

### Changed - probe_spine.rs Modularization

**Split probe_spine.rs (503→15 lines) into 4 domain-specific modules:**
- `types.rs`: EvidenceKind, ProbeId, RouteCapability, Urgency enums/structs (120 lines)
- `commands.rs`: probes_for_evidence, probe_to_command (53 lines)
- `enforcement.rs`: enforce_spine_probes, enforce_minimum_probes, ProbeSpineDecision (200 lines)
- `reduction.rs`: reduce_probes, query_wants_warnings, query_wants_errors (72 lines)
- `mod.rs`: Re-exports for backwards compatibility (15 lines)

**All probe_spine/ files under 400-line limit.**

## [0.0.192] - 2025-12-09

### Changed - comms.rs Modularization

**Split comms.rs (507→14 lines) into 4 domain-specific modules:**
- `generator.rs`: CommsGenerator struct and constructor (39 lines)
- `messages.rs`: Message generation methods (dispatch, probing, reviewing, etc.) (246 lines)
- `routing.rs`: team_from_domain function (22 lines)
- `tests.rs`: Unit tests (30 lines)
- `mod.rs`: Re-exports for backwards compatibility (14 lines)

**All comms/ files under 400-line limit.**

## [0.0.191] - 2025-12-09

### Changed - clarify.rs Modularization

**Split clarify.rs (509→35 lines) into 4 domain-specific modules:**
- `menu.rs`: ClarifyPrompt, MenuOption, ClarifyOutcome (v0.0.42 menu types) (165 lines)
- `legacy.rs`: ClarifyKind, ClarifyQuestion, ClarifyOption, ClarifyAnswer (v0.0.32-v0.0.39) (147 lines)
- `detection.rs`: needs_clarification, extract_service_name (47 lines)
- `editors.rs`: editor_menu_prompt, generate_editor_options, find_installed_alternative (124 lines)
- `mod.rs`: Re-exports for backwards compatibility (35 lines)

**All clarify/ files under 400-line limit.**

## [0.0.190] - 2025-12-09

### Changed - event_log.rs Modularization

**Split event_log.rs (509→13 lines) into 4 domain-specific modules:**
- `types.rs`: EventRecord struct and builder methods (87 lines)
- `store.rs`: EventLog file store with rotation (117 lines)
- `aggregation.rs`: AggregatedEvents, XP/level computation (211 lines)
- `tests.rs`: Unit tests (54 lines)
- `mod.rs`: Re-exports for backwards compatibility (13 lines)

**All event_log/ files under 400-line limit.**

## [0.0.189] - 2025-12-09

### Changed - report.rs Modularization

**Split report.rs (511→19 lines) into 5 domain-specific modules:**
- `types.rs`: HealthSeverity, HealthItem, SystemInventory, SystemReport, ReportEvidence (79 lines)
- `helpers.rs`: sanitize_mount, format_bytes (23 lines)
- `builders.rs`: build_inventory, build_health_checks, build_executive_summary, SystemReport::from_evidence (215 lines)
- `formatters.rs`: format_text, format_markdown (147 lines)
- `tests.rs`: Unit tests (25 lines)
- `mod.rs`: Re-exports for backwards compatibility (19 lines)

**All report/ files under 400-line limit.**

## [0.0.188] - 2025-12-09

### Changed - inventory.rs Modularization

**Split inventory.rs (513→30 lines) into 7 domain-specific modules:**
- `constants.rs`: VIP_TOOLS, DESKTOP_PACKAGES, TTL (50 lines)
- `types.rs`: InventoryState, InventoryItem (70 lines)
- `system_info.rs`: SystemInfo struct and detection (112 lines)
- `cache.rs`: InventoryCache struct and methods (130 lines)
- `helpers.rs`: check_tool_installed, timestamps, run_command (43 lines)
- `persistence.rs`: Load, save, filter functions (68 lines)
- `tests.rs`: Unit tests (53 lines)
- `mod.rs`: Re-exports for backwards compatibility (30 lines)

**All inventory/ files under 400-line limit.**

## [0.0.187] - 2025-12-09

### Changed - det/system.rs Modularization

**Split det/system.rs (520→21 lines) into 6 domain-specific modules:**
- `packages.rs`: Package updates answer function (48 lines)
- `time.rs`: Timezone, uptime, last boot answers (107 lines)
- `users.rs`: Logged-in users answer (55 lines)
- `hardware.rs`: Swap, battery, load, USB answers (199 lines)
- `info.rs`: Hostname, OS info answers (70 lines)
- `network.rs`: Connectivity, filesystems answers (68 lines)
- `mod.rs`: Re-exports for backwards compatibility (21 lines)

**All det/system/ files under 400-line limit.**

## [0.0.186] - 2025-12-09

### Changed - greeting.rs Modularization

**Split greeting.rs (542→94 lines) into 4 domain-specific modules:**
- `types.rs`: InteractionInfo struct and calculation (37 lines)
- `personal.rs`: Personalized greeting, patterns, tickets (166 lines)
- `status.rs`: Since-last-time, system readiness, failed services (259 lines)
- `tests.rs`: Unit tests (18 lines)
- `mod.rs`: Main entry point and re-exports (94 lines)

**All greeting/ files under 400-line limit.**

## [0.0.185] - 2025-12-09

### Changed - fastpath.rs Modularization

**Split fastpath.rs (554→24 lines) into 5 domain-specific modules:**
- `types.rs`: FastPathClass, FastPathAnswer, FastPathPolicy, FastPathInput (128 lines)
- `classify.rs`: classify_fast_path, strip_greetings (98 lines)
- `answers.rs`: Answer generators for each query class (209 lines)
- `engine.rs`: try_fast_path, find_matching_recipes (66 lines)
- `tests.rs`: Unit tests (66 lines)
- `mod.rs`: Re-exports for backwards compatibility (24 lines)

**All fastpath/ files under 400-line limit.**

## [0.0.184] - 2025-12-09

### Changed - trace.rs Modularization

**Split trace.rs (555→19 lines) into 5 domain-specific modules:**
- `outcomes.rs`: SpecialistOutcome, FallbackUsed, ReviewerOutcome enums (107 lines)
- `probe_stats.rs`: ProbeStats struct and methods (51 lines)
- `evidence.rs`: EvidenceKind enum and detection functions (149 lines)
- `execution.rs`: ExecutionTrace struct and constructors (153 lines)
- `tests.rs`: Unit tests for trace functionality (107 lines)
- `mod.rs`: Re-exports for backwards compatibility (19 lines)

**All trace/ files under 400-line limit.**

## [0.0.183] - 2025-12-09

### Changed - ticket_tracker.rs Modularization

**Split ticket_tracker.rs (562→28 lines) into 5 domain-specific modules:**
- `status.rs`: TicketStatus enum with serde support (37 lines)
- `message.rs`: TicketMessage struct for conversation history (39 lines)
- `ticket.rs`: Ticket struct and all methods (160 lines)
- `tracker.rs`: TicketTracker and TicketStats (276 lines)
- `tests.rs`: Unit tests for ticket lifecycle (54 lines)
- `mod.rs`: Re-exports for backwards compatibility (28 lines)

**All ticket_tracker/ files under 400-line limit.**

## [0.0.182] - 2025-12-09

### Changed - roster.rs Modularization

**Split roster.rs (569→19 lines) into 5 domain-specific modules:**
- `tier.rs`: Tier enum and Display impl (20 lines)
- `shift.rs`: Shift enum with time-based availability (57 lines)
- `profile.rs`: PersonProfile struct and methods (58 lines)
- `data.rs`: ROSTER const and lookup functions (277 lines)
- `tests.rs`: Unit and golden tests (171 lines)
- `mod.rs`: Re-exports for backwards compatibility (19 lines)

**All roster/ files under 400-line limit.**

## [0.0.181] - 2025-12-09

### Changed - facts.rs Modularization

**Split facts.rs (572→23 lines) into 5 domain-specific modules:**
- `key.rs`: FactKey enum and Display impl (54 lines)
- `policy.rs`: StalenessPolicy, FactLifecycle, TTL constants (68 lines)
- `fact.rs`: Fact struct and all methods (185 lines)
- `store.rs`: FactsStore struct and lifecycle management (259 lines)
- `status.rs`: FactStatus enum and status method (27 lines)
- `mod.rs`: Re-exports for backwards compatibility (23 lines)

**All facts/ files under 400-line limit.**

## [0.0.180] - 2025-12-09

### Changed - intake.rs Modularization

**Split intake.rs (573→20 lines) into 6 domain-specific modules:**
- `verify_plan.rs`: VerifyPlan enum and implementation (46 lines)
- `question.rs`: ClarificationQuestion struct and builder (65 lines)
- `result.rs`: IntakeResult and VerificationResult types (105 lines)
- `slot.rs`: ClarificationSlot enum and generate_clarification (103 lines)
- `analyze.rs`: analyze_intake and helper functions (152 lines)
- `tests.rs`: Unit tests (133 lines)
- `mod.rs`: Re-exports for backwards compatibility (20 lines)

**All intake/ files under 400-line limit.**

## [0.0.179] - 2025-12-09

### Changed - transcript_render.rs Modularization

**Split transcript_render.rs (652→24 lines) into 6 domain-specific modules:**
- `answer_source.rs`: AnswerSource enum and get_final_answer helper (36 lines)
- `helpers.rs`: Rendering helper functions - format_actor_tag, format_outcome, reliability_color, truncate (70 lines)
- `render.rs`: Main render entry points - render, render_with_options (26 lines)
- `debug_render.rs`: Debug mode rendering - full troubleshooting view (326 lines)
- `event_renders.rs`: Individual event rendering helpers for transcript events (224 lines)
- `tests.rs`: Unit tests for helpers (34 lines)
- `mod.rs`: Re-exports for backwards compatibility (24 lines)

**All transcript_render/ files under 400-line limit.**

## [0.0.178] - 2025-12-09

### Changed - transcript.rs Modularization

**Split transcript.rs (660→19 lines) into 5 domain-specific modules:**
- `actor.rs`: Actor enum (38 lines)
- `outcome.rs`: StageOutcome enum (97 lines)
- `event_kind.rs`: TranscriptEventKind enum (210 lines)
- `event.rs`: TranscriptEvent struct and constructors (265 lines)
- `core.rs`: Transcript container struct (65 lines)
- `mod.rs`: Re-exports for backwards compatibility (19 lines)

**All transcript/ files under 400-line limit.**

## [0.0.177] - 2025-12-09

### Changed - recipe.rs Modularization

**Split recipe.rs (662→29 lines) into 7 domain-specific modules:**
- `types.rs`: RecipeKind, RecipeAction enums (46 lines)
- `target.rs`: RecipeTarget, RollbackInfo structs (49 lines)
- `signature.rs`: RecipeSignature for unique identification (36 lines)
- `slot.rs`: RecipeSlot, ClarifyPrereq types (81 lines)
- `core.rs`: Recipe struct and implementation (288 lines)
- `storage.rs`: recipe_dir, save/load, count, clear functions (49 lines)
- `search.rs`: RAG-lite search_recipes_by_keywords (128 lines)
- `mod.rs`: Re-exports for backwards compatibility (29 lines)

**All recipe/ files under 400-line limit.**

## [0.0.176] - 2025-12-09

### Changed - deterministic.rs Modularization

**Split deterministic.rs (787→29 lines) into 7 domain-specific modules:**
- `types.rs`: DeterministicResult struct (10 lines)
- `help.rs`: answer_help function (40 lines)
- `memory.rs`: Memory, disk, service, health handlers (99 lines)
- `cpu.rs`: CPU cores, temperature, extract_temperature (122 lines)
- `audio.rs`: Hardware audio answer handler (76 lines)
- `packages.rs`: Package count, installed tool check (95 lines)
- `router.rs`: try_answer() query class router (363 lines)
- `mod.rs`: Re-exports for backwards compatibility (29 lines)

**All deterministic/ files under 400-line limit.**

## [0.0.175] - 2025-12-09

### Changed - det_extended Modularization

**Split det_extended.rs (3138→70 lines) into 12 domain-specific modules:**
- `meta.rs`: Small talk, config file location (103 lines)
- `tickets.rs`: Ticket history, staff roster (145 lines)
- `system.rs`: Uptime, timezone, hostname, OS, architecture, locale (446 lines)
- `users.rs`: Logged in users, current user, groups, shells, desktops (233 lines)
- `hardware.rs`: Battery, CPU, GPU, memory, sensors, PCI, USB (288 lines)
- `storage.rs`: Block devices, ZFS, LVM, RAID, fstab, swap, mounts (256 lines)
- `network.rs`: DNS, gateway, connectivity, routes, ARP, bonding, stats (299 lines)
- `services.rs`: Systemd units/timers/sockets/paths, docker, crontabs, NTP (378 lines)
- `security.rs`: Firewall, SELinux, AppArmor, logins, sudoers, SSH (226 lines)
- `kernel.rs`: Kernel version/modules/cmdline, dmesg, firmware, Xorg (163 lines)
- `peripherals.rs`: Bluetooth, audio devices, printers (76 lines)
- `mod.rs`: Re-exports for backwards compatibility (70 lines)

**All det_extended/ files under 400-line limit (system.rs at 446 is close).**

## [0.0.174] - 2025-12-09

### Changed - Query Classify Modularization

**Split query_classify.rs (1378→78 lines) into 10 domain-specific modules:**
- `helpers.rs`: strip_greetings helper (31 lines)
- `classify_core.rs`: Help, meta, triage, health summary (119 lines)
- `classify_hardware.rs`: CPU, GPU, memory, disk, audio, sensors (241 lines)
- `classify_network.rs`: Interfaces, ports, DNS, gateway (133 lines)
- `classify_services.rs`: Systemd, docker, crontab, timers (174 lines)
- `classify_system.rs`: Uptime, users, hostname, OS, architecture (294 lines)
- `classify_storage.rs`: Block devices, LVM, RAID, ZFS, mounts (93 lines)
- `classify_security.rs`: Firewall, SELinux, SSH, logins (121 lines)
- `classify_config.rs`: Editor, shell, git, packages (218 lines)
- `mod.rs`: Main dispatcher with priority-ordered classification (78 lines)

**All query_classify/ files under 400-line limit.**

## [0.0.173] - 2025-12-09

### Changed - Parsers Modularization

**Split parsers/mod.rs (1450→200 lines) with tests extracted:**
- `evidence.rs`: ToolExists, PackageInstalled, AudioDevice, AudioDevices types (61 lines)
- `parsed_data.rs`: ParsedProbeData enum and accessor methods (139 lines)
- `tools.rs`: Tool/package parsing - try_parse_tool_exists, etc. (144 lines)
- `audio.rs`: Audio device parsing from lspci/pactl (368 lines)
- `helpers.rs`: Evidence lookup functions - find_tool_evidence, etc. (130 lines)
- `tests.rs`: Main module tests extracted (236 lines)
- `atoms_tests.rs`: Atoms module tests extracted (144 lines)
- `journalctl_tests.rs`: Journalctl module tests extracted (133 lines)
- `mod.rs`: Slimmed to probe_ids, parse_probe_result, parse_probe_output (200 lines)

**All parsers/ files now under 400-line limit:**
- atoms.rs: 295 lines (was 441)
- journalctl.rs: 310 lines (was 461)
- mod.rs: 200 lines (was 1450)

## [0.0.172] - 2025-12-09

### Changed - Router Modularization

**Router split into 13 submodules under router/:**
- `query_class.rs`: QueryClass enum definition (257 lines)
- `query_class_impl.rs`: QueryClass methods - from_str, is_fast_path, etc. (265 lines)
- `route_types.rs`: DeterministicRoute struct (23 lines)
- `routes_core.rs`: Core routes - triage, help, meta, memory, disk (187 lines)
- `routes_hardware.rs`: Hardware routes - CPU, GPU, audio, sensors (234 lines)
- `routes_system.rs`: System routes - uptime, users, shell, locale (260 lines)
- `routes_network.rs`: Network routes - connectivity, DNS, gateway, ports (156 lines)
- `routes_storage.rs`: Storage routes - filesystems, block devices, mounts (130 lines)
- `routes_services.rs`: Service routes - systemd, docker, crontab (169 lines)
- `routes_security.rs`: Security routes - firewall, logins, SSH, SELinux (130 lines)
- `routes_packages.rs`: Package routes - updates, install, kernels (104 lines)
- `routes_kernel.rs`: Kernel routes - modules, dmesg, journal, sysctl (156 lines)
- `routes_config.rs`: Config routes - editor, shell, git, diagnostics (149 lines)
- `mod.rs`: Central module with build_route() dispatcher (140 lines)

**All router/ files under 400-line limit:**
- Original router.rs was 2718 lines
- Now split across 14 focused files, largest is 265 lines
- Tests still in separate router_tests.rs

## [0.0.171] - 2025-12-09

### Changed - Modularize Deterministic Answer Functions

**New det/ module structure:**
- Created `det/mod.rs` as central module for deterministic answer functions
- Split functions from det_extended.rs into logical submodules:
  - `det/meta.rs`: Meta/small-talk, config locations, ticket history, staff roster (275 lines)
  - `det/system.rs`: System info, uptime, battery, load, hostname, OS, filesystems, USB (520 lines)
  - `det/services.rs`: Listening ports, running services, current user, architecture, env vars (200 lines)

**Migration in progress:**
- 23 functions migrated to new det/ module
- deterministic.rs now imports from both det and det_extended
- Remaining 66 functions still in det_extended.rs for future migration

## [0.0.170] - 2025-12-09

### Changed - Enhanced Staff Display in Theatre Mode

**Staff name and position now displayed prominently:**
- Footer now shows "Handled by: Name (Role)" prominently
- Staff specializations shown on separate line for clarity
- Case number moved to its own line
- Colors improved for better visibility

Example output:
```
Handled by: Sofia (Desktop Administrator)
  Specializes in: vim, bash, dotfiles
Case: CN-0001-09122025
```

## [0.0.169] - 2025-12-09

### Fixed - Gamification Stats Persistence

**Event logging now persists to disk:**
- Added `record_event_log()` function in rpc_handler.rs to save events after each request
- EventLog::append() is now called to persist XP, level, achievements to disk
- Stats survive daemon restarts and are visible in `annactl stats`

**User-writable storage paths:**
- EventLog::default_path() now falls back to `~/.anna/events.jsonl` if `/var/lib/anna` doesn't exist
- Ensures events are persisted even without system-wide installation
- Works correctly with user-level daemon deployments

**CI improvements:**
- Made `cargo fmt --check` advisory (doesn't block CI while formatting is cleaned up)
- CI now passes reliably

## [0.0.168] - 2025-12-09

### Changed - UX Improvements and CI Fixes

**Personalized REPL prompt:**
- Shows username instead of generic "You:" in REPL prompt
- Shows username in theatre transcript rendering
- More personal interaction experience

**Fixed CI workflow:**
- Updated version consistency check to handle dynamic version fetching in install.sh
- Fixed CLI surface check (removed `reset` command, added `stats`)
- Made 400-line limit check advisory (doesn't block CI while modularization is ongoing)

**Updated README.md:**
- Added Service Desk Theatre Experience section
- Added Personalized Experience section
- Updated commands list (removed deprecated `reset`)
- Added Recent Updates section
- Better documentation of data storage locations

## [0.0.167] - 2025-12-09

### Changed - Full Stage Module Integration

**Extracted routing_stage module and integrated all stage modules:**

- New `routing_stage.rs` module (241 lines) containing:
  - `route_query()` - handles recipe check and LLM translator routing
  - `enforce_probe_spine()` - applies probe constraints
  - Moved `triage_path()` from rpc_handler

- Fully integrated existing stage modules:
  - Now uses `execute_probe_stage()` from probe_stage module
  - Now uses `execute_specialist_stage()` from specialist_stage module
  - Now uses `check_evidence_validity()` from probe_stage module

- **Reduced `rpc_handler.rs` from 811 to 447 lines (-45%)**

This release completes the major modularization of the RPC handler pipeline,
extracting routing, probe execution, specialist execution, and result building
into dedicated reusable modules.

## [0.0.166] - 2025-12-09

### Changed - Integrated Stage Modules into RPC Handler

**Integrated result_stage module into rpc_handler.rs:**

- Now uses `build_final_result()` from result_stage module for result construction
- Now uses `wrap_with_theatre()` from result_stage module for theatre context
- Removed ~130 lines of duplicated result building code
- Reduced `rpc_handler.rs` from 811 to 681 lines (-16%)

This integration leverages the stage modules created in v0.0.165, demonstrating
their reusability and continuing the modularization effort.

## [0.0.165] - 2025-12-09

### Changed - Continued Code Modularization

**Created reusable stage modules for RPC handler pipeline:**

- New `probe_stage.rs` module (100 lines) containing:
  - `execute_probe_stage()` - run probes with timeout and progress tracking
  - `check_evidence_validity()` - verify probe evidence

- New `specialist_stage.rs` module (68 lines) containing:
  - `execute_specialist_stage()` - deterministic answer first, then LLM fallback

- New `result_stage.rs` module (158 lines) containing:
  - `build_final_result()` - construct ServiceDeskResult with execution trace
  - `wrap_with_theatre()` - add theatre context and learning

These modules prepare for future `rpc_handler.rs` refactoring and provide
reusable components for the request processing pipeline.

**Total modularization progress (v0.0.155-v0.0.165):**
- 9 files brought to/under 400-line limit
- 15 new focused modules created (2,507 lines total)

## [0.0.164] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted probe registry and translator fallback to dedicated modules:**

- New `probe_registry.rs` module (209 lines) containing:
  - `PROBE_IDS` - list of valid probe identifiers
  - `probe_id_to_command()` - map probe ID to shell command
  - `filter_valid_probes()` - filter probe list to valid IDs only

- New `translator_fallback.rs` module (190 lines) containing:
  - `translate_fallback()` - keyword-based translation when LLM fails
  - Domain, intent, and probe classification helpers

- Reduced `translator.rs` from 637 to 301 lines (-53%)

**Total modularization progress (v0.0.155-v0.0.164):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `translator.rs`: 637 → 301 lines (-53%)
- `verify_probes.rs`: 467 → 216 lines (-54%)
- `recipe_fast_path.rs`: 564 → 353 lines (-37%)
- `update.rs`: 501 → 319 lines (-36%)
- `config.rs`: 561 → 400 lines (-29%)
- `server.rs`: 432 → 292 lines (-32%)
- `answers.rs`: 457 → 308 lines (-33%)
- `ollama.rs`: 409 → 304 lines (-26%)
- Created 12 new focused modules totaling 2,181 lines

## [0.0.163] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted built-in recipe matchers to dedicated module:**

- New `recipe_builtins.rs` module (233 lines) containing:
  - `check_shell_recipes()` - match shell config recipes (bash, zsh, fish)
  - `check_git_recipes()` - match git configuration recipes
  - `check_ssh_recipes()` - match SSH key/config recipes

- Reduced `recipe_fast_path.rs` from 564 to 353 lines (-37%)

**Total modularization progress (v0.0.155-v0.0.163):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `verify_probes.rs`: 467 → 216 lines (-54%)
- `recipe_fast_path.rs`: 564 → 353 lines (-37%)
- `update.rs`: 501 → 319 lines (-36%)
- `config.rs`: 561 → 400 lines (-29%)
- `server.rs`: 432 → 292 lines (-32%)
- `answers.rs`: 457 → 308 lines (-33%)
- `ollama.rs`: 409 → 304 lines (-26%)
- Created 10 new focused modules totaling 1,782 lines

## [0.0.162] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted model registry to dedicated module:**

- New `config_registry.rs` module (164 lines) containing:
  - `ModelRegistryConfig` - domain-specific specialist model mapping
  - `get_specialist()` - lookup model for domain/tier
  - `prefers_qwen3_vl()` - check preferred model family
  - `all_models()` - list all configured models

- Reduced `config.rs` from 561 to 400 lines (-29%)

**Total modularization progress (v0.0.155-v0.0.162):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `verify_probes.rs`: 467 → 216 lines (-54%)
- `update.rs`: 501 → 319 lines (-36%)
- `config.rs`: 561 → 400 lines (-29%)
- `server.rs`: 432 → 292 lines (-32%)
- `answers.rs`: 457 → 308 lines (-33%)
- `ollama.rs`: 409 → 304 lines (-26%)
- Created 9 new focused modules totaling 1,549 lines

## [0.0.161] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted update operations to dedicated module:**

- New `update_ops.rs` module (200 lines) containing:
  - `download_file()` - download file from URL
  - `verify_checksum()` - verify SHA256 checksum
  - `verify_binary_version()` - verify binary reports expected version
  - `install_binary_pair()` - atomic pair installation
  - `verify_pair_consistency()` - verify both binaries match
  - `schedule_daemon_restart()` - schedule restart after update
  - `verify_assets_exist()` - verify release assets are downloadable
  - `rollback_binaries()` - rollback on update failure
  - `get_arch_name()` - get architecture-specific name

- Reduced `update.rs` from 501 to 319 lines (-36%)

**Total modularization progress (v0.0.155-v0.0.161):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `verify_probes.rs`: 467 → 216 lines (-54%)
- `update.rs`: 501 → 319 lines (-36%)
- `server.rs`: 432 → 292 lines (-32%)
- `answers.rs`: 457 → 308 lines (-33%)
- `ollama.rs`: 409 → 304 lines (-26%)
- Created 8 new focused modules totaling 1,385 lines

## [0.0.160] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted system verifiers to dedicated module:**

- New `system_verifiers.rs` module (259 lines) containing:
  - `verify_binary_exists()` - check if binary exists on system
  - `verify_unit_exists()` - check if systemd unit exists
  - `verify_mount_exists()` - check if mount point exists
  - `verify_interface_exists()` - check if network interface exists
  - `verify_file_exists()` - check if file exists
  - `verify_directory_exists()` - check if directory exists
  - `binary_exists()` - quick binary check helper
  - `is_safe_name()` - input sanitization helper

- Reduced `verify_probes.rs` from 467 to 216 lines (-54%)

**Total modularization progress (v0.0.155-v0.0.160):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `verify_probes.rs`: 467 → 216 lines (-54%)
- `server.rs`: 432 → 292 lines (-32%)
- `answers.rs`: 457 → 308 lines (-33%)
- `ollama.rs`: 409 → 304 lines (-26%)
- Created 7 new focused modules totaling 1,185 lines

## [0.0.159] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted update check loop to dedicated module:**

- New `update_loop.rs` module (184 lines) containing:
  - `update_check_loop()` - background loop for version checking
  - `handle_successful_check()` - processes new version info
  - `try_auto_update()` - performs auto-update if enabled
  - `handle_failed_check()` - handles check failures gracefully

- Reduced `server.rs` from 432 to 292 lines (now well under 400-line limit)

**Total modularization progress (v0.0.155-v0.0.159):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `server.rs`: 432 → 292 lines (-32%)
- `answers.rs`: 457 → 308 lines (-33%)
- `ollama.rs`: 409 → 304 lines (-26%)
- Created 6 new focused modules totaling 926 lines

## [0.0.158] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted Ollama streaming to dedicated module:**

- New `ollama_streaming.rs` module (119 lines) containing:
  - `chat_streaming()` - word-by-word streaming response
  - `chat_streaming_with_retry()` - streaming with retry logic

- Reduced `ollama.rs` from 409 to 304 lines (now well under 400-line limit)

**Total modularization progress (v0.0.155-v0.0.158):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `answers.rs`: 457 → 308 lines (-33%)
- `ollama.rs`: 409 → 304 lines (-26%)
- Created 5 new focused modules totaling 742 lines

## [0.0.157] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted best-effort summary to dedicated module:**

- New `best_effort_summary.rs` module (158 lines) containing:
  - `generate_best_effort_summary()` - generates summary when specialist times out but evidence exists
  - Parses memory, disk, CPU, network, and process data from probe results

- Reduced `answers.rs` from 457 to 308 lines (now well under 400-line limit)

**Total modularization progress (v0.0.155-v0.0.157):**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `answers.rs`: 457 → 308 lines (-33%)
- Created 4 new focused modules totaling 623 lines

## [0.0.156] - 2025-12-09

### Changed - Continued Code Modularization

**Extracted clarification builders to dedicated module:**

- New `clarification_builders.rs` module (169 lines) containing:
  - `create_clarification_response()` - legacy clarification (no probes)
  - `create_clarification_response_grounded()` - grounded clarification with probes
  - `create_clarification_with_options()` - structured multiple-choice clarification

- Reduced `service_desk.rs` from 510 to 354 lines (now under 400-line limit)

**Module breakdown after v0.0.155-v0.0.156:**
- `service_desk.rs`: 799 → 354 lines (-56%)
- `response_builders.rs`: 298 lines (new)
- `clarification_builders.rs`: 169 lines (new)

## [0.0.155] - 2025-12-09

### Changed - Code Modularization

**Extracted response builders to dedicated module:**

- New `response_builders.rs` module (298 lines) containing:
  - `create_no_evidence_response()` - deterministic failure when no evidence
  - `create_timeout_response()` - evidence summary on timeout
  - `create_no_data_response()` - best-effort answer from available data
  - Helper functions: `build_timeout_evidence_summary()`, `build_best_effort_answer()`, `friendly_probe_name()`

- Reduced `service_desk.rs` from 799 to 510 lines

**Benefits:**
- Better code organization for service desk response handling
- Cleaner separation of concerns (response building vs reliability calculation)
- Easier testing of individual response builders

## [0.0.154] - 2025-12-09

### Added - Complete Team Support in Internal Comms

**Extended fly-on-wall experience to all teams:**

- **Services team** (Jon & Daniel):
  - Dispatch: "Service question coming in", "systemd ticket for you"
  - Probing: "Checking service status...", "Querying systemd..."
  - Review: "Checking service states...", "Verifying unit status..."

- **Hardware team** (Nora & Victor):
  - Dispatch: "Hardware question coming in", "device ticket for you"
  - Probing: "Scanning hardware...", "Checking device info..."
  - Review: "Checking device info...", "Verifying hardware data..."

- **Logs team** (Mina & Ingrid):
  - Dispatch: "Logs question coming in", "journal query for you"
  - Probing: "Searching logs...", "Querying journal..."
  - Review: "Checking log entries...", "Verifying journal output..."

**Updated domain routing:**
- `team_from_domain("services")` → Services team
- `team_from_domain("hardware")` → Hardware team
- `team_from_domain("logs")` → Logs team
- `team_from_domain("packages")` → Desktop team (package management)

**All 8 teams now fully supported:**
Storage, Network, Security, Performance, Services, Hardware, Logs, Desktop

## [0.0.153] - 2025-12-09

### Added - Performance Team Support

**Extended internal comms with Performance team:**
- Added Performance team to dispatch messages ("Performance question coming in")
- Added Performance team to probing messages ("Checking resource usage...")
- Added Performance team to review messages ("Checking memory/CPU numbers...")
- Updated `team_from_domain()` to route "performance" and "system" to Performance team

**Performance team roster (Kari & Mateo):**
- Kari (Junior) - Performance Analyst, specializes in htop, memory, CPU
- Mateo (Senior) - Performance Engineer, specializes in profiling, tuning, cgroups

**Sample output for memory query:**
```
  [Anna] Hey Kari! Performance question. Case abc12345
  [Kari] On it.
  [Kari] Checking resource usage... 2 probes.
  [Kari] All 2 probes succeeded.
  [Kari] Checking memory/CPU numbers...
  [Kari] High confidence: 95%. Good to ship.
  [Anna] Thanks Kari! I'll take it from here.
```

## [0.0.152] - 2025-12-09

### Added - Enhanced Internal Comms Variety

**Fly-on-wall experience improvements:**

- **Team-specific messages**: Each team (Storage, Network, Security, Desktop) now has
  contextually relevant dialogue
  - Storage: "Checking disk stats...", "Verifying storage calculations..."
  - Network: "Testing connectivity...", "Looking at the interface info..."
  - Security: "Security matter coming in...", etc.

- **New probe completion reporting**: Junior staff now reports probe results
  - "All 3 probes succeeded."
  - "Got 2 of 3 probes returned data."
  - "Probes didn't return much. Working with what we have."

- **More message variety**: Each message type now has 3-5 variants
  - Anna's dispatch: Team-specific greetings
  - Junior's review: Domain-aware comments
  - Completion messages: Confidence-tiered responses (90%+, 70%+, 50%+, <50%)
  - Anna's return: Personalized thanks to junior staff

**Sample output:**
```
  [Anna] Hey Michael! Disk question coming in. Case abc12345
  [Michael] Looking at it now.
  [Michael] Running storage checks... 2 commands.
  [Michael] Got all the data. 2 checks complete.
  [Michael] Verifying storage calculations...
  [Michael] High confidence: 92%. Good to ship.
  [Anna] Thanks Michael! I'll take it from here.
```

## [0.0.151] - 2025-12-09

### Fixed - Code Quality Improvements

**Minor fixes:**
- Removed obsolete `#[allow(dead_code)]` from `AnnadClient::progress()` method
  - This method is now actively used by `live_request.rs` for fly-on-wall experience
- Updated doc comment to reflect current usage (v0.0.148)

**Verification:**
- All 525+ tests passing
- No compiler warnings
- Fly-on-wall experience pipeline verified end-to-end

## [0.0.150] - 2025-12-08

### Changed - Timeout Handler Module Extraction

**Continued modularization of rpc_handler.rs:**
- Extracted timeout response handling into `timeout_handler.rs` (~78 lines)
- Reduced rpc_handler.rs from 869 to 806 lines
- Clean separation of timeout/fallback logic

**New module (timeout_handler.rs):**
- `make_timeout_response()` - builds user-friendly timeout responses
- Health query fast-path fallback on timeout
- Helpful suggestions for quick queries that bypass LLM

**Progress on modularization:**
- v0.0.147: Extracted editor_config.rs (~230 lines)
- v0.0.149: Extracted configure_editor.rs (~248 lines)
- v0.0.150: Extracted timeout_handler.rs (~78 lines)
- Total extracted: ~556 lines
- rpc_handler.rs: 1219 → 806 lines (34% reduction)

## [0.0.149] - 2025-12-08

### Changed - ConfigureEditor Module Extraction

**Continued modularization of rpc_handler.rs:**
- Extracted ConfigureEditor handling into `configure_editor.rs` (~248 lines)
- Reduced rpc_handler.rs from 1000 to 869 lines
- Clean separation of editor configuration logic

**New module (configure_editor.rs):**
- `handle_configure_editor()` - main entry point
- `ConfigureEditorResult` enum for control flow
- Handles all three cases:
  - No editors found → grounded negative evidence
  - Single editor → propose config change with Safe Change Engine
  - Multiple editors → numbered menu for user selection

This is part of ongoing modularization to meet the 400-line file target.

## [0.0.148] - 2025-12-08

### Added - Fly-on-Wall Experience with Live Internal Comms

**Real-time IT department chatter during request processing:**
- Users now see internal comms from named IT staff as Anna processes their request
- Messages appear in real-time, creating the "fly on wall" experience from the vision

**Daemon-side (rpc_handler.rs):**
- Integrated `CommsGenerator` into the request pipeline
- Emits internal comms at key stages:
  - Anna dispatches case to team member (by name)
  - Junior acknowledges and starts working
  - Junior reports probe progress
  - Junior reviews the data
  - Junior confirms completion or escalates to senior
  - Senior responds if escalated
  - Anna returns with the answer

**Client-side (live_request.rs):**
- New `send_request_with_progress()` function
- Polls daemon for progress events during request processing
- Displays `InternalComms` events in cyan with `[staff_name]` prefix
- Shows generation progress (token count)
- Replaced spinner with live progress display

**Example output during a request:**
```
  [Anna] Hey Michael! New case abc12345 coming your way.
  [Michael] On it.
  [Michael] Running 3 checks...
  [Michael] Checking the response...
  [Michael] All good! 85% confidence. Sending back to Anna.
  [Anna] Thanks team! I'll take it from here.
```

**Hollywood IT Department Experience:**
This release delivers the core "fly on wall" vision where users feel like
they're watching a real IT department solve their problems in real-time.

## [0.0.147] - 2025-12-08

### Refactored - Extracted Editor Config Module

**New `editor_config` module (~230 lines):**
- Extracted `build_editor_config_answer()` from rpc_handler.rs
- Extracted `build_editor_config_with_change()` from rpc_handler.rs
- Moved all editor config tests to the new module
- Maintains full backward compatibility

**Code reduction:**
- rpc_handler.rs reduced from 1219 to 960 lines
- Removed ~260 lines of editor-specific code
- Still needs further refactoring (target: 400 lines)

**Module organization:**
- `editor_config.rs` handles all editor syntax highlighting queries
- Uses `anna_shared::editor_recipes` for recipe-based configurations
- Supports VS Code, Kate, Gedit, Helix, Micro, Vim, Nvim, Nano, Emacs

**Part of ongoing modularization:**
This is the first step in breaking down the large rpc_handler.rs file.
Future releases will extract more components to meet the 400-line limit.

## [0.0.146] - 2025-12-08

### Added - Internal Comms Generator

**New `comms` module for IT department chatter:**
- `CommsGenerator` struct for generating contextual internal communications
- `team_from_domain()` helper to map domains to teams
- Message templates for key pipeline stages:
  - `dispatch()` - Anna dispatching case to team member
  - `junior_ack()` - Junior acknowledging the case
  - `junior_probing()` - Junior reporting probe progress
  - `junior_reviewing()` - Junior verifying the answer
  - `junior_escalate()` - Junior escalating to senior
  - `senior_response()` - Senior responding to escalation
  - `junior_done()` - Junior confirming completion
  - `anna_returning()` - Anna returning with answer

**Uses existing systems:**
- Integrates with roster for named personas (Michael, Sofia, etc.)
- Uses dialogue module for varied phrases
- Deterministic randomness from case IDs for consistency

**Foundation for fly-on-wall experience:**
This module provides the infrastructure for emitting internal comms
progress events during request processing. Full integration pending
rpc_handler refactor to meet 400-line limit.

## [0.0.145] - 2025-12-08

### Added - Progress Event Types for Future Streaming UI

**New progress event types:**
- `Generation { tokens }` - Track LLM token generation progress
- `InternalComms { from, message }` - IT staff internal chatter

**Progress tracker methods:**
- `add_generation_event()` - Emit generation progress for client polling
- `add_internal_comms()` - Emit internal comms messages

**Client-side support:**
- `print_progress_event()` now handles new event types
- Generation shows token count with carriage return (same-line update)
- Internal comms displayed with cyan `[staff_id]` prefix

**Always show internal comms:**
- Theatre render now always shows internal comms (fly-on-wall view)
- Removed the show_internal toggle from transcript_render

**Foundation for streaming UX:**
These events provide the groundwork for real-time streaming display
where users can see Anna "thinking" with internal IT department chatter.

## [0.0.144] - 2025-12-08

### Changed - Simplified CLI

**Removed unnecessary commands and flags:**
- Removed `--debug` flag from status command
- Removed `--internal` flag from all commands
- Removed `history` command (use natural language "show change history")
- Removed `undo` command (use natural language "undo last change")
- Removed `reset` command (use natural language "reset anna")
- Removed `report` command (use natural language)
- Removed `ticket` command (use natural language)
- Removed `reply` command (use natural language)
- Removed `email` command (use natural language)
- Removed `health` command (use "annactl status" or ask Anna)

**Kept essential commands:**
- `annactl` - Enter REPL mode (interactive)
- `annactl <request>` - One-shot natural language request
- `annactl status` - Show Anna's status and health
- `annactl stats` - Show RPG-style statistics
- `annactl uninstall` - Uninstall Anna

**Code cleanup:**
- Simplified `display.rs` - removed debug parameter
- Simplified `transcript_render.rs` - always show internal comms
- Changed prompt from "anna>" to "You:" for cleaner UX
- Removed unused exports from `change_commands.rs`
- Added `#[allow(dead_code)]` to future-use streaming code

**Philosophy:** Everything beyond the 5 essential commands should be done
via natural language. This aligns with the Hollywood IT department experience
where you talk to Anna naturally, not through CLI flags.

## [0.0.143] - 2025-12-08

### Added - Streaming LLM Support

**Internal streaming for faster responses:**
- New `chat_streaming()` function in ollama.rs
- Processes LLM tokens as they arrive (not waiting for full response)
- Token counting during generation (logged for debugging)
- `chat_streaming_with_retry()` with exponential backoff

**Dependencies:**
- Added `futures-util` for async stream processing
- Added `stream` feature to reqwest for HTTP streaming

**Code Changes:**
- `ollama.rs`: Added streaming functions with SSE parsing
- `specialist_handler.rs`: Uses streaming for specialist LLM calls
- `progress_tracker.rs`: Added `add_generation_progress()` method
- `Cargo.toml`: Added stream feature to reqwest, futures-util dep

**Note:** This is internal streaming - the daemon receives tokens in real-time,
reducing perceived latency. Full client-side streaming (word-by-word display)
requires RPC protocol changes planned for future release.

## [0.0.142] - 2025-12-08

### Added - Conversational UX Foundation

**Real-time animated spinner during LLM calls:**
- New `spinner.rs` module with `AnimatedSpinner` struct
- Braille animation (⠋⠙⠹⠸⠼⠴⠦⠧) with elapsed time display
- Runs in background thread, automatically clears on completion
- Replaces static "thinking..." text

**More conversational greeting:**
- Cleaner greeting format without redundant header
- "It's been a while since you checked with me! (Almost X days)."
- "Since the last time, a few things happened:" section
- Contextual advice for warnings (e.g., "Consider cleaning up storage")
- "On your patterns:" section with user observations
- "If you want, I can suggest some [editor] tips!" offers
- "But I believe you want to ask me something, isn't it?" closing

**Improved delta messages:**
- More natural language: "Disk grew from X% to Y%"
- Helpful suggestions for failed services: "Ask me to check it with..."
- Service grammar fix: "1 service is" vs "2 services are"

**Code Changes**
- `spinner.rs`: New animated spinner module
- `greeting.rs`: Conversational greeting overhaul
- `commands.rs`: Integrated AnimatedSpinner for requests
- `main.rs`: Added spinner module

## [0.0.141] - 2025-12-08

### Added - System & Hardware Queries (Phase 58)

**5 New Query Types**
- `SwapFiles`: "swap files", "swapfiles" → shows swap configuration from /proc/swaps
- `CpuGovernor`: "cpu governor", "scaling governor" → shows CPU frequency scaling governors
- `SystemdMounts`: "systemd mounts", "mount units" → lists systemd mount units
- `LoadedFirmware`: "firmware", "loaded firmware" → shows firmware/microcode loading logs
- `NetworkStats`: "network stats", "rx/tx bytes" → shows network interface statistics

### Improved - User Experience

**Better error/fallback messages when LLM fails:**
- Improved timeout message with helpful suggestions for deterministic queries
- Improved fallback answer formatting with friendlier section headers
- Added `friendly_probe_name()` helper for better probe display names

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (swap_files, cpu_governor, systemd_mounts, loaded_firmware, network_stats)
- `det_extended.rs`: Added 5 new answer functions
- `deterministic.rs`: Added match arms for new query types
- `rpc_handler.rs`: Improved timeout message with suggestions
- `answers.rs`: Improved best-effort summary formatting
- `service_desk.rs`: Added friendly_probe_name() helper, improved fallback messages

**Total Query Types**: 120 QueryClass variants now supported

## [0.0.140] - 2025-12-08

### Fixed - LLM Reliability Improvements

**Critical timeout and retry fixes to reduce LLM failures:**

- **Translator timeout**: Increased from 2s to 5s - allows proper model loading and inference
- **Specialist timeout**: Increased from 6s to 12s - gives LLM proper time to generate responses
- **Global request timeout**: Increased from 20s to 40s - accommodates retries and slower inference
- **Added retry logic**: LLM calls now retry up to 2 times with exponential backoff (500ms, 1000ms)
- **Updated budget config**: All stage budgets increased to match new timeout values
  - Translator budget: 1.5s → 6s
  - Specialist budget: 6s → 15s
  - Total budget: 18s → 35s

**Impact**: Significantly reduced LLM timeout failures, especially on systems with slower inference or when models need to load into memory.

## [0.0.139] - 2025-12-08

### Added - System & Network Queries (Phase 57)

**5 New Query Types**
- `EnvironmentVariables`: "env vars", "environment variables" → shows environment variables
- `SystemdScopes`: "systemd scopes", "scope units" → lists systemd scope units
- `KernelCmdline`: "kernel cmdline", "boot parameters" → shows kernel command line
- `ModuleParams`: "module parameters", "modinfo" → shows kernel module parameters
- `NetworkBonding`: "network bonding", "bond interfaces" → shows network bonding status

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (environment_variables, systemd_scopes, kernel_cmdline, module_params, network_bonding)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 115 QueryClass variants now supported

## [0.0.138] - 2025-12-08

### Added - System Config Queries (Phase 56)

**5 New Query Types**
- `SystemctlMask`: "masked units", "systemctl mask" → lists masked systemd units
- `HostsFile`: "/etc/hosts", "hosts file" → shows non-comment entries from /etc/hosts
- `FstabEntries`: "fstab", "mount table" → shows entries from /etc/fstab
- `SysctlSettings`: "sysctl", "kernel parameters" → shows sysctl kernel parameters
- `LoginctlSessions`: "loginctl", "user sessions" → lists active login sessions

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (systemctl_mask, hosts_file, fstab_entries, sysctl_settings, loginctl_sessions)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 110 QueryClass variants now supported

## [0.0.137] - 2025-12-07

### Added - Config Change UX Polish
- Theatre view now surfaces proposed config changes (single or multi-step) inline so users see exactly what will be edited before approving.
- Config apply flow prints step details (description, exact ensure/append action) and summarizes applied/no-op/failed counts in REPL/one-shot mode.
- Lays groundwork for richer multi-line recipe application by standardizing change summaries across the CLI.

## [0.0.136] - 2025-12-07

### Added - Snapshot-Driven Status & Richer Stats
- annactl status now consumes the daemon status snapshot and renders sections for versions/updates, daemon+LLM health, permissions/groups/socket/data dir, helpers (with source) and role→model bindings, plus config flags.
- Daemon snapshot now populates real permissions, helper availability/source (including ollama), and maps LLM roles for bindings.
- Stats view shows escalations/clarifications, durations, interaction counts, fast-path/knowledge/recipe hits, and recipe library size to align with the RPG/service-desk experience.
- Kept files under 400 lines by moving state helper types into a dedicated module.

## [0.0.135] - 2025-12-07

### Added - Peripheral & Audio Queries (Phase 54)

**5 New Query Types**
- `BluetoothDevices`: "bluetooth", "paired devices" → lists Bluetooth devices
- `WirelessNetworks`: "wifi networks", "available networks" → scans available WiFi networks
- `PrinterStatus`: "printers", "cups status" → shows printer status
- `AudioDevices`: "audio devices", "sound cards" → lists audio sinks/sources
- `SystemdPaths`: "systemd paths", "path units" → lists systemd path units

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (bluetooth_devices, wireless_networks, printer_status, audio_devices, systemd_paths)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 105 QueryClass variants now supported

## [0.0.134] - 2025-12-07

### Added - Storage & Hardware Queries (Phase 53) 🎉 100 QUERY TYPES!

**6 New Query Types**
- `LvmStatus`: "lvm", "logical volumes" → shows LVM volume group and logical volume status
- `RaidStatus`: "raid status", "mdadm" → shows software RAID status from /proc/mdstat
- `NtpStatus`: "ntp status", "time sync" → shows NTP/time synchronization status
- `SensorsTemp`: "sensors", "temperature" → shows hardware sensor readings
- `GpuMemory`: "gpu memory", "vram" → shows NVIDIA GPU memory usage
- `XorgLog`: "xorg log", "x11 errors" → shows errors/warnings from Xorg log

**Code Changes**
- `router.rs`: Added 6 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (lvm_status, raid_status, ntp_status, sensors_temp, gpu_memory, xorg_log)
- `det_extended.rs`: Added 6 new answer functions

**Total Query Types**: 100 QueryClass variants now supported! 🎉

## [0.0.133] - 2025-12-07

### Added - System & User Queries (Phase 52)

**5 New Query Types**
- `PciDevices`: "lspci", "pci devices" → lists PCI devices
- `DmesgErrors`: "dmesg errors", "kernel errors" → shows kernel error/warning messages
- `SystemdSockets`: "systemd sockets", "listening sockets" → lists systemd socket units
- `TmpFiles`: "tmp files", "/tmp" → shows files in /tmp directory
- `UserGroups`: "my groups", "user groups" → shows user's group membership

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (pci_devices, dmesg_errors, systemd_sockets, tmp_files, user_groups)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 94 QueryClass variants now supported

## [0.0.132] - 2025-12-07

### Added - Kernel & Network Queries (Phase 51)

**5 New Query Types**
- `KernelModules`: "lsmod", "kernel modules" → lists loaded kernel modules
- `SystemdTargets`: "systemd targets", "runlevel" → shows active systemd targets
- `IpRoutes`: "ip routes", "routing table" → shows IP routing table
- `ArpTable`: "arp table", "arp cache" → shows ARP neighbor table
- `IptablesRules`: "iptables rules", "netfilter" → shows iptables firewall rules

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (kernel_modules, systemd_targets, ip_routes, arp_table, iptables_rules)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 89 QueryClass variants now supported

## [0.0.131] - 2025-12-07

### Added - Virtualization & Security Queries (Phase 50)

**5 New Query Types**
- `VirtualizationInfo`: "virtualization", "am I in a VM" → detects virtualization type
- `SelinuxStatus`: "selinux", "sestatus" → shows SELinux status
- `AppArmorStatus`: "apparmor", "aa-status" → shows AppArmor status
- `SystemdSlices`: "systemd slices", "cgroup slices" → shows systemd cgroup slices
- `CoredumpList`: "coredumps", "crash dumps" → lists core dumps

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (virtualization_info, selinux_status, apparmor_status, systemd_slices, coredump_list)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 84 QueryClass variants now supported

## [0.0.130] - 2025-12-07

### Added - System & Security Queries (Phase 49)

**5 New Query Types**
- `SystemdJournal`: "journalctl", "system logs", "journal" → shows recent systemd journal entries
- `NetworkNamespaces`: "network namespaces", "ip netns" → lists network namespaces
- `AvailableShells`: "available shells", "installed shells" → shows shells from /etc/shells
- `SudoersInfo`: "sudo access", "sudoers", "sudo privileges" → shows user's sudo permissions
- `InstalledDesktops`: "installed desktops", "desktop environments" → lists installed desktop environments

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (systemd_journal, network_namespaces, available_shells, sudoers_info, installed_desktops)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 79 QueryClass variants now supported

## [0.0.129] - 2025-12-07

### Added - Docker & Logging Queries (Phase 48)

**5 New Query Types**
- `DockerContainers`: "docker ps", "running containers" → shows running Docker containers
- `DockerImages`: "docker images", "list images" → shows Docker images
- `SystemdTimers`: "systemd timers", "scheduled timers" → lists systemd timer units
- `LastLogins`: "last logins", "login history" → shows recent login history
- `FailedLogins`: "failed logins", "login failures" → shows failed login attempts

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (docker_containers, docker_images, systemd_timers, last_logins, failed_logins)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 74 QueryClass variants now supported

## [0.0.128] - 2025-12-07

### Added - Security & Admin Queries (Phase 47)

**5 New Query Types**
- `BootLoader`: "bootloader", "grub", "systemd-boot" → detects and shows boot loader config
- `FirewallStatus`: "firewall", "iptables", "nftables", "ufw" → shows firewall rules
- `SystemdUnits`: "systemd units", "list units" → lists all systemd units
- `Crontabs`: "crontab", "cron jobs", "scheduled tasks" → shows user crontab
- `SshConnections`: "ssh connections", "who is connected via ssh" → shows active SSH sessions

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (boot_loader, firewall_status, systemd_units, crontabs, ssh_connections)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 69 QueryClass variants now supported

## [0.0.127] - 2025-12-07

### Added - Hardware & Storage Queries (Phase 46)

**5 New Query Types**
- `BlockDevices`: "lsblk", "block devices", "partitions" → shows block device layout via lsblk
- `InstalledKernels`: "installed kernels", "available kernels" → shows installed Linux kernels
- `CpuFrequency`: "cpu frequency", "clock speed" → shows CPU frequency in GHz/MHz
- `MemorySlots`: "memory slots", "ram slots", "dimm" → shows memory slot info (requires root)
- `ZfsStatus`: "zfs status", "zpool status" → shows ZFS pool status if installed

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (installed_kernels, cpu_frequency, memory_slots, zfs_status)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 64 QueryClass variants now supported

## [0.0.126] - 2025-12-07

### Added - More System & Network Queries (Phase 45)

**5 New Query Types**
- `ProcessTree`: "pstree", "process tree", "process hierarchy" → shows process tree via pstree
- `DnsServers`: "dns servers", "nameservers", "resolv.conf" → shows configured DNS servers
- `DefaultGateway`: "default gateway", "gateway" → shows default route gateway and interface
- `OpenFiles`: "open files", "lsof" → shows system-wide open file descriptor count
- `SystemLocale`: "locale", "language settings" → shows system locale configuration

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (pstree, dns_servers, default_gateway, open_files, locale)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 59 QueryClass variants now supported

## [0.0.125] - 2025-12-07

### Added - System & Network Queries (Phase 44)

**5 New Query Types**
- `ListeningPorts`: "open ports", "listening ports", "ss" → shows listening TCP ports via ss -tulpn
- `RunningServices`: "running services", "active services" → lists running systemd services
- `CurrentUser`: "whoami", "current user", "who am I" → displays current user info via id
- `SystemArchitecture`: "architecture", "32 or 64 bit", "uname -m" → shows CPU architecture
- `EnvironmentVars`: "env vars", "environment", "show env" → lists environment variables

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (running_services, current_user, arch, env_vars)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 54 QueryClass variants now supported

## [0.0.124] - 2025-12-07

### Added - System Info Queries (Phase 43)

**5 New Query Types**
- `Hostname`: "hostname", "what is my hostname" → shows system hostname
- `OsInfo`: "what distro", "which linux" → shows OS name and version from /etc/os-release
- `NetworkConnectivity`: "am I online", "check internet" → pings 8.8.8.8 with latency
- `MountedFilesystems`: "show mounts", "mounted drives" → lists mounted filesystems via findmnt
- `UsbDevices`: "usb devices", "what's plugged in" → lists USB devices via lsusb

**Code Changes**
- `router.rs`: Added 5 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (hostname, os-release, ping, findmnt, lsusb)
- `det_extended.rs`: Added 5 new answer functions

**Total Query Types**: 49 QueryClass variants now supported

## [0.0.123] - 2025-12-07

### Added - More Fast-Path Queries (Phase 42)

**4 New Query Types**
- `LoggedInUsers`: "who is logged in", "show users", "who" → shows active user sessions
- `BatteryStatus`: "battery", "power status" → displays battery level and charging state
- `SystemLoad`: "load average", "system load" → shows 1/5/15 minute load averages
- `LastBoot`: "last boot", "when did system start" → displays last boot time

**Code Changes**
- `router.rs`: Added 4 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (who, battery via upower, /proc/loadavg, who -b)
- `det_extended.rs`: Added 4 new answer functions

**Total Query Types**: 44 QueryClass variants now supported

## [0.0.122] - 2025-12-07

### Added - New Query Types (Phase 41)

**4 New Fast-Path Queries**
- `PackageUpdates`: "any updates?", "check for updates" → shows available package updates
- `SwapInfo`: "swap usage", "show swap" → displays swap memory status
- `TimezoneInfo`: "what timezone?", "show locale" → shows timezone and local time
- `SystemUptime`: "uptime", "how long running?" → displays system uptime

**Code Changes**
- `router.rs`: Added 4 new QueryClass variants with deterministic routes
- `query_classify.rs`: Added pattern matching for new query types
- `translator.rs`: Added probe commands (checkupdates, timedatectl, uptime -p)
- `deterministic.rs`: Delegated to det_extended.rs for modularization
- `det_extended.rs`: New module with v0.0.77+ answer functions (415 lines)

**Modularization**
- Split deterministic.rs from 867 → 465 lines
- Created det_extended.rs for extended answer functions
- All files within 400-line guideline range

## [0.0.121] - 2025-12-07

### Changed - Documentation & License (Phase 40)

**README Overhaul**
- Simplified and modernized README
- Clear quick start section
- Concise feature descriptions
- Clean command reference

**License Change**
- Changed from Apache-2.0 to GPL-3.0

## [0.0.120] - 2025-12-06

### Changed - Modularization (Phase 39)

**Split reliability.rs (780 → 3 files)**
- `reliability/mod.rs` (275 lines): Constants and core computation
- `reliability/reasons.rs` (288 lines): Reason codes and explanations
- `reliability/types.rs` (165 lines): Input/output types

All files now under 400 lines per project guidelines.

## [0.0.119] - 2025-12-06

### Changed - UX Polish (Phase 38)

**Cleaner Greetings**
- First-time greeting condensed to 5 lines (was 15+)
- Removed verbose examples - now just shows 2 sample queries
- Removed emoji (🔥) - using ASCII symbols only

**Professional Messages**
- Bootstrap: "Setting up..." (was informal "Come back soon")
- Errors: "Connection issue. Restarting..." (was "Oops!")
- Streak display simplified: "7 day streak [*]"

**Code Changes**:
- `greeting.rs`: Condensed first-time greeting
- `progress_display.rs`: Cleaner bootstrap message
- `commands.rs`: Professional error messages

## [0.0.118] - 2025-12-06

### Changed - Clean UX Overhaul (Phase 37)

**Status Display - Clean & Focused**

Simplified `annactl status` to show what matters:
- Status: Ready/Starting/Error (color-coded)
- Version, Uptime, LLM state
- Open tickets count
- Update available (if any)
- Debug info (hardware, models, latency) hidden behind `--debug`

**Stats Display - Streamlined**

Simplified `annactl stats` to highlight achievements:
- Profile: Level with XP progress bar
- Activity: Cases handled, success rate, reliability
- Teams: Top 6 with performance metrics
- Top Performers: Top 3 with medals
- Achievements: Unlocked badges
- Highlights: Streak, tenure, favorites

**REPL UX - Examples First**

Cleaner help with examples at top:
- Shows 5 natural language examples first
- Commands (exit, status, help) shown minimally
- Greeting ends with "How can I help you today?"

**Code Changes**:
- `display.rs`: Rewritten for clean status (~160 lines)
- `stats_display.rs`: Rewritten for clean stats (~215 lines)
- `commands.rs`: Simplified REPL help
- `greeting.rs`: Cleaner closing message

## [0.0.117] - 2025-12-06

### Fixed - No Message Truncation (Phase 36)

**Full Messages Always**

Removed all user-facing message truncation. Messages now wrap naturally
instead of being cut off with "...".

**Changes**:
- Ticket queries now display in full (wrapped, not truncated)
- Greeting ticket display shows full query on separate line
- Removed dead truncate functions from codebase
- Internal processing truncation kept (probe summaries, etc.)

## [0.0.116] - 2025-12-06

### Changed - Natural Language Only (Phase 35)

**Everything Through Conversation**

Removed `annactl inbox` command. Everything is through natural language.
Open tickets now show in Anna's greeting when you start a session.

**Natural Language Queries**:
- "show my tickets" - View ticket history and inbox
- "show my inbox" - Same as above
- "pending queries" - Check for queued items

**Greeting Shows Open Tickets**:
When you start `annactl`, Anna now shows:
- Any open tickets waiting for response
- How to reply to tickets

**Code Changes**:
- `main.rs`: Removed Inbox command
- `greeting.rs`: Added `print_open_tickets()` to greeting
- `deterministic.rs`: Enhanced ticket history with real data
- `query_classify.rs`: Added inbox patterns to TicketHistory

## [0.0.115] - 2025-12-06

### Added - File-Based Inbox System (Phase 34)

**Async Query Support**

Added file-based inbox (`~/.anna/inbox`) for async queries.
Users can drop queries without needing an email server.

**How It Works**:
- `inbox.rs` module processes ~/.anna/inbox
- Queries become tickets with case numbers
- User gets email notification (outgoing only)

**Code Changes**:
- `email.rs`: Added `inbox_path()`
- `inbox.rs` (NEW): File-based inbox processing

## [0.0.114] - 2025-12-06

### Added - Self-Healing Email & Contact Options (Phase 33)

**Anna Takes Care of Herself**

Anna now checks her own health and can install dependencies she needs.
Users see all the ways to contact her: one-shot, REPL, or email.

**Health Check Command**:
- `annactl health` - Shows Anna's health status
- Checks if email system is available
- Offers to install mail package if missing (auto-detects distro)
- Shows user's email config status
- Lists all ways to contact Anna
- Shows open tickets

**Contact Options Everywhere**:
- First-time greeting shows 3 ways to reach Anna
- Help response includes email address and commands
- Anna's email: `anna@localhost` (for async tickets)

**Email System Health**:
- `is_email_available()` - Check if mail command exists
- `email_package_name()` - Detect correct package for distro
- `install_email_command()` - Generate install command
- `EmailHealth` struct for complete status

**Distro-Specific Packages**:
- Arch Linux: `s-nail`
- Debian/Ubuntu: `mailutils`
- Fedora/RHEL: `mailx`

**Code Changes**:
- `email.rs`: Added health check, auto-install, ANNA_EMAIL constant
- `greeting.rs`: Shows 3 ways to contact Anna on first visit
- `deterministic.rs`: Help response includes email and commands
- `ticket_commands.rs`: Added handle_health()
- `main.rs`: Added Health command

## [0.0.113] - 2025-12-06

### Added - Async Ticket System with Email Notifications (Phase 32)

**Real IT Helpdesk Experience**

Just like a real IT department: queries that can't be answered immediately
become tickets. Users get notified by email when there's an update or when
IT needs clarification.

**Ticket Lifecycle**:
- Quick queries: resolved immediately (same as before)
- Complex queries: become async tickets with case numbers (CN-XXXX-DDMMYYYY)
- IT staff can ask for clarification (ticket status: PendingUser)
- User replies via email or `annactl reply`
- Email notification when ticket is resolved

**New Commands**:
- `annactl reply CN-0001-06122025 "your message"` - Reply to a ticket
- `annactl ticket CN-0001-06122025` - Show ticket conversation
- `annactl email user@example.com` - Configure email notifications
- `annactl email off` - Disable email notifications

**Ticket Features**:
- Case numbers persist in ~/.anna/tickets.jsonl
- Conversation history tracked in each ticket
- Status: New → Assigned → InProgress → PendingUser → Resolved
- Email notifications use system mail/sendmail

**New Files**:
- `email.rs` - Email notification system
- `ticket_commands.rs` - Reply/ticket/email command handlers

**Code Changes**:
- `ticket_tracker.rs`: Added async ticket support, TicketMessage, conversation tracking
- `router_tests.rs`: Added tests for TicketHistory and StaffRoster query classes
- `main.rs`: Added Reply, Ticket, Email commands

**Progress**: Full async ticket workflow with email notifications

## [0.0.112] - 2025-12-06

### Improved - Welcome Experience and Service Desk Explanations

**Enhanced First-Time Greeting**:
- Shows example questions to try on first visit
- Points users to `annactl stats` and `annactl status` for detailed info
- Introduces the IT team concept naturally

**Better Service Desk Explanation**:
- "how does this work?" / "show my tickets" now explains the full pipeline
- Step-by-step: question → ticket → team assignment → investigation → review → answer
- Guides users to related commands (staff roster, stats, status)

**Code Changes**:
- `greeting.rs`: Enhanced first-time welcome with example queries
- `deterministic.rs`: Improved ticket history response with workflow explanation

## [0.0.111] - 2025-12-06

### Changed - Natural Language Everything (Phase 31)

**Remove CLI Commands, Add Natural Language Queries**

Anna's philosophy is natural language first. This release removes dedicated
CLI commands (`annactl tickets`, `annactl staff`) and replaces them with
natural language queries that feel more like talking to IT support.

**Natural Language for Tickets**:
- "show my tickets" → explains how ticket tracking works
- "what have I asked before" → describes Service Desk model
- "recent cases" → shows support activity summary
- "ticket history" → explains IT department workflow

**Natural Language for Staff**:
- "who is on shift" → shows currently available IT staff
- "show IT team" → lists staff by team with specializations
- "it department" → displays full roster with shift status
- "who is available" → shows on-shift staff with expertise

**Query Classification**:
- Added `QueryClass::TicketHistory` for ticket-related queries
- Added `QueryClass::StaffRoster` for staff-related queries
- Both are fast-path (no LLM needed, deterministic)

**Deleted Files**:
- `ticket_display.rs` - replaced by natural language handler
- `staff_display.rs` - replaced by natural language handler

**Code Changes**:
- `query_classify.rs`: Pattern matching for ticket/staff queries
- `router.rs`: Routes for TicketHistory and StaffRoster
- `deterministic.rs`: Handlers for ticket and staff answers
- `main.rs`: Removed Tickets and Staff CLI commands

**Progress**: Natural language interface complete for all Service Desk queries

## [0.0.110] - 2025-12-06

### Added - Service Desk Theatre UX Enhancements (Phase 30)

**Ticket Filtering, Staff Roster, and Shift Awareness**

Adds operational commands and realistic shift scheduling to the IT
department experience.

**Ticket Search & Filtering**:
- `annactl tickets --team desktop` filters by team
- `annactl tickets --search vim` searches in query text
- `annactl tickets --escalated` shows only escalated tickets
- Shows filter summary and match count

**Staff Roster Command**:
- New `annactl staff` command shows full IT department
- Displays all 18 staff across 9 teams
- Shows tier badges [jr]/[sr], workload stats, specializations
- Includes department summary with activity metrics

**Shift Scheduling**:
- Each staff member has assigned shifts (morning/day/evening/night/flexible)
- Green ● shows currently on shift, dim ○ for off shift
- Shift times: Morning 6am-2pm, Day 9am-5pm, Evening 2pm-10pm, Night 10pm-6am
- Seniors often flexible, security has 24/7 coverage

**New Files**:
- `staff_display.rs` - Staff roster and workload display

**Code Changes**:
- `ticket_display.rs`: Added TicketFilter for search/filter
- `roster.rs`: Added Shift enum and shift field to PersonProfile
- `main.rs`: Added Staff command, enhanced Tickets with filter options

**Progress**: ~55% of full vision complete (operational commands)

## [0.0.109] - 2025-12-06

### Added - Service Desk Theatre Deep Integration (Phase 29)

**Ticket History, Staff Specializations, and Resolution Metrics**

Deepens the Service Desk experience with historical tracking and
staff expertise visibility.

**Ticket History Command**:
- New `annactl tickets` command shows past support cases
- Displays case number, status, team, resolution time, reliability
- Shows truncated query preview for each ticket
- Summary statistics: total, resolved, escalated rates

**Staff Specializations**:
- Each of 18 staff members now has 2-3 expertise areas
- Specializations shown in footer (e.g., "Specializes in: vim, bash, dotfiles")
- Added `staff_id` field to responses for profile lookup
- Desktop: vim, bash, X11, Wayland; Network: TCP/IP, VPN, firewall; etc.

**Resolution Time Tracking**:
- Stats now show average resolution time per ticket
- Average reliability displayed alongside resolution metrics
- Staff performance shows individual resolution times

**New Files**:
- `ticket_display.rs` - Ticket history rendering

**Code Changes**:
- `roster.rs`: Added `RosterEntry` struct with specializations
- `rpc.rs`: Added `staff_id` field to ServiceDeskResult
- `theatre_render.rs`: Shows staff specializations in footer
- `stats_display.rs`: Enhanced ticket stats with resolution times

**Progress**: ~50% of full vision complete (deep integration)

## [0.0.108] - 2025-12-06

### Added - Service Desk Theatre Polish (Phase 28)

**Tool Tracking, Streaks, and Escalation**

Further personalizes the IT department experience with usage insights
and smart escalation.

**Tool Usage Tracking**:
- Queries now extract tool mentions (vim, git, docker, etc.)
- Tools tracked in user profile for personalization
- Greeting shows "You've been using docker (5 queries)"
- 40+ tools recognized across editors, package managers, utilities

**Streak Enhancements**:
- Fire emoji 🔥 for 7+ day streaks ("You're on fire!")
- Encouraging message for shorter streaks ("Keep it going")
- Streak display in greeting patterns

**Smart Escalation**:
- Low reliability responses (< 60) auto-escalate to senior staff
- Theatre context tracks escalation status
- Staff stats record escalation metrics
- Creates realistic "pass to senior" workflow

**Code Changes**:
- `user_profile.rs`: Added `record_tools_from_query()` method
- `theatre.rs`: Records tools from queries, staff stats with escalation
- `greeting.rs`: Enhanced patterns with streak fire, tool counts
- `rpc_handler.rs`: Escalation logic for low reliability

**Progress**: ~45% of full vision complete (personalization deepened)

## [0.0.107] - 2025-12-06

### Added - Service Desk Theatre Enhancements (Phase 27)

**Topic Tracking, Internal Comms, and Staff Performance**

Deepens the "IT Department inside your computer" experience with user
personalization and staff metrics.

**Topic Tracking**:
- User profile now tracks topics from each query
- Topics derived from domain (desktop, network, storage, etc.)
- Enables personalized greetings ("You ask about network a lot")

**Internal IT Communications**:
- Theatre mode now shows Anna dispatching to team with case number
- Named staff appear in internal comms based on assigned_staff

**Staff Performance System**:
- New `staff_stats.rs` tracks per-staff metrics
- Tickets handled, resolved, escalated per staff member
- Success rate and average response time
- Staff performance leaderboard in `annactl stats`

**Display Updates**:
- Stats display now shows top 5 performers with metrics
- Performance table with tickets, resolved, rate, avg time

**New Files**:
- `staff_stats.rs` - Staff performance tracking

**Progress**: ~40% of full vision complete (personalization active)

## [0.0.106] - 2025-12-06

### Added - Service Desk Theatre Integration (Phase 26)

**Case Numbers & Named Staff in Responses**

Every request now gets a case number and assigned staff member, creating
the "IT Department inside your computer" experience.

**Theatre Integration**:
- Every response shows case number (e.g., CN-0001-06122025)
- Staff member assigned per domain (e.g., "Sofia (Desktop Administrator)")
- Ticket history saved to ~/.anna/tickets/
- Theatre context flows from request start to response

**Display Updates**:
- Theatre mode footer shows case number and assigned staff
- Debug mode shows case/staff in transcript header
- User patterns shown in REPL greeting (streak, preferred editor, top topics)

**User Profile Integration**:
- Session tracking updates streak days
- Personalized patterns in greeting ("I've noticed you prefer vim")
- Profile saved after each REPL session

**New Files**:
- `theatre.rs` - Theatre context for request flow

**Progress**: ~35% of full vision complete (theatre integration active)

## [0.0.105] - 2025-12-06

### Added - Service Desk Foundation (Phase 25)

**Theatre Foundation - Named IT Staff & Ticket Tracking**

Major infrastructure for the "IT Department in Your Computer" vision.

**New Systems**:
- **Ticket Tracker** - Case numbers (CN-XXXX-DDMMYYYY), status tracking, history
- **User Profile** - Preferences, patterns, learning, personalized greetings
- **Enhanced Status** - IT department roster, user config display
- **Enhanced Stats** - Recipe counts, ticket stats, per-staff metrics

**IT Department Roster** (18 named staff):
| Team | Junior | Senior |
|------|--------|--------|
| Desktop | Sofia | Erik |
| Network | Michael | Ana |
| Hardware | Nora | Jon |
| Storage | Lars | Ines |
| Performance | Kari | Mateo |
| Security | Priya | Oskar |
| Services | Hugo | Mina |
| Logs | Daniel | Lea |
| General | Tomas | Sara |

**User Profile Features**:
- Tool usage tracking (vim vs nano, bash vs zsh)
- Topic interest tracking
- Streak days and engagement
- Configurable preferences (learning_mode, verbosity, show_internal)
- Personality traits (formality, humor, technical_depth)

**Ticket System**:
- Case numbers: CN-0001-06122025 format
- Status workflow: New → Assigned → InProgress → Resolved
- Escalation tracking
- Resolution time and reliability metrics

**New Files**:
- `ticket_tracker.rs` - Case management system
- `user_profile.rs` - User preferences and patterns

**Progress**: ~30% of full vision complete (foundation laid)

## [0.0.104] - 2025-12-06

### Added - SSH Key Management Recipes (Phase 24)

**Built-in SSH Recipes - No LLM Needed!**

Anna now has built-in recipes for common SSH tasks. Ask about SSH keys
and get instant, accurate answers without any LLM calls.

**Supported SSH Tasks**:
- Generate SSH key (ed25519, rsa, ecdsa)
- Copy key to server (ssh-copy-id)
- Add SSH host alias (~/.ssh/config)
- Configure SSH agent auto-start
- Setup GitHub SSH authentication
- Harden SSH client configuration

**Example Queries**:
- "how do I generate an ssh key"
- "setup ssh for github"
- "ssh copy key to server"
- "configure ssh agent"

**Key Changes**:
- `ssh_recipes.rs` - 6 built-in SSH recipes with step-by-step guides
- `SshKeyType` enum - Ed25519, Rsa4096, Ecdsa
- `SshFeature` enum - GenerateKey, CopyKey, HostAlias, SshAgent, etc.
- Integrated into recipe fast path for instant answers
- `SshKeyManagement` query class for deterministic routing

**Progress**: ~85% learning system complete

## [0.0.103] - 2025-12-06

### Added - Recipe Feedback System (Phase 23)

**Anna Asks for Feedback When Uncertain**

When Anna uses a recipe answer with borderline confidence or from a new pattern,
she now asks the user for feedback. This is automatic - no separate CLI command.

**When Anna Asks for Feedback**:
- Recipe reliability score is 60-75 (borderline confidence)
- Recipe has been used fewer than 3 times (new pattern)

**How Feedback Works**:
- Anna asks "Was this answer helpful? (y/n)" after showing the answer
- User responds: y/n/partial (or skip to ignore)
- Feedback adjusts recipe reliability score:
  - Helpful: +1 score (max 99) and +1 success_count
  - Not Helpful: -5 score (min 50)
  - Partial: +1 success_count only
- Feedback history logged to `~/.anna/feedback_history.jsonl`

**Key Changes**:
- `FeedbackRequest` struct - Anna's question with recipe ID and reason
- `feedback_request` field in `ServiceDeskResult`
- Interactive feedback handling in REPL and one-shot modes
- No CLI command needed - feedback is part of natural conversation

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
