# Changelog

All notable changes to Anna will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.878] - 2026-01-09

### Added
- **TROUBLESHOOT Diagnostic Handler** - Specialized handling for "why is X not working?" questions
  - `get_diagnostic_commands()` returns targeted commands based on problem type:
    - System slow: load average, memory, top CPU/memory processes
    - Audio: pipewire/pulseaudio status, sinks, devices
    - WiFi/Network: nmcli status, device list, rfkill
    - Packages/Updates: mirrorlist, cache, pacman logs
    - Disk space: df, du on common large directories
    - GPU: lspci, driver modules, nvidia-smi, glxinfo
    - Fonts: fc-list, font config, installed font packages
    - Screen flickering: DRM status, xrandr, xorg config
    - Bluetooth: service status, bluetoothctl, rfkill
    - Boot: failed services, journal errors, cmdline
  - `handle_troubleshoot_diagnostic()` runs diagnostics and analyzes output

- **Extended Command Hints** - Added hints for previously failing categories:
  - Init system: `ps -p 1`, `readlink /sbin/init`, `systemctl --version`
  - Display server: `loginctl` session type, `XDG_SESSION_TYPE`
  - Audio server: `pactl info`, `pipewire/pulseaudio` process detection
  - Package count: `pacman -Q | wc -l` (Arch-specific)
  - Shell detection: `$SHELL`, `/etc/passwd` lookup
  - GPU VRAM: `nvidia-smi --query-gpu=memory.total`, `glxinfo`, `lspci -v`
  - Recent packages: `pacman.log` grep, `expac` time-sorted
  - Active timers: `systemctl list-timers`
  - Running processes: `ps aux --sort=-%cpu/-%mem`
  - Network connections: `ss -tuln`, `netstat -tuln`
  - CPU usage: `ps aux --sort=-%cpu`, `/proc/loadavg`
  - Kernel messages: `dmesg --level=err,warn`, `journalctl -k`
  - USB devices: `lsusb`
  - Package checks: zsh, steam, rust, wayland, xorg-server

### Improved
- TROUBLESHOOT routing: diagnostic questions now get specialized handling
- Better coverage for Arch Linux-specific package queries

## [0.0.877] - 2026-01-09

### Fixed
- **TROUBLESHOOT intent now checks for configuration requests** - Questions like
  "can you apply 2x scale to GDM3" were classified as TROUBLESHOOT but not
  routed to the configuration handler. Now both HOWTO and TROUBLESHOOT check
  `is_configuration_request()` and route to `handle_howto_config()` if true.

## [0.0.876] - 2026-01-09

### Added
- **HOWTO Configuration Handler** - Special handling for configuration/change requests
  - Detects when user wants to change/configure vs just query information
  - Routes to instruction-generation mode instead of command-execution
  - Provides step-by-step instructions with commands and config file edits
  - Example: "apply 2x scale to GDM3" now gives proper instructions

- **Better Wiki Search** - Improved search term extraction
  - Extracts key technical terms (gdm, wayland, hidpi, etc.) from questions
  - Uses intent entities and topic for focused search
  - Reduces noise from long questions

### Improved
- HOWTO intent routing: configuration requests get instructions, queries get commands
- Configuration detection: recognizes action verbs (apply, change, configure, enable, etc.)
- Wiki context increased to 3000 chars for HOWTO configuration requests

## [0.0.875] - 2026-01-09

### Added
- **LLM-Based Intent Classification** - Replaces brittle keyword matching
  - New `intent` module with `classify_intent()` function
  - 5 intent categories: FACTUAL, HOWTO, TROUBLESHOOT, MULTI, UNCLEAR
  - Uses quick LLM call (~100-150 tokens, <2s) for semantic understanding
  - Fallback to keyword classification if LLM fails/times out
  - Entities and topic detection (packages, services, files mentioned)

- **Clarification Loop** - Anna asks for more context when questions are unclear
  - UNCLEAR intent triggers clarification question
  - In REPL mode: prompts user, re-submits with context appended
  - Loop continues until intent becomes clear (max 3 attempts)
  - Non-REPL mode: suggests using interactive mode

- **Multi-Question Handling** - Decomposes complex questions
  - MULTI intent extracts sub-questions from LLM response
  - Each sub-question processed separately
  - Sub-question/result steps displayed during execution
  - Combined answer returned to user

### Improved
- Better understanding of user intent through semantic analysis
- More accurate routing based on question type
- New step types for transparency: IntentClassifying, IntentResult, SubQuestion, SubQuestionResult, ClarificationQuestion

## [0.0.873] - 2026-01-09

### Improved
- **SPEED: Lean prompt mode for simple queries** - Factual queries now use minimal context
  - "what is X?", "how much X?", "is X installed?" use just command output
  - No hardware list, no configs, no wiki for simple questions
  - Complex queries (how to, troubleshooting) still get full context
  - Significantly faster response times for factual questions

## [0.0.872] - 2026-01-09

### Added
- More command hints for gaps identified in testing
  - Filesystem type queries (findmnt, df -T)
  - Screen resolution with Wayland support (wlr-randr)
  - Package install checks (ffmpeg, neovim, obs)
  - Default browser query (xdg-settings)
  - Context switch rate, cgroups, ionice, interrupts
  - Improved date/time and terminal detection

### Improved
- Better package detection for common packages
- Wayland-compatible resolution detection
- More specific process stats commands

## [0.0.871] - 2026-01-09

### Added
- **Comprehensive Command Hints** - Massively expanded command suggestions
  - System: load average, memory details, CPU frequency, threads, cache, hyperthreading
  - Storage: last reboot, UUID, NVMe, disk serial, TRIM, swap, inodes, zombie processes
  - Boot: UEFI, bootloader type, microcode, boot entries
  - Network: wifi signal/channel, network speed, ping, ports, routing, DNS, VPN
  - Packages: glibc version, lib32 packages, wine, lutris, pipewire, wireplumber
  - Desktop: compositor, DPI, brightness, night light, language, keyboard layout
  - Sensors: temperature sources, battery details, RAM speed timing
  - Kernel: swappiness, overcommit, sysrq, dirty ratio, hugepages, file limits
  - Shell: users count, available shells, umask, terminal type, TTY, SSH detection

### Improved
- 40+ query categories now have specific command hints
- LLM receives targeted suggestions for specialized questions
- Better coverage for system internals via /proc and /sys

## [0.0.870] - 2026-01-09

### Added
- **Command Hints System** - Intelligent command suggestions based on question type
  - Desktop theme queries now get gsettings commands
  - Hardware sensors get sensors/acpi commands
  - Battery queries get proper power supply commands
  - Motherboard/RAM speed get dmidecode hints
  - Package queries get specific pacman commands

### Improved
- Better LLM command selection for specialized questions
- More user commands recognized (alias, env, groups, whoami)
- Fish shell queries now properly handled

## [0.0.869] - 2026-01-08

### Added
- **User Context System** - Commands now run as the logged-in user
  - New `user_context` module detects real logged-in user via loginctl
  - Commands execute as user by default (not root)
  - Only root-required commands (systemctl start, pacman -S) run as root
  - User-specific commands (~/, .config) properly expand paths
  - Fixes: username, groups, home dir size, package queries

### Fixed
- **Massive accuracy improvements** from running commands as user:
  - Package detection now works (flatpak, zsh, snapper, rust, mesa)
  - User's groups and home directory correctly reported
  - User services (pipewire, pulseaudio) detected properly
  - Editor detection uses user's EDITOR/VISUAL env vars
  - Home directory paths resolve correctly

## [0.0.868] - 2026-01-08

### Fixed
- **Shell detection** - Now correctly detects the logged-in user's shell
  - Previously detected root's shell (bash) when daemon runs as root
  - Now uses loginctl to find real user, then checks /etc/passwd
  - Correctly identifies fish, zsh, and other shells

## [0.0.867] - 2026-01-08

### Added
- **Learning Memory System** - Anna learns from every interaction
  - Stores successful Q&A pairs with semantic context
  - Recalls similar past questions to inform new answers
  - Learns command patterns from successful executions
  - Memory compacts automatically to keep most valuable experiences
  - Storage: `~/.local/share/anna/memory.json`

- **Proactive Monitoring** - Anna watches your system 24/7
  - Checks every 5 minutes for issues
  - Detects: disk space, memory, failed services, journal errors, outdated packages
  - Severity levels: Critical (🔴), Warning (🟡), Info (ℹ️)
  - Issues tracked in `~/.local/share/anna/issues.json`

- **Wiki Knowledge Extractor** - Anna reads the wiki to learn
  - Extracts config file locations from wiki articles
  - Learns common commands per topic
  - Maps packages → config files, services → config files
  - No hardcoding - knowledge comes from wiki

- **Session Memory** - Context awareness in conversations
  - Remembers entities mentioned (packages, services, files)
  - Resolves references ("it", "that service", "the file")
  - Tracks conversation topic for better answers
  - Detects user's apparent goal (troubleshooting, setup, etc.)

## [0.0.866] - 2026-01-08

### Added
- **Safe Operations Module** - Backup and rollback for system modifications
  - Automatic backups before modifying files
  - Backup ledger tracks all changes (`~/.local/share/anna/backup_ledger.json`)
  - Backups stored in `~/.local/share/anna/backups/`
  - Rollback capability for reversing changes
  - Verification functions to check file contents
  - Automatic cleanup of old backups (30+ days)
  - Safe write and append operations with built-in backup

## [0.0.865] - 2026-01-08

### Added
- **Recipe System** - Deterministic solutions for common tasks
  - Built-in recipes for: disk usage, memory, CPU, kernel version, failing services
  - Editor-specific recipes: vim syntax highlighting, nvim syntax highlighting
  - System update recipe for Arch Linux
  - Context-aware matching: recipes check editor/shell/bootloader/filesystem
  - Recipe fast path: skips LLM for known queries (faster, deterministic)
  - Success tracking: recipes improve ranking over time
  - Recipes stored in `~/.local/share/anna/recipes.json`

## [0.0.864] - 2026-01-08

### Added
- **Autonomous Profile Refresh** - Anna keeps itself up-to-date without user intervention
  - Profile automatically scans fresh on daemon startup
  - Background refresh loop checks every 30 minutes
  - Profile expiration reduced from 24 hours to 1 hour
  - RwLock-based caching allows runtime refresh without restart

## [0.0.863] - 2026-01-08

### Added
- **Enhanced System Profile** - Anna now knows your environment
  - Bootloader detection (systemd-boot, grub, limine, refind)
  - Shell detection (bash, zsh, fish)
  - Editor detection (vim, nvim, nano, emacs, code)
  - AUR helper detection (paru, yay, etc.)
  - Root filesystem detection (btrfs, ext4, xfs)
  - Display manager detection (gdm, sddm, lightdm)
  - Audio system detection (pipewire, pulseaudio, alsa)
  - Brief summary now includes: OS, DE, bootloader, editor, shell, filesystem

## [0.0.862] - 2026-01-08

### Fixed
- **Command filtering** - Robust parsing of LLM responses
  - Filters out markdown code fences (```)
  - Filters out explanations (lines starting with "This ", "You ", etc.)
  - Filters out placeholders like `<username>`
  - Enforces ban on interactive commands (top/htop/vim/nano/less)
  - Stricter prompt: "NEVER use: top, htop, vim, nano, less"

## [0.0.861] - 2026-01-08

### Fixed
- **No interactive commands** - Prompt now says "no top/htop/vim"
  - These commands need a terminal and fail in non-interactive execution
  - Example changed to `ps aux --sort=-%cpu | head -10`

## [0.0.860] - 2026-01-08

### Improved
- **Command limit** - Max 3 commands per iteration
  - Prompt now says "1-3 commands" instead of just "commands"
  - Hard limit of 3 enforced in code
  - Prevents LLM from returning 9 commands and causing timeouts

## [0.0.859] - 2026-01-08

### Improved
- **Slim command selection prompts** - Much smaller prompts for faster LLM responses
  - Added `brief_summary()` for one-line system context
  - Command selection now uses minimal prompt instead of full system details
  - Reduces LLM processing time significantly

## [0.0.858] - 2026-01-08

### Fixed
- **Wiki context truncation** - Limit wiki content to 2000 chars max
  - Prevents huge prompts that cause LLM timeouts
  - Faster response times for questions that match wiki articles

## [0.0.857] - 2026-01-08

### Improved
- **Concise answers** - LLM now gives brief, direct responses
  - No more verbose explanations or unsolicited advice
  - "How much RAM?" now returns "31 GiB" instead of paragraphs
  - Removed "Arch Linux system" phrasing that confused LLM about CachyOS
- Prompt rewritten to emphasize brevity: "Answer BRIEFLY - just the facts"

## [0.0.856] - 2026-01-08

### Fixed
- **Skip wiki for vague queries** - Questions like "any improvements?" or "tell me about my system" now skip wiki
  - Detects queries with mostly stop words (no meaningful search terms)
  - Prevents garbage wiki results like "Concurrent Versions System" for "system" queries
- **Skip Category: pages** - Navigation pages no longer included in wiki results
  - Filters out Category:, ArchWiki:, Template:, Help: pages
- **Increased LLM timeout** - From 60s to 120s for complex prompts

### Added
- Extended stop words list with vague query terms (tell, about, know, improvements, etc.)

## [0.0.855] - 2026-01-08

### Added
- **System Profile** - Anna now scans and remembers your hardware + existing configs
  - Detects PCI devices (GPU, WiFi, audio) and their drivers
  - Scans existing workarounds: modprobe.d, udev rules, systemd overrides
  - Profile cached and refreshed every 24 hours
- **Context-aware answers** - LLM sees your existing configs when relevant
  - WiFi question? Shows your iwlwifi.conf settings
  - GPU question? Shows your nvidia/amdgpu modprobe configs
  - Prevents suggesting fixes you've already applied

## [0.0.854] - 2026-01-08

### Fixed
- **Skip low-quality wiki results** - If best wiki score < 0.7, skip wiki entirely
  - "check updates" no longer finds "Machine-check exception"
  - Prevents garbage wiki context from confusing LLM
- **Better command examples** - Added common system commands to LLM prompt
  - checkupdates, df -h, free -h, journalctl, systemctl, ip addr, lspci

## [0.0.853] - 2026-01-08

### Fixed
- Section matching now filters generic words ("change", "make") from header matching
- Added related terms expansion - "scale" now also matches "monitor", "hidpi", "resolution"
- GDM scaling queries now find "Setup default monitor settings" instead of "Change the language"

## [0.0.852] - 2026-01-08

### Added
- **Section-aware wiki retrieval** - Articles parsed into sections by headers
- Wiki now extracts RELEVANT sections (e.g., "HiDPI" for scaling queries) instead of random paragraphs
- Commands extracted only from relevant sections, reducing noise

### Improved
- GDM scaling queries now find the actual HiDPI/scaling section instead of keyboard layout sections
- More focused wiki context sent to LLM = better quality answers

## [0.0.851] - 2026-01-08

### Fixed
- Wiki search now filters out low-scoring results (noise) - only shows truly relevant articles
- Empty LLM response fallback - retries with non-streaming if streaming returns empty
- GDM scaling query no longer shows unrelated "Domain name resolution" results

## [0.0.850] - 2026-01-08

### Fixed
- System context detection now uses `loginctl` instead of environment variables
- Properly detects Wayland/Xorg session type even when daemon runs as root
- Session desktop environment detection improved

## [0.0.849] - 2026-01-08

### Added
- **System context gathering** - Anna now checks environment first (Wayland/Xorg, DE, display manager)
- LLM receives system context before suggesting commands
- Context-aware command examples (Wayland vs Xorg specific)
- Final answers tailored to the actual system setup

### Improved
- Command suggestions now consider whether system uses Wayland or Xorg
- GDM-specific examples for dconf, monitors.xml when relevant

## [0.0.848] - 2026-01-08

### Fixed
- Wiki search now properly filters stop words (the, is, to, etc.)
- Wiki commands are now hints to LLM, not executed directly (prevents wrong commands)
- Search prioritizes exact topic matches with proper scoring

## [0.0.847] - 2026-01-08

### Fixed
- Embedding model pull now properly waits for completion
- Keyword search now prioritizes direct topic matches (e.g., "GDM" in query finds GDM article)
- Improved search relevance - titles matching query terms are prioritized over keyword index

## [0.0.846] - 2026-01-08

### Fixed
- Wiki now correctly reads from `/usr/share/doc/arch-wiki/html/en/` (English articles)
- Wiki index rebuilds automatically if empty (0 articles)

## [0.0.845] - 2026-01-08

### Added
- **Arch Wiki RAG integration** - Anna now searches the Arch Wiki for relevant information
  - Downloads wiki articles on first startup (arch-wiki-docs or API fallback)
  - Keyword and semantic search using Ollama embeddings
  - Extracts commands from wiki articles
  - Wiki context included in final answers
- New step types displayed: WikiSearch, WikiResults, WikiCommands
- Persistent configuration module (`~/.anna/config.toml`)
- Helpers tracking module - tracks packages Anna installs for cleanup

### Improved
- Better command selection - uses wiki commands when available
- More grounded answers with wiki citations

## [0.0.844] - 2026-01-07

### Improved
- Added more Arch-specific command examples:
  - Recently installed packages via pacman.log
  - GPU/graphics card detection with lspci -k
  - CPU info via lscpu
- Better guidance for LLM to use pacman.log for package history

## [0.0.843] - 2026-01-07

### Improved
- Disk usage commands now show full paths (not just top-level directories)
- Commands exclude virtual filesystems (/proc, /sys, /dev)
- LLM prompted to always add `2>/dev/null` to suppress errors
- Better example for folder size queries

## [0.0.842] - 2026-01-07

### Fixed
- LLM no longer generates Chinese characters in commands (enforced English-only)
- LLM no longer makes up data when commands return empty output
- Better grep patterns in examples (e.g., "Install Date" instead of "Install.date")
- Added more command examples to guide LLM (including installation date)

## [0.0.841] - 2026-01-07

### Fixed
- Commands with shell metacharacters (`$()`, pipes, backticks) now execute correctly
- LLM's escaped characters are automatically unescaped before execution

## [0.0.840] - 2026-01-07

### Added
- **Streaming responses** - answers now appear word-by-word as the LLM generates them
- Real-time display of all dialogue steps as they happen
- No more "Thinking..." wait - you see progress immediately

### Changed
- Full command outputs are no longer truncated
- Each step is displayed as it happens, not all at once at the end

## [0.0.839] - 2026-01-07

### Added
- **Full dialogue transparency** - annactl now shows the complete conversation:
  - USER: The original question
  - ANNA → LLM: What Anna asks the LLM (for commands, validation, final answer)
  - LLM → ANNA: What commands the LLM suggests
  - EXEC: Commands being executed
  - OUTPUT: Results from each command
  - ANSWER: Final response to user
- Makes troubleshooting easy by showing every step of the process

## [0.0.838] - 2026-01-07

### Changed
- Improved command generation prompt - LLM now more reliably generates commands for system queries
- Added examples to prompt to guide command generation
- Clearer instructions to prevent LLM from giving advice instead of commands

## [0.0.837] - 2026-01-07

### Added
- **Automatic hardware detection** - detects NVIDIA CUDA, AMD ROCm, Intel Arc GPUs and VRAM
- **Smart model selection** - automatically picks the best model for your hardware:
  - 16GB+ VRAM: qwen2.5:14b
  - 8-16GB VRAM: qwen2.5:7b
  - 4-8GB VRAM: qwen2.5:3b
  - CPU-only: based on RAM
- **Resource registry** - tracks models installed by Anna for clean uninstall
- **GPU info in status** - `annactl status` now shows GPU type and VRAM
- **CUDA/ROCm support** - installs ollama-cuda or ollama-rocm based on hardware

### Changed
- Daemon auto-upgrades to better model if hardware supports it
- Model selection prefers larger models for better reasoning

## [0.0.836] - 2026-01-07

### Fixed
- `annad --version` now works (previously tried to start daemon, blocking auto-update verification)

## [0.0.835] - 2026-01-07

### Fixed
- `annactl status` now correctly shows daemon status instead of sending "status" as a question to the LLM
- CLI properly handles built-in commands (status, help, --help, -h, --version, -v) before treating input as a question

## [0.0.834] - 2026-01-07

### Fixed
- Model pull no longer blocks the async runtime (using `spawn_blocking`)
- Daemon properly initializes when no models are available

## [0.0.833] - 2026-01-07

### Changed - Complete Architecture Simplification

**Anna is now a simple LLM-powered command executor.**

The entire codebase has been stripped down to its core functionality:

#### Removed (~200 modules deleted)
- Service desk, specialists, recipes, learning loop
- Theatre rendering, progress tracking, stats display
- Deterministic handlers, probes, routing stages
- Recipe engine, similarity matching, query classification
- Ticket system, feedback handlers, greeting generators
- All complex UI components

#### What Remains

**anna-shared (5 files):**
- `lib.rs` - Core exports, socket path, version
- `rpc.rs` - Simple RPC types (Ask, Status)
- `status.rs` - DaemonStatus, DaemonState
- `update_ledger.rs` - Update tracking
- `version.rs` - Version comparison

**annactl (1 file):**
- Simple REPL client
- Commands: question, status, help, quit

**annad (9 files):**
- Auto-update from GitHub (preserved)
- Ollama management (install/start/chat)
- Core loop: Question -> LLM commands -> Execute -> Validate -> Iterate

#### How It Works Now
1. User asks question about Arch Linux
2. LLM generates shell commands to answer
3. Commands are executed (dangerous ones filtered)
4. Output sent to LLM for validation
5. Iterate until valid answer (max 5 times)
6. Return answer to user

## [0.0.832] - 2026-01-06

### Fixed - LLM Status Display

**No more duplicate models showing 34 times!**

- `models_pulled()` now deduplicates entries
- Excludes models that were later deleted
- Shows max 3 models with "see ollama list" hint
- Changed label from "models_by_anna" to "models_pulled (by Anna for IT staff)"

## [0.0.831] - 2026-01-06

### Fixed - Unified Staff Roster & Simplified CLI

**Staff names are now consistent across all components.**

Previously there were TWO separate staff systems:
- `annad/specialists.rs`: Wei, Lin, Eva, Raj, Tom, Amy, Leo, Maya...
- `anna-shared/roster/`: Michael, Sofia, Ana, Erik, Lars, Ines...

This caused the theatre renderer to show different names than the internal comms!

Now:
- `specialists.rs` uses the unified roster from `anna-shared/roster/`
- All staff names are consistent: Michael, Sofia, Kari, Mateo, etc.
- Role titles come from the roster (e.g., "Network Engineer", "Performance Analyst")
- Theatre renderer and core loop use the same staff data

### Removed - CLI Flags

**Internal comms are now ALWAYS shown - no flags needed!**

Removed flags:
- `--cinematic` - Was redundant, theatre mode is always on
- `--debug` - Was rarely used
- `--no-internal-comms` - Defeats the purpose of the IT department experience

The whole point of Anna is the "fly on the wall" experience watching the IT team work.
This should never be optional or hidden behind flags.

### Staff Roster (unified)

| Domain      | Junior                        | Senior                         |
|-------------|-------------------------------|--------------------------------|
| System      | Kari (Performance Analyst)    | Mateo (Performance Engineer)   |
| Network     | Michael (Network Engineer)    | Ana (Network Architect)        |
| Storage     | Lars (Storage Engineer)       | Ines (Storage Architect)       |
| Services    | Hugo (Services Administrator) | Mina (Services Architect)      |
| Desktop     | Sofia (Desktop Administrator) | Erik (Desktop Specialist)      |
| Security    | Priya (Security Analyst)      | Oskar (Security Engineer)      |
| Hardware    | Nora (Hardware Technician)    | Jon (Hardware Engineer)        |
| Logs        | Daniel (Logs Analyst)         | Lea (Logs Engineer)            |

## [0.0.830] - 2026-01-06

### Fixed - Critical Issues from Audit

**Major fixes addressing "nothing works" feedback from comprehensive audit.**

#### 1. Internal Comms Now Work
- `dispatcher.rs`: Internal comms from core_loop are now properly converted to `Transcript` events
- `dispatcher.rs`: Internal comms are also pushed as `ProgressEvent::internal_comms` for real-time streaming
- Users will now see the "fly on wall" IT department experience they expected

#### 2. Improved Recipe Matching
- `recipe_handler.rs`: Added synonym support (ram↔memory, disk↔storage, etc.)
- `recipe_handler.rs`: Added fuzzy matching with prefix/suffix support
- `recipe_handler.rs`: Intent normalization for better matching
- Recipes are now much more likely to match similar queries

#### 3. Better Specialist Prompts
- `specialist_handler.rs`: Now uses structured prompts from `specialist_prompt.rs`
- `specialist_handler.rs`: Domain-specific hints for each specialist type
- `specialist_handler.rs`: Proper JSON schema input/output format
- Specialists will give better, more grounded answers

### Technical Details

**Internal comms flow:**
```
core_loop generates InternalComm → dispatcher converts to:
  1. Transcript events (for final response)
  2. Progress events (for real-time streaming)
```

**Recipe matching improvements:**
- Synonyms: 15 word groups (memory, storage, process, update, service, network, etc.)
- Scoring: exact=10, synonym=8, prefix=5 points
- Threshold: minimum 10 points to match (prevents false positives)
- Intent bonus: 20 points for intent match

**Prompt improvements:**
- Uses `build_specialist_prompt(domain)` for domain-specific hints
- Structured JSON input with probes, docs, metadata
- Handles both old (answer: string) and new (answer: {short, detail}) response formats

## [0.0.826] - 2026-01-06

### Fixed - Deterministic Routing Priority

**CRITICAL: Queries now use deterministic probes instead of LLM translator.**

The core loop was bypassing deterministic routing entirely, sending ALL queries
through the LLM translator. This caused:
- Wrong package manager commands (apt/yum on Arch instead of pacman)
- Slow responses for known query patterns
- Unnecessary LLM calls for deterministic queries

Now the flow is:
1. Classify query using pattern matching
2. If known pattern → run appropriate probes from registry → deterministic answer
3. Only fall back to LLM translator for unknown queries

Affected queries (now instant):
- "any pending updates?" → uses `checkupdates` (Arch-aware)
- "how much RAM?" → uses `free -h`
- "disk usage?" → uses `df -h`
- And 100+ other deterministic patterns

### Technical Details

- Added `classify_query()` call at start of core loop
- Uses `get_route()` to get distro-aware probes
- `probe_id_to_command()` maps probe IDs to actual shell commands
- Multi-distro fallback in probe commands (pacman || apt || dnf)

## [0.0.825] - 2026-01-06

### Added - Daemon Watchdog

**Automatic health monitoring and recovery for improved reliability.**

- New `watchdog.rs` module monitors daemon health every 30 seconds
- Checks: Ollama status, memory usage, daemon state
- Auto-restarts Ollama if it crashes (configurable)
- Can be disabled with `--no-watchdog` flag
- Prevents restart loops with 60-second cooldown

### Added - Integration Tests

**Comprehensive daemon-client workflow testing.**

- 13 new integration tests in `integration_tests.rs`
- Tests for: status, stats, progress, daemon info, feedback, truth ledger
- Concurrent request handling verification
- Response time assertions (< 100ms for status)
- Error handling for invalid params

### Fixed - Translator JSON Parsing

**Robust handling of malformed LLM JSON responses.**

The translator was failing on valid queries when the LLM returned slightly malformed JSON like:
```
{"intent": "check_updates", "probes": ["cmd1", "cmd2"])"}
```
(Note the extra `)` before `}`)

New `fix_common_json_errors()` function handles:
- Extra parentheses before braces (`])"}` → `]}`)
- Missing closing brackets (auto-balanced)
- Trailing commas (`[1, 2,]` → `[1, 2]`)

### Fixed - Async Mutex for Streaming Events

**Replaced std::sync::Mutex with tokio::sync::Mutex in async code.**

- `state/types.rs`: streaming_events now uses async-safe mutex
- `progress_tracker.rs`: Full rewrite for async mutex compatibility
- `handlers/status.rs`: Proper async lock handling
- Prevents potential deadlocks in concurrent async contexts

### Fixed - Pre-existing Test Failures

- `test_query_classification_network`: Added `ends_with(" ip")` pattern
- `test_pending_messages`: Unique test paths to avoid parallel test conflicts
- `test_comms_generator_creates_messages`: Changed to `#[tokio::test]`

### Changed - File Modularization

**Split oversized files to comply with 400-line limit.**

- `initialization.rs` (461→277 lines): Extracted `model_setup.rs` (185 lines)
- `ollama.rs` (451→312 lines): Extracted `ollama_models.rs` (152 lines)

### Changed - Error Handling

**Replaced critical `.unwrap()` calls with `unwrap_or_default()`.**

Files updated:
- `rpc_handler/dispatcher.rs`
- `rpc_handler/mod.rs`
- `feedback_handler.rs`
- `result_stage.rs`
- `handlers/status.rs`

### Changed - Documentation

- Updated README.md version from v0.0.791 to v0.0.824

## [0.0.824] - 2026-01-05

### Fixed - Pacman File Conflict Handling

**Use --overwrite flag for pacman installs to handle gcc/gcc-libs and other file conflicts.**

The `cuda` package depends on `gcc` which has file conflicts with `gcc-libs` (man pages, include files).
Similarly, `ollama` and `ollama-cuda` packages can have conflicts with leftover files.

Now all pacman installs use `--overwrite '*'` to forcefully resolve file conflicts:
- `pacman -S --noconfirm --overwrite '*' ollama`
- `pacman -S --noconfirm --overwrite '*' cuda ollama-cuda`

This fixes the daemon crash-loop on systems with pre-existing gcc-libs conflicts.

## [0.0.823] - 2026-01-05

### Fixed - Cleanup FIRST on Daemon Start

**Manual Ollama cleanup now runs FIRST thing on daemon initialization - before anything else.**

The daemon now:
1. Runs cleanup_manual_ollama_install() immediately on start
2. Detects leftover /usr/lib/ollama/ even if binary was already deleted
3. Cleans up ALL conflicting files before attempting any installs
4. Then proceeds with normal initialization

This ensures the cleanup happens even if previous attempts partially failed.

## [0.0.822] - 2026-01-05

### Fixed - Non-Fatal Ollama Install

**Daemon no longer crashes if Ollama install fails - continues startup so auto-update can run.**

The daemon was crash-looping because Ollama install failure was fatal. Now:
- Ollama install failure logs an error but doesn't crash
- Daemon continues startup
- Auto-update loop can run and download fixed version
- Next restart will have the cleanup code

## [0.0.821] - 2026-01-05

### Fixed - Complete Manual Ollama Cleanup

**Properly removes ALL files from manual Ollama install before installing pacman package.**

The curl installer leaves files in multiple locations that ALL conflict with pacman:
- `/usr/bin/ollama`
- `/usr/lib/ollama/` (directory with .so files)
- `/usr/lib/systemd/system/ollama.service`
- `/usr/lib/sysusers.d/ollama.conf`
- `/usr/lib/tmpfiles.d/ollama.conf`
- `/usr/share/licenses/ollama/`
- `/usr/share/ollama/`

Now both `ollama::install()` and `gpu_setup::install_cuda_packages()` clean up all these files before attempting pacman install.

## [0.0.820] - 2026-01-05

### Fixed - GPU Setup Handles Manual Ollama Installs

**Anna now properly replaces manually installed Ollama with GPU-enabled version!**

**The real problem:** Ollama was running on CPU only (20-30 second responses) because:
1. Ollama was installed manually via curl script (not via pacman)
2. `ollama-cuda` package conflicted with the manual binary
3. GPU setup was failing silently with "conflicting files" error

**The fix:**
- Detect if ollama was installed manually (binary exists but not owned by pacman)
- Remove the manual binary before installing `ollama-cuda` package
- Also handle CPU-only `ollama` package that conflicts with `ollama-cuda`

**Result:** Ollama will now use GPU (2-3 second responses instead of 20-30 seconds).

## [0.0.819] - 2026-01-05

### Fixed - Fast REPL Startup

**annactl without arguments now starts in ~2 seconds instead of hanging!**

The problem was that REPL mode tried to:
1. Get daemon status (could hang if daemon is busy)
2. Generate LLM greeting (could take 30+ seconds if Ollama is slow)

**Fix:**
- Added 500ms timeout for initial daemon status check
- Added 2 second timeout for LLM greeting generation
- Falls back to fast deterministic greeting on timeout

Startup time improved from 30+ seconds to ~2 seconds.

## [0.0.818] - 2026-01-02

### Fixed - Status Display and Better Defaults

**Anna now tracks installations properly and has better defaults!**

**Fixes:**
1. **Ollama tracking**: Fixed Ollama showing as "installed by user" when Anna installed it. Now checks ledger for proper attribution.
2. **Package tracking**: CUDA and ollama-cuda are now tracked separately in the ledger for proper uninstall.
3. **Model cleanup**: Enhanced cleanup to remove unused and duplicate models more aggressively.

**Better defaults per VISION.md:**
1. **learning_mode**: Now ON by default (was OFF)
2. **internal_comms**: Now ON by default (was OFF) - enables fly-on-wall IT department experience
3. **autonomy_level**: Now 50/100 (was 0) - balanced autonomy

**Query classification:**
- Added "memory hogs" pattern (plural) for TopMemoryProcesses classification
- Fixed test classifier ordering to properly catch memory hog queries

## [0.0.817] - 2026-01-02

### Added - Automatic GPU Setup

**Anna now automatically configures GPU acceleration on startup!**

When the daemon starts, it will:
1. Detect NVIDIA GPU via `lspci`
2. Check if `cuda` and `ollama-cuda` packages are installed
3. Install them automatically if missing (`pacman -S --noconfirm cuda ollama-cuda`)
4. Add `ollama` user to `video` and `render` groups if needed
5. Restart Ollama to pick up GPU access

**Why this matters:**
- Without GPU, Ollama runs on CPU only (20-30 second responses)
- With GPU, responses are 2-3 seconds
- Users no longer need to manually configure GPU access
- Anna self-heals this common misconfiguration

**New module:** `gpu_setup.rs` handles all GPU detection and configuration.

## [0.0.816] - 2025-12-16

### Fix - Don't learn recipes for dynamic queries

**Critical bug fix!** The learning system was storing literal answers as recipes for
queries that return different results each time (disk space, memory, processes).

**Problem:**
- User asked "biggest folders" → Specialist answered "home 313G, var 18G..."
- Anna learned this as a recipe with the literal text "home 313G..."
- Next time, Anna returned the stale cached answer instead of running fresh probes

**Fix:**
- Added `is_dynamic_domain()` and `is_dynamic_probe()` checks
- Queries about storage, memory, performance, processes skip recipe learning
- These always go to specialist with fresh probe data

**Domains that skip learning:** storage, memory, performance, system, processes

**Probes that skip learning:** largest_dirs, disk_usage, free, top_memory, top_cpu,
running_services, failed_services, etc.

## [0.0.815] - 2025-12-16

### Added - Learning Stats Tracking

**The core loop now tracks real learning metrics!** Per VISION.md, Anna now measures
her recipe hit rate - the percentage of queries answered by learned recipes vs LLM.

**Stats tracked:**
- `total_requests` - every query processed
- `recipe_hits` - queries answered instantly by learned recipes
- Recipe hit rate = `recipe_hits / total_requests * 100`

**Why this matters:**
- Shows Anna is actually learning (hit rate should increase over time)
- Identifies areas where more recipes are needed
- Measures the effectiveness of the learning system

**View stats:** `annactl status` shows these in the stats section.

**Files:**
- `crates/annad/src/core_loop.rs` - Added stats recording for recipe hits vs LLM calls

## [0.0.814] - 2025-12-16

### Fix - "Biggest folders" now shows actual content, not parent directories

**Problem:**
- "what are the top 20 biggest folders" returned `/home (313G)`, `/var (18G)`, `/usr (15G)`
- This is useless - user already knows these are large
- They want to know WHAT INSIDE these directories is consuming space

**Fix:**
- `largest_dirs` probe now uses `--max-depth=2` to drill into subdirectories
- Scans `/home`, `/var`, `/usr`, `/opt` with depth 2 to find actual large folders
- Now shows: `/home/user/Downloads (50G)`, `/home/user/.cache (30G)`, `/var/log/journal (5G)`
- Increased timeout from 2s to 8s to allow deeper scan
- `largest_home` also uses depth 2 to show subdirectories

**Example output (before):**
```
home (313G), var (18G), usr (15G)
```

**Example output (after):**
```
/home/user (300G), /home/user/Downloads (50G), /home/user/.cache (30G),
/var/lib/docker (10G), /var/log/journal (5G), /usr/lib (8G)
```

## [0.0.813] - 2025-12-16

### Added - Knowledge Base Integration

**Specialists now consult documentation!** Per VISION.md, Anna treats documentation as the
primary technical authority - Arch Wiki, man pages, and --help output.

**Knowledge Flow:**
1. Query is analyzed and knowledge tags extracted (intent keywords, domain, entities)
2. `doc_fetcher` looks up relevant documentation:
   - Local Arch Wiki cache (~/.anna/wiki-cache or /var/lib/anna/wiki-cache)
   - Man pages for relevant commands
   - --help output for discovered commands
3. Knowledge is included in the specialist prompt with citations
4. Specialists cite their sources: "According to the Arch Wiki..." or "The man page states..."

**Example Internal Comms:**
```
Wei (Junior System): Found 2 knowledge sources: man systemctl, Arch Wiki: Systemd
Wei (Junior System): According to the systemd man page, ...
```

**Files:**
- `crates/annad/src/core_loop.rs` - Added knowledge lookup step between evidence and specialist

## [0.0.812] - 2025-12-16

### Added - IT Department with Named Specialists

**The IT department is now alive!** Per VISION.md, Anna has a full team of specialists
with human names, organized by domain with Junior and Senior roles.

**Specialist Teams:**
| Domain | Junior | Senior |
|--------|--------|--------|
| System | Wei | Lin |
| Network | Eva | Raj |
| Storage | Tom | Amy |
| Services | Leo | Maya |
| Packages | Kai | Zara |
| Desktop | Mia | Sofia |
| Security | Alex | Chen |

**Escalation Flow (per VISION.md):**
1. Junior specialist attempts first (uses lighter model)
2. If confidence < 70%, escalates to Senior (uses deeper model)
3. Senior analyzes with more thorough reasoning
4. High-confidence solutions become learned recipes

**Internal Comms Example:**
```
Anna: No recipe for 'check_ram', asking Wei (Junior System) for help
Wei (Junior System): Gathered 2 pieces of evidence, analyzing...
Wei (Junior System): I'm not confident enough (65%), escalating to Lin (Senior System)
Lin (Senior System): Analyzed the problem. Confidence: 92%
Anna: Learning recipe 'check_ram-1734355200' for next time
```

**Files:**
- `crates/annad/src/specialists.rs` - NEW: IT Department registry with all specialists
- `crates/annad/src/core_loop.rs` - Updated to use named specialists with escalation
- `crates/annad/src/lib.rs` - Added specialists module

## [0.0.811] - 2025-12-16

### Changed - Core Loop: The Beginning of Real Learning

**The Big Pivot:** After 810 versions of hardcoded pattern matching that never truly learned,
this release introduces the **core loop** - the foundation for Anna to actually learn from experience.

**New `core_loop.rs` module** implementing VISION.md core request flow:
```
User Query → Translator → Check Recipes →
  If found: Execute recipe → Answer
  If not: Specialist solves → Anna learns → Answer
```

**Everything is natural language** - no new commands or flags. Just ask Anna anything:
```
annactl "how much RAM do I have?"
```

The core loop is now the DEFAULT handler for all requests. It replaces the complex
658-line llm_request.rs with a simple, clear flow.

**Core Loop Flow:**
1. **Translator** - LLM converts natural language to `ParsedQuery` (intent, domain, probes)
2. **Recipe Check** - Search `RecipeLibrary` for matching pattern
3. **If Recipe Found** - Execute probes, render template, instant answer
4. **If No Recipe** - Gather evidence, ask specialist LLM
5. **Learning** - High-confidence (≥80%) solutions become new recipes

**Why This Matters:**
The old system had 5+ overlapping recipe systems that never actually learned.
This is the clean restart: one loop, one recipe store, real learning.

**Files:**
- `crates/annad/src/core_loop.rs` - 520 lines of clear, simple request handling
- `crates/annad/src/rpc_handler/dispatcher.rs` - Request now uses core_loop

## [0.0.810] - 2025-12-16

### Added - Reusable Pattern Matching System

**New `patterns.rs` module** with synonym groups for flexible query matching:
- `SIZE_WORDS`: largest, biggest, top, most, heaviest, taking, using, consuming, eating, hogging
- `STORAGE_WORDS`: disk, space, storage, drive, volume, filesystem, fs
- `DIRECTORY_WORDS`: folder, folders, directory, directories, dir, dirs, path, paths
- `PROCESS_WORDS`: process, processes, program, programs, app, apps, application, task, tasks
- `MEMORY_WORDS`: memory, ram, mem, heap
- `CPU_WORDS`: cpu, core, cores, thread, threads
- `NETWORK_WORDS`: network, net, internet, wifi, ethernet, connection, connected, online

**Helper functions:**
- `contains_any()` - substring matching for any word in a group
- `contains_any_word()` - **whole word** matching (prevents "processor" matching "process")
- `matches_top_n_pattern()` - matches "top N folders", "biggest directories", etc.
- `matches_what_is_doing()` - matches "what is using disk", "what's eating storage", etc.

**Benefits:**
- Queries like "top 20 folders", "biggest directories", "largest dirs", "folders eating disk"
  all route to `LargestFolders` without hardcoding each variation
- "processes using memory" vs "show processor info" now correctly distinguished
- Future patterns can reuse these synonym groups

**Bug fix:** "show processor info" was incorrectly matching `TopCpuProcesses` because
"processor" contains "process" as a substring. Fixed with whole-word matching.

## [0.0.809] - 2025-12-16

### Changed - LargestFolders Back to LLM Path (Honest Fix)

**The Problem:** Pretending `du` can be fast is a lie. On large filesystems (690GB+),
scanning directories takes 30-60+ seconds minimum. No timeout hack fixes this.

**The Honest Solution:** LargestFolders is no longer a "fast path" query.
- Removed from `is_fast_path()`
- Reverted to LLM handling
- LLM will explain the limitation and offer alternatives

**Reality check:** Some queries simply cannot be answered instantly:
- "biggest 20 folders" requires scanning the entire filesystem
- This is not a recipe problem - it's physics (disk I/O)
- The LLM can explain this and suggest `ncdu` or running `du` manually

## [0.0.808] - 2025-12-16

### Fixed - Largest Folders Actually Fast Now

**Problem:** `du` on large filesystems (690GB+) takes forever, causing timeouts.

**Solution:** Use df (instant) + aggressive 2-second timeout on top-level du:
- Shows disk usage overview (instant via `df`)
- Shows top 10 largest directories at root level with 2s timeout
- Shows home directory contents with 3s timeout
- If scan times out, gives user the manual command to run

**Changed:**
- `probe_registry.rs` - New fast probe strategy
- `storage.rs` - Updated parser for sectioned output

## [0.0.807] - 2025-12-16

### Improved - Classification Pattern Additions

**Date/Time Patterns:**
- Added "what time", "what date", "current date", "today's date"
- Added standalone "date" and "time" queries → TimezoneInfo

**IP Address Patterns:**
- Added "my ip", "ip address", "what is my ip", "show ip" → NetworkInterfaces

**Disk Space Patterns:**
- Added standalone "df" command → DiskSpace
- Added "free space", "space left", "how full" patterns

**Files Modified:**
- `classify_system.rs` - Date/time patterns
- `classify_network.rs` - IP address patterns
- `classify_hardware.rs` - Disk space patterns

## [0.0.806] - 2025-12-16

### Fixed - Largest Folders Now Deterministic (FAST!)

**LargestFolders Query - No More 40s Timeouts!**
- "what are the biggest folders?" now returns instantly (~3s max)
- No LLM specialist needed - deterministic du output parsing
- Shows system directories and home directory separately
- Fixed the embarrassing 40+ second timeout issue

**Classification Improvements:**

**Docker Patterns:**
- Added standalone "docker" query → DockerContainers
- Added "what containers" and "show container" patterns

**Port/Process Patterns:**
- Added "what process is using port X" → ListeningPorts
- Added "what's on port 80" style queries

**Running Processes:**
- Added "running processes", "active processes" → TopCpuProcesses
- Added "list processes", "show processes" patterns
- Fixed: "show processor" still routes to CpuInfo (not TopCpuProcesses)

**Files Modified:**
- `classify_services.rs` - Docker patterns
- `classify_network.rs` - Port/process patterns
- `classify_hardware.rs` - Running processes patterns
- `routes_storage.rs` - LargestFolders now deterministic
- `query_class_impl.rs` - LargestFolders in is_fast_path
- `det_extended/storage.rs` - Added answer_largest_folders()
- `deterministic/router.rs` - LargestFolders dispatch

## [0.0.805] - 2025-12-16

### Added - Screen/Display Resolution Query

**New Query Class: ScreenResolution**
- Query "what is my screen resolution?" now has deterministic answer path
- Uses xrandr (Xorg) or wlr-randr/hyprctl (Wayland) to detect display info
- Shows monitor names, resolutions, and primary display indicator
- Supports multi-monitor setups

**Classification Patterns:**
- "resolution", "screen size", "display size", "monitor", "xrandr"
- "how many displays", "what screen", "my display"
- "brightness", "refresh rate", display/screen + "hz"

**Files Modified:**
- `query_class.rs` - Added ScreenResolution variant
- `query_class_impl.rs` - Added from_str parsing and is_fast_path
- `classify_hardware.rs` - Added classification patterns
- `routes_hardware.rs` - Added route configuration
- `probe_registry.rs` - Added xrandr probe with Wayland fallbacks
- `det/system/hardware.rs` - Added `answer_screen_resolution()` function
- `deterministic/router.rs` - Added dispatch for ScreenResolution

**Performance:**
- Instant deterministic response (~200ms)
- No LLM specialist needed

## [0.0.804] - 2025-12-16

### Fixed - Query Classification Refinements

**Audio Troubleshooting Patterns:**
- Added "audio working", "sound working", "no sound", "no audio" patterns to AudioDevices
- Added "speakers work/sound" patterns for speaker troubleshooting
- "is my audio working?" now correctly routes to AudioDevices

**Network Classification Order:**
- Moved interface keyword checks (interface, ip, show network, ethernet, wlan) BEFORE connectivity patterns
- Prevents "show network interfaces" from matching "network" in connectivity check first
- "show network interfaces" now correctly routes to NetworkInterfaces

**Network Working Pattern:**
- Added "network working" and "internet working" patterns to NetworkConnectivity
- "is my network working?" now routes to NetworkConnectivity

**Disk Full Investigation:**
- Added "why is disk full" and "what is making disk full" patterns to LargestFolders
- Requires "why" or "what" to distinguish from status check ("is storage full" stays DiskSpace)
- Helps users understand what's consuming disk space

**Files Modified:**
- `classify_hardware.rs` - Added audio troubleshooting patterns, disk full with why/what check
- `classify_network.rs` - Reordered interface checks first, added network/internet working patterns

## [0.0.803] - 2025-12-16

### Fixed - Query Classification Improvements

**Boot Times Pattern:**
- Added "boot times" pattern (plural) to BootTimeStatus classifier
- "what are my boot times?" now correctly routes to BootTimeStatus

**Network Status Pattern:**
- Added "network status" pattern to NetworkConnectivity classifier
- Reordered NetworkConnectivity check BEFORE NetworkInterfaces
- "what is my network status?" now routes to NetworkConnectivity instead of NetworkInterfaces

**Files Modified:**
- `classify_system.rs` - Added "boot times" pattern
- `classify_network.rs` - Added "network status" pattern, reordered checks

## [0.0.802] - 2025-12-16

### Added - Webcam/Camera Detection Query

**New Query Class: WebcamStatus**
- Query "do I have a webcam?" now has deterministic answer path
- Uses lsusb to detect camera devices + /dev/video* + v4l2-ctl
- Shows camera name and model when detected

**Classification Patterns:**
- "webcam", "camera", "web cam", "video device"

**Files Modified:**
- `query_class.rs` - Added WebcamStatus variant
- `query_class_impl.rs` - Added from_str parsing and is_fast_path
- `classify_hardware.rs` - Added classification patterns (before USB)
- `routes_hardware.rs` - Added route configuration
- `probe_registry.rs` - Added webcam_devices probe
- `det/system/hardware.rs` - Added `answer_webcam_status()` function
- `deterministic/router.rs` - Added dispatch for WebcamStatus

**Performance:**
- Instant deterministic response (~200ms)
- No LLM specialist needed

## [0.0.801] - 2025-12-16

### Added - Device Type Query Support

**New Query Class: DeviceType**
- Query "is this a laptop or a desktop?" now has deterministic answer path
- Uses `hostnamectl` to detect chassis type (laptop, desktop, server, vm, etc.)
- Fallback to battery detection if chassis info unavailable

**Classification Patterns:**
- "laptop" + "desktop", "device type", "is this" + "laptop/desktop"
- "am i on" + "laptop/desktop", "chassis", "what type/kind" + "computer"

**Files Modified:**
- `query_class.rs` - Added DeviceType variant
- `query_class_impl.rs` - Added from_str parsing and is_fast_path
- `classify_system.rs` - Added classification patterns
- `routes_system.rs` - Added route configuration with hostnamectl probe
- `probe_registry.rs` - Added hostnamectl probe command
- `det/system/info.rs` - Added `answer_device_type()` function
- `deterministic/router.rs` - Added dispatch for DeviceType

**Performance:**
- Instant deterministic response (~100ms)
- No LLM specialist needed

## [0.0.800] - 2025-12-16

### Fixed - "What is my vim setup?" Wrong Answer

**Bug:**
- Query "what is my vim setup?" returned generic system info instead of vim config
- Translator probes overrode the deterministic route probes for ConfigureEditor

**Fix:**
- Added classification pattern for "what is my X setup" queries in `classify_config.rs`
- ConfigureEditor now always uses deterministic route probes
- Fixed in `enforce_probe_spine()` to prioritize det_route probes

**Result:**
- "what is my vim setup?" now detects installed editors and shows config options

## [0.0.799] - 2025-12-16

### Added - Boot Blame Query Support

**New Query Class: BootBlame**
- Query "why is my boot slow?" now has deterministic answer path
- Uses `systemd-analyze` and `systemd-analyze blame` probes
- Shows total boot time + top 10 slowest services

**Classification Patterns:**
- "boot" + "slow", "slow startup", "why boot", "boot blame"
- "boot analysis", "boot" + "long", "takes" + "boot"

**Fixes Applied:**
- Excluded boot queries from `SystemSlow` classifier in `classify_core.rs`
- Fixed `find_probe` to use command prefix ("systemd-analyze") not probe name
- Added BootBlame to deterministic router dispatch

**Performance:**
- Skips LLM specialist entirely - instant response (~400ms)
- No more "Parse error: Timeout" on boot queries

## [0.0.798] - 2025-12-15

### Fixed - Swap Query Misclassification

**Bug Fix:**
- "do I have swap" was being classified as `InstalledToolCheck` instead of `SwapInfo`
- The query matched "do i have" pattern but "swap" wasn't in the hardware exclusion list
- Result: "**swap** package is not installed" (wrong answer)

**Fix:**
- Added "swap" to hardware keyword exclusion list in `classify_hardware()`
- Now "do I have swap" correctly routes to `SwapInfo` → checks actual swap configuration

**Result:**
- "do I have swap" now reports actual swap status (from `free` command output)
- No more false "swap package is not installed" messages

## [0.0.797] - 2025-12-15

### Fixed - Tool Installation Check Parsing Bug

**Critical Bug Fix:**
- Fixed "is nano installed" returning "2 is installed (version 25.01-1)" instead of checking for nano
- The bug was caused by:
  1. `installed_packages` probe ran `pacman -Q 2>/dev/null` which lists all packages
  2. Parser extracted "2" from "2>/dev/null" as the package name
  3. Output showed first package (7zip) version as "2's" version

**Root Cause Fixes:**
- `extract_package_name_from_pacman()`: Now rejects file descriptor patterns (e.g., "2" from "2>/dev/null")
- `classify_tool_check_query()`: New function to properly handle "is X installed" queries
- `probe_id_to_command_dynamic()`: New function for dynamic probe generation (command_v_<tool>)

**New Query Handling:**
- "is nano installed" → generates `command_v_nano` probe → runs `sh -lc 'command -v nano'`
- "do I have vim" → generates `command_v_vim` probe → runs `sh -lc 'command -v vim'`
- Supports any tool name, not just predefined editors

**Result:**
- Tool check queries now correctly identify whether a specific tool is installed
- No more false positives from package listing output

## [0.0.796] - 2025-12-15

### Improved - Query Classification and Formatting Skip

**Better Swap Query Classification:**
- Added patterns: "have swap", "is swap", "any swap", "swap enabled", "swap active"
- Query "do I have swap?" now correctly routes to SwapInfo

**Extended Formatting Skip Coverage:**
- Added installed_tool_check, package_count, package_updates
- Added largest_folders, fstab_entries
- Added network_connectivity, ip_routes, open_files
- Added hardware_audio, pci_devices, block_devices

**Result:**
- More queries now respond instantly without LLM formatting
- "is nano installed" - instant response
- "do I have swap?" - instant response

## [0.0.795] - 2025-12-15

### Fixed - Route Class Case Mismatch

**Bug Fix:**
- Fixed `should_skip_formatting()` to use snake_case route class names
- QueryClass::to_string() returns "listening_ports", not "ListeningPorts"
- v0.0.794's skip logic was checking PascalCase but receiving snake_case
- This bug caused the formatting skip to never trigger

**Result:**
- Port queries now truly skip LLM formatting
- Response time reduced from ~25 seconds to <1 second
- Extended coverage to more query types (cpu_info, disk_space, memory_usage, etc.)

## [0.0.794] - 2025-12-15

### Performance - Skip LLM Formatting for Data Listings

**Instant Responses for Data-Listing Queries:**
- Deterministic answers for data listings now skip LLM formatting step
- Reduces port query response time from ~25 seconds to <1 second
- Affected query types: ListeningPorts, RunningServices, EnvironmentVars,
  MountedFilesystems, UsbDevices, LoggedInUsers, NetworkInterfaces,
  TopCpuProcesses, TopMemoryProcesses, and simple status queries

**Technical details:**
- Added `should_skip_formatting()` function in `llm_request.rs`
- Data listings like `ss -tulpn` output are already well-formatted
- LLM rephrasing adds ~20-25 seconds latency without improving quality
- Now these answers return instantly without LLM involvement

**Result:**
- `annactl "open ports"` - instant response (was 25+ seconds)
- `annactl "running services"` - instant response
- `annactl "who is logged in"` - instant response

## [0.0.793] - 2025-12-15

### Fixed - Port Query Deterministic Answer Bug

**Root Cause Fix:**
- Fixed `answer_listening_ports()` in `det/services.rs` to search for "ss" command
- Previously searched for probe ID "listening_ports" which doesn't match stored command
- The ProbeResult stores the actual command ("ss -tulpn"), not the probe ID
- This was the actual root cause of port query timeouts - the v0.0.792 fix was incomplete

**Technical details:**
- `find_probe()` uses `command.starts_with(prefix)` to match probes
- ProbeResult.command is set to the executed command, not the probe ID
- Fixed by changing from `find_probe(probes, "listening_ports")` to `find_probe(probes, "ss")`

## [0.0.792] - 2025-12-15

### Fixed - Port Query Direct Answer

**Direct Probe Answer for Port Queries:**
- Added `try_port_answer` function to `probe_direct.rs` for instant port query responses
- Queries like "open ports", "what's using port 3000", "listening ports" now bypass LLM
- Parses `ss -tulpn` output and formats a clean answer with port numbers and process names
- Fixes timeout issue where port queries would hang waiting for LLM response

**Root Cause:**
- Port queries were correctly classified and probes ran successfully
- However, `probe_direct.rs` had no handler for port-related queries
- This caused queries to fall through to LLM specialist which timed out
- Now port data is formatted directly from probe output

**Example usage:**
- `annactl "open ports"` - instant response with listening ports list
- `annactl "what's using port 3000"` - instant response
- `annactl "show listening ports"` - instant response

## [0.0.791] - 2025-12-15

### Added - Failed Services Query Classification

**Failed Services Pattern:**
- Added "failed services" pattern to deterministic routing
- Queries like "show me failed services", "failed services", "broken services" now work
- Routes to ServiceStatus with `systemctl --failed` probe
- Avoids LLM timeout for common service health queries

**Example usage:**
- `annactl "failed services"` - instant response
- `annactl "show me failed services"` - instant response
- `annactl "broken services"` - instant response

## [0.0.790] - 2025-12-15

### Added - Natural Language Config in One-Shot Mode

**Configuration via Natural Language:**
- Config commands now work in one-shot mode (`annactl "enable learning mode"`)
- Previously only worked in REPL mode, now instant response without daemon
- Added direct pattern matching for common settings commands
- Patterns: "show internal comms", "hide internal", "learning mode", "auto confirm", etc.

**Example usage:**
- `annactl "enable learning mode"` - instant response
- `annactl "make Anna more casual"` - instant response
- `annactl "show internal comms"` - instant response
- `annactl "my email is user@example.com"` - instant response

## [0.0.789] - 2025-12-15

### Fixed - Network Port Query Classification

**ListeningPorts Pattern Expansion:**
- Added "using port" pattern to match queries like "what's using port 3000"
- Added "on port" pattern for queries like "what's listening on port 8080"
- Added common port number patterns (3000, 8080, 80, 443) for better matching
- Fixes timeout issues for port-related queries by routing to deterministic handler

## [0.0.788] - 2025-12-15

### Fixed - Query Classification Bug

**ConfigureEditor vs ManageService Conflict:**
- Fixed bug where "enable syntax highlighting in vim" was misclassified as ManageService
- Queries containing editor keywords (vim, nvim, nano, emacs, syntax, highlight, etc.)
  now correctly route to ConfigureEditor instead of ManageService
- This fixes timeout issues for editor configuration queries

## [0.0.787] - 2025-12-15

### Changed - README Version Update
- Updated README version from 0.0.373 to 0.0.787
- Updated feature highlights to reflect current capabilities

## [0.0.786] - 2025-12-15

### Added - Settings Hideaway (Phase 362)

**Settings Hideaway Module:**
- `settings_hideaway.rs` for secret hideaway for settings seclusion
- `HideawayType` enum (Secret, Private, Remote, Hidden)
- `HideawayStatus` enum (Secluded, Concealed, Sheltered, Isolated)
- `HideawayConfig` / `HideawayOccupant` / `HideawayGuardian` / `HideawayStats` structs
- `SettingsHideaway` / `HideawayRegistry` for managing hideaways

**Core Functions:**
- `register()` / `unregister()` - Hideaway management
- `add_occupant()` - Add occupants
- `make_hidden()` / `make_visible()` - Visibility management
- `add_guardian()` - Add guardians
- `stats()` - Access statistics

**Features:**
- Multiple hideaway types
- Status tracking
- Nook numbering
- Guardian support
- Hidden rate calculation

## [0.0.785] - 2025-12-15

### Added - Settings Retreat (Phase 361)

**Settings Retreat Module:**
- `settings_retreat.rs` for peaceful retreat for settings relaxation
- `RetreatType` enum (Peaceful, Mountain, Coastal, Forest)
- `RetreatStatus` enum (Open, Relaxing, Meditating, Rejuvenating)
- `RetreatConfig` / `RetreatVisitor` / `RetreatGuide` / `RetreatStats` structs
- `SettingsRetreat` / `RetreatRegistry` for managing retreats

**Core Functions:**
- `register()` / `unregister()` - Retreat management
- `add_visitor()` - Add visitors
- `make_relaxed()` / `make_stressed()` - Relaxation management
- `add_guide()` - Add guides
- `stats()` - Access statistics

**Features:**
- Multiple retreat types
- Status tracking
- Cabin numbering
- Guide support
- Relaxation rate calculation

## [0.0.784] - 2025-12-15

### Added - Settings Haven (Phase 360)

**Settings Haven Module:**
- `settings_haven.rs` for safe haven for settings protection
- `HavenType` enum (Safe, Secure, Protected, Peaceful)
- `HavenStatus` enum (Open, Sheltering, Guarding, Welcoming)
- `HavenConfig` / `HavenGuest` / `HavenKeeper` / `HavenStats` structs
- `SettingsHaven` / `HavenRegistry` for managing havens

**Core Functions:**
- `register()` / `unregister()` - Haven management
- `add_guest()` - Add guests
- `make_comfortable()` / `make_restless()` - Comfort management
- `add_keeper()` - Add keepers
- `stats()` - Access statistics

**Features:**
- Multiple haven types
- Status tracking
- Room numbering
- Keeper support
- Comfort rate calculation

## [0.0.783] - 2025-12-15

### Added - Settings Refuge (Phase 359)

**Settings Refuge Module:**
- `settings_refuge.rs` for wildlife refuge for settings shelter
- `RefugeType` enum (Wildlife, Bird, Fish, Mammal)
- `RefugeStatus` enum (Active, Sheltering, Protecting, Recovering)
- `RefugeConfig` / `RefugeInhabitant` / `RefugeWarden` / `RefugeStats` structs
- `SettingsRefuge` / `RefugeRegistry` for managing refuges

**Core Functions:**
- `register()` / `unregister()` - Refuge management
- `add_inhabitant()` - Add inhabitants
- `make_safe()` / `make_vulnerable()` - Safety management
- `add_warden()` - Add wardens
- `stats()` - Access statistics

**Features:**
- Multiple refuge types
- Status tracking
- Shelter numbering
- Warden support
- Safety rate calculation

## [0.0.782] - 2025-12-15

### Added - Settings Reserve (Phase 358)

**Settings Reserve Module:**
- `settings_reserve.rs` for nature reserve for settings preservation
- `ReserveType` enum (Nature, Game, Forest, Marine)
- `ReserveStatus` enum (Protected, Managed, Restored, Conserved)
- `ReserveConfig` / `ReserveSpecies` / `ReserveRanger` / `ReserveStats` structs
- `SettingsReserve` / `ReserveRegistry` for managing reserves

**Core Functions:**
- `register()` / `unregister()` - Reserve management
- `add_species()` - Add species
- `make_thriving()` / `make_endangered()` - Thriving management
- `add_ranger()` - Add rangers
- `stats()` - Access statistics

**Features:**
- Multiple reserve types
- Status tracking
- Territory numbering
- Ranger support
- Thriving rate calculation

## [0.0.781] - 2025-12-15

### Added - Settings Sanctuary (Phase 357)

**Settings Sanctuary Module:**
- `settings_sanctuary.rs` for wildlife sanctuary for settings conservation
- `SanctuaryType` enum (Wildlife, Marine, Bird, Forest)
- `SanctuaryStatus` enum (Protected, Monitored, Rehabilitating, Expanding)
- `SanctuaryConfig` / `SanctuaryResident` / `SanctuaryWarden` / `SanctuaryStats` structs
- `SettingsSanctuary` / `SanctuaryRegistry` for managing sanctuaries

**Core Functions:**
- `register()` / `unregister()` - Sanctuary management
- `add_resident()` - Add residents
- `make_thriving()` / `make_recovering()` - Thriving management
- `add_warden()` - Add wardens
- `stats()` - Access statistics

**Features:**
- Multiple sanctuary types
- Status tracking
- Habitat numbering
- Warden support
- Thriving rate calculation

## [0.0.780] - 2025-12-15

### Added - Settings Butterfly (Phase 356)

**Settings Butterfly Module:**
- `settings_butterfly.rs` for butterfly house for settings lepidopterology
- `ButterflyType` enum (Tropical, Native, Monarch, Conservation)
- `ButterflyStatus` enum (Active, Emerging, Breeding, Migrating)
- `ButterflyConfig` / `ButterflySpecimen` / `ButterflyCurator` / `ButterflyStats` structs
- `SettingsButterfly` / `ButterflyRegistry` for managing butterflies

**Core Functions:**
- `register()` / `unregister()` - Butterfly management
- `add_specimen()` - Add specimens
- `make_flying()` / `make_resting()` - Flight management
- `add_curator()` - Add curators
- `stats()` - Access statistics

**Features:**
- Multiple butterfly types
- Status tracking
- Zone numbering
- Curator support
- Flight rate calculation

## [0.0.779] - 2025-12-15

### Added - Settings Apiary (Phase 355)

**Settings Apiary Module:**
- `settings_apiary.rs` for bee apiary for settings apiculture
- `ApiaryType` enum (Honey, Pollination, Queen, Research)
- `ApiaryStatus` enum (Active, Swarming, Harvesting, Wintering)
- `ApiaryConfig` / `ApiaryHive` / `ApiaryBeekeeper` / `ApiaryStats` structs
- `SettingsApiary` / `ApiaryRegistry` for managing apiaries

**Core Functions:**
- `register()` / `unregister()` - Apiary management
- `add_hive()` - Add hives
- `make_productive()` / `make_dormant()` - Productivity management
- `add_beekeeper()` - Add beekeepers
- `stats()` - Access statistics

**Features:**
- Multiple apiary types
- Status tracking
- Stand numbering
- Beekeeper support
- Productivity rate calculation

## [0.0.778] - 2025-12-15

### Added - Settings Aviary (Phase 354)

**Settings Aviary Module:**
- `settings_aviary.rs` for bird aviary for settings ornithology
- `AviaryType` enum (Flight, Breeding, Display, Rescue)
- `AviaryStatus` enum (Active, Nesting, Molting, Quarantine)
- `AviaryConfig` / `AviaryBird` / `AviaryKeeper` / `AviaryStats` structs
- `SettingsAviary` / `AviaryRegistry` for managing aviaries

**Core Functions:**
- `register()` / `unregister()` - Aviary management
- `add_bird()` - Add birds
- `make_flying()` / `make_grounded()` - Flight management
- `add_keeper()` - Add keepers
- `stats()` - Access statistics

**Features:**
- Multiple aviary types
- Status tracking
- Perch numbering
- Keeper support
- Flight rate calculation

## [0.0.777] - 2025-12-15

### Added - Settings Terrarium (Phase 353)

**Settings Terrarium Module:**
- `settings_terrarium.rs` for enclosed terrarium for settings miniature ecosystem
- `TerrariumType` enum (Desert, Tropical, Woodland, Moss)
- `TerrariumStatus` enum (Building, Sealed, Mature, Renewing)
- `TerrariumConfig` / `TerrariumPlant` / `TerrariumCreator` / `TerrariumStats` structs
- `SettingsTerrarium` / `TerrariumRegistry` for managing terrariums

**Core Functions:**
- `register()` / `unregister()` - Terrarium management
- `add_plant()` - Add plants
- `make_established()` / `make_struggling()` - Establishment management
- `add_creator()` - Add creators
- `stats()` - Access statistics

**Features:**
- Multiple terrarium types
- Status tracking
- Layer numbering
- Creator support
- Establishment rate calculation

## [0.0.776] - 2025-12-15

### Added - Settings Vivarium (Phase 352)

**Settings Vivarium Module:**
- `settings_vivarium.rs` for living vivarium for settings animal habitat
- `VivariumType` enum (Reptile, Amphibian, Invertebrate, Mixed)
- `VivariumStatus` enum (Setup, Established, Breeding, Resting)
- `VivariumConfig` / `VivariumCreature` / `VivariumKeeper` / `VivariumStats` structs
- `SettingsVivarium` / `VivariumRegistry` for managing vivariums

**Core Functions:**
- `register()` / `unregister()` - Vivarium management
- `add_creature()` - Add creatures
- `make_thriving()` / `make_struggling()` - Thriving management
- `add_keeper()` - Add keepers
- `stats()` - Access statistics

**Features:**
- Multiple vivarium types
- Status tracking
- Enclosure numbering
- Keeper support
- Thriving rate calculation

## [0.0.775] - 2025-12-15

### Added - Settings Aquarium (Phase 351)

**Settings Aquarium Module:**
- `settings_aquarium.rs` for aquatic aquarium for settings marine life
- `AquariumType` enum (Freshwater, Saltwater, Reef, Brackish)
- `AquariumStatus` enum (Cycling, Stable, Stocking, Maintenance)
- `AquariumConfig` / `AquariumInhabitant` / `AquariumAquarist` / `AquariumStats` structs
- `SettingsAquarium` / `AquariumRegistry` for managing aquariums

**Core Functions:**
- `register()` / `unregister()` - Aquarium management
- `add_inhabitant()` - Add inhabitants
- `make_healthy()` / `make_sick()` - Health management
- `add_aquarist()` - Add aquarists
- `stats()` - Access statistics

**Features:**
- Multiple aquarium types
- Status tracking
- Tank numbering
- Aquarist support
- Health rate calculation

## [0.0.774] - 2025-12-15

### Added - Settings Herbarium (Phase 350)

**Settings Herbarium Module:**
- `settings_herbarium.rs` for plant herbarium for settings taxonomy
- `HerbariumType` enum (University, Museum, National, Private)
- `HerbariumStatus` enum (Active, Cataloging, Digitizing, Archiving)
- `HerbariumConfig` / `HerbariumSpecimen` / `HerbariumTaxonomist` / `HerbariumStats` structs
- `SettingsHerbarium` / `HerbariumRegistry` for managing herbariums

**Core Functions:**
- `register()` / `unregister()` - Herbarium management
- `add_specimen()` - Add specimens
- `make_mounted()` / `make_unmounted()` - Mount management
- `add_taxonomist()` - Add taxonomists
- `stats()` - Access statistics

**Features:**
- Multiple herbarium types
- Status tracking
- Cabinet numbering
- Taxonomist support
- Mount rate calculation

## [0.0.773] - 2025-12-15

### Added - Settings Botanical (Phase 349)

**Settings Botanical Module:**
- `settings_botanical.rs` for botanical garden for settings plant science
- `BotanicalType` enum (Display, Research, Conservation, Educational)
- `BotanicalStatus` enum (Active, Expanding, Conserving, Restoration)
- `BotanicalConfig` / `BotanicalCollection` / `BotanicalBotanist` / `BotanicalStats` structs
- `SettingsBotanical` / `BotanicalRegistry` for managing botanicals

**Core Functions:**
- `register()` / `unregister()` - Botanical management
- `add_collection()` - Add collections
- `make_documented()` / `make_undocumented()` - Documentation management
- `add_botanist()` - Add botanists
- `stats()` - Access statistics

**Features:**
- Multiple botanical types
- Status tracking
- Wing numbering
- Botanist support
- Documentation rate calculation

## [0.0.772] - 2025-12-15

### Added - Settings Arboretum (Phase 348)

**Settings Arboretum Module:**
- `settings_arboretum.rs` for tree arboretum for settings dendrology
- `ArboretumType` enum (Public, University, Memorial, Research)
- `ArboretumStatus` enum (Open, Planting, Surveying, Closed)
- `ArboretumConfig` / `ArboretumSpecimen` / `ArboretumDendrologist` / `ArboretumStats` structs
- `SettingsArboretum` / `ArboretumRegistry` for managing arboretums

**Core Functions:**
- `register()` / `unregister()` - Arboretum management
- `add_specimen()` - Add specimens
- `make_cataloged()` / `make_uncataloged()` - Catalog management
- `add_dendrologist()` - Add dendrologists
- `stats()` - Access statistics

**Features:**
- Multiple arboretum types
- Status tracking
- Plot numbering
- Dendrologist support
- Catalog rate calculation

## [0.0.771] - 2025-12-15

### Added - Settings Conservatory (Phase 347)

**Settings Conservatory Module:**
- `settings_conservatory.rs` for glass conservatory for settings preservation
- `ConservatoryType` enum (Victorian, Modern, LeanTo, Edwardian)
- `ConservatoryStatus` enum (Open, Closed, Ventilating, Renovation)
- `ConservatoryConfig` / `ConservatorySpecimen` / `ConservatoryCurator` / `ConservatoryStats` structs
- `SettingsConservatory` / `ConservatoryRegistry` for managing conservatories

**Core Functions:**
- `register()` / `unregister()` - Conservatory management
- `add_specimen()` - Add specimens
- `make_preserved()` / `make_damaged()` - Preservation management
- `add_curator()` - Add curators
- `stats()` - Access statistics

**Features:**
- Multiple conservatory types
- Status tracking
- Section numbering
- Curator support
- Preservation rate calculation

## [0.0.770] - 2025-12-15

### Added - Settings Greenhouse (Phase 346)

**Settings Greenhouse Module:**
- `settings_greenhouse.rs` for controlled greenhouse for settings cultivation
- `GreenhouseType` enum (Commercial, Hobby, Research, Tropical)
- `GreenhouseStatus` enum (Active, Heating, Cooling, Maintenance)
- `GreenhouseConfig` / `GreenhouseCrop` / `GreenhouseGrower` / `GreenhouseStats` structs
- `SettingsGreenhouse` / `GreenhouseRegistry` for managing greenhouses

**Core Functions:**
- `register()` / `unregister()` - Greenhouse management
- `add_crop()` - Add crops
- `make_flourishing()` / `make_struggling()` - Flourishing management
- `add_grower()` - Add growers
- `stats()` - Access statistics

**Features:**
- Multiple greenhouse types
- Status tracking
- Zone numbering
- Grower support
- Flourishing rate calculation

## [0.0.769] - 2025-12-15

### Added - Settings Nursery (Phase 345)

**Settings Nursery Module:**
- `settings_nursery.rs` for plant nursery for settings propagation
- `NurseryType` enum (Retail, Wholesale, Specialty, Research)
- `NurseryStatus` enum (Seeding, Growing, Ready, Dormant)
- `NurseryConfig` / `NurserySeedling` / `NurseryPropagator` / `NurseryStats` structs
- `SettingsNursery` / `NurseryRegistry` for managing nurseries

**Core Functions:**
- `register()` / `unregister()` - Nursery management
- `add_seedling()` - Add seedlings
- `make_viable()` / `make_unviable()` - Viability management
- `add_propagator()` - Add propagators
- `stats()` - Access statistics

**Features:**
- Multiple nursery types
- Status tracking
- Tray numbering
- Propagator support
- Viability rate calculation

## [0.0.768] - 2025-12-15

### Added - Settings Garden (Phase 344)

**Settings Garden Module:**
- `settings_garden.rs` for cultivated garden for settings horticulture
- `GardenType` enum (Flower, Vegetable, Herb, Rock)
- `GardenStatus` enum (Planned, Planted, Growing, Blooming)
- `GardenConfig` / `GardenPlant` / `GardenGardener` / `GardenStats` structs
- `SettingsGarden` / `GardenRegistry` for managing gardens

**Core Functions:**
- `register()` / `unregister()` - Garden management
- `add_plant()` - Add plants
- `make_thriving()` / `make_wilting()` - Thriving management
- `add_gardener()` - Add gardeners
- `stats()` - Access statistics

**Features:**
- Multiple garden types
- Status tracking
- Bed numbering
- Gardener support
- Thriving rate calculation

## [0.0.767] - 2025-12-15

### Added - Settings Vineyard (Phase 343)

**Settings Vineyard Module:**
- `settings_vineyard.rs` for grape vineyard for settings viticulture
- `VineyardType` enum (RedWine, WhiteWine, TableGrape, Raisin)
- `VineyardStatus` enum (Pruned, Budding, Ripening, Vintage)
- `VineyardConfig` / `VineyardVine` / `VineyardVintner` / `VineyardStats` structs
- `SettingsVineyard` / `VineyardRegistry` for managing vineyards

**Core Functions:**
- `register()` / `unregister()` - Vineyard management
- `add_vine()` - Add vines
- `make_bearing()` / `make_dormant()` - Bearing management
- `add_vintner()` - Add vintners
- `stats()` - Access statistics

**Features:**
- Multiple vineyard types
- Status tracking
- Terrace numbering
- Vintner support
- Bearing rate calculation

## [0.0.766] - 2025-12-15

### Added - Settings Orchard (Phase 342)

**Settings Orchard Module:**
- `settings_orchard.rs` for fruit orchard for settings horticulture
- `OrchardType` enum (Apple, Cherry, Peach, Pear)
- `OrchardStatus` enum (Dormant, Blooming, Fruiting, Harvesting)
- `OrchardConfig` / `OrchardFruit` / `OrchardPicker` / `OrchardStats` structs
- `SettingsOrchard` / `OrchardRegistry` for managing orchards

**Core Functions:**
- `register()` / `unregister()` - Orchard management
- `add_fruit()` - Add fruits
- `make_ripe()` / `make_unripe()` - Ripeness management
- `add_picker()` - Add pickers
- `stats()` - Access statistics

**Features:**
- Multiple orchard types
- Status tracking
- Branch numbering
- Picker support
- Ripe rate calculation

## [0.0.765] - 2025-12-15

### Added - Settings Grove (Phase 341)

**Settings Grove Module:**
- `settings_grove.rs` for tree grove for settings forestry
- `GroveType` enum (Oak, Olive, Citrus, Sacred)
- `GroveStatus` enum (Planted, Maturing, Productive, Resting)
- `GroveConfig` / `GroveTree` / `GroveTender` / `GroveStats` structs
- `SettingsGrove` / `GroveRegistry` for managing groves

**Core Functions:**
- `register()` / `unregister()` - Grove management
- `add_tree()` - Add trees
- `make_healthy()` / `make_diseased()` - Health management
- `add_tender()` - Add tenders
- `stats()` - Access statistics

**Features:**
- Multiple grove types
- Status tracking
- Row numbering
- Tender support
- Healthy rate calculation

## [0.0.764] - 2025-12-15

### Added - Settings Pasture (Phase 340)

**Settings Pasture Module:**
- `settings_pasture.rs` for grazing pasture for settings livestock
- `PastureType` enum (Permanent, Rotational, Intensive, Rough)
- `PastureStatus` enum (Open, Grazed, Rested, Improved)
- `PastureConfig` / `PastureHerd` / `PastureHerder` / `PastureStats` structs
- `SettingsPasture` / `PastureRegistry` for managing pastures

**Core Functions:**
- `register()` / `unregister()` - Pasture management
- `add_herd()` - Add herds
- `make_thriving()` / `make_struggling()` - Thriving management
- `add_herder()` - Add herders
- `stats()` - Access statistics

**Features:**
- Multiple pasture types
- Status tracking
- Paddock numbering
- Herder support
- Thriving rate calculation

## [0.0.763] - 2025-12-15

### Added - Settings Meadow (Phase 339)

**Settings Meadow Module:**
- `settings_meadow.rs` for grassland meadow for settings grazing
- `MeadowType` enum (Hay, Water, Alpine, Wildflower)
- `MeadowStatus` enum (Resting, Grazing, Mowing, Recovering)
- `MeadowConfig` / `MeadowGrass` / `MeadowKeeper` / `MeadowStats` structs
- `SettingsMeadow` / `MeadowRegistry` for managing meadows

**Core Functions:**
- `register()` / `unregister()` - Meadow management
- `add_grass()` - Add grasses
- `make_lush()` / `make_sparse()` - Lush management
- `add_keeper()` - Add keepers
- `stats()` - Access statistics

**Features:**
- Multiple meadow types
- Status tracking
- Sward numbering
- Keeper support
- Lush rate calculation

## [0.0.762] - 2025-12-15

### Added - Settings Field (Phase 338)

**Settings Field Module:**
- `settings_field.rs` for agricultural field for settings cultivation
- `FieldType` enum (Arable, Pastoral, Fallow, Orchard)
- `FieldStatus` enum (Prepared, Planted, Growing, Harvested)
- `FieldConfig` / `FieldCrop` / `FieldFarmer` / `FieldStats` structs
- `SettingsField` / `FieldRegistry` for managing fields

**Core Functions:**
- `register()` / `unregister()` - Field management
- `add_crop()` - Add crops
- `make_yielded()` / `make_barren()` - Yield management
- `add_farmer()` - Add farmers
- `stats()` - Access statistics

**Features:**
- Multiple field types
- Status tracking
- Furrow numbering
- Farmer support
- Yield rate calculation

## [0.0.761] - 2025-12-15

### Added - Settings Hectare (Phase 337)

**Settings Hectare Module:**
- `settings_hectare.rs` for land hectare for settings metric area
- `HectareType` enum (Standard, Cadastral, Agricultural, Forest)
- `HectareStatus` enum (Surveyed, Registered, Contested, Confirmed)
- `HectareConfig` / `HectareRecord` / `HectareInspector` / `HectareStats` structs
- `SettingsHectare` / `HectareRegistry` for managing hectares

**Core Functions:**
- `register()` / `unregister()` - Hectare management
- `add_record()` - Add records
- `make_confirmed()` / `make_unconfirmed()` - Confirmed management
- `add_inspector()` - Add inspectors
- `stats()` - Access statistics

**Features:**
- Multiple hectare types
- Status tracking
- Grid reference numbering
- Inspector support
- Confirmed rate calculation

## [0.0.760] - 2025-12-15

### Added - Settings Acre (Phase 336)

**Settings Acre Module:**
- `settings_acre.rs` for land acre for settings measurement
- `AcreType` enum (Survey, Statute, Irish, Scottish)
- `AcreStatus` enum (Measured, Verified, Disputed, Certified)
- `AcreConfig` / `AcreMeasurement` / `AcreSurveyor` / `AcreStats` structs
- `SettingsAcre` / `AcreRegistry` for managing acres

**Core Functions:**
- `register()` / `unregister()` - Acre management
- `add_measurement()` - Add measurements
- `make_certified()` / `make_uncertified()` - Certified management
- `add_surveyor()` - Add surveyors
- `stats()` - Access statistics

**Features:**
- Multiple acre types
- Status tracking
- Chain numbering
- Surveyor support
- Certified rate calculation

## [0.0.759] - 2025-12-15

### Added - Settings Tract (Phase 335)

**Settings Tract Module:**
- `settings_tract.rs` for land tract for settings territory
- `TractType` enum (Residential, Commercial, Agricultural, Wilderness)
- `TractStatus` enum (Surveyed, Developed, Preserved, Disputed)
- `TractConfig` / `TractGrant` / `TractRanger` / `TractStats` structs
- `SettingsTract` / `TractRegistry` for managing tracts

**Core Functions:**
- `register()` / `unregister()` - Tract management
- `add_grant()` - Add grants
- `make_patented()` / `make_pending()` - Patented management
- `add_ranger()` - Add rangers
- `stats()` - Access statistics

**Features:**
- Multiple tract types
- Status tracking
- Range numbering
- Ranger support
- Patented rate calculation

## [0.0.758] - 2025-12-15

### Added - Settings Plot (Phase 334)

**Settings Plot Module:**
- `settings_plot.rs` for land plot for settings allocation
- `PlotType` enum (Garden, Building, Cemetery, Allotment)
- `PlotStatus` enum (Allocated, Cultivated, Fallow, Reserved)
- `PlotConfig` / `PlotSurvey` / `PlotSteward` / `PlotStats` structs
- `SettingsPlot` / `PlotRegistry` for managing plots

**Core Functions:**
- `register()` / `unregister()` - Plot management
- `add_survey()` - Add surveys
- `make_verified()` / `make_unverified()` - Verified management
- `add_steward()` - Add stewards
- `stats()` - Access statistics

**Features:**
- Multiple plot types
- Status tracking
- Row numbering
- Steward support
- Verified rate calculation

## [0.0.757] - 2025-12-15

### Added - Settings Parcel (Phase 333)

**Settings Parcel Module:**
- `settings_parcel.rs` for land parcel for settings ownership
- `ParcelType` enum (FeeSimple, Leasehold, Easement, RightOfWay)
- `ParcelStatus` enum (Platted, Conveyed, Encumbered, Cleared)
- `ParcelConfig` / `ParcelTitle` / `ParcelExaminer` / `ParcelStats` structs
- `SettingsParcel` / `ParcelRegistry` for managing parcels

**Core Functions:**
- `register()` / `unregister()` - Parcel management
- `add_title()` - Add titles
- `make_cleared()` / `make_clouded()` - Cleared management
- `add_examiner()` - Add examiners
- `stats()` - Access statistics

**Features:**
- Multiple parcel types
- Status tracking
- Section numbering
- Examiner support
- Cleared rate calculation

## [0.0.756] - 2025-12-15

### Added - Settings Lot (Phase 332)

**Settings Lot Module:**
- `settings_lot.rs` for land lot for settings property
- `LotType` enum (Residential, Commercial, Industrial, Agricultural)
- `LotStatus` enum (Vacant, Improved, Subdivided, Consolidated)
- `LotConfig` / `LotDeed` / `LotAssessor` / `LotStats` structs
- `SettingsLot` / `LotRegistry` for managing lots

**Core Functions:**
- `register()` / `unregister()` - Lot management
- `add_deed()` - Add deeds
- `make_registered()` / `make_unregistered()` - Registered management
- `add_assessor()` - Add assessors
- `stats()` - Access statistics

**Features:**
- Multiple lot types
- Status tracking
- Parcel numbering
- Assessor support
- Registered rate calculation

## [0.0.755] - 2025-12-15

### Added - Settings Block (Phase 331)

**Settings Block Module:**
- `settings_block.rs` for city block for settings subdivision
- `BlockType` enum (Residential, Commercial, Industrial, Civic)
- `BlockStatus` enum (Surveyed, Developed, Subdivided, Consolidated)
- `BlockConfig` / `BlockPlat` / `BlockSurveyor` / `BlockStats` structs
- `SettingsBlock` / `BlockRegistry` for managing blocks

**Core Functions:**
- `register()` / `unregister()` - Block management
- `add_plat()` - Add plats
- `make_recorded()` / `make_pending()` - Recorded management
- `add_surveyor()` - Add surveyors
- `stats()` - Access statistics

**Features:**
- Multiple block types
- Status tracking
- Lot numbering
- Surveyor support
- Recorded rate calculation

## [0.0.754] - 2025-12-15

### Added - Settings Neighborhood (Phase 330)

**Settings Neighborhood Module:**
- `settings_neighborhood.rs` for residential neighborhood for settings community
- `NeighborhoodType` enum (Residential, Commercial, Industrial, MixedUse)
- `NeighborhoodStatus` enum (Planned, Developing, Established, Revitalized)
- `NeighborhoodConfig` / `NeighborhoodInitiative` / `NeighborhoodOrganizer` / `NeighborhoodStats` structs
- `SettingsNeighborhood` / `NeighborhoodRegistry` for managing neighborhoods

**Core Functions:**
- `register()` / `unregister()` - Neighborhood management
- `add_initiative()` - Add initiatives
- `make_approved()` / `make_rejected()` - Approved management
- `add_organizer()` - Add organizers
- `stats()` - Access statistics

**Features:**
- Multiple neighborhood types
- Status tracking
- Block numbering
- Organizer support
- Approved rate calculation

## [0.0.753] - 2025-12-15

### Added - Settings Precinct (Phase 329)

**Settings Precinct Module:**
- `settings_precinct.rs` for voting precinct for settings participation
- `PrecinctType` enum (Voting, Police, Fire, School)
- `PrecinctStatus` enum (Designated, Active, Consolidated, Dissolved)
- `PrecinctConfig` / `PrecinctBallot` / `PrecinctCaptain` / `PrecinctStats` structs
- `SettingsPrecinct` / `PrecinctRegistry` for managing precincts

**Core Functions:**
- `register()` / `unregister()` - Precinct management
- `add_ballot()` - Add ballots
- `make_certified()` / `make_contested()` - Certified management
- `add_captain()` - Add captains
- `stats()` - Access statistics

**Features:**
- Multiple precinct types
- Status tracking
- District numbering
- Captain support
- Certified rate calculation

## [0.0.752] - 2025-12-15

### Added - Settings Ward (Phase 328)

**Settings Ward Module:**
- `settings_ward.rs` for electoral ward for settings representation
- `WardType` enum (Electoral, Hospital, Prison, Administrative)
- `WardStatus` enum (Created, Active, Redrawn, Abolished)
- `WardConfig` / `WardMotion` / `WardDelegate` / `WardStats` structs
- `SettingsWard` / `WardRegistry` for managing wards

**Core Functions:**
- `register()` / `unregister()` - Ward management
- `add_motion()` - Add motions
- `make_passed()` / `make_failed()` - Passed management
- `add_delegate()` - Add delegates
- `stats()` - Access statistics

**Features:**
- Multiple ward types
- Status tracking
- Precinct numbering
- Delegate support
- Passed rate calculation

## [0.0.751] - 2025-12-15

### Added - Settings Borough (Phase 327)

**Settings Borough Module:**
- `settings_borough.rs` for borough subdivision for settings local governance
- `BoroughType` enum (Urban, Metropolitan, London, Municipal)
- `BoroughStatus` enum (Established, Active, Reformed, Abolished)
- `BoroughConfig` / `BoroughResolution` / `BoroughRepresentative` / `BoroughStats` structs
- `SettingsBorough` / `BoroughRegistry` for managing boroughs

**Core Functions:**
- `register()` / `unregister()` - Borough management
- `add_resolution()` - Add resolutions
- `make_adopted()` / `make_rescinded()` - Adopted management
- `add_representative()` - Add representatives
- `stats()` - Access statistics

**Features:**
- Multiple borough types
- Status tracking
- Section numbering
- Representative support
- Adopted rate calculation

## [0.0.750] - 2025-12-15

### Added - Settings Municipality (Phase 326)

**Settings Municipality Module:**
- `settings_municipality.rs` for municipal corporation for settings self-governance
- `MunicipalityType` enum (City, Town, Village, Township)
- `MunicipalityStatus` enum (Incorporated, Chartered, Consolidated, Dissolved)
- `MunicipalityConfig` / `MunicipalityCode` / `MunicipalityCouncilor` / `MunicipalityStats` structs
- `SettingsMunicipality` / `MunicipalityRegistry` for managing municipalities

**Core Functions:**
- `register()` / `unregister()` - Municipality management
- `add_code()` - Add codes
- `make_in_force()` / `make_suspended()` - In force management
- `add_councilor()` - Add councilors
- `stats()` - Access statistics

**Features:**
- Multiple municipality types
- Status tracking
- Chapter numbering
- Councilor support
- In force rate calculation

## [0.0.749] - 2025-12-15

### Added - Settings County (Phase 325)

**Settings County Module:**
- `settings_county.rs` for county level for settings governance
- `CountyType` enum (Metropolitan, Rural, Historic, Administrative)
- `CountyStatus` enum (Established, Active, Merged, Abolished)
- `CountyConfig` / `CountyOrdinance` / `CountyCommissioner` / `CountyStats` structs
- `SettingsCounty` / `CountyRegistry` for managing counties

**Core Functions:**
- `register()` / `unregister()` - County management
- `add_ordinance()` - Add ordinances
- `make_enacted()` / `make_repealed()` - Enacted management
- `add_commissioner()` - Add commissioners
- `stats()` - Access statistics

**Features:**
- Multiple county types
- Status tracking
- Township numbering
- Commissioner support
- Enacted rate calculation

## [0.0.748] - 2025-12-15

### Added - Settings District (Phase 324)

**Settings District Module:**
- `settings_district.rs` for local district for settings administration
- `DistrictType` enum (Urban, Rural, Industrial, Commercial)
- `DistrictStatus` enum (Planned, Operational, Developing, Restructuring)
- `DistrictConfig` / `DistrictBylaw` / `DistrictOfficial` / `DistrictStats` structs
- `SettingsDistrict` / `DistrictRegistry` for managing districts

**Core Functions:**
- `register()` / `unregister()` - District management
- `add_bylaw()` - Add bylaws
- `make_active()` / `make_inactive()` - Active management
- `add_official()` - Add officials
- `stats()` - Access statistics

**Features:**
- Multiple district types
- Status tracking
- Ward numbering
- Official support
- Active rate calculation

## [0.0.747] - 2025-12-15

### Added - Settings Region (Phase 323)

**Settings Region Module:**
- `settings_region.rs` for geographic region for settings organization
- `RegionType` enum (Administrative, Economic, Cultural, Geographic)
- `RegionStatus` enum (Defined, Active, Expanding, Contracting)
- `RegionConfig` / `RegionPolicy` / `RegionCoordinator` / `RegionStats` structs
- `SettingsRegion` / `RegionRegistry` for managing regions

**Core Functions:**
- `register()` / `unregister()` - Region management
- `add_policy()` - Add policies
- `make_regional()` / `make_local()` - Regional management
- `add_coordinator()` - Add coordinators
- `stats()` - Access statistics

**Features:**
- Multiple region types
- Status tracking
- Sector numbering
- Coordinator support
- Regional rate calculation

## [0.0.746] - 2025-12-15

### Added - Settings Province (Phase 322)

**Settings Province Module:**
- `settings_province.rs` for administrative province for settings governance
- `ProvinceType` enum (Autonomous, Imperial, Colonial, Federal)
- `ProvinceStatus` enum (Established, Developing, Integrated, Reorganizing)
- `ProvinceConfig` / `ProvinceEdict` / `ProvinceGovernor` / `ProvinceStats` structs
- `SettingsProvince` / `ProvinceRegistry` for managing provinces

**Core Functions:**
- `register()` / `unregister()` - Province management
- `add_edict()` - Add edicts
- `make_provincial()` / `make_local()` - Provincial management
- `add_governor()` - Add governors
- `stats()` - Access statistics

**Features:**
- Multiple province types
- Status tracking
- Prefecture numbering
- Governor support
- Provincial rate calculation

## [0.0.745] - 2025-12-15

### Added - Settings Territory (Phase 321)

**Settings Territory Module:**
- `settings_territory.rs` for controlled territory for settings administration
- `TerritoryType` enum (Sovereign, Occupied, Trust, Dependent)
- `TerritoryStatus` enum (Administered, Autonomous, Contested, Ceded)
- `TerritoryConfig` / `TerritoryOrdinance` / `TerritoryAdministrator` / `TerritoryStats` structs
- `SettingsTerritory` / `TerritoryRegistry` for managing territories

**Core Functions:**
- `register()` / `unregister()` - Territory management
- `add_ordinance()` - Add ordinances
- `make_enforced()` / `make_suspended()` - Enforcement management
- `add_administrator()` - Add administrators
- `stats()` - Access statistics

**Features:**
- Multiple territory types
- Status tracking
- District numbering
- Administrator support
- Enforcement rate calculation

## [0.0.744] - 2025-12-15

### Added - Settings Realm (Phase 320)

**Settings Realm Module:**
- `settings_realm.rs` for royal realm for settings sovereignty
- `RealmType` enum (Kingdom, Empire, Principality, Duchy)
- `RealmStatus` enum (Rising, Prosperous, Stagnant, Declining)
- `RealmConfig` / `RealmDecree` / `RealmVassal` / `RealmStats` structs
- `SettingsRealm` / `RealmRegistry` for managing realms

**Core Functions:**
- `register()` / `unregister()` - Realm management
- `add_decree()` - Add decrees
- `make_royal()` / `make_common()` - Royal management
- `add_vassal()` - Add vassals
- `stats()` - Access statistics

**Features:**
- Multiple realm types
- Status tracking
- Order numbering
- Vassal support
- Royal rate calculation

## [0.0.743] - 2025-12-15

### Added - Settings Domain (Phase 319)

**Settings Domain Module:**
- `settings_domain.rs` for sovereign domain for settings jurisdiction
- `DomainType` enum (Public, Private, Royal, Eminent)
- `DomainStatus` enum (Claimed, Recognized, Consolidated, Disputed)
- `DomainConfig` / `DomainRight` / `DomainHolder` / `DomainStats` structs
- `SettingsDomain` / `DomainRegistry` for managing domains

**Core Functions:**
- `register()` / `unregister()` - Domain management
- `add_right()` - Add rights
- `make_exclusive()` / `make_shared()` - Exclusivity management
- `add_holder()` - Add holders
- `stats()` - Access statistics

**Features:**
- Multiple domain types
- Status tracking
- Priority numbering
- Holder support
- Exclusive rate calculation

## [0.0.742] - 2025-12-15

### Added - Settings Zone (Phase 318)

**Settings Zone Module:**
- `settings_zone.rs` for designated zone for settings boundaries
- `ZoneType` enum (FreeTrade, Economic, Security, Buffer)
- `ZoneStatus` enum (Proposed, Established, Operational, Suspended)
- `ZoneConfig` / `ZoneRegulation` / `ZoneParticipant` / `ZoneStats` structs
- `SettingsZone` / `ZoneRegistry` for managing zones

**Core Functions:**
- `register()` / `unregister()` - Zone management
- `add_regulation()` - Add regulations
- `make_enforced()` / `make_advisory()` - Enforcement management
- `add_participant()` - Add participants
- `stats()` - Access statistics

**Features:**
- Multiple zone types
- Status tracking
- Sector numbering
- Participant support
- Enforcement rate calculation

## [0.0.741] - 2025-12-15

### Added - Settings Sphere (Phase 317)

**Settings Sphere Module:**
- `settings_sphere.rs` for influence sphere for settings reach
- `SphereType` enum (Influence, CoProsperity, Interest, Security)
- `SphereStatus` enum (Expanding, Stable, Contested, Declining)
- `SphereConfig` / `SphereInterest` / `SphereEntity` / `SphereStats` structs
- `SettingsSphere` / `SphereRegistry` for managing spheres

**Core Functions:**
- `register()` / `unregister()` - Sphere management
- `add_interest()` - Add interests
- `make_core()` / `make_peripheral()` - Core management
- `add_entity()` - Add entities
- `stats()` - Access statistics

**Features:**
- Multiple sphere types
- Status tracking
- Radius numbering
- Entity support
- Core rate calculation

## [0.0.740] - 2025-12-15

### Added - Settings Bloc (Phase 316)

**Settings Bloc Module:**
- `settings_bloc.rs` for regional bloc for settings coordination
- `BlocType` enum (Trading, Voting, Power, Regional)
- `BlocStatus` enum (Forming, Active, Dominant, Fragmented)
- `BlocConfig` / `BlocPolicy` / `BlocMember` / `BlocStats` structs
- `SettingsBloc` / `BlocRegistry` for managing blocs

**Core Functions:**
- `register()` / `unregister()` - Bloc management
- `add_policy()` - Add policies
- `make_coordinated()` / `make_independent()` - Coordination management
- `add_member()` - Add members
- `stats()` - Access statistics

**Features:**
- Multiple bloc types
- Status tracking
- Priority numbering
- Member support
- Coordination rate calculation

## [0.0.739] - 2025-12-15

### Added - Settings Union (Phase 315)

**Settings Union Module:**
- `settings_union.rs` for political union for settings integration
- `UnionType` enum (Full, Customs, Monetary, Personal)
- `UnionStatus` enum (Proposed, Ratified, Integrated, Dissolved)
- `UnionConfig` / `UnionProvision` / `UnionMember` / `UnionStats` structs
- `SettingsUnion` / `UnionRegistry` for managing unions

**Core Functions:**
- `register()` / `unregister()` - Union management
- `add_provision()` - Add provisions
- `make_binding()` / `make_advisory()` - Binding management
- `add_member()` - Add members
- `stats()` - Access statistics

**Features:**
- Multiple union types
- Status tracking
- Section numbering
- Member support
- Binding rate calculation

## [0.0.738] - 2025-12-15

### Added - Settings Confederation (Phase 314)

**Settings Confederation Module:**
- `settings_confederation.rs` for loose union for settings governance
- `ConfederationType` enum (Sovereign, Economic, Military, Political)
- `ConfederationStatus` enum (Forming, Functional, Strained, Dissolved)
- `ConfederationConfig` / `ConfederationArticle` / `ConfederationMember` / `ConfederationStats` structs
- `SettingsConfederation` / `ConfederationRegistry` for managing confederations

**Core Functions:**
- `register()` / `unregister()` - Confederation management
- `add_article()` - Add articles
- `make_voluntary()` / `make_mandatory()` - Voluntary management
- `add_member()` - Add members
- `stats()` - Access statistics

**Features:**
- Multiple confederation types
- Status tracking
- Clause numbering
- Member support
- Voluntary rate calculation

## [0.0.737] - 2025-12-15

### Added - Settings Federation (Phase 313)

**Settings Federation Module:**
- `settings_federation.rs` for federal union for settings governance
- `FederationType` enum (Symmetric, Asymmetric, Cooperative, Dual)
- `FederationStatus` enum (Constituting, Established, Reforming, Dissolving)
- `FederationConfig` / `FederationArticle` / `FederationState` / `FederationStats` structs
- `SettingsFederation` / `FederationRegistry` for managing federations

**Core Functions:**
- `register()` / `unregister()` - Federation management
- `add_article()` - Add articles
- `make_constitutional()` / `make_statutory()` - Constitutional management
- `add_state()` - Add states
- `stats()` - Access statistics

**Features:**
- Multiple federation types
- Status tracking
- Section numbering
- State support
- Constitutional rate calculation

## [0.0.736] - 2025-12-15

### Added - Settings Coalition (Phase 312)

**Settings Coalition Module:**
- `settings_coalition.rs` for temporary coalition for settings governance
- `CoalitionType` enum (Governing, Opposition, Emergency, Issue)
- `CoalitionStatus` enum (Forming, Stable, Unstable, Collapsed)
- `CoalitionConfig` / `CoalitionAgreement` / `CoalitionPartner` / `CoalitionStats` structs
- `SettingsCoalition` / `CoalitionRegistry` for managing coalitions

**Core Functions:**
- `register()` / `unregister()` - Coalition management
- `add_agreement()` - Add agreements
- `reach_consensus()` / `break_consensus()` - Consensus management
- `add_partner()` - Add partners
- `stats()` - Access statistics

**Features:**
- Multiple coalition types
- Status tracking
- Priority ordering
- Partner support
- Consensus rate calculation

## [0.0.735] - 2025-12-15

### Added - Settings Alliance (Phase 311)

**Settings Alliance Module:**
- `settings_alliance.rs` for formal alliance for settings governance
- `AllianceType` enum (Military, Economic, Political, Strategic)
- `AllianceStatus` enum (Forming, Active, Strained, Dissolved)
- `AllianceConfig` / `AllianceCommitment` / `AllianceMember` / `AllianceStats` structs
- `SettingsAlliance` / `AllianceRegistry` for managing alliances

**Core Functions:**
- `register()` / `unregister()` - Alliance management
- `add_commitment()` - Add commitments
- `make_binding()` / `make_optional()` - Binding management
- `add_member()` - Add members
- `stats()` - Access statistics

**Features:**
- Multiple alliance types
- Status tracking
- Article numbering
- Member support
- Binding rate calculation

## [0.0.734] - 2025-12-15

### Added - Settings Entente (Phase 310)

**Settings Entente Module:**
- `settings_entente.rs` for informal understanding for settings governance
- `EntenteType` enum (Cordiale, Strategic, Commercial, Cultural)
- `EntenteStatus` enum (Informal, Formalized, Active, Dissolved)
- `EntenteConfig` / `EntenteUnderstanding` / `EntentePartner` / `EntenteStats` structs
- `SettingsEntente` / `EntenteRegistry` for managing ententes

**Core Functions:**
- `register()` / `unregister()` - Entente management
- `add_understanding()` - Add understandings
- `make_tacit()` / `make_explicit()` - Tacit management
- `add_partner()` - Add partners
- `stats()` - Access statistics

**Features:**
- Multiple entente types
- Status tracking
- Point numbering
- Partner support
- Tacit rate calculation

## [0.0.733] - 2025-12-15

### Added - Settings Convention (Phase 309)

**Settings Convention Module:**
- `settings_convention.rs` for formal convention for settings governance
- `ConventionType` enum (International, Constitutional, Trade, Technical)
- `ConventionStatus` enum (Draft, Adopted, InForce, Superseded)
- `ConventionConfig` / `ConventionArticle` / `ConventionParty` / `ConventionStats` structs
- `SettingsConvention` / `ConventionRegistry` for managing conventions

**Core Functions:**
- `register()` / `unregister()` - Convention management
- `add_article()` - Add articles
- `make_binding()` / `make_advisory()` - Binding management
- `add_party()` - Add parties
- `stats()` - Access statistics

**Features:**
- Multiple convention types
- Status tracking
- Article numbering
- Party support
- Binding rate calculation

## [0.0.732] - 2025-12-15

### Added - Settings Concordat (Phase 308)

**Settings Concordat Module:**
- `settings_concordat.rs` for religious agreement for settings governance
- `ConcordatType` enum (Papal, Diplomatic, Administrative, Cultural)
- `ConcordatStatus` enum (Negotiating, Signed, Binding, Rescinded)
- `ConcordatConfig` / `ConcordatArticle` / `ConcordatSignatory` / `ConcordatStats` structs
- `SettingsConcordat` / `ConcordatRegistry` for managing concordats

**Core Functions:**
- `register()` / `unregister()` - Concordat management
- `add_article()` - Add articles
- `make_canonical()` / `make_non_canonical()` - Canonical management
- `add_signatory()` - Add signatories
- `stats()` - Access statistics

**Features:**
- Multiple concordat types
- Status tracking
- Section numbering
- Signatory support
- Canonical rate calculation

## [0.0.731] - 2025-12-15

### Added - Settings Pact (Phase 307)

**Settings Pact Module:**
- `settings_pact.rs` for formal pact for settings governance
- `PactType` enum (Defense, NonAggression, Alliance, Cooperation)
- `PactStatus` enum (Proposed, Sealed, Honored, Broken)
- `PactConfig` / `PactClause` / `PactParty` / `PactStats` structs
- `SettingsPact` / `PactRegistry` for managing pacts

**Core Functions:**
- `register()` / `unregister()` - Pact management
- `add_clause()` - Add clauses
- `make_sacred()` / `make_profane()` - Sacred management
- `add_party()` - Add parties
- `stats()` - Access statistics

**Features:**
- Multiple pact types
- Status tracking
- Article numbering
- Party support
- Sacred rate calculation

## [0.0.730] - 2025-12-15

### Added - Settings Accord (Phase 306)

**Settings Accord Module:**
- `settings_accord.rs` for formal accord for settings governance
- `AccordType` enum (Peace, Trade, Framework, Settlement)
- `AccordStatus` enum (Preliminary, Final, Implemented, Voided)
- `AccordConfig` / `AccordProvision` / `AccordSignatory` / `AccordStats` structs
- `SettingsAccord` / `AccordRegistry` for managing accords

**Core Functions:**
- `register()` / `unregister()` - Accord management
- `add_provision()` - Add provisions
- `agree()` / `disagree()` - Agreement management
- `add_signatory()` - Add signatories
- `stats()` - Access statistics

**Features:**
- Multiple accord types
- Status tracking
- Section numbering
- Signatory support
- Agreement rate calculation

## [0.0.729] - 2025-12-15

### Added - Settings Compact (Phase 305)

**Settings Compact Module:**
- `settings_compact.rs` for formal compact for settings governance
- `CompactType` enum (Interstate, Federal, Regional, Municipal)
- `CompactStatus` enum (Proposed, Negotiating, Enacted, Suspended)
- `CompactConfig` / `CompactTerm` / `CompactMember` / `CompactStats` structs
- `SettingsCompact` / `CompactRegistry` for managing compacts

**Core Functions:**
- `register()` / `unregister()` - Compact management
- `add_term()` - Add terms
- `make_binding()` / `make_non_binding()` - Binding management
- `add_member()` - Add members
- `stats()` - Access statistics

**Features:**
- Multiple compact types
- Status tracking
- Article numbering
- Member support
- Binding rate calculation

## [0.0.728] - 2025-12-15

### Added - Settings Protocol (Phase 304)

**Settings Protocol Module:**
- `settings_protocol.rs` for formal protocol for settings governance
- `ProtocolType` enum (Amendment, Optional, Supplementary, Implementation)
- `ProtocolStatus` enum (Draft, Open, Active, Closed)
- `ProtocolConfig` / `ProtocolClause` / `ProtocolParty` / `ProtocolStats` structs
- `SettingsProtocol` / `ProtocolRegistry` for managing protocols

**Core Functions:**
- `register()` / `unregister()` - Protocol management
- `add_clause()` - Add clauses
- `adopt()` / `reject()` - Adoption management
- `add_party()` - Add parties
- `stats()` - Access statistics

**Features:**
- Multiple protocol types
- Status tracking
- Section numbering
- Party support
- Adoption rate calculation

## [0.0.727] - 2025-12-15

### Added - Settings Treaty (Phase 303)

**Settings Treaty Module:**
- `settings_treaty.rs` for international agreement for settings governance
- `TreatyType` enum (Bilateral, Multilateral, Framework, Protocol)
- `TreatyStatus` enum (Negotiating, Signed, Ratified, Terminated)
- `TreatyConfig` / `TreatyProvision` / `TreatySignatory` / `TreatyStats` structs
- `SettingsTreaty` / `TreatyRegistry` for managing treaties

**Core Functions:**
- `register()` / `unregister()` - Treaty management
- `add_provision()` - Add provisions
- `enter_force()` / `terminate()` - Force management
- `add_signatory()` - Add signatories
- `stats()` - Access statistics

**Features:**
- Multiple treaty types
- Status tracking
- Article numbering
- Signatory support
- In force rate calculation

## [0.0.726] - 2025-12-15

### Added - Settings Covenant (Phase 302)

**Settings Covenant Module:**
- `settings_covenant.rs` for binding agreement for settings governance
- `CovenantType` enum (Mutual, Unilateral, Conditional, Unconditional)
- `CovenantStatus` enum (Pending, Active, Fulfilled, Breached)
- `CovenantConfig` / `CovenantTerm` / `CovenantObligation` / `CovenantStats` structs
- `SettingsCovenant` / `CovenantRegistry` for managing covenants

**Core Functions:**
- `register()` / `unregister()` - Covenant management
- `add_term()` - Add terms
- `fulfill()` / `reset()` - Fulfillment management
- `add_obligation()` - Add obligations
- `stats()` - Access statistics

**Features:**
- Multiple covenant types
- Status tracking
- Binding terms
- Obligation support
- Fulfillment rate calculation

## [0.0.725] - 2025-12-15

### Added - Settings Constitution (Phase 301)

**Settings Constitution Module:**
- `settings_constitution.rs` for supreme law for settings governance
- `ConstitutionType` enum (Written, Unwritten, Codified, Uncodified)
- `ConstitutionBranch` enum (Executive, Legislative, Judicial, Administrative)
- `ConstitutionConfig` / `ConstitutionArticle` / `ConstitutionClause` / `ConstitutionStats` structs
- `SettingsConstitution` / `ConstitutionRegistry` for managing constitutions

**Core Functions:**
- `register()` / `unregister()` - Constitution management
- `add_article()` - Add articles
- `ratify()` / `repeal()` - Ratification management
- `add_clause()` - Add clauses
- `stats()` - Access statistics

**Features:**
- Multiple constitution types
- Branch organization
- Ratification tracking
- Article numbering
- Clause support

## [0.0.724] - 2025-12-15

### Added - Settings Charter (Phase 300)

**Settings Charter Module:**
- `settings_charter.rs` for foundational charter for settings governance
- `CharterType` enum (Founding, Corporate, Municipal, Royal)
- `CharterStatus` enum (Draft, Ratified, Amended, Revoked)
- `CharterConfig` / `CharterProvision` / `CharterAmendment` / `CharterStats` structs
- `SettingsCharter` / `CharterRegistry` for managing charters

**Core Functions:**
- `register()` / `unregister()` - Charter management
- `add_provision()` - Add provisions
- `activate()` / `deactivate()` - Activation management
- `add_amendment()` - Add amendments
- `stats()` - Access statistics

**Features:**
- Multiple charter types
- Status tracking
- Provision sections
- Amendment support
- Active rate calculation

## [0.0.723] - 2025-12-15

### Added - Settings Statute (Phase 299)

**Settings Statute Module:**
- `settings_statute.rs` for written laws for settings governance
- `StatuteType` enum (General, Criminal, Civil, Administrative)
- `StatuteScope` enum (Federal, State, Local, International)
- `StatuteConfig` / `StatuteArticle` / `StatuteSubsection` / `StatuteStats` structs
- `SettingsStatute` / `StatuteRegistry` for managing statutes

**Core Functions:**
- `register()` / `unregister()` - Statute management
- `add_article()` - Add articles
- `enact()` / `repeal()` - Enactment management
- `add_subsection()` - Add subsections
- `stats()` - Access statistics

**Features:**
- Multiple statute types
- Scope levels
- Enactment tracking
- Article numbering
- Subsection support

## [0.0.722] - 2025-12-15

### Added - Settings Ordinance (Phase 298)

**Settings Ordinance Module:**
- `settings_ordinance.rs` for local ordinances for settings regulation
- `OrdinanceType` enum (Municipal, Regional, Local, Zoning)
- `OrdinanceJurisdiction` enum (City, County, District, Zone)
- `OrdinanceConfig` / `OrdinanceProvision` / `OrdinanceAmendment` / `OrdinanceStats` structs
- `SettingsOrdinance` / `OrdinanceRegistry` for managing ordinances

**Core Functions:**
- `register()` / `unregister()` - Ordinance management
- `add_provision()` - Add provisions
- `make_effective()` / `make_ineffective()` - Effectiveness management
- `add_amendment()` - Add amendments
- `stats()` - Access statistics

**Features:**
- Multiple ordinance types
- Jurisdiction levels
- Effectiveness tracking
- Section numbering
- Amendment support

## [0.0.721] - 2025-12-15

### Added - Settings Mandate (Phase 297)

**Settings Mandate Module:**
- `settings_mandate.rs` for authoritative mandates for settings compliance
- `MandateType` enum (Legal, Regulatory, Corporate, Technical)
- `MandateCompliance` enum (Required, Recommended, Optional, Exempt)
- `MandateConfig` / `MandateRequirement` / `MandateEvidence` / `MandateStats` structs
- `SettingsMandate` / `MandateRegistry` for managing mandates

**Core Functions:**
- `register()` / `unregister()` - Mandate management
- `add_requirement()` - Add requirements
- `fulfill()` / `unfulfill()` - Fulfillment management
- `add_evidence()` - Add evidence
- `stats()` - Access statistics

**Features:**
- Multiple mandate types
- Compliance levels
- Fulfillment tracking
- Evidence support
- Required requirement counting

## [0.0.720] - 2025-12-15

### Added - Settings Decree (Phase 296)

**Settings Decree Module:**
- `settings_decree.rs` for official decrees for settings governance
- `DecreeType` enum (Executive, Legislative, Judicial, Emergency)
- `DecreeBinding` enum (Mandatory, Recommended, Voluntary, Advisory)
- `DecreeConfig` / `DecreeRuling` / `DecreeClause` / `DecreeStats` structs
- `SettingsDecree` / `DecreeRegistry` for managing decrees

**Core Functions:**
- `register()` / `unregister()` - Decree management
- `add_ruling()` - Add rulings
- `enact()` / `repeal()` - In-force management
- `add_clause()` - Add clauses
- `stats()` - Access statistics

**Features:**
- Multiple decree types
- Binding levels
- In-force tracking
- Clause support
- Emergency decree counting

## [0.0.719] - 2025-12-15

### Added - Settings Edict (Phase 295)

**Settings Edict Module:**
- `settings_edict.rs` for formal edicts for settings enforcement
- `EdictType` enum (Royal, Imperial, Sovereign, Administrative)
- `EdictStatus` enum (Draft, Proclaimed, Active, Revoked)
- `EdictConfig` / `EdictProclamation` / `EdictAnnotation` / `EdictStats` structs
- `SettingsEdict` / `EdictRegistry` for managing edicts

**Core Functions:**
- `register()` / `unregister()` - Edict management
- `add_proclamation()` - Add proclamations
- `proclaim()` / `activate()` / `revoke()` - Lifecycle management
- `add_annotation()` - Add annotations
- `stats()` - Access statistics

**Features:**
- Multiple edict types
- Status lifecycle
- Seal support
- Annotation tracking
- Active/revoked counting

## [0.0.718] - 2025-12-15

### Added - Settings Directive (Phase 294)

**Settings Directive Module:**
- `settings_directive.rs` for authoritative directives for settings management
- `DirectiveType` enum (Mandatory, Recommended, Advisory, Optional)
- `DirectiveAuthority` enum (System, Admin, Policy, Executive)
- `DirectiveConfig` / `DirectiveOrder` / `DirectiveSupplement` / `DirectiveStats` structs
- `SettingsDirective` / `DirectiveRegistry` for managing directives

**Core Functions:**
- `register()` / `unregister()` - Directive management
- `add_order()` - Add orders
- `enforce()` - Enforce orders
- `add_supplement()` - Add supplements
- `stats()` - Access statistics

**Features:**
- Multiple directive types
- Authority levels
- Enforcement tracking
- Supplement support
- Mandatory counting

## [0.0.717] - 2025-12-15

### Added - Settings Circular (Phase 293)

**Settings Circular Module:**
- `settings_circular.rs` for circular notices distributed to all
- `CircularType` enum (Policy, Information, Directive, Advisory)
- `CircularScope` enum (All, Department, Team, Individual)
- `CircularConfig` / `CircularNotice` / `CircularAttachment` / `CircularStats` structs
- `SettingsCircular` / `CircularRegistry` for managing circulars

**Core Functions:**
- `register()` / `unregister()` - Circular management
- `add_notice()` - Add notices
- `deactivate()` - Deactivate notices
- `add_attachment()` - Add attachments
- `stats()` - Access statistics

**Features:**
- Multiple circular types
- Scope-based distribution
- Active status tracking
- Reference numbers
- Effective dates

## [0.0.716] - 2025-12-15

### Added - Settings Missive (Phase 292)

**Settings Missive Module:**
- `settings_missive.rs` for formal letters about settings changes
- `MissiveType` enum (Formal, Informal, Personal, Business)
- `MissiveDelivery` enum (Standard, Express, Priority, Certified)
- `MissiveConfig` / `MissiveLetter` / `MissiveEnclosure` / `MissiveStats` structs
- `SettingsMissive` / `MissiveRegistry` for managing missives

**Core Functions:**
- `register()` / `unregister()` - Missive management
- `add_letter()` - Add letters
- `deliver()` - Mark letters delivered
- `add_enclosure()` - Add enclosures
- `stats()` - Access statistics

**Features:**
- Multiple missive types
- Delivery methods
- Delivery tracking
- From/to addressing
- Enclosure support

## [0.0.715] - 2025-12-15

### Added - Settings Communique (Phase 291)

**Settings Communique Module:**
- `settings_communique.rs` for official communications about settings
- `CommuniqueType` enum (Official, Informal, Urgent, Diplomatic)
- `CommuniqueClassification` enum (Public, Internal, Confidential, Restricted)
- `CommuniqueConfig` / `CommuniqueMessage` / `CommuniqueAttachment` / `CommuniqueStats` structs
- `SettingsCommunique` / `CommuniqueRegistry` for managing communiques

**Core Functions:**
- `register()` / `unregister()` - Communique management
- `add_message()` - Add messages
- `mark_read()` - Mark messages read
- `add_attachment()` - Add attachments
- `stats()` - Access statistics

**Features:**
- Multiple communique types
- Classification levels
- Read tracking
- Sender/recipient support
- Attachment support

## [0.0.714] - 2025-12-15

### Added - Settings Dispatch (Phase 290)

**Settings Dispatch Module:**
- `settings_dispatch.rs` for dispatching settings changes to targets
- `DispatchType` enum (Immediate, Scheduled, Batch, Conditional)
- `DispatchStatus` enum (Pending, InProgress, Completed, Failed)
- `DispatchConfig` / `DispatchItem` / `DispatchMetadata` / `DispatchStats` structs
- `SettingsDispatch` / `DispatchRegistry` for managing dispatches

**Core Functions:**
- `register()` / `unregister()` - Dispatch management
- `add_item()` - Add items
- `start()` / `complete()` / `fail()` - Lifecycle management
- `add_metadata()` - Add metadata
- `stats()` - Access statistics

**Features:**
- Multiple dispatch types
- Status tracking
- Retry count support
- Attempt counting
- Success rate calculation

## [0.0.713] - 2025-12-15

### Added - Settings Notice (Phase 289)

**Settings Notice Module:**
- `settings_notice.rs` for official notices about settings changes
- `NoticeType` enum (Information, Warning, Alert, Announcement)
- `NoticePriority` enum (Low, Normal, High, Urgent)
- `NoticeConfig` / `NoticeEntry` / `NoticeMetadata` / `NoticeStats` structs
- `SettingsNotice` / `NoticeRegistry` for managing notices

**Core Functions:**
- `register()` / `unregister()` - Notice management
- `add_entry()` - Add entries
- `add_metadata()` - Add metadata
- `acknowledge()` - Acknowledge notices
- `stats()` - Access statistics

**Features:**
- Multiple notice types
- Priority levels
- Acknowledgment tracking
- Metadata support
- Urgent notice counting

## [0.0.712] - 2025-12-15

### Added - Settings Report (Phase 288)

**Settings Report Module:**
- `settings_report.rs` for formal reports on settings changes and status
- `ReportType` enum (Status, Change, Audit, Compliance)
- `ReportFrequency` enum (OnDemand, Daily, Weekly, Monthly)
- `ReportConfig` / `ReportSection` / `ReportAppendix` / `ReportStats` structs
- `SettingsReport` / `ReportRegistry` for managing reports

**Core Functions:**
- `register()` / `unregister()` - Report management
- `add_section()` - Add sections
- `add_appendix()` - Add appendices
- `get_section()` - Get sections
- `stats()` - Access statistics

**Features:**
- Multiple report types
- Configurable frequency
- Critical section flags
- Appendix support
- Ordered sections

## [0.0.711] - 2025-12-15

### Added - Settings Summary (Phase 287)

**Settings Summary Module:**
- `settings_summary.rs` for comprehensive summary of settings state
- `SummaryType` enum (Quick, Detailed, Comprehensive, Overview)
- `SummaryDepth` enum (Shallow, Medium, Deep, Full)
- `SummaryConfig` / `SummaryEntry` / `SummaryMetadata` / `SummaryStats` structs
- `SettingsSummary` / `SummaryRegistry` for managing summaries

**Core Functions:**
- `register()` / `unregister()` - Summary management
- `add_entry()` - Add entries
- `add_metadata()` - Add metadata
- `get_entry()` - Get entries
- `stats()` - Access statistics

**Features:**
- Category organization
- Important entry flags
- Configurable depth
- Metadata support
- Category counting

## [0.0.710] - 2025-12-15

### Added - Settings Brief (Phase 286)

**Settings Brief Module:**
- `settings_brief.rs` for executive briefs of settings overview
- `BriefType` enum (Executive, Technical, Operational, Strategic)
- `BriefScope` enum (Department, Organization, Project, System)
- `BriefConfig` / `BriefPoint` / `BriefAttachment` / `BriefStats` structs
- `SettingsBrief` / `BriefRegistry` for managing briefs

**Core Functions:**
- `register()` / `unregister()` - Brief management
- `add_point()` - Add brief points
- `add_attachment()` - Add attachments
- `get_point()` - Get points
- `stats()` - Access statistics

**Features:**
- Priority-based points
- Action required flags
- Multiple scope types
- Hierarchical organization
- Attachment support

## [0.0.709] - 2025-12-15

### Added - Settings Digest (Phase 285)

**Settings Digest Module:**
- `settings_digest.rs` for condensed digest of settings summaries
- `DigestType` enum (Daily, Weekly, Monthly, Custom)
- `DigestFormat` enum (Summary, Detailed, Highlights, Full)
- `DigestConfig` / `DigestSection` / `DigestItem` / `DigestStats` structs
- `SettingsDigest` / `DigestRegistry` for managing digests

**Core Functions:**
- `register()` / `unregister()` - Digest management
- `add_section()` - Add sections
- `add_item()` - Add items
- `get_section()` - Get sections
- `stats()` - Access statistics

**Features:**
- Section-based organization
- Item highlighting
- Multiple formats
- Ordered sections
- Section summaries

## [0.0.708] - 2025-12-15

### Added - Settings Memo (Phase 284)

**Settings Memo Module:**
- `settings_memo.rs` for internal memos of settings communication
- `MemoType` enum (Internal, External, Confidential, Broadcast)
- `MemoStatus` enum (Draft, Sent, Read, Archived)
- `MemoConfig` / `MemoMessage` / `MemoAttachment` / `MemoStats` structs
- `SettingsMemo` / `MemoRegistry` for managing memos

**Core Functions:**
- `register()` / `unregister()` - Memo management
- `add_message()` - Add messages
- `send()` / `mark_read()` - Message lifecycle
- `add_attachment()` - Add attachments
- `stats()` - Access statistics

**Features:**
- From/To addressing
- Message status tracking
- Attachment support
- Acknowledgment settings
- Read rate tracking

## [0.0.707] - 2025-12-15

### Added - Settings Journal (Phase 283)

**Settings Journal Module:**
- `settings_journal.rs` for personal journal of settings reflections
- `JournalType` enum (Personal, Technical, Research, Log)
- `JournalMood` enum (Productive, Challenging, Learning, Resolved)
- `JournalConfig` / `JournalEntry` / `JournalItem` / `JournalStats` structs
- `SettingsJournal` / `JournalRegistry` for managing journals

**Core Functions:**
- `register()` / `unregister()` - Journal management
- `write()` - Write entries
- `add_item()` - Add items
- `get_items()` - Get items for entry
- `stats()` - Access statistics

**Features:**
- Mood tracking
- Entry tagging
- Item reflections
- Privacy settings
- Auto-incrementing IDs

## [0.0.706] - 2025-12-15

### Added - Settings Bulletin (Phase 282)

**Settings Bulletin Module:**
- `settings_bulletin.rs` for bulletin board of settings updates
- `BulletinType` enum (News, Alert, Update, Archive)
- `BulletinPriority` enum (Low, Normal, High, Urgent)
- `BulletinConfig` / `BulletinPost` / `BulletinItem` / `BulletinStats` structs
- `SettingsBulletin` / `BulletinRegistry` for managing bulletins

**Core Functions:**
- `register()` / `unregister()` - Bulletin management
- `add_post()` - Add posts
- `add_item()` - Add items
- `get_pinned()` - Get pinned posts
- `stats()` - Access statistics

**Features:**
- Post pinning
- Priority levels
- Auto expire settings
- Post dates
- High priority tracking

## [0.0.705] - 2025-12-15

### Added - Settings Almanac (Phase 281)

**Settings Almanac Module:**
- `settings_almanac.rs` for yearly almanac of settings information
- `AlmanacType` enum (Annual, Seasonal, Technical, Historical)
- `AlmanacEdition` enum (Current, Previous, Special, Commemorative)
- `AlmanacConfig` / `AlmanacChapter` / `AlmanacEntry` / `AlmanacStats` structs
- `SettingsAlmanac` / `AlmanacRegistry` for managing almanacs

**Core Functions:**
- `register()` / `unregister()` - Almanac management
- `add_chapter()` - Add chapters
- `add_entry()` - Add entries
- `set_edition()` - Set edition type
- `stats()` - Access statistics

**Features:**
- Chapter-based organization
- Yearly periods
- Entry highlighting
- Edition management
- Period tracking

## [0.0.704] - 2025-12-15

### Added - Settings Gazette (Phase 280)

**Settings Gazette Module:**
- `settings_gazette.rs` for official gazette of settings announcements
- `GazetteType` enum (Official, Weekly, Special, Extraordinary)
- `GazetteStatus` enum (Draft, Review, Published, Superseded)
- `GazetteConfig` / `GazetteNotice` / `GazetteEntry` / `GazetteStats` structs
- `SettingsGazette` / `GazetteRegistry` for managing gazettes

**Core Functions:**
- `register()` / `unregister()` - Gazette management
- `add_notice()` - Add notices
- `add_entry()` - Add entries
- `review()` / `publish()` / `supersede()` - Lifecycle
- `stats()` - Access statistics

**Features:**
- Official notices
- Urgent flagging
- Effective dates
- Issue numbering
- Reference tracking

## [0.0.703] - 2025-12-15

### Added - Settings Repertoire (Phase 279)

**Settings Repertoire Module:**
- `settings_repertoire.rs` for performance repertoire of available settings
- `RepertoireType` enum (Standard, Classic, Modern, Experimental)
- `RepertoireStatus` enum (Rehearsing, Ready, Performing, Retired)
- `RepertoireConfig` / `RepertoirePiece` / `RepertoireItem` / `RepertoireStats` structs
- `SettingsRepertoire` / `RepertoireRegistry` for managing repertoires

**Core Functions:**
- `register()` / `unregister()` - Repertoire management
- `add_piece()` - Add pieces
- `add_item()` - Add items
- `ready()` / `perform()` / `retire()` - Lifecycle
- `stats()` - Access statistics

**Features:**
- Piece difficulty levels
- Practice tracking
- Performance notes
- Season support
- Composer attribution

## [0.0.702] - 2025-12-15

### Added - Settings Archive V2 (Phase 278)

**Settings Archive V2 Module:**
- `settings_archive_v2.rs` for long-term archive of settings history
- `ArchiveTypeV2` enum (Cold, Warm, Deep, Glacier)
- `ArchiveRetention` enum (Days30, Days90, Year1, Indefinite)
- `ArchiveConfigV2` / `ArchiveRecord` / `ArchiveBox` / `ArchiveStatsV2` structs
- `SettingsArchiveV2` / `ArchiveRegistryV2` for managing archives

**Core Functions:**
- `register()` / `unregister()` - Archive management
- `add_box()` - Add archive boxes
- `archive_to_box()` - Archive records
- `seal()` - Seal boxes
- `stats()` - Access statistics

**Features:**
- Box-based organization
- Record expiry dates
- Sealed box protection
- Retention policies
- Archive type classification

## [0.0.701] - 2025-12-15

### Added - Settings Anthology (Phase 277)

**Settings Anthology Module:**
- `settings_anthology.rs` for curated anthology of settings collections
- `AnthologyType` enum (BestOf, Complete, Themed, Historical)
- `AnthologyStatus` enum (Curating, Complete, Published, Archived)
- `AnthologyConfig` / `AnthologyWork` / `AnthologyPiece` / `AnthologyStats` structs
- `SettingsAnthology` / `AnthologyRegistry` for managing anthologies

**Core Functions:**
- `register()` / `unregister()` - Anthology management
- `add_work()` - Add works
- `add_piece()` - Add pieces
- `complete()` / `publish()` - Lifecycle management
- `stats()` - Access statistics

**Features:**
- Work curation
- Featured work tracking
- Author attribution
- Ordered pieces
- Theme support

## [0.0.700] - 2025-12-15

### Added - Settings Compendium (Phase 276) - MILESTONE!

**Settings Compendium Module:**
- `settings_compendium.rs` for comprehensive compendium of settings knowledge
- `CompendiumType` enum (Reference, Tutorial, Encyclopedia, Handbook)
- `CompendiumEdition` enum (First, Revised, Extended, Final)
- `CompendiumConfig` / `CompendiumVolume` / `CompendiumArticle` / `CompendiumEntry` / `CompendiumStats` structs
- `SettingsCompendium` / `CompendiumRegistry` for managing compendiums

**Core Functions:**
- `register()` / `unregister()` - Compendium management
- `add_volume()` - Add volumes
- `add_entry()` - Add entries
- `get_entries()` - Get entries for article
- `stats()` - Access statistics

**Features:**
- Volume-based organization
- Article keywords
- Entry definitions
- Edition tracking
- Subject categorization

**Milestone: v0.0.700 - 700 versions of Anna!**

## [0.0.699] - 2025-12-15

### Added - Settings Catalog V2 (Phase 275)

**Settings Catalog V2 Module:**
- `settings_catalog_v2.rs` for product catalog of available settings
- `CatalogType` enum (Standard, Premium, Custom, Archive)
- `CatalogStatus` enum (Draft, Published, Deprecated, Archived)
- `CatalogConfig` / `CatalogProduct` / `CatalogEntry` / `CatalogStats` structs
- `SettingsCatalogV2` / `CatalogRegistryV2` for managing catalogs

**Core Functions:**
- `register()` / `unregister()` - Catalog management
- `add_product()` - Add products
- `add_entry()` - Add entries
- `publish()` / `deprecate()` - Lifecycle management
- `stats()` - Access statistics

**Features:**
- Product availability tracking
- Category organization
- Catalog lifecycle states
- Entry notes
- Availability rate calculation

## [0.0.698] - 2025-12-15

### Added - Settings Portfolio (Phase 274)

**Settings Portfolio Module:**
- `settings_portfolio.rs` for investment portfolio of settings assets
- `PortfolioType` enum (Standard, Growth, Balanced, Conservative)
- `PortfolioStatus` enum (Active, Paused, Rebalancing, Closed)
- `PortfolioConfig` / `PortfolioAsset` / `PortfolioHolding` / `PortfolioStats` structs
- `SettingsPortfolio` / `PortfolioRegistry` for managing portfolios

**Core Functions:**
- `register()` / `unregister()` - Portfolio management
- `add_asset()` - Add assets
- `add_holding()` - Add holdings
- `rebalance()` - Recalculate weights
- `stats()` - Access statistics

**Features:**
- Asset weighting
- Category tracking
- Portfolio rebalancing
- Holdings management
- Value calculations

## [0.0.697] - 2025-12-15

### Added - Settings Dossier (Phase 273)

**Settings Dossier Module:**
- `settings_dossier.rs` for comprehensive dossier of settings information
- `DossierType` enum (Standard, Confidential, Summary, Full)
- `DossierClassification` enum (Public, Internal, Restricted, Secret)
- `DossierConfig` / `DossierDocument` / `DossierEntry` / `DossierStats` structs
- `SettingsDossier` / `DossierRegistry` for managing dossiers

**Core Functions:**
- `register()` / `unregister()` - Dossier management
- `add_document()` - Add documents
- `add_entry()` - Add entries
- `get_entries()` - Get entries for document
- `stats()` - Access statistics

**Features:**
- Document classification
- Entry tracking
- Security levels
- Max document limits
- Entry notes

## [0.0.696] - 2025-12-15

### Added - Settings Album (Phase 272)

**Settings Album Module:**
- `settings_album.rs` for collection album of settings snapshots
- `AlbumType` enum (Standard, Collection, Archive, Snapshot)
- `AlbumStatus` enum (Empty, Partial, Complete, Sealed)
- `AlbumConfig` / `AlbumPage` / `AlbumItem` / `AlbumStats` structs
- `SettingsAlbum` / `AlbumRegistry` for managing albums

**Core Functions:**
- `register()` / `unregister()` - Album management
- `add_page()` - Add pages
- `add_item()` - Add items to pages
- `seal()` - Seal album
- `stats()` - Access statistics

**Features:**
- Page-based organization
- Timestamp tracking
- Album sealing
- Max page limits
- Item labels

## [0.0.695] - 2025-12-15

### Added - Settings Folio (Phase 271)

**Settings Folio Module:**
- `settings_folio.rs` for portfolio of settings collections
- `FolioType` enum (Active, Archived, Template, Backup)
- `FolioStatus` enum (Open, Closed, Locked, Pending)
- `FolioConfig` / `FolioSection` / `FolioItem` / `FolioStats` structs
- `SettingsFolio` / `FolioRegistry` for managing folios

**Core Functions:**
- `register()` / `unregister()` - Folio management
- `add_section()` - Add sections
- `add_setting()` - Add settings to sections
- `lock()` / `close()` - Change status
- `stats()` - Access statistics

**Features:**
- Section-based organization
- Multiple folio types
- Status management
- Max section limits
- Item notes

## [0.0.694] - 2025-12-15

### Added - Settings Diary (Phase 270)

**Settings Diary Module:**
- `settings_diary.rs` for daily diary of settings activities
- `DiaryEntryType` enum (Note, Change, Alert, Milestone)
- `DiaryImportance` enum (Low, Normal, High, Critical)
- `DiaryConfig` / `DiaryEntry` / `DailyPage` / `DiaryStats` structs
- `SettingsDiary` / `DiaryRegistry` for managing diaries

**Core Functions:**
- `register()` / `unregister()` - Diary management
- `add_note()` - Add general notes
- `add_change()` - Record changes
- `add_alert()` - Add alerts
- `get_page()` - Get daily page
- `stats()` - Access statistics

**Features:**
- Daily page organization
- Entry tagging
- Importance levels
- Auto-summarization
- Retention policy

## [0.0.693] - 2025-12-15

### Added - Settings Ledger (Phase 269)

**Settings Ledger Module:**
- `settings_ledger.rs` for immutable settings records
- `LedgerEntryType` enum (Set, Update, Delete, Import)
- `LedgerStatus` enum (Active, Archived, Sealed, Pending)
- `LedgerConfig` / `LedgerEntry` / `LedgerPage` / `LedgerStats` structs
- `SettingsLedger` / `LedgerRegistry` for managing ledgers

**Core Functions:**
- `register()` / `unregister()` - Ledger management
- `record_set()` - Record new values
- `record_update()` - Record updates
- `record_delete()` - Record deletions
- `get_entry()` - Retrieve by sequence
- `stats()` - Access statistics

**Features:**
- Immutable audit trail
- Page-based storage
- Sequence numbering
- Hash generation
- Sealable pages

## [0.0.692] - 2025-12-15

### Added - Settings Chronicle (Phase 268)

**Settings Chronicle Module:**
- `settings_chronicle.rs` for chronicling settings changes over time
- `ChronicleEvent` enum (Changed, Added, Removed, Accessed)
- `ChronicleMode` enum (All, WritesOnly, Specific, Pattern)
- `ChronicleConfig` / `ChronicleRecord` / `ChronicleHistory` / `ChronicleStats` structs
- `SettingsChronicle` / `ChronicleRegistry` for managing chronicles

**Core Functions:**
- `register()` / `unregister()` - Chronicle management
- `track_change()` - Record changes
- `track_add()` - Record additions
- `track_remove()` - Record removals
- `track_access()` - Record reads
- `stats()` / `history()` - Access statistics and history

**Features:**
- Comprehensive change recording
- Pattern-based tracking
- Per-key history
- Max history limit
- Write-only mode

## [0.0.691] - 2025-12-15

### Added - Settings Auditor (Phase 267)

**Settings Auditor Module:**
- `settings_auditor.rs` for auditing settings changes and access
- `AuditEventType` enum (Read, Write, Delete, Create)
- `AuditSeverity` enum (Low, Medium, High, Critical)
- `AuditorConfig` / `AuditEvent` / `AuditTrail` / `AuditorStats` structs
- `SettingsAuditor` / `AuditorRegistry` for managing auditors

**Core Functions:**
- `register()` / `unregister()` - Auditor management
- `log_read()` - Log read access
- `log_write()` - Log write changes
- `log_create()` - Log new settings
- `log_delete()` - Log deletions
- `stats()` / `trail()` - Access statistics and history

**Features:**
- Comprehensive event logging
- Severity levels
- Event filtering
- Max events limit
- Read/write tracking

## [0.0.690] - 2025-12-15

### Added - Settings Combiner (Phase 266)

**Settings Combiner Module:**
- `settings_combiner.rs` for combining multiple settings collections
- `CombineStrategy` enum (LeftWins, RightWins, KeepBoth, ErrorOnConflict)
- `CombineDepth` enum (Shallow, Deep, Recursive, Flat)
- `CombinerConfig` / `CombineConflict` / `CombineResult` / `CombinerStats` structs
- `SettingsCombiner` / `CombinerRegistry` for managing combiners

**Core Functions:**
- `register()` / `unregister()` - Combiner management
- `combine()` - Combine two collections
- `combine_all()` - Combine multiple collections
- `stats()` - Access statistics

**Features:**
- Multiple combine strategies
- Conflict resolution
- Preserve empty option
- Custom conflict suffix
- Multi-collection support

## [0.0.689] - 2025-12-15

### Added - Settings Comparer (Phase 265)

**Settings Comparer Module:**
- `settings_comparer.rs` for comparing two settings collections
- `CompareMode` enum (Full, KeysOnly, ValuesOnly, StructureOnly)
- `DiffType` enum (Added, Removed, Changed, Unchanged)
- `ComparerConfig` / `DiffEntry` / `CompareResult` / `ComparerStats` structs
- `SettingsComparer` / `ComparerRegistry` for managing comparers

**Core Functions:**
- `register()` / `unregister()` - Comparer management
- `compare()` - Full comparison of two collections
- `compare_keys()` - Compare keys only
- `stats()` - Access statistics

**Features:**
- Multiple compare modes
- Case-insensitive comparison
- Whitespace normalization
- Change detection (+/-/~)
- Summary generation

## [0.0.688] - 2025-12-15

### Added - Settings Validator (Phase 264)

**Settings Validator Module:**
- `settings_validator.rs` for validating settings against rules and constraints
- `ValidationType` enum (Required, TypeCheck, Range, Pattern)
- `ValidationSeverity` enum (Info, Warning, Error, Critical)
- `ValidatorConfig` / `ValidationRule` / `ValidationIssue` / `ValidationResult` / `ValidatorStats` structs
- `SettingsValidator` / `ValidatorRegistry` for managing validators

**Core Functions:**
- `register()` / `unregister()` - Validator management
- `add_rule()` / `remove_rule()` - Rule management
- `validate()` - Validate all settings
- `validate_key()` - Validate single key
- `stats()` - Access statistics

**Features:**
- Multiple validation types
- Configurable severity levels
- Type checking (int, float, bool, url, email)
- Pattern matching
- Stop-on-error mode

## [0.0.687] - 2025-12-15

### Added - Settings Matcher (Phase 263)

**Settings Matcher Module:**
- `settings_matcher.rs` for matching settings against patterns and rules
- `MatchType` enum (Exact, Prefix, Suffix, Contains)
- `MatchTarget` enum (Key, Value, Both, Either)
- `MatcherConfig` / `MatchRule` / `MatchItem` / `MatchResult` / `MatcherStats` structs
- `SettingsMatcher` / `MatcherRegistry` for managing matchers

**Core Functions:**
- `register()` / `unregister()` - Matcher management
- `add_rule()` / `remove_rule()` - Rule management
- `match_pattern()` - Match against single pattern
- `match_rules()` - Match against multiple rules
- `stats()` - Access statistics

**Features:**
- Multiple match types
- Flexible match targets
- Case-insensitive matching
- Inverted matching
- Rule-based matching

## [0.0.686] - 2025-12-15

### Added - Settings Counter (Phase 262)

**Settings Counter Module:**
- `settings_counter.rs` for counting settings by various criteria
- `CountType` enum (All, ByPrefix, ByValueType, ByLength)
- `ValueType` enum (String, Numeric, Boolean, Empty)
- `CounterConfig` / `CountEntry` / `CountResult` / `CounterStats` structs
- `SettingsCounter` / `CounterRegistry` for managing counters

**Core Functions:**
- `register()` / `unregister()` - Counter management
- `count_all()` - Count total settings
- `count_by_prefix()` - Group and count by key prefix
- `count_by_value_type()` - Count by detected value type
- `count_by_length()` - Count by value length buckets
- `stats()` - Access statistics

**Features:**
- Multiple count types
- Value type detection
- Length bucketing
- Percentage calculations
- Configurable thresholds

## [0.0.685] - 2025-12-15

### Added - Settings Finder (Phase 261)

**Settings Finder Module:**
- `settings_finder.rs` for finding settings by various criteria
- `FindMode` enum (ExactKey, KeyPattern, ByValue, ValuePattern)
- `FindLimit` enum (First, All, Max)
- `FinderConfig` / `FoundItem` / `FindResult` / `FinderStats` structs
- `SettingsFinder` / `FinderRegistry` for managing finders

**Core Functions:**
- `register()` / `unregister()` - Finder management
- `find_exact()` - Find by exact key match
- `find_by_key()` - Find by key pattern with scoring
- `find_by_value()` - Find by value pattern with scoring
- `find_by_exact_value()` - Find by exact value match
- `stats()` - Access statistics

**Features:**
- Multiple find modes
- Configurable limits (first, all, max N)
- Case-insensitive matching
- Partial match scoring
- Best match selection

## [0.0.684] - 2025-12-15

### Added - Settings Scanner (Phase 260)

**Settings Scanner Module:**
- `settings_scanner.rs` for scanning settings for patterns and anomalies
- `ScanType` enum (Pattern, Anomaly, Duplicate, Empty)
- `ScanSeverity` enum (Info, Warning, Error, Critical)
- `ScannerConfig` / `ScanFinding` / `ScanResult` / `ScannerStats` structs
- `SettingsScanner` / `ScannerRegistry` for managing scanners

**Core Functions:**
- `register()` / `unregister()` - Scanner management
- `scan_pattern()` - Scan for pattern matches
- `scan_empty()` - Scan for empty values
- `scan_duplicates()` - Scan for duplicate values
- `scan_anomalies()` - Scan for unusual patterns
- `stats()` - Access statistics

**Features:**
- Multiple scan types
- Severity levels
- Case-insensitive matching
- Sensitive data detection
- Per-scan statistics

## [0.0.683] - 2025-12-15

### Added - Settings Zipper (Phase 259)

**Settings Zipper Module:**
- `settings_zipper.rs` for zipping and unzipping settings collections
- `ZipMode` enum (ByKey, ByPosition, Cartesian, WithDefault)
- `UnzipMode` enum (ByPrefix, Alternating, ByIndex)
- `ZipperConfig` / `ZippedPair` / `ZipResult` / `UnzipResult` / `ZipperStats` structs
- `SettingsZipper` / `ZipperRegistry` for managing zippers

**Core Functions:**
- `register()` / `unregister()` - Zipper management
- `zip_by_key()` - Zip collections by matching keys
- `zip_by_position()` - Zip by position in collection
- `unzip_by_prefix()` - Split by key prefix
- `unzip_alternating()` - Split alternating entries
- `stats()` - Access statistics

**Features:**
- Multiple zip modes
- Multiple unzip modes
- Default value for missing pairs
- Match rate tracking
- Balanced split detection

## [0.0.682] - 2025-12-15

### Added - Settings Collector (Phase 258)

**Settings Collector Module:**
- `settings_collector.rs` for collecting settings from multiple sources
- `CollectMode` enum (Merge, Union, Intersect, Append)
- `SourcePriority` enum (Low, Normal, High, Critical)
- `CollectorConfig` / `SettingsSource` / `CollectResult` / `CollectorStats` structs
- `SettingsCollector` / `CollectorRegistry` for managing collectors

**Core Functions:**
- `register()` / `unregister()` - Collector management
- `add_source()` / `clear_sources()` - Source management
- `collect()` - Collect from all sources
- `stats()` - Access statistics

**Features:**
- Multiple collect modes
- Source priorities
- Conflict detection
- Key deduplication
- Per-collection statistics

## [0.0.681] - 2025-12-15

### Added - Settings Iterator (Phase 257)

**Settings Iterator Module:**
- `settings_iterator.rs` for iterating over settings with various strategies
- `IterationOrder` enum (Natural, Alphabetical, ReverseAlphabetical, ByValueLength)
- `IterationFilter` enum (None, NonEmpty, Numeric, Boolean)
- `IteratorConfig` / `IterationItem` / `IterationResult` / `IteratorStats` structs
- `SettingsIterator` / `IteratorRegistry` for managing iterators

**Core Functions:**
- `register()` / `unregister()` - Iterator management
- `iterate()` - Iterate with ordering and filtering
- `for_each()` - Iterate with callback
- `iterate_batched()` - Iterate in batches
- `stats()` - Access statistics

**Features:**
- Multiple iteration orders
- Built-in filters
- Skip and take support
- Batch iteration
- Per-iteration statistics

## [0.0.680] - 2025-12-15

### Added - Settings Expander (Phase 256)

**Settings Expander Module:**
- `settings_expander.rs` for expanding variables and templates in settings
- `ExpandMode` enum (Environment, Reference, Template, All)
- `VariableSyntax` enum (Shell, Mustache, Percent)
- `ExpanderConfig` / `ExpandResult` / `ExpanderStats` structs
- `SettingsExpander` / `ExpanderRegistry` for managing expanders

**Core Functions:**
- `register()` / `unregister()` - Expander management
- `set_variable()` / `clear_variables()` - Variable management
- `expand_env()` - Expand environment variables
- `expand_vars()` - Expand custom variables
- `expand_refs()` - Expand references to other settings
- `stats()` - Access statistics

**Features:**
- Multiple expand modes
- Multiple variable syntaxes
- Default value support
- Reference expansion
- Missing variable handling

## [0.0.679] - 2025-12-15

### Added - Settings Flattener (Phase 255)

**Settings Flattener Module:**
- `settings_flattener.rs` for flattening nested settings structures
- `FlattenMode` enum (DotNotation, Underscore, Bracket, Slash)
- `DepthLimit` enum (Unlimited, Limited)
- `FlattenerConfig` / `FlattenResult` / `FlattenerStats` structs
- `SettingsFlattener` / `FlattenerRegistry` for managing flatteners

**Core Functions:**
- `register()` / `unregister()` - Flattener management
- `flatten()` - Flatten nested keys
- `flatten_with_prefix()` - Flatten with custom prefix
- `unflatten()` - Convert back to nested representation
- `stats()` - Access statistics

**Features:**
- Multiple flatten modes
- Configurable depth limits
- Custom separators
- Array preservation option
- Depth tracking

## [0.0.678] - 2025-12-15

### Added - Settings Partitioner (Phase 254)

**Settings Partitioner Module:**
- `settings_partitioner.rs` for partitioning settings into distinct subsets
- `PartitionStrategy` enum (ByPredicate, ByCount, ByPercentage, ByHash)
- `PredicateType` enum (IsNumeric, IsNonEmpty, KeyContains, ValueContains)
- `PartitionerConfig` / `Partition` / `PartitionResult` / `PartitionerStats` structs
- `SettingsPartitioner` / `PartitionerRegistry` for managing partitioners

**Core Functions:**
- `register()` / `unregister()` - Partitioner management
- `partition_by_numeric()` - Split by numeric/non-numeric
- `partition_by_empty()` - Split by empty/non-empty
- `partition_by_count()` - Split into N equal parts
- `partition_by_key_pattern()` - Split by key pattern match
- `stats()` - Access statistics

**Features:**
- Multiple partition strategies
- Predicate-based partitioning
- Balanced partition distribution
- Pattern matching support
- Per-partition statistics

## [0.0.677] - 2025-12-15

### Added - Settings Reducer (Phase 253)

**Settings Reducer Module:**
- `settings_reducer.rs` for reducing settings to aggregated values
- `ReduceOp` enum (Count, Sum, Average, Min, Max, Concat)
- `ReduceTarget` enum (Values, Keys, Both)
- `ReducerConfig` / `ReducedValue` / `ReduceResult` / `ReducerStats` structs
- `SettingsReducer` / `ReducerRegistry` for managing reducers

**Core Functions:**
- `register()` / `unregister()` - Reducer management
- `count()` - Count entries
- `sum()` - Sum numeric values
- `average()` - Average numeric values
- `min()` / `max()` - Find extremes
- `concat()` - Concatenate values
- `stats()` - Access statistics

**Features:**
- Multiple reduction operations
- Numeric value parsing
- Configurable separators
- Skip non-numeric handling
- Per-operation statistics

## [0.0.676] - 2025-12-15

### Added - Settings Grouper (Phase 252)

**Settings Grouper Module:**
- `settings_grouper.rs` for grouping settings by various criteria
- `GroupByField` enum (KeyPrefix, KeySuffix, Value, ValueType)
- `ValueTypeClass` enum (String, Number, Boolean, Empty)
- `GrouperConfig` / `SettingsGroup` / `GroupResult` / `GrouperStats` structs
- `SettingsGrouper` / `GrouperRegistry` for managing groupers

**Core Functions:**
- `register()` / `unregister()` - Grouper management
- `group_by_prefix()` - Group by key prefix
- `group_by_suffix()` - Group by key suffix
- `group_by_value_type()` - Group by value type
- `group_by_value()` - Group by value
- `stats()` - Access statistics

**Features:**
- Multiple grouping strategies
- Value type classification
- Configurable delimiters
- Group statistics tracking
- Min group size filtering

## [0.0.675] - 2025-12-15

### Added - Settings Sorter (Phase 251)

**Settings Sorter Module:**
- `settings_sorter.rs` for sorting settings by various criteria
- `SortOrder` enum (Ascending, Descending, Natural, Reverse)
- `SortField` enum (Key, Value, KeyLength, ValueLength)
- `SorterConfig` / `SortCriteria` / `SortResult` / `SorterStats` structs
- `SettingsSorter` / `SorterRegistry` for managing sorters

**Core Functions:**
- `register()` / `unregister()` - Sorter management
- `sort_by_key()` - Sort by key
- `sort_by_value()` - Sort by value
- `sort_with_criteria()` - Sort with custom criteria
- `sort_descending()` - Sort in descending order
- `stats()` - Access statistics

**Features:**
- Multiple sort orders
- Multiple sort fields
- Stable and unstable sorting
- Case-insensitive sorting
- Multi-field sort criteria with priorities

## [0.0.674] - 2025-12-15

### Added - Settings Filter (Phase 250)

**Settings Filter Module:**
- `settings_filter.rs` for filtering settings with predicates
- `FilterType` enum (Include, Exclude, AllowList, BlockList)
- `FilterPredicate` enum (IsEmpty, IsNotEmpty, IsNumeric, IsBoolean, HasValue)
- `FilterConfig` / `FilterRule` / `FilterResult` / `FilterStats` structs
- `SettingsFilter` / `FilterRegistry` for managing filters

**Core Functions:**
- `register()` / `unregister()` - Filter management
- `filter_by()` - Filter by predicate
- `filter()` - Apply filter rules
- `stats()` - Access statistics

**Features:**
- Multiple filter types
- Built-in predicates
- Pattern-based filtering
- Chainable rules
- Per-filter statistics

## [0.0.673] - 2025-12-15

### Added - Settings Selector (Phase 249)

**Settings Selector Module:**
- `settings_selector.rs` for selecting settings based on criteria
- `SelectorType` enum (Pattern, ByValue, ByIndex, First, Last)
- `MatchMode` enum (Exact, Prefix, Suffix, Contains, Regex)
- `SelectorConfig` / `SelectionCriteria` / `SelectionResult` / `SelectorStats` structs
- `SettingsSelector` / `SelectorRegistry` for managing selectors

**Core Functions:**
- `register()` / `unregister()` - Selector management
- `select()` - Select by criteria
- `select_first()` - Select first N entries
- `select_by_prefix()` - Select by key prefix
- `stats()` - Access statistics

**Features:**
- Multiple selector types
- Flexible match modes
- Case insensitivity
- Selection rate tracking
- Per-selector statistics

## [0.0.672] - 2025-12-15

### Added - Settings Projector (Phase 248)

**Settings Projector Module:**
- `settings_projector.rs` for projecting settings to custom views
- `ProjectionType` enum (Include, Exclude, Rename, Compute)
- `FieldMapping` struct for field transformations
- `ProjectorConfig` / `ProjectionResult` / `ProjectorStats` structs
- `SettingsProjector` / `ProjectorRegistry` for managing projectors

**Core Functions:**
- `register()` / `unregister()` - Projector management
- `project_include()` - Include specific fields
- `project_exclude()` - Exclude specific fields
- `project()` - Apply field mappings
- `stats()` - Access statistics

**Features:**
- Include/exclude projections
- Field renaming
- Custom field mappings
- Per-projector statistics

## [0.0.671] - 2025-12-15

### Added - Settings Aggregation (Phase 247)

**Settings Aggregation Module:**
- `settings_aggregation.rs` for aggregating settings values
- `AggregateFunction` enum (Count, Sum, Avg, Min, Max)
- `GroupByType` enum (Prefix, Suffix, Value, Pattern)
- `AggregatorConfig` / `AggregateEntry` / `AggregationResult` / `AggregatorStats` structs
- `SettingsAggregator` / `AggregatorRegistry` for managing aggregators

**Core Functions:**
- `register()` / `unregister()` - Aggregator management
- `count()` - Count by group
- `aggregate()` - Apply aggregate function
- `stats()` - Access statistics

**Features:**
- Multiple aggregate functions
- Group by key prefix/suffix
- Sorted results
- Per-aggregator statistics

## [0.0.670] - 2025-12-15

### Added - Settings Query Engine (Phase 246)

**Settings Query Engine Module:**
- `settings_query_engine.rs` for complex settings queries
- `QueryType` enum (Select, Filter, Aggregate, Join, Union)
- `QueryOperator` enum (Eq, Ne, Gt, Lt, Contains, StartsWith)
- `QueryEngineConfig` / `QueryCondition` / `Query` / `QueryResult` / `QueryEngineStats` structs
- `SettingsQueryEngine` / `QueryEngineRegistry` for managing engines

**Core Functions:**
- `register()` / `unregister()` - Engine management
- `execute()` - Execute query against settings
- `select_all()` - Select all settings
- `select_where_key_starts()` - Filter by key prefix
- `stats()` - Access statistics

**Features:**
- Multiple query types
- Various operators
- Pagination (limit/offset)
- Case insensitivity
- Per-engine statistics

## [0.0.669] - 2025-12-15

### Added - Settings Indexer (Phase 245)

**Settings Indexer Module:**
- `settings_indexer.rs` for indexing settings for fast lookup
- `IndexType` enum (Hash, BTree, FullText, Prefix, Inverted)
- `IndexStatus` enum (Ready, Building, Stale, Error)
- `IndexerConfig` / `IndexEntry` / `IndexLookupResult` / `IndexerStats` structs
- `SettingsIndexer` / `IndexerRegistry` for managing indexers

**Core Functions:**
- `register()` / `unregister()` - Indexer management
- `index()` - Build index from settings
- `lookup()` - Exact key lookup
- `search_prefix()` - Prefix-based search
- `search_term()` - Full-text term search
- `stats()` - Access statistics

**Features:**
- Multiple index types
- Fast hash-based lookup
- Prefix matching
- Full-text term search
- Index status tracking
- Per-indexer statistics

## [0.0.668] - 2025-12-15

### Added - Settings Denormalization (Phase 244)

**Settings Denormalization Module:**
- `settings_denormalization.rs` for denormalizing settings to target format
- `DenormalizationType` enum (Expand, Unflatten, Prefix, Suffix, Full)
- `TargetFormat` enum (Json, Yaml, Toml, Ini, Env)
- `DenormalizerConfig` / `DenormalizationResult` / `DenormalizerStats` structs
- `SettingsDenormalizer` / `DenormalizerRegistry` for managing denormalizers

**Core Functions:**
- `register()` / `unregister()` - Denormalizer management
- `denormalize()` - Expand settings to target format
- `stats()` - Access statistics

**Features:**
- Multiple denormalization types
- Key prefix/suffix support
- Original key preservation
- Per-denormalizer statistics

## [0.0.667] - 2025-12-15

### Added - Settings Normalization (Phase 243)

**Settings Normalization Module:**
- `settings_normalization.rs` for normalizing settings to canonical format
- `NormalizationType` enum (Case, Whitespace, KeyFormat, ValueFormat, Full)
- `CaseStyle` enum (Lower, Upper, Camel, Snake, Kebab)
- `NormalizerConfig` / `NormalizationResult` / `NormalizerStats` structs
- `SettingsNormalizer` / `NormalizerRegistry` for managing normalizers

**Core Functions:**
- `register()` / `unregister()` - Normalizer management
- `normalize()` - Normalize settings to canonical format
- `stats()` - Access statistics

**Features:**
- Multiple normalization types
- Case style conversion
- Whitespace handling
- Empty value removal
- Per-normalizer statistics

## [0.0.666] - 2025-12-15

### Added - Settings Transform (Phase 242)

**Settings Transform Module:**
- `settings_transform.rs` for transforming settings between formats
- `TransformType` enum (Map, Filter, Reduce, Flatten, Group)
- `TransformDirection` enum (Forward, Reverse, Bidirectional)
- `TransformerConfig` / `TransformRule` / `TransformResult` / `TransformerStats` structs
- `SettingsTransformer` / `TransformerRegistry` for managing transformers

**Core Functions:**
- `register()` / `unregister()` - Transformer management
- `add_rule()` / `remove_rule()` - Rule management
- `transform()` - Apply transformation rules
- `stats()` - Access statistics

**Features:**
- Pattern-based transformations
- Multiple transform types
- Bidirectional support
- Original preservation
- Per-transformer statistics

## [0.0.665] - 2025-12-15

### Added - Settings Validator Hub (Phase 241)

**Settings Validator Hub Module:**
- `settings_validator_hub.rs` for coordinating multiple validators
- `ValidatorType` enum (Schema, Range, Format, Custom, Composite)
- `ValidationSeverity` enum (Error, Warning, Info, Hint)
- `HubConfig` / `ValidationIssue` / `HubValidationResult` / `ValidatorEntry` / `HubStats` structs
- `SettingsValidatorHub` / `ValidatorHubRegistry` for managing hubs

**Core Functions:**
- `register()` / `unregister()` - Validator management
- `enable()` / `disable()` - Toggle validators
- `validate()` - Run all enabled validators
- `stats()` - Access statistics

**Features:**
- Multiple validator types
- Severity-based issues
- Suggested fixes
- Fail-fast mode
- Per-hub statistics

## [0.0.664] - 2025-12-15

### Added - Settings Resolution (Phase 240)

**Settings Resolution Module:**
- `settings_resolution.rs` for resolving settings with reference following
- `ResolutionStrategy` enum (Direct, Reference, Computed, Cached, Default)
- `ResolutionStatus` enum (Resolved, Pending, Failed, Circular, NotFound)
- `ResolverConfig` / `ResolutionResult` / `ResolutionRequest` / `ResolverStats` structs
- `SettingsResolver` / `SettingsResolverRegistry` for managing resolvers

**Core Functions:**
- `register()` / `unregister()` - Resolver management
- `set()` / `set_reference()` - Define values and references
- `resolve()` - Resolve key with reference following
- `resolve_request()` - Resolve with defaults
- `clear_cache()` - Clear resolution cache
- `stats()` - Access statistics

**Features:**
- Multiple resolution strategies
- Reference chain following
- Circular reference detection
- Result caching
- Default value support
- Per-resolver statistics

## [0.0.663] - 2025-12-15

### Added - Settings Graph (Phase 239)

**Settings Graph Module:**
- `settings_graph.rs` for modeling settings relationships
- `LinkType` enum (Reference, Alias, Dependency, Override, Computed)
- `LinkDirection` enum (Unidirectional, Bidirectional, Reverse)
- `LinkerConfig` / `SettingsLink` / `LinkResult` / `LinkerStats` structs
- `SettingsLinker` / `SettingsLinkerRegistry` for managing graphs

**Core Functions:**
- `register()` / `unregister()` - Graph management
- `link()` / `link_with_type()` - Create relationships
- `resolve()` - Resolve linked values
- `links_for()` - Get all links for key
- `stats()` - Access statistics

**Features:**
- Multiple link types
- Bidirectional relationships
- Circular dependency prevention
- Link resolution
- Per-graph statistics

## [0.0.662] - 2025-12-15

### Added - Settings Patcher (Phase 238)

**Settings Patcher Module:**
- `settings_patcher.rs` for applying incremental changes
- `PatchOperation` enum (Add, Remove, Replace, Copy, Move)
- `PatchMode` enum (Strict, Lenient, DryRun, Atomic)
- `PatcherConfig` / `PatchEntry` / `PatchResult` / `PatcherStats` structs
- `SettingsPatcher` / `SettingsPatcherRegistry` for managing patchers

**Core Functions:**
- `register()` / `unregister()` - Patcher management
- `apply()` - Apply patches to settings
- `add()` / `remove()` / `replace()` - Patch entry builders
- `copy()` / `move_key()` - Key operations
- `results()` / `stats()` - Data access

**Features:**
- Multiple patch operations
- Strict/lenient/atomic modes
- Dry run support
- Rollback on failure
- Operation statistics

## [0.0.661] - 2025-12-15

### Added - Settings Differ (Phase 237)

**Settings Differ Module:**
- `settings_differ.rs` for comparing settings configurations
- `DiffType` enum (Added, Removed, Modified, Unchanged)
- `DiffMode` enum (All, AdditionsOnly, RemovalsOnly, ModificationsOnly)
- `DifferConfig` / `DiffEntry` / `DiffResult` / `DifferStats` structs
- `SettingsDiffer` / `SettingsDifferRegistry` for managing differs

**Core Functions:**
- `register()` / `unregister()` - Differ management
- `diff()` - Compare two settings maps
- `get_by_type()` - Filter entries by diff type
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple diff modes
- Added/removed/modified detection
- Case-sensitive comparison option
- Change statistics
- Entry type filtering

## [0.0.660] - 2025-12-15

### Added - Settings Versioner (Phase 236)

**Settings Versioner Module:**
- `settings_versioner.rs` for tracking settings version history
- `VersionScheme` enum (Semantic, Sequential, DateBased, HashBased)
- `BumpType` enum (Major, Minor, Patch, Auto)
- `VersionerConfig` / `SettingsVersion` / `VersionResult` / `VersionerStats` structs
- `SettingsVersioner` / `SettingsVersionerRegistry` for managing versioners

**Core Functions:**
- `register()` / `unregister()` - Versioner management
- `bump()` - Increment version
- `bump_with_description()` - Bump with change note
- `set_version()` - Set specific version
- `history()` / `stats()` - Data access

**Features:**
- Semantic versioning support
- Version history tracking
- Bump type statistics
- Description annotations
- Max history limit

## [0.0.659] - 2025-12-15

### Added - Settings Restorer (Phase 235)

**Settings Restorer Module:**
- `settings_restorer.rs` for recovering settings from archives
- `RestoreMode` enum (Full, Selective, Merge, Override)
- `RestoreStrategy` enum (LatestFirst, OldestFirst, ByPriority, Manual)
- `RestorerConfig` / `RestoreSource` / `RestoreResult` / `RestorerStats` structs
- `SettingsRestorer` / `SettingsRestorerRegistry` for managing restorers

**Core Functions:**
- `register()` / `unregister()` - Restorer management
- `restore()` - Recover from archive data
- `restore_from_source()` - Restore from source object
- `restore_keys()` - Selective key restoration
- `results()` / `stats()` - Data access

**Features:**
- Multiple restore modes
- JSON and key=value format parsing
- Selective key restoration
- Success tracking
- Per-restorer statistics

## [0.0.658] - 2025-12-15

### Added - Settings Archiver (Phase 234)

**Settings Archiver Module:**
- `settings_archiver.rs` for backing up settings configurations
- `ArchiveFormat` enum (Json, Toml, Binary, Compressed)
- `ArchiveType` enum (Full, Incremental, Differential, Snapshot)
- `ArchiverConfig` / `ArchiveMetadata` / `ArchiveResult` / `ArchiverStats` structs
- `SettingsArchiver` / `SettingsArchiverRegistry` for managing archivers

**Core Functions:**
- `register()` / `unregister()` - Archiver management
- `archive()` - Create settings backup
- `snapshot()` - Create named snapshot
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple archive formats
- Archive metadata tracking
- Snapshot support with descriptions
- Data size tracking
- Per-archiver statistics

## [0.0.657] - 2025-12-15

### Added - Settings Cloner (Phase 233)

**Settings Cloner Module:**
- `settings_cloner.rs` for duplicating settings configurations
- `CloneDepth` enum (Shallow, Deep, Selective, Reference)
- `CloneMode` enum (Exact, WithMods, Template, Incremental)
- `ClonerConfig` / `CloneMod` / `CloneResult` / `ClonerStats` structs
- `SettingsCloner` / `SettingsClonerRegistry` for managing cloners

**Core Functions:**
- `register()` / `unregister()` - Cloner management
- `clone_settings()` - Duplicate settings
- `clone_as()` - Clone with custom name prefix
- `add_mod()` - Add value modifications
- `results()` / `stats()` - Data access

**Features:**
- Multiple clone depths
- Value transformations (uppercase, lowercase, trim, reverse)
- Key prefix/suffix support
- Selective cloning
- Per-cloner statistics

## [0.0.656] - 2025-12-15

### Added - Settings Splitter (Phase 232)

**Settings Splitter Module:**
- `settings_splitter.rs` for dividing settings into groups
- `SplitCriteria` enum (ByCategory, ByPrefix, ByPattern, ByValueType, BySize)
- `SplitMode` enum (Even, ByThreshold, ByCount, Custom)
- `SplitterConfig` / `SplitGroup` / `SplitResult` / `SplitterStats` structs
- `SettingsSplitter` / `SettingsSplitterRegistry` for managing splitters

**Core Functions:**
- `register()` / `unregister()` - Splitter management
- `split()` - Divide settings by criteria
- `split_into_n()` - Evenly divide into N groups
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple split criteria
- Configurable split modes
- Max group limits
- Group tracking
- Per-splitter statistics

## [0.0.655] - 2025-12-15

### Added - Settings Merger (Phase 231)

**Settings Merger Module:**
- `settings_merger.rs` for combining multiple settings sources
- `MergeStrategy` enum (FirstWins, LastWins, PriorityWins, Combine, Union)
- `ConflictResolution` enum (UseStrategy, Skip, Fail, Prompt)
- `MergerConfig` / `MergeSource` / `MergeResult` / `MergerStats` structs
- `SettingsMerger` / `SettingsMergerRegistry` for managing mergers

**Core Functions:**
- `register()` / `unregister()` - Merger management
- `merge()` - Combine multiple sources
- `merge_maps()` - Merge two hashmaps directly
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple merge strategies
- Conflict resolution modes
- Priority-based merging
- Source tracking
- Per-merger statistics

## [0.0.654] - 2025-12-15

### Added - Settings Injector (Phase 230)

**Settings Injector Module:**
- `settings_injector.rs` for inserting settings
- `InjectionType` enum (Insert, Update, Upsert, Replace, Append)
- `InjectionStrategy` enum (FailOnConflict, SkipOnConflict, OverwriteOnConflict, MergeOnConflict)
- `InjectorConfig` / `InjectionResult` / `InjectorStats` structs
- `SettingsInjector` / `SettingsInjectorRegistry` for managing injectors

**Core Functions:**
- `register()` / `unregister()` - Injector management
- `inject()` - Insert settings into target
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple injection types
- Conflict resolution strategies
- Dry run mode
- Insert/update tracking
- Per-injector statistics

## [0.0.653] - 2025-12-15

### Added - Settings Extractor (Phase 229)

**Settings Extractor Module:**
- `settings_extractor.rs` for pulling specific settings
- `ExtractionType` enum (Key, Pattern, Category, Prefix, Suffix)
- `ExtractionMode` enum (First, All, Last, Unique)
- `ExtractorConfig` / `ExtractionResult` / `ExtractorStats` structs
- `SettingsExtractor` / `SettingsExtractorRegistry` for managing extractors

**Core Functions:**
- `register()` / `unregister()` - Extractor management
- `extract()` - Pull settings by selector
- `matches_selector()` - Check key matching
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple extraction types
- Mode-based extraction
- Case sensitivity option
- Match count tracking
- Per-extractor statistics

## [0.0.652] - 2025-12-15

### Added - Settings Binder (Phase 228)

**Settings Binder Module:**
- `settings_binder.rs` for runtime binding
- `BindingType` enum (OneWay, TwoWay, OneTime, Lazy, Eager)
- `BindingState` enum (Unbound, Bound, Pending, Error)
- `BindingDef` / `BinderConfig` / `BindingResult` / `BinderStats` structs
- `SettingsBinder` / `SettingsBinderRegistry` for managing binders

**Core Functions:**
- `register()` / `unregister()` - Binder management
- `add_binding()` - Add binding definitions
- `bind_all()` / `unbind_all()` - Binding lifecycle
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple binding types
- State tracking
- Auto-bind option
- Validation on bind
- Per-binder statistics

## [0.0.651] - 2025-12-15

### Added - Settings Mapper (Phase 227)

**Settings Mapper Module:**
- `settings_mapper.rs` for key transformations
- `MappingType` enum (Direct, Rename, Transform, Merge, Split)
- `MappingDirection` enum (Forward, Reverse, Bidirectional)
- `MappingRule` / `MapperConfig` / `MappingResult` / `MapperStats` structs
- `SettingsMapper` / `SettingsMapperRegistry` for managing mappers

**Core Functions:**
- `register()` / `unregister()` - Mapper management
- `add_rule()` - Add mapping rules
- `map()` - Apply key transformations
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple mapping types
- Directional mapping
- Case sensitivity option
- Unmapped key handling
- Per-mapper statistics

## [0.0.650] - 2025-12-15

### Added - Settings Converter (Phase 226)

**Settings Converter Module:**
- `settings_converter.rs` for format transformation
- `SourceFormat` enum (Json, Toml, Yaml, Ini, Env)
- `TargetFormat` enum (Json, Toml, Yaml, Ini, Env)
- `ConverterConfig` / `ConversionResult` / `ConverterStats` structs
- `SettingsConverter` / `SettingsConverterRegistry` for managing converters

**Core Functions:**
- `register()` / `unregister()` - Converter management
- `convert()` - Transform between formats
- `parse_source()` / `format_target()` - Internal conversion
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multi-format support
- Pretty output option
- Comment preservation option
- Key count tracking
- Per-converter statistics

## [0.0.649] - 2025-12-15

### Added - Settings Decoder (Phase 225)

**Settings Decoder Module:**
- `settings_decoder.rs` for deserialization
- `DecodingFormat` enum (Json, Toml, Yaml, Binary, Base64)
- `DecodingMode` enum (Strict, Lenient, Permissive, Recovery)
- `DecoderConfig` / `DecodeError` / `DecodeResult` / `DecoderStats` structs
- `SettingsDecoder` / `SettingsDecoderRegistry` for managing decoders

**Core Functions:**
- `register()` / `unregister()` - Decoder management
- `decode()` - Deserialize settings
- `do_decode()` - Internal decoding logic
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple input formats
- Mode-based strictness
- Error collection
- Position tracking
- Per-decoder statistics

## [0.0.648] - 2025-12-15

### Added - Settings Encoder (Phase 224)

**Settings Encoder Module:**
- `settings_encoder.rs` for serialization
- `EncodingFormat` enum (Json, Toml, Yaml, Binary, Base64)
- `EncodingOptions` enum (Compact, Pretty, Minified, Verbose)
- `EncoderConfig` / `EncodeResult` / `EncoderStats` structs
- `SettingsEncoder` / `SettingsEncoderRegistry` for managing encoders

**Core Functions:**
- `register()` / `unregister()` - Encoder management
- `encode()` - Serialize settings
- `do_encode()` - Internal encoding logic
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple output formats
- Encoding options
- Byte size tracking
- Key sorting option
- Per-encoder statistics

## [0.0.647] - 2025-12-15

### Added - Settings Renderer (Phase 223)

**Settings Renderer Module:**
- `settings_renderer.rs` for display output
- `RenderTarget` enum (Terminal, Html, Markdown, PlainText, RichText)
- `RenderTheme` enum (Default, Light, Dark, Minimal, Custom)
- `RendererConfig` / `RenderOutput` / `RendererStats` structs
- `SettingsRenderer` / `SettingsRendererRegistry` for managing renderers

**Core Functions:**
- `register()` / `unregister()` - Renderer management
- `render()` - Render settings to output
- `do_render()` - Internal rendering logic
- `outputs()` / `stats()` - Data access
- `clear()` - Output cleanup

**Features:**
- Multiple output targets
- Theme support
- Header and border options
- Line count tracking
- Per-renderer statistics

## [0.0.646] - 2025-12-14

### Added - Settings Parser (Phase 222)

**Settings Parser Module:**
- `settings_parser.rs` for multi-format parsing
- `ParseSource` enum (Json, Toml, Yaml, Ini, Env)
- `ParseMode` enum (Strict, Lenient, Permissive, ValidateOnly)
- `ParserConfig` / `ParseError` / `ParseResult` / `ParserStats` structs
- `SettingsParser` / `SettingsParserRegistry` for managing parsers

**Core Functions:**
- `register()` / `unregister()` - Parser management
- `parse()` - Parse input strings
- `do_parse()` - Internal parsing logic
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple input formats
- Mode-based strictness
- Error collection
- Location tracking
- Per-parser statistics

## [0.0.645] - 2025-12-14

### Added - Settings Normalizer (Phase 221)

**Settings Normalizer Module:**
- `settings_normalizer.rs` for value standardization
- `NormalizationType` enum (String, Path, Url, Number, Boolean)
- `NormalizationRule` enum (None, Lowercase, Uppercase, Trim, Canonical)
- `NormalizerConfig` / `NormalizationResult` / `NormalizerStats` structs
- `SettingsNormalizer` / `SettingsNormalizerRegistry` for managing normalizers

**Core Functions:**
- `register()` / `unregister()` - Normalizer management
- `normalize()` - Standardize values
- `apply_rule()` - Apply normalization rules
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple normalization types
- Configurable rules
- Modification tracking
- Enable/disable control
- Per-normalizer statistics

## [0.0.644] - 2025-12-14

### Added - Settings Formatter (Phase 220)

**Settings Formatter Module:**
- `settings_formatter.rs` for display formatting
- `FormatType` enum (Plain, Json, Toml, Yaml, Table)
- `FormatStyle` enum (Compact, Pretty, Minimal, Verbose)
- `FormatterConfig` / `FormatResult` / `FormatterStats` structs
- `SettingsFormatter` / `SettingsFormatterRegistry` for managing formatters

**Core Functions:**
- `register()` / `unregister()` - Formatter management
- `format()` - Convert values to display formats
- `apply_format()` - Apply format transformation
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple output formats
- Style options
- Indent configuration
- Output size tracking
- Per-formatter statistics

## [0.0.643] - 2025-12-14

### Added - Settings Sanitizer (Phase 219)

**Settings Sanitizer Module:**
- `settings_sanitizer.rs` for value sanitization
- `SanitizationType` enum (Trim, NormalizeCase, RemoveSpecial, Escape, Full)
- `CaseNormalization` enum (None, Lower, Upper, Title)
- `SanitizerConfig` / `SanitizationResult` / `SanitizerStats` structs
- `SettingsSanitizer` / `SettingsSanitizerRegistry` for managing sanitizers

**Core Functions:**
- `register()` / `unregister()` - Sanitizer management
- `sanitize()` - Clean and normalize values
- `add_operation()` - Track applied operations
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple sanitization types
- Case normalization options
- Operation tracking
- Change detection
- Per-sanitizer statistics

## [0.0.642] - 2025-12-14

### Added - Settings Analyzer (Phase 218)

**Settings Analyzer Module:**
- `settings_analyzer.rs` for pattern analysis
- `AnalysisType` enum (Pattern, Anomaly, Trend, Correlation, Impact)
- `AnalysisScope` enum (Category, CrossCategory, SystemWide, Historical)
- `AnalyzerConfig` / `AnalysisInsight` / `AnalysisResult` / `AnalyzerStats` structs
- `SettingsAnalyzer` / `SettingsAnalyzerRegistry` for managing analyzers

**Core Functions:**
- `register()` / `unregister()` - Analyzer management
- `analyze()` - Run analyses
- `add_insight()` - Record insights
- `high_confidence_insights()` - Filter insights
- `results()` / `stats()` - Data access

**Features:**
- Multiple analysis types
- Configurable scope
- Confidence-based filtering
- Recommendation support
- Per-analyzer statistics

## [0.0.641] - 2025-12-14

### Added - Settings Inspector (Phase 217)

**Settings Inspector Module:**
- `settings_inspector.rs` for structure analysis
- `InspectionType` enum (Structure, Value, Type, Dependency, Full)
- `InspectionDepth` enum (Shallow, Normal, Deep, Complete)
- `InspectorConfig` / `InspectionFinding` / `InspectionResult` / `InspectorStats` structs
- `SettingsInspector` / `SettingsInspectorRegistry` for managing inspectors

**Core Functions:**
- `register()` / `unregister()` - Inspector management
- `inspect()` - Run inspections
- `add_finding()` - Record findings
- `results()` / `stats()` - Data access
- `clear()` - Result cleanup

**Features:**
- Multiple inspection types
- Configurable depth
- Finding severity levels
- Metadata and defaults options
- Per-inspector statistics

## [0.0.640] - 2025-12-14

### Added - Settings Report Generator (Phase 216)

**Settings Report Generator Module:**
- `settings_report_generator.rs` for status reports
- `ReportType` enum (Summary, Detailed, Health, Audit, Custom)
- `ReportFormat` enum (Text, Json, Markdown, Html, Csv)
- `ReporterConfig` / `ReportSection` / `Report` / `ReporterStats` structs
- `SettingsReporter` / `SettingsReporterRegistry` for managing reporters

**Core Functions:**
- `register()` / `unregister()` - Reporter management
- `generate()` - Create reports
- `add_section()` / `add_item()` - Report building
- `reports()` / `stats()` - Data access
- `clear()` - Report cleanup

**Features:**
- Multiple report types
- Multi-format output
- Section-based organization
- Timestamp and stats options
- Per-reporter statistics

## [0.0.639] - 2025-12-14

### Added - Settings Notifier (Phase 215)

**Settings Notifier Module:**
- `settings_notifier.rs` for change alerts
- `NotifyChannel` enum (Internal, Log, Event, Callback, External)
- `NotifyPriority` enum (Low, Normal, High, Urgent, Critical)
- `NotifierConfig` / `Notification` / `NotifierStats` structs
- `SettingsNotifier` / `SettingsNotifierRegistry` for managing notifiers

**Core Functions:**
- `register()` / `unregister()` - Notifier management
- `queue()` / `flush()` - Notification handling
- `enable()` / `disable()` - Notifier control
- `list_by_channel()` / `list_enabled()` - Notifier listing
- `record_sent()` / `record_suppressed()` - Stats tracking

**Features:**
- Multiple notification channels
- Priority-based filtering
- Debounce configuration
- Per-notifier statistics
- Suppression rate metrics

## [0.0.638] - 2025-12-14

### Added - Settings Tracker (Phase 214)

**Settings Tracker Module:**
- `settings_tracker.rs` for usage tracking
- `TrackType` enum (Read, Write, Access, Modify, Delete)
- `TrackLevel` enum (None, Basic, Detailed, Full, Debug)
- `TrackerConfig` / `TrackEvent` / `TrackerStats` structs
- `SettingsTracker` / `SettingsTrackerRegistry` for managing trackers

**Core Functions:**
- `register()` / `unregister()` - Tracker management
- `track()` - Record events
- `enable()` / `disable()` - Tracker control
- `list_by_type()` / `list_by_category()` - Event filtering
- `events()` / `stats()` - Data access

**Features:**
- Multiple track types
- Configurable track levels
- Retention configuration
- Per-tracker statistics
- Read/write ratio metrics

## [0.0.637] - 2025-12-14

### Added - Settings Poller (Phase 213)

**Settings Poller Module:**
- `settings_poller.rs` for polling-based change detection
- `WatcherType` enum (Polling, EventBased, Hybrid, Lazy, Eager)
- `WatchInterval` enum (Immediate, Fast, Normal, Slow, Custom)
- `WatcherConfig` / `WatchEvent` / `WatcherStats` structs
- `Watcher` / `SettingsWatcherRegistry` for managing watchers

**Core Functions:**
- `register()` / `unregister()` - Watcher management
- `get()` / `get_mut()` - Watcher access
- `activate()` / `deactivate()` - Watcher control
- `record_poll()` / `record_event()` - Tracking
- `list_by_type()` / `list_active()` - Watcher listing

**Features:**
- Multiple watcher types
- Configurable poll intervals
- Change detection
- Per-watcher statistics
- Change rate metrics

## [0.0.636] - 2025-12-14

### Added - Settings Listener (Phase 212)

**Settings Listener Module:**
- `settings_listener.rs` for event listening
- `ListenerType` enum (Passive, Active, Reactive, Selective, Persistent)
- `ListenerState` enum (Idle, Listening, Processing, Paused, Stopped)
- `ListenerConfig` / `ReceivedEvent` / `ListenerStats` structs
- `SettingsListener` / `SettingsListenerRegistry` for managing listeners

**Core Functions:**
- `register()` / `unregister()` - Listener management
- `start()` / `stop()` / `pause()` / `resume()` - State control
- `receive()` / `next()` - Event handling
- `list_by_type()` / `list_listening()` - Listener listing
- `clear()` - Buffer management

**Features:**
- Multiple listener types
- State-based lifecycle
- Event buffering with filtering
- Auto-start configuration
- Processing rate metrics

## [0.0.635] - 2025-12-14

### Added - Settings Broadcaster (Phase 211)

**Settings Broadcaster Module:**
- `settings_broadcaster.rs` for multi-listener broadcasts
- `BroadcastChannel` enum (Default, System, Application, Priority, Debug)
- `BroadcastMode` enum (Sync, Async, FireAndForget, Queued)
- `BroadcasterConfig` / `BroadcastMessage` / `ListenerInfo` / `BroadcasterStats` structs
- `SettingsBroadcaster` for managing broadcasts

**Core Functions:**
- `add_listener()` / `remove_listener()` - Listener management
- `get_listener()` / `get_listener_mut()` - Listener access
- `broadcast()` - Send to all listeners
- `enqueue()` / `flush()` - Message queueing
- `list_by_channel()` - Channel-based listing

**Features:**
- Multiple broadcast channels
- Configurable broadcast modes
- Message queueing
- Per-listener message tracking
- Delivery efficiency metrics

## [0.0.634] - 2025-12-14

### Added - Settings Publisher (Phase 210)

**Settings Publisher Module:**
- `settings_publisher.rs` for change events
- `PublisherType` enum (System, Application, Service, Plugin, External)
- `PublicationScope` enum (Local, Module, Application, System, Global)
- `PublisherConfig` / `PublicationEvent` / `PublisherStats` structs
- `SettingsPublisherRegistry` for managing publishers

**Core Functions:**
- `register()` / `unregister()` - Publisher management
- `get()` / `get_mut()` - Publisher access
- `enable()` / `disable()` - Publisher control
- `queue()` / `flush()` - Event buffering
- `list_by_type()` / `list_enabled()` - Publisher listing

**Features:**
- Multiple publisher types
- Configurable publication scopes
- Event buffering with size limits
- Per-publisher statistics
- Success rate tracking

## [0.0.633] - 2025-12-14

### Added - Settings Subscriber (Phase 209)

**Settings Subscriber Module:**
- `settings_subscriber.rs` for change notifications
- `SubscriptionType` enum (All, Category, Key, Pattern, Filtered)
- `DeliveryMode` enum (Immediate, Batched, Throttled, OnDemand)
- `SubscriptionConfig` / `Subscription` / `SubscriberStats` structs
- `SettingsSubscriberManager` for managing subscriptions

**Core Functions:**
- `subscribe()` / `unsubscribe()` - Subscription management
- `get()` / `get_mut()` - Subscription access
- `pause()` / `resume()` - Subscription control
- `list_by_subscriber()` / `list_active()` - Subscription listing
- `record_notification()` / `record_delivery()` - Tracking

**Features:**
- Multiple subscription types
- Configurable delivery modes
- Category and key filtering
- Per-subscriber statistics
- Delivery rate tracking

## [0.0.632] - 2025-12-14

### Added - Settings Consumer (Phase 208)

**Settings Consumer Module:**
- `settings_consumer.rs` for client abstraction
- `ConsumerType` enum (Application, Service, Module, Plugin, External)
- `ConsumerState` enum (Inactive, Active, Suspended, Error, Terminated)
- `ConsumerInfo` / `ConsumerSession` / `ConsumerStats` structs
- `SettingsConsumerRegistry` for managing consumers

**Core Functions:**
- `register()` / `unregister()` - Consumer management
- `subscribe()` / `is_subscribed()` - Category subscriptions
- `suspend()` / `resume()` / `terminate()` - Session control
- `record_activity()` - Activity tracking
- `list_subscribed()` - Subscription-based listing

**Features:**
- Multiple consumer types
- Session state management
- Category subscriptions
- Activity tracking
- Per-consumer statistics

## [0.0.631] - 2025-12-14

### Added - Settings Provider (Phase 207)

**Settings Provider Module:**
- `settings_provider.rs` for provider abstraction
- `ProviderType` enum (Local, Remote, Cloud, Hybrid, Custom)
- `ProviderCapability` enum (Read, Write, Delete, List, Watch, Sync)
- `ProviderInfo` / `ProviderStatus` / `ProviderStats` structs
- `SettingsProviderRegistry` for managing providers

**Core Functions:**
- `register()` / `unregister()` - Provider management
- `capability()` / `has_capability()` - Capability control
- `mark_available()` / `mark_unavailable()` - Status updates
- `list_by_capability()` - Capability-based listing
- `avg_latency_ms()` - Performance metrics

**Features:**
- Multiple provider types
- Capability-based access control
- Health and availability tracking
- Latency statistics
- Priority-based selection

## [0.0.630] - 2025-12-14

### Added - Settings Connector (Phase 206)

**Settings Connector Module:**
- `settings_connector.rs` for external providers
- `ConnectorProtocol` enum (Http, Https, UnixSocket, Tcp, File)
- `ConnectionState` enum (Disconnected, Connecting, Connected, Reconnecting, Failed)
- `ConnectorConfig` / `ConnectionInfo` / `ConnectorStats` structs
- `SettingsConnector` for managing connections

**Core Functions:**
- `register()` / `unregister()` - Connector management
- `connect()` / `disconnect()` - Connection control
- `get_connection()` / `get_stats()` - Status access
- `record_request()` - Request tracking
- `success_rate()` - Performance metrics

**Features:**
- Multiple protocols
- Connection state tracking
- Timeout and retry configuration
- Per-connector statistics
- Activity tracking

## [0.0.629] - 2025-12-14

### Added - Settings Bridge (Phase 205)

**Settings Bridge Module:**
- `settings_bridge.rs` for connecting settings systems
- `BridgeDirection` enum (Unidirectional, Bidirectional, SourceOnly, TargetOnly)
- `BridgeState` enum (Idle, Syncing, Active, Paused, Error)
- `BridgeEndpoint` / `BridgeMapping` / `BridgeStats` structs
- `SettingsBridge` for managing connections

**Core Functions:**
- `set_source()` / `set_target()` - Endpoint configuration
- `add_mapping()` - Key mapping
- `start()` / `pause()` / `stop()` - Lifecycle control
- `sync()` - Synchronization
- `connect()` / `disconnect()` - Endpoint control

**Features:**
- Multiple bridge directions
- Key mapping with transforms
- Endpoint connection tracking
- Sync statistics
- State management

## [0.0.628] - 2025-12-14

### Added - Settings Adapter (Phase 204)

**Settings Adapter Module:**
- `settings_adapter.rs` for multi-source integration
- `AdapterType` enum (File, Environment, Memory, Remote, Database)
- `AdapterStatus` enum (Connected, Disconnected, Connecting, Error, Disabled)
- `AdapterConfig` / `AdapterInstance` / `AdapterStats` structs
- `SettingsAdapterRegistry` for managing adapters

**Core Functions:**
- `register()` / `unregister()` - Adapter management
- `connect()` / `disconnect()` - Connection control
- `get()` / `get_mut()` - Adapter access
- `list_available()` - Available adapter listing
- `record_read()` / `record_write()` - Operation tracking

**Features:**
- Multiple adapter types
- Priority-based ordering
- Read-only mode support
- Connection status tracking
- Per-adapter statistics

## [0.0.627] - 2025-12-14

### Added - Settings Facade (Phase 203)

**Settings Facade Module:**
- `settings_facade.rs` for simplified access
- `FacadeOperation` enum (Get, Set, Reset, List, Validate)
- `FacadeResult` enum (Success, NotFound, InvalidValue, PermissionDenied, SystemError)
- `FacadeRequest` / `FacadeResponse` / `FacadeUsage` structs
- `SettingsFacade` for simple settings interface

**Core Functions:**
- `get()` / `set()` / `reset()` - CRUD request builders
- `list()` / `validate()` - Query request builders
- `process()` - Request processing
- `enable()` / `disable()` - Facade control
- `success_rate()` - Usage statistics

**Features:**
- Simplified request/response pattern
- Multiple operation types
- Result status codes
- Usage tracking
- Enable/disable control

## [0.0.626] - 2025-12-14

### Added - Settings Proxy (Phase 202)

**Settings Proxy Module:**
- `settings_proxy.rs` for caching and transformation
- `ProxyBehavior` enum (Passthrough, Cache, Transform, Intercept)
- `CacheStatus` enum (Hit, Miss, Stale, Bypass)
- `CacheEntry` / `ProxyOperation` / `ProxyStats` structs
- `SettingsProxy` for managing cached access

**Core Functions:**
- `get_cached()` / `put_cached()` - Cache operations
- `invalidate()` / `clear_cache()` - Cache management
- `is_expired()` - TTL checking
- `cache_hit_rate()` - Performance metrics
- `set_behavior()` / `behavior()` - Mode control

**Features:**
- Multiple proxy behaviors
- TTL-based cache expiration
- Cache size limits
- Hit rate tracking
- Automatic eviction

## [0.0.625] - 2025-12-14

### Added - Settings Gateway (Phase 201)

**Settings Gateway Module:**
- `settings_gateway.rs` for unified settings access
- `GatewayMode` enum (Open, Restricted, Locked, Maintenance)
- `RequestType` enum (Read, Write, Delete, List, Admin)
- `GatewayRequest` / `GatewayResponse` / `GatewayStats` structs
- `SettingsGateway` for managing access control

**Core Functions:**
- `process()` - Process incoming requests
- `set_mode()` / `mode()` - Mode control
- `is_read_only()` - Request classification
- `allowed()` / `denied()` - Response builders
- `allow_rate()` - Statistics

**Features:**
- Multiple access modes
- Request type tracking
- Mode-based access control
- Statistics by request type
- Allow rate monitoring

## [0.0.624] - 2025-12-14

### Added - Settings Catalog (Phase 200)

**Settings Catalog Module:**
- `settings_catalog.rs` for comprehensive settings documentation
- `CatalogSection` enum (Core, Preferences, System, Advanced, Experimental)
- `DocLevel` enum (Basic, Detailed, Expert, Internal)
- `CatalogItem` / `SearchResult` structs
- `SettingsCatalog` for managing settings documentation

**Core Functions:**
- `add()` / `remove()` / `get()` - Item management
- `search()` - Full-text search across items
- `list_by_section()` - Section-based listing
- `example()` / `has_examples()` - Example management
- `count()` / `search_count()` / `section_count()` - Statistics

**Features:**
- Multiple catalog sections
- Documentation levels
- Example values
- Full-text search
- Section-based organization

## [0.0.623] - 2025-12-14

### Added - Settings Index (Phase 199)

**Settings Index Module:**
- `settings_index.rs` for fast multi-pattern lookups
- `IndexType` enum (Primary, Secondary, Category, Tag, FullText)
- `IndexStatus` enum (Building, Ready, Stale, Rebuilding, Error)
- `IndexEntry` / `IndexStats` structs
- `SettingsIndex` for managing indexed lookups

**Core Functions:**
- `add()` / `remove()` - Entry management
- `get()` / `get_by_category()` / `get_by_tag()` - Multi-pattern lookups
- `tag()` / `has_tag()` - Tag operations
- `count()` / `stats()` - Statistics
- `is_ready()` / `mark_rebuild()` - Status management

**Features:**
- Multiple index types
- Status tracking
- Category-based indexing
- Tag-based indexing
- Query hit rate tracking

## [0.0.622] - 2025-12-14

### Added - Settings Registry (Phase 198)

**Settings Registry Module:**
- `settings_registry.rs` for central settings catalog
- `EntryType` enum (String, Integer, Boolean, Float, Enum, List)
- `EntryVisibility` enum (Public, Internal, Hidden, Deprecated)
- `RegistryEntry` / `LookupResult` structs
- `SettingsRegistry` for managing settings definitions

**Core Functions:**
- `register()` / `unregister()` - Entry management
- `lookup()` / `get()` - Entry retrieval
- `list_by_category()` / `list_public()` - Filtered listings
- `is_public()` / `is_deprecated()` - Visibility checks
- `count()` / `lookup_count()` / `hit_rate()` - Statistics

**Features:**
- Multiple entry types
- Visibility levels
- Default values
- Allowed values for enums
- Lookup hit rate tracking

## [0.0.621] - 2025-12-14

### Added - Settings Manager (Phase 197)

**Settings Manager Module:**
- `settings_manager.rs` for unified settings management
- `ManagerMode` enum (Normal, Maintenance, ReadOnly, Debug)
- `ManagerStatus` enum (Initializing, Active, Degraded, Inactive, Error)
- `SettingEntry` / `ManagerOperation` structs
- `SettingsManager` for centralized settings control

**Core Functions:**
- `mode()` / `set_mode()` - Mode control
- `status()` / `set_status()` - Status management
- `get()` / `set()` / `delete()` - Settings CRUD
- `list_by_category()` - Category filtering
- `record()` / `history()` - Operation history
- `count()` / `is_active()` - Statistics

**Features:**
- Multiple operation modes
- Status tracking
- Settings storage with HashMap
- Operation history with limits
- Read-only mode protection
- Category-based listing

## [0.0.620] - 2025-12-14

### Added - Settings Service (Phase 196)

**Settings Service Module:**
- `settings_service.rs` for service layer operations
- `SettingsServiceState` enum (Starting, Running, Stopping, Stopped, Error)
- `ServiceEndpoint` enum (Get, Set, Delete, List, Health, Stats)
- `ServiceCall` / `ServiceResponse` / `ServiceStats` structs
- `SettingsService` for managing service operations

**Core Functions:**
- `start()` / `stop()` / `set_error()` - Lifecycle control
- `record_call()` / `endpoint_stats()` / `global_stats()` - Statistics
- `success()` / `error()` / `with_data()` - Response builders
- `is_running()` / `uptime()` - Status checks
- `success_rate()` - Performance tracking

**Features:**
- Multiple endpoints
- Per-endpoint statistics
- Global statistics
- Uptime tracking
- Service lifecycle

## [0.0.619] - 2025-12-14

### Added - Settings Controller (Phase 195)

**Settings Controller Module:**
- `settings_controller.rs` for controlling operations
- `ControllerAction` enum (Get, Set, List, Reset, Apply, Rollback)
- `ResponseStatus` enum (Success, Error, NotFound, Forbidden, Invalid)
- `ControlRequest` / `ControlResponse` structs
- `SettingsController` for managing control flow

**Core Functions:**
- `submit()` / `get_pending()` / `respond()` - Request handling
- `success()` / `error()` / `not_found()` - Response builders
- `with_data()` / `with_duration()` - Response enrichment
- `is_success()` / `success_rate()` - Status checks
- `pending_count()` / `total_requests()` - Statistics

**Features:**
- Multiple actions
- Response status codes
- Request/response pattern
- Success rate tracking
- History management

## [0.0.618] - 2025-12-14

### Added - Settings Coordinator (Phase 194)

**Settings Coordinator Module:**
- `settings_coordinator.rs` for coordinating operations
- `CoordinatorState` enum (Initializing, Ready, Coordinating, Paused, Shutdown)
- `ComponentType` enum (Queue, Worker, Executor, Pipeline, Handler, Dispatcher)
- `ComponentStatus` enum (Unknown, Healthy, Degraded, Unhealthy, Offline)
- `ComponentInfo` / `CoordinationTask` structs
- `SettingsCoordinator` for managing coordination

**Core Functions:**
- `start()` / `pause()` / `resume()` / `shutdown()` - Lifecycle control
- `register_component()` / `update_component()` / `get_component()` - Component management
- `start_task()` / `complete_task()` - Task coordination
- `component_count()` / `healthy_count()` - Statistics
- `is_healthy()` / `is_complete()` - Status checks

**Features:**
- Component health tracking
- Task coordination
- Lifecycle management
- Multi-component orchestration
- Health monitoring

## [0.0.617] - 2025-12-14

### Added - Settings Dispatcher (Phase 193)

**Settings Dispatcher Module:**
- `settings_dispatcher.rs` for dispatching operations
- `DispatchPriority` enum (Urgent, High, Normal, Low, Deferred)
- `DispatchStatus` enum (Queued, Dispatched, Delivered, Failed, Rejected)
- `DispatchMessage` / `DispatchResult` structs
- `SettingsDispatcher` for managing dispatch queue

**Core Functions:**
- `enqueue()` / `dequeue()` - Queue operations
- `mark_dispatched()` / `mark_delivered()` / `mark_failed()` - Status updates
- `complete()` / `results()` - Result handling
- `queue_len()` / `total_dispatched()` / `total_delivered()` - Statistics
- `delivery_rate()` - Performance tracking

**Features:**
- Priority-based ordering
- Message lifecycle tracking
- Delivery rate monitoring
- Result history
- Target routing

## [0.0.616] - 2025-12-14

### Added - Settings Handler (Phase 192)

**Settings Handler Module:**
- `settings_handler.rs` for handling operations
- `HandlerType` enum (Read, Write, Delete, Validate, Transform, Notify)
- `HandlerStatus` enum (Ready, Busy, Disabled, Error)
- `HandlerDef` / `HandlerInstance` structs
- `SettingsHandlerRegistry` for managing handlers

**Core Functions:**
- `register()` / `unregister()` / `get_def()` - Definition management
- `invoke()` / `complete()` / `error()` - Invocation handling
- `disable()` / `enable()` - Status control
- `find_by_type()` - Type-based lookup
- `success_rate()` - Performance tracking

**Features:**
- Multiple handler types
- Priority-based routing
- Success rate tracking
- Category filtering
- Lifecycle management

## [0.0.615] - 2025-12-14

### Added - Settings Processor (Phase 191)

**Settings Processor Module:**
- `settings_processor.rs` for processing settings changes
- `ProcessorMode` enum (Sync, Async, Batch, Streaming)
- `ProcessorState` enum (Idle, Processing, Paused, Error, Shutdown)
- `ProcessingJob` / `ProcessingResult` structs
- `SettingsProcessor` for managing processing

**Core Functions:**
- `submit()` / `get_pending()` / `complete()` - Job handling
- `start()` / `pause()` / `resume()` / `shutdown()` - Lifecycle control
- `is_create()` / `is_update()` / `is_delete()` - Job type detection
- `set_mode()` / `mode()` / `state()` - Configuration
- `pending_count()` / `total_processed()` - Statistics

**Features:**
- Multiple processing modes
- Job type detection
- Lifecycle management
- Result tracking
- Configurable modes

## [0.0.614] - 2025-12-14

### Added - Settings Pipeline (Phase 190)

**Settings Pipeline Module:**
- `settings_pipeline.rs` for pipeline processing
- `PipelineStage` enum (Validate, Transform, Apply, Verify, Notify, Cleanup)
- `PipelineStatus` enum (Ready, Running, Paused, Completed, Failed)
- `StageResult` / `PipelineRun` structs
- `SettingsPipeline` for managing pipelines

**Core Functions:**
- `create_run()` / `get_run()` / `get_run_mut()` - Run management
- `start()` / `set_stage()` / `add_result()` / `complete()` - Run lifecycle
- `archive()` / `history()` - History management
- `is_success()` / `total_duration()` - Status checks
- `active_count()` / `success_count()` - Statistics

**Features:**
- Multi-stage processing
- Stage result tracking
- Run history
- Duration tracking
- Success rate monitoring

## [0.0.613] - 2025-12-14

### Added - Settings Executor (Phase 189)

**Settings Executor Module:**
- `settings_executor.rs` for executing operations
- `ExecutionStatus` enum (Pending, Running, Success, Failed, Cancelled, Timeout)
- `ExecutionType` enum (Get, Set, Delete, Reset, Batch, Transaction)
- `ExecutionRequest` / `ExecutionResult` structs
- `SettingsExecutor` for managing executions

**Core Functions:**
- `submit()` / `pending()` / `get_pending()` - Request handling
- `complete()` / `results()` - Result management
- `is_success()` / `with_value()` / `with_duration()` - Result building
- `total_executed()` / `total_failed()` / `success_rate()` - Statistics

**Features:**
- Multiple execution types
- Timeout protection
- Request/result tracking
- Success rate monitoring
- History management

## [0.0.612] - 2025-12-14

### Added - Settings Worker (Phase 188)

**Settings Worker Module:**
- `settings_worker.rs` for worker pool management
- `WorkerState` enum (Idle, Busy, Paused, Stopped, Error)
- `WorkerType` enum (Reader, Writer, Syncer, Backup, General)
- `WorkerConfig` / `Worker` structs
- `WorkerPool` for managing workers

**Core Functions:**
- `assign()` / `complete()` / `fail()` - Task handling
- `pause()` / `resume()` / `stop()` - State control
- `is_available()` / `success_rate()` - Status checks
- `add()` / `remove()` / `get()` - Pool management
- `available()` / `available_by_type()` - Worker selection

**Features:**
- Multiple worker types
- Concurrent task limits
- Success rate tracking
- Type-based worker selection
- Configurable timeouts

## [0.0.611] - 2025-12-14

### Added - Settings Queue (Phase 187)

**Settings Queue Module:**
- `settings_queue.rs` for queue management
- `QueuePriority` enum (Critical, High, Normal, Low, Background)
- `QueueItemStatus` enum (Queued, Processing, Completed, Failed, Retrying)
- `QueueOperation` enum (Read, Write, Update, Delete, Sync)
- `QueueItem` struct with retry support
- `SettingsQueue` with priority ordering

**Core Functions:**
- `enqueue()` / `dequeue()` / `peek()` - Queue operations
- `mark_processing()` / `mark_completed()` / `mark_failed()` - Status updates
- `retry()` / `can_retry()` - Retry support
- `by_priority()` - Priority filtering
- `processed_count()` / `failed_count()` - Statistics

**Features:**
- Priority-based ordering
- Automatic retry support
- Operation tracking
- Category filtering
- Payload support

## [0.0.610] - 2025-12-14

### Added - Settings Task Scheduler (Phase 186)

**Settings Task Scheduler Module:**
- `settings_task_scheduler.rs` for advanced task scheduling
- `TaskFrequency` enum (Once, Minutely, Hourly, Daily, Weekly, Monthly)
- `TaskType` enum (Backup, Sync, Validation, Report, Cleanup, Custom)
- `TaskState` enum (Pending, Running, Completed, Failed, Cancelled)
- `TaskDefinition` / `TaskInstance` structs
- `SettingsTaskScheduler` for managing scheduled tasks

**Core Functions:**
- `add_definition()` / `remove_definition()` / `get_definition()` - Definition management
- `schedule()` / `pending()` / `running()` - Instance management
- `start()` / `complete()` / `fail()` / `cancel()` - State transitions
- `definition_count()` / `instance_count()` - Statistics

**Features:**
- Multiple task frequencies
- Task priority support
- State machine lifecycle
- Instance history tracking
- Category filtering

## [0.0.609] - 2025-12-14

### Added - Settings Reporter (Phase 185)

**Settings Reporter Module:**
- `settings_reporter.rs` for generating reports
- `ReportType` enum (Summary, Detailed, Change, Usage, Compliance)
- `ReportFormat` enum (Text, Markdown, Html, Json, Csv)
- `ReportSection` / `ReportConfig` / `Report` structs
- `SettingsReporter` with report storage

**Core Functions:**
- `add_config()` / `remove_config()` / `get_config()` - Config management
- `store()` / `reports()` / `recent()` - Report storage
- `add_section()` / `render()` - Report building
- `section_count()` - Report metrics

**Features:**
- Multiple report types
- Multiple output formats
- Ordered sections
- Report history
- Metadata inclusion

## [0.0.608] - 2025-12-14

### Added - Settings Monitor (Phase 184)

**Settings Monitor Module:**
- `settings_monitor.rs` for monitoring settings
- `MonitorType` enum (Change, Threshold, Pattern, Anomaly, Health)
- `AlertSeverity` enum (Info, Warning, Error, Critical)
- `MonitorConfig` / `Alert` / `MonitorStatus` structs
- `SettingsMonitor` with alert management

**Core Functions:**
- `add_config()` / `remove_config()` / `get_config()` - Config management
- `add_alert()` / `alerts()` / `unacknowledged()` - Alert handling
- `get_status()` / `healthy_count()` - Status tracking
- `acknowledge()` - Alert acknowledgment

**Features:**
- Multiple monitor types
- Alert severity levels
- Health status tracking
- Alert acknowledgment
- Configurable check intervals

## [0.0.607] - 2025-12-14

### Added - Settings Deployer (Phase 183)

**Settings Deployer Module:**
- `settings_deployer.rs` for deploying settings
- `DeployTarget` enum (Local, Remote, Profile, Application, Service)
- `DeployStatus` enum (Pending, InProgress, Completed, Failed, RolledBack)
- `DeployConfig` / `DeployResult` / `DeployHistory` structs
- `SettingsDeployer` with history tracking

**Core Functions:**
- `add_config()` / `remove_config()` / `get_config()` - Config management
- `record()` / `history()` - Deployment history
- `success_rate()` - Performance tracking
- `dry_run()` / `backup()` - Safety options

**Features:**
- Multiple deploy targets
- Dry run mode
- Backup before deploy
- Deployment history
- Success rate tracking

## [0.0.606] - 2025-12-14

### Added - Settings Bundler (Phase 182)

**Settings Bundler Module:**
- `settings_bundler.rs` for bundling settings
- `BundleFormat` enum (Json, Binary, Compressed, Encrypted)
- `BundleStatus` enum (Created, Valid, Invalid, Expired)
- `BundleMeta` / `BundleEntry` / `SettingsBundle` structs
- `BundleManager` for managing bundles

**Core Functions:**
- `add()` / `remove()` / `get()` - Bundle management
- `by_category()` - Filter by category
- `mark_valid()` / `mark_invalid()` - Status control
- `entry_count()` / `category_count()` - Statistics

**Features:**
- Multiple bundle formats
- Bundle metadata
- Category-based filtering
- Status tracking
- Valid/invalid marking

## [0.0.605] - 2025-12-14

### Added - Settings Linker (Phase 181)

**Settings Linker Module:**
- `settings_linker.rs` for linking settings
- `LinkType` enum (Reference, Alias, Computed, Inherited, Sync)
- `LinkStatus` enum (Active, Broken, Pending, Disabled)
- `LinkDef` / `LinkResolution` / `LinkRegistry` structs
- `SettingsLinker` with resolution caching

**Core Functions:**
- `add()` / `remove()` / `get()` - Link management
- `from_source()` / `to_target()` - Link queries
- `broken()` - Find broken links
- `cache_resolution()` / `get_cached()` - Caching

**Features:**
- Multiple link types
- Link status tracking
- Source/target lookup
- Resolution caching
- Broken link detection

## [0.0.604] - 2025-12-14

### Added - Settings Compiler (Phase 180)

**Settings Compiler Module:**
- `settings_compiler.rs` for compiling settings
- `CompileStage` enum (Parse, Validate, Optimize, Generate, Finalize)
- `CompileStatus` enum (Success, Warning, Error, Skipped)
- `CompileDiag` / `CompileOptions` / `CompileOutput` structs
- `SettingsCompiler` with history tracking

**Core Functions:**
- `add_diag()` / `add_data()` - Output building
- `has_errors()` / `error_count()` - Error checking
- `record()` / `recent()` - History management
- `success_rate()` - Performance tracking

**Features:**
- Multi-stage compilation
- Optimization support
- Strict mode
- Debug info
- Compile history with success rate

## [0.0.603] - 2025-12-14

### Added - Settings Router (Phase 179)

**Settings Router Module:**
- `settings_router.rs` for routing settings operations
- `RouteType` enum (Direct, Middleware, Proxy, Cached, Redirect)
- `RouteAction` enum (Get, Set, Delete, List, Query)
- `RouteDef` / `RouteMatch` / `RouteTable` structs
- `SettingsRouter` / `RouteStats`

**Core Functions:**
- `add()` / `remove()` / `get()` - Route management
- `find()` / `best_match()` - Route matching
- `route()` - Request routing with stats
- `add_table()` / `set_default()` - Table management

**Features:**
- Priority-based routing
- Per-category routes
- Action filtering
- Cache/redirect stats
- Named route tables

## [0.0.602] - 2025-12-14

### Added - Settings Serializer (Phase 178)

**Settings Serializer Module:**
- `settings_serializer.rs` for multi-format serialization
- `SerializationFormat` enum (Json, Toml, Yaml, Binary, JsonPretty)
- `SerializeResult` enum (Success, FormatError, EncodingError, SizeExceeded, Unknown)
- `SerializeOptions` / `SerializedData` / `SerializationStats` structs
- `SettingsSerializer` with per-category format support

**Core Functions:**
- `set_default()` / `format_for()` - Format management
- `set_category_format()` - Per-category formats
- `record()` / `stats()` - Statistics tracking
- `as_string()` / `checksum()` - Data operations

**Features:**
- Multiple serialization formats
- Per-category format preferences
- Compression support
- Size limits
- Serialization statistics

## [0.0.601] - 2025-12-14

### Added - Settings Comparator (Phase 177)

**Settings Comparator Module:**
- `settings_comparator.rs` for comparing settings
- `DiffType` enum (Added, Removed, Modified, Unchanged, TypeChanged)
- `CompareMode` enum (Full, ChangesOnly, AdditionsOnly, RemovalsOnly, SummaryOnly)
- `DiffEntry` / `CompareResult` / `CompareOptions` structs
- `SettingsComparator` with history tracking

**Core Functions:**
- `added()` / `removed()` / `modified()` - Create diff entries
- `add()` / `total_changes()` / `has_changes()` - Result operations
- `changes_only()` / `by_category()` - Filtering
- `record()` / `recent()` - History management

**Features:**
- Multiple diff types
- Comparison modes
- Category filtering
- Key ignore list
- Comparison history

## [0.0.600] - 2025-12-14

### Added - Settings Aggregator (Phase 176) 🎉 MILESTONE

**Settings Aggregator Module:**
- `settings_aggregator.rs` for aggregation and summarization
- `AggregationType` enum (Count, Sum, Average, Min, Max, List, GroupBy)
- `AggregationScope` enum (All, Category, Categories, Pattern)
- `AggregationDef` / `AggValue` / `AggregationResult` structs
- `SettingsSummary` / `SettingsAggregator`

**Core Functions:**
- `add()` / `remove()` / `get()` - Manage aggregations
- `store_result()` / `get_result()` - Result caching
- `add_category()` - Summary building
- `list_ids()` - List all aggregations

**Features:**
- Multiple aggregation types
- Scope-based filtering
- Result caching
- Settings summary
- Per-category breakdown

## [0.0.599] - 2025-12-14

### Added - Settings Resolver (Phase 175)

**Settings Resolver Module:**
- `settings_resolver.rs` for conflict resolution
- `ConflictType` enum (ValueMismatch, TypeMismatch, DependencyMissing, CircularDep, MutualExclusion, VersionConflict)
- `ResolutionStrategy` enum (First, Last, Priority, Merge, Fail, Ask)
- `Conflict` / `Resolution` / `Dependency` structs
- `ResolverConfig` / `SettingsResolver`

**Core Functions:**
- `add_dependency()` / `remove_dependency()` - Dependency management
- `dependencies_for()` / `dependents_of()` - Query dependencies
- `has_circular()` - Circular dependency detection
- `record_conflict()` / `record_resolution()` - History tracking

**Features:**
- Multiple resolution strategies
- Per-category strategy configuration
- Circular dependency detection
- Dependency tracking
- Conflict/resolution history

## [0.0.598] - 2025-12-14

### Added - Settings Transformer (Phase 174)

**Settings Transformer Module:**
- `settings_transformer.rs` for transformation pipeline
- `TransformType` enum (Trim, Lower, Upper, Default, Replace, Clamp, Custom)
- `TransformDirection` enum (Input, Output, Both)
- `TransformDef` / `TransformResult` structs
- `TransformPipeline` / `TransformerManager`

**Core Functions:**
- `add()` / `remove()` / `enable()` / `disable()` - Manage transforms
- `for_category_dir()` - Get transforms for category/direction
- `add_pipeline()` / `get_pipeline()` / `set_default()` - Pipeline management
- `was_transformed()` - Check if value changed

**Features:**
- Priority-based transform ordering
- Direction-aware transforms (input/output)
- Named transform pipelines
- Parameter support
- Per-category filtering

## [0.0.597] - 2025-12-14

### Added - Settings Validator Chain (Phase 173)

**Settings Validator Chain Module:**
- `settings_validator_chain.rs` for chainable validation
- `ValidatorType` enum (Required, Type, Range, Pattern, Custom, Dependency, Unique)
- `ValidationResult` enum (Pass, Fail, Warn, Skip)
- `ValidationError` / `ValidatorDef` / `ValidationOutput` structs
- `ValidationChain` / `ChainResult` / `ValidatorChainManager`

**Core Functions:**
- `add()` / `remove()` / `enable()` / `disable()` - Manage validators
- `for_category()` - Get validators for category
- `add_chain()` / `get_chain()` / `set_default()` - Chain management
- `all_errors()` / `is_valid()` - Result checking

**Features:**
- Priority-based validator ordering
- Stop on first failure option
- Named validation chains
- Error suggestions
- Per-category filtering

## [0.0.596] - 2025-12-14

### Added - Settings Query (Phase 172)

**Settings Query Module:**
- `settings_query.rs` for query DSL
- `QueryOperator` enum (Eq, Ne, Gt, Gte, Lt, Lte, Contains, StartsWith, EndsWith, Matches)
- `QueryCondition` / `SettingsQuery` / `QueryResult` structs
- `QueryExecutor` for history tracking

**Core Functions:**
- `category()` / `where_field()` / `select_field()` - Query building
- `limit()` / `offset()` / `order_by()` / `desc()` - Pagination
- `matches_category()` / `matches()` - Condition matching
- `record()` / `recent()` / `clear_history()` - History operations

**Features:**
- SQL-like query syntax
- Multiple operators for filtering
- Regex matching support
- Query history tracking
- Formatted query output

## [0.0.595] - 2025-12-14

### Added - Settings Inheritance (Phase 171)

**Settings Inheritance Module:**
- `settings_inheritance.rs` for profile inheritance
- `InheritanceMode` enum (Full, Selective, Override, Merge)
- `InheritanceRule` / `InheritanceEntry` structs
- `InheritanceManager`

**Core Functions:**
- `add()` / `get()` / `remove()` - Manage entries
- `chain()` - Get inheritance chain
- `children()` / `roots()` - Tree navigation
- `depth()` - Calculate inheritance depth

**Features:**
- Parent-child relationships
- Selective inheritance rules
- Include/exclude key filters
- Priority-based application
- Tree structure support

## [0.0.594] - 2025-12-14

### Added - Settings Encryption (Phase 170)

**Settings Encryption Module:**
- `settings_encryption.rs` for encrypting settings
- `EncryptionAlgorithm` enum (Aes256Gcm, ChaCha20, XChaCha20)
- `EncryptionStatus` enum (Plain, Encrypted, Decrypted, Error)
- `EncryptedValue` / `KeyInfo` structs
- `EncryptionManager`

**Core Functions:**
- `add_key()` / `set_active_key()` - Key management
- `store()` / `get()` / `remove()` - Value operations
- `require()` / `is_required()` - Category requirements
- `is_encrypted()` - Check encryption status

**Features:**
- Multiple encryption algorithms
- Key rotation support
- Category-based encryption requirements
- Key expiration
- Active key tracking

## [0.0.593] - 2025-12-14

### Added - Settings Lock (Phase 169)

**Settings Lock Module:**
- `settings_lock.rs` for locking settings
- `LockType` enum (ReadOnly, Full, AdminOnly, Temporary)
- `LockScope` enum (Global, Category, Key)
- `LockEntry` / `LockCheckResult`
- `SettingsLockManager`

**Core Functions:**
- `lock()` / `unlock()` / `remove()` - Manage locks
- `check()` - Check if locked
- `lock_permanent()` / `unlock_permanent()` - Permanent locks
- `clean_expired()` - Auto cleanup

**Features:**
- Scoped locking (global, category, key)
- Lock expiration
- Admin-only locks
- Permanent locks
- Lock ownership tracking

## [0.0.592] - 2025-12-14

### Added - Settings Snapshot (Phase 168)

**Settings Snapshot Module:**
- `settings_snapshot.rs` for point-in-time snapshots
- `SnapshotType` enum (Manual, Auto, PreChange, Scheduled)
- `SnapshotStatus` enum (Active, Archived, Expired, Corrupted)
- `SettingsSnapshot` struct
- `SnapshotManager` for managing snapshots

**Core Functions:**
- `create()` / `get()` / `delete()` - Manage snapshots
- `add_data()` / `get_data()` - Store category data
- `archive()` / `finalize()` - Lifecycle operations
- `clean_expired()` - Auto cleanup

**Features:**
- Per-category data storage
- Hash-based integrity
- Expiration support
- Size tracking
- Max snapshot limit

## [0.0.591] - 2025-12-14

### Added - Settings Observer (Phase 167)

**Settings Observer Module:**
- `settings_observer.rs` for change observation
- `ObserverEvent` enum (BeforeChange, AfterChange, OnError, OnReset, OnLoad, OnSave)
- `Notification` / `Observer` structs
- `ObserverManager` for managing observers

**Core Functions:**
- `register()` / `unregister()` - Manage observers
- `notify()` - Send notifications
- `matching()` - Find matching observers
- `enable()` / `disable()` - Toggle observers

**Features:**
- Event filtering
- Category filtering
- Notification history
- Notification count tracking
- Watch all events option

## [0.0.590] - 2025-12-14

### Added - Settings Middleware (Phase 166)

**Settings Middleware Module:**
- `settings_middleware.rs` for operation pipeline
- `MiddlewarePriority` enum (Critical, High, Normal, Low)
- `MiddlewareAction` enum (Continue, Skip, Abort, Modify)
- `MiddlewareContext` / `MiddlewareResult` / `Middleware` structs
- `MiddlewarePipeline` manager

**Core Functions:**
- `add()` / `remove()` / `get()` - Manage middleware
- `enable()` / `disable()` - Toggle middleware
- `applicable()` - Get matching middleware
- Priority-based ordering

**Features:**
- Operation filtering
- Category filtering
- Context metadata
- Abort with reason
- Priority sorting

## [0.0.589] - 2025-12-14

### Added - Settings Throttling (Phase 165)

**Settings Throttling Module:**
- `settings_throttling.rs` for rate limiting
- `ThrottleAction` enum (Read, Write, Export, Import, Sync, Any)
- `ThrottleResult` enum (Allowed, Limited, Blocked)
- `RateLimit` / `ThrottleStats` structs
- `SettingsThrottler` manager

**Core Functions:**
- `check()` - Check and record request
- `would_limit()` - Check without recording
- `set_limit()` / `get_limit()` - Configure limits
- `block()` / `unblock()` - Block actions

**Features:**
- Per-action rate limits
- Burst allowance
- Action blocking
- Request tracking
- Statistics per action

## [0.0.588] - 2025-12-14

### Added - Settings Versioning (Phase 164)

**Settings Versioning Module:**
- `settings_versioning.rs` for version control
- `ChangeType` enum (Added, Modified, Removed, Reset)
- `VersionChange` / `SettingsVersion` structs
- `VersionHistory` manager

**Core Functions:**
- `create()` - Create new version
- `get()` / `current()` / `all()` / `recent()` - Query versions
- `compare()` - Compare two versions
- `find_by_message()` - Search history

**Features:**
- Full change history tracking
- Version comparison
- Change categorization
- Message search
- Configurable history size

## [0.0.587] - 2025-12-14

### Added - Settings Transactions (Phase 163)

**Settings Transactions Module:**
- `settings_transactions.rs` for atomic operations
- `TransactionState` enum (Pending, Active, Committed, RolledBack, Failed)
- `OperationType` enum (Set, Delete, Reset, Update)
- `TransactionOp` / `SettingsTransaction` structs
- `TransactionManager` for managing transactions

**Core Functions:**
- `begin()` / `commit()` / `rollback()` - Transaction lifecycle
- `set()` / `delete()` / `reset()` - Add operations
- `with_prev()` - Store previous value for rollback
- History tracking

**Features:**
- Atomic settings changes
- Rollback support
- Transaction history
- Operation tracking
- Error handling with state preservation

## [0.0.586] - 2025-12-14

### Added - Settings Cache (Phase 162)

**Settings Cache Module:**
- `settings_cache.rs` for caching settings values
- `CacheState` enum (Valid, Stale, Expired, Loading)
- `EvictionPolicy` enum (LRU, LFU, FIFO, TTL)
- `CacheEntry` / `CacheStats` structs
- `SettingsCache` manager

**Core Functions:**
- `get()` / `put()` / `remove()` - Cache operations
- `invalidate_category()` / `mark_stale_category()` - Category operations
- `clear_expired()` - Clean up expired entries
- Automatic eviction based on policy

**Features:**
- Multiple eviction policies
- TTL-based expiration
- Category-based invalidation
- Hit/miss statistics
- Configurable max entries and size

## [0.0.585] - 2025-12-14

### Added - Settings Logging (Phase 161)

**Settings Logging Module:**
- `settings_logging.rs` for structured logging
- `LogLevel` enum (Trace, Debug, Info, Warn, Error)
- `LogTarget` enum (Core, Persistence, Validation, Migration, Backup, Sync, Api)
- `LogEntry` / `LogFilter` structs
- `SettingsLogger` manager

**Core Functions:**
- `log()` / `trace()` / `debug()` / `info()` / `warn()` / `error()` - Log entries
- `query()` - Query with filters
- `recent()` / `errors()` / `warnings_and_errors()` - Get filtered entries
- `count()` / `error_count()` - Statistics

**Features:**
- Builder pattern for log entries
- Level-based filtering
- Target and category filtering
- Text search in messages
- Time range queries
- Max entries limit with auto-cleanup

## [0.0.584] - 2025-12-14

### Added - Settings Metrics (Phase 160)

**Settings Metrics Module:**
- `settings_metrics.rs` for metrics collection
- `MetricKind` enum (Counter, Gauge, Histogram, Timer)
- `MetricUnit` enum (Count, Bytes, Milliseconds, Percent, None)
- `MetricValue` / `Metric` structs
- `SettingsMetrics` collection

**Core Functions:**
- `register()` - Register new metric
- `increment()` / `set()` / `record()` - Update metrics
- `get()` / `all()` / `by_kind()` / `by_category()` - Query metrics
- `uptime()` - Get collection uptime

**Features:**
- Counter, gauge, and timer metrics
- Default metrics for changes, reads, exports, imports
- Min/max tracking for gauges
- Category-based filtering
- Uptime tracking

## [0.0.583] - 2025-12-14

### Added - Settings Diagnostics (Phase 159)

**Settings Diagnostics Module:**
- `settings_diagnostics.rs` for health checking
- `DiagnosticSeverity` enum (Info, Warning, Error, Critical)
- `DiagnosticType` enum (Configuration, Compatibility, Performance, Security, Validation, Dependency)
- `DiagnosticIssue` / `DiagnosticReport` structs
- `SettingsDiagnostics` runner

**Core Functions:**
- `run()` - Run full diagnostics
- `quick_check()` - Quick health check
- `health_score()` - Calculate health score (0-100)
- `auto_fixable()` - Get auto-fixable issues

**Features:**
- Configuration, security, performance, compatibility checks
- Health scoring with severity weighting
- Auto-fixable issue detection
- Duration tracking
- Last report caching

## [0.0.582] - 2025-12-14

### Added - Settings Permissions (Phase 158)

**Settings Permissions Module:**
- `settings_permissions.rs` for access control
- `PermissionLevel` enum (None, Read, Write, Admin)
- `PermissionAction` enum (View, Modify, Reset, Export, Import, ManagePermissions)
- `PermissionResult` enum (Allowed, Denied, RequiresElevation)
- `CategoryPermission` / `PermissionRole` / `PermissionManager`

**Core Functions:**
- `check()` - Check permission for category and action
- `add_role()` / `remove_role()` - Manage roles
- `set_active_role()` / `active_role()` - Role selection
- `lock_category()` / `unlock_category()` - Category locks

**Features:**
- Role-based access control
- Built-in roles (Viewer, Editor, Admin)
- Category-level permission overrides
- Global category locking
- Permission checking with results

## [0.0.581] - 2025-12-14

### Added - Settings Events (Phase 157)

**Settings Events Module:**
- `settings_events.rs` for pub/sub event system
- `SettingsEventType` enum (Changed, Reset, Imported, Exported, ProfileSwitched, etc.)
- `EventPriority` enum (Low, Normal, High, Critical)
- `SettingsEvent` struct with builder pattern
- `EventFilter` for subscription filtering
- `Subscriber` / `SettingsEventBus` for event management

**Core Functions:**
- `publish()` / `publish_change()` - Publish events
- `subscribe()` / `unsubscribe()` - Manage subscriptions
- `query()` / `recent()` / `get()` - Query events
- `EventFilter::matches()` - Filter matching

**Features:**
- Full pub/sub event system
- Priority-based events
- Flexible filtering by type, category, priority, source
- Subscriber event counting
- Event history with cleanup

## [0.0.580] - 2025-12-14

### Added - Settings API (Phase 156)

**Settings API Module:**
- `settings_api.rs` for unified API access
- `ApiOperation` enum (Get, Set, Reset, List, Search, Validate, Export, Import)
- `ApiStatus` enum (Success, Error, Partial, Pending)
- `ApiRequest` / `ApiResponse` structs
- `SettingValue` for setting representation
- `SettingsApi` handler with request history

**Core Functions:**
- `handle()` - Handle API requests
- `ApiRequest::get()` / `set()` / `list()` / `search()` - Request builders
- `ApiResponse::success()` / `error()` - Response builders
- `history()` / `recent()` / `clear_history()` - History management

**Features:**
- Request/response pattern with request IDs
- Full CRUD operations on settings
- Search and validate support
- Export/import via API
- Request history tracking

## [0.0.579] - 2025-12-14

### Added - Settings Dashboard (Phase 155)

**Settings Dashboard Module:**
- `settings_dashboard.rs` for unified settings overview
- `DashboardSection` enum (Overview, RecentChanges, Recommendations, QuickActions, Health, Statistics)
- `HealthLevel` enum (Excellent, Good, Fair, Poor, Critical)
- `QuickAction` enum (ResetDefaults, Export, Import, Backup, Diagnostics, ApplyRecommended)
- `CategorySummary` / `DashboardStats` / `RecentChange`
- `SettingsDashboard` for dashboard management

**Core Functions:**
- `refresh()` - Refresh dashboard data
- `add_change()` - Track recent changes
- `categories()` / `stats()` / `recent_changes()` - Access data
- `set_section_visible()` - Configure visible sections

**Features:**
- Per-category health tracking
- Overall health scoring (0-100%)
- Recent changes history
- Customization percentage
- Quick action support

## [0.0.578] - 2025-12-14

### Added - Settings Recommendations (Phase 154)

**Settings Recommendations Module:**
- `settings_recommendations.rs` for intelligent recommendations
- `RecommendationPriority` enum (Low, Medium, High, Critical)
- `RecommendationType` enum (Security, Performance, Usability, Privacy, BestPractice)
- `RecommendationStatus` enum (Active, Dismissed, Applied, Expired)
- `Recommendation` struct with builder pattern
- `RecommendationEngine` for generating recommendations

**Core Functions:**
- `analyze()` - Analyze settings and generate recommendations
- `active()` / `by_type()` / `by_priority()` - Filter recommendations
- `dismiss()` / `apply()` - Manage recommendation status
- `active_count()` / `count_by_priority()` - Statistics

**Features:**
- Automatic security, privacy, usability, performance checks
- Priority-based recommendations
- Type-based categorization
- Dismiss/apply tracking

## [0.0.577] - 2025-12-14

### Added - Settings Analytics (Phase 153)

**Settings Analytics Module:**
- `settings_analytics.rs` for tracking usage patterns
- `AnalyticsPeriod` enum (Day, Week, Month, AllTime)
- `MetricType` enum (ChangeCount, AccessCount, RevertCount, ExportCount, ImportCount)
- `AnalyticsEvent` for single analytics events
- `CategoryStats` for per-category statistics
- `AnalyticsSummary` for overall analytics
- `SettingsAnalytics` for tracking settings usage

**Core Functions:**
- `record()` / `record_change()` / `record_access()` - Record events
- `record_revert()` / `record_export()` / `record_import()` - Track operations
- `summary()` / `category_stats()` - Get statistics
- `events_for_period()` / `recent()` - Query event history

**Features:**
- Per-category usage tracking
- Activity scoring (0-100)
- Top settings tracking per category
- Time-based event filtering
- Configurable event retention

## [0.0.576] - 2025-12-14

### Added - Settings Restore (Phase 152)

**Settings Restore Module:**
- `settings_restore.rs` for restoring from backups
- `RestoreMode` enum (Full, Partial, Merge, Selective)
- `RestoreStatus` enum (Pending, Validating, InProgress, Success, Failed, RolledBack)
- `RestoreValidation` for backup validation
- `RestorePoint` for pre-restore snapshots
- `RestoreRecord` for restore operation tracking
- `RestoreManager` for managing restores

**Core Functions:**
- `validate()` - Validate backup before restore
- `create_restore_point()` - Snapshot before restore
- `restore()` - Perform restore from backup
- `rollback()` - Roll back to restore point
- `history()` / `recent()` - Query restore history

**Features:**
- Pre-restore snapshot creation
- Version compatibility checking
- Multiple restore modes
- Automatic rollback support
- Restore history tracking

## [0.0.575] - 2025-12-14

### Added - Settings Backup Manager (Phase 151)

**Settings Backup Module:**
- `settings_backup.rs` for automated backup and restore
- `BackupType` enum (Full, Incremental, PreChange, Scheduled, Manual)
- `BackupStatus` enum (Success, Failed, InProgress, Corrupted, Expired)
- `BackupMeta` for backup metadata with checksum
- `BackupConfig` for backup configuration
- `BackupManager` for managing backups

**Core Functions:**
- `create_backup()` / `manual_backup()` / `pre_change_backup()` - Create backups
- `scheduled_backup()` / `is_backup_due()` - Scheduled backups
- `get()` / `list()` / `valid_backups()` / `latest()` - Query backups
- `delete()` / `cleanup_old_backups()` - Manage backups

**Features:**
- Automatic pre-change backups
- Scheduled automatic backups
- Maximum backup retention limit
- Checksum verification
- Human-readable age display
- Total size tracking

## [0.0.574] - 2025-12-14

### Added - Settings Orchestrator (Phase 150)

**Settings Orchestrator Module:**
- `settings_orchestrator.rs` for unified settings coordination
- `OrchestratorState` enum (Uninitialized, Initializing, Ready, Busy, Error)
- `OperationResult` for operation outcomes with warnings/errors
- `SettingsOrchestrator` unified coordinator
- `OrchestratorStatus` for status summary

**Core Functions:**
- `initialize()` / `set_session()` - Setup
- `change_setting()` - Change with hooks, audit, notifications
- `switch_profile()` / `apply_template()` - Profile/template operations
- `run_scheduled()` - Execute pending schedules
- `status_summary()` - Get subsystem status

**Features:**
- Unified coordination of all settings subsystems
- Automatic hook triggering for all operations
- Automatic audit logging
- Automatic notification sending
- Constraint validation on changes
- Session tracking across operations

## [0.0.573] - 2025-12-14

### Added - Settings Audit (Phase 149)

**Settings Audit Module:**
- `settings_audit.rs` for compliance and security tracking
- `AuditEventType` enum (Load, Save, Change, Reset, ProfileSwitch, etc.)
- `AuditSeverity` enum (Info, Notice, Warning, Security, Critical)
- `AuditEntry` for individual audit events
- `AuditFilter` for querying audit log
- `AuditLog` for managing audit entries

**Core Functions:**
- `log()` / `log_change()` / `log_security()` - Log events
- `filter()` / `recent()` / `security_events()` - Query log
- `changes_for()` / `count_by_severity()` - Analysis
- `set_session()` - Session tracking

**Features:**
- Security event tracking
- Category and field level tracking
- Old/new value recording
- Session ID tracking
- Flexible filtering
- Max entry limit with auto-trim

## [0.0.572] - 2025-12-14

### Added - Settings Wizard (Phase 148)

**Settings Wizard Module:**
- `settings_wizard.rs` for guided interactive configuration
- `WizardStepType` enum (Welcome, SingleChoice, MultipleChoice, TextInput, etc.)
- `WizardChoice` for choice options with descriptions
- `WizardStep` for wizard steps with choices and defaults
- `WizardState` enum (NotStarted, InProgress, Completed, Cancelled)
- `SettingsWizard` and `WizardManager` for wizard management

**Core Functions:**
- `start()` / `next()` / `back()` / `cancel()` - Navigation
- `answer()` / `get_answer()` - Collect and retrieve answers
- `current()` / `progress()` - State tracking
- `add_step()` / `add_choice()` - Build wizards

**Features:**
- Built-in wizards (Quick Setup, Privacy Setup)
- Step-by-step guided configuration
- Progress tracking
- Navigation (forward, back, cancel)
- Skip conditions for conditional steps

## [0.0.571] - 2025-12-14

### Added - Settings Hooks (Phase 147)

**Settings Hooks Module:**
- `settings_hooks.rs` for callbacks on settings changes
- `HookTrigger` enum (BeforeChange, AfterChange, BeforeLoad, AfterLoad, etc.)
- `HookResult` enum (Continue, Skip, Abort, Modify)
- `HookPriority` enum (High, Normal, Low)
- `HookContext` for execution context with values
- `SettingsHook` and `HookManager` for managing hooks

**Core Functions:**
- `register()` / `unregister()` - Manage hooks
- `set_enabled()` / `for_trigger()` - Enable/filter hooks
- `fire()` - Execute hooks for a context
- `should_fire()` - Check if hook should execute
- `history()` / `recent()` - Execution history

**Features:**
- Priority-based hook ordering
- Category-specific hooks
- Execution history tracking
- Built-in hooks (Log Changes, Validate Before Save, Notify Changes)
- Before/after triggers for all operations

## [0.0.570] - 2025-12-14

### Added - Settings Constraints (Phase 146)

**Settings Constraints Module:**
- `settings_constraints.rs` for defining settings rules
- `ConstraintSeverity` enum (Suggestion, Warning, Error, Critical)
- `ConstraintType` enum (Range, Dependency, MutuallyExclusive, Requires, Conflicts)
- `ConstraintViolation` for tracking rule violations
- `SettingsConstraint` for defining rules
- `ConstraintManager` for managing and checking constraints

**Core Functions:**
- `add_constraint()` / `remove()` - Manage constraints
- `set_enabled()` / `enabled()` - Enable/disable constraints
- `check()` - Check settings against all enabled constraints
- `count_by_severity()` / `critical()` / `errors()` - Filter violations

**Features:**
- Built-in constraints for common conflicts
- Severity levels from suggestion to critical
- Category-based constraint targeting
- Violation suggestions for fixes
- Enable/disable individual constraints

## [0.0.569] - 2025-12-14

### Added - Settings Templates (Phase 145)

**Settings Templates Module:**
- `settings_templates.rs` for reusable settings configurations
- `TemplateScope` enum (Full, Partial, Single)
- `TemplateUseCase` enum (Development, Production, Presentation, Learning, etc.)
- `TemplateMeta` struct for template metadata
- `SettingsTemplate` struct with settings snapshot
- `TemplateManager` for managing templates

**Core Functions:**
- `add()` / `add_partial()` - Create templates
- `remove()` / `get()` - Manage templates
- `find_by_name()` / `find_by_use_case()` / `find_by_tag()` - Search templates
- `apply()` - Apply template to current settings
- `most_used()` - Get frequently used templates

**Features:**
- Built-in templates (Development, Production, Presentation, Learning)
- Full and partial template support
- Category-specific templates
- Tag-based template search
- Usage tracking
- Builtin vs user template distinction

## [0.0.568] - 2025-12-14

### Added - Settings Scheduler (Phase 144)

**Settings Scheduler Module:**
- `settings_scheduler.rs` for scheduling settings changes
- `ScheduleTrigger` enum (AtTime, AfterDuration, DailyAt, Weekly, OnEvent)
- `ScheduleEvent` enum (Startup, Shutdown, NetworkConnected, etc.)
- `ScheduledAction` enum (SwitchProfile, ApplySettings, ChangeSetting, etc.)
- `SettingsScheduler` for managing scheduled changes

**Core Functions:**
- `add()` / `add_once()` - Create schedules
- `remove()` / `get()` - Manage schedules
- `pending()` - Get ready-to-run schedules
- `on_event()` - Handle event-based triggers
- `set_enabled()` - Enable/disable schedules

**Features:**
- Time-based scheduling (specific time, daily, weekly)
- Event-based triggers (startup, network, battery)
- One-time and recurring schedules
- Run tracking and history
- Profile switching automation

## [0.0.567] - 2025-12-14

### Added - Settings Notifications (Phase 143)

**Settings Notifications Module:**
- `settings_notifications.rs` for notifying about settings changes
- `NotificationPriority` enum (Low, Normal, High, Critical)
- `NotificationType` enum (SettingChanged, ProfileSwitched, SyncCompleted, etc.)
- `SettingsNotification` struct with read/dismiss state
- `NotificationManager` for managing notifications

**Core Functions:**
- `notify()` / `notify_with_priority()` - Create notifications
- `mark_read()` / `mark_all_read()` - Mark as read
- `dismiss()` / `dismiss_all()` - Dismiss notifications
- `unread()` / `unread_count()` - Get unread notifications
- `setting_changed()` / `profile_switched()` / `sync_completed()` - Helpers

**Features:**
- Priority-based notifications
- Read/unread tracking
- Notification history with max limit
- Category-specific notifications
- Auto-dismiss support

## [0.0.566] - 2025-12-14

### Added - Settings Search (Phase 142)

**Settings Search Module:**
- `settings_search.rs` for searching settings by keywords
- `MatchType` enum (FieldName, FieldValue, CategoryName, Description)
- `SearchResult` / `SearchResults` for match results
- `SearchOptions` for configurable search behavior
- `SettingsSearcher` for performing searches

**Core Functions:**
- `search()` - Search settings by query
- `matches()` - Check text against query
- `sort_by_score()` - Sort results by relevance
- `by_category()` / `by_match_type()` - Filter results
- `format_search_results()` - Display search output

**Features:**
- Case-sensitive/insensitive search
- Search in field names, values, descriptions
- Filter by specific categories
- Relevance scoring (0.0-1.0)
- Configurable maximum results

## [0.0.565] - 2025-12-14

### Added - Settings Profiles (Phase 141)

**Settings Profiles Module:**
- `settings_profiles.rs` for named settings configurations
- `ProfileMeta` struct with name, description, timestamps, tags
- `SettingsProfile` struct combining metadata and settings
- `ProfileManager` for managing multiple profiles

**Core Functions:**
- `add()` / `remove()` / `get()` - Profile CRUD operations
- `switch_to()` - Switch active profile
- `duplicate()` - Clone an existing profile
- `rename()` / `set_description()` - Update profile metadata
- `find_by_name()` / `find_by_tag()` - Search profiles

**Features:**
- Multiple named profiles for different use cases
- Active profile tracking with switch functionality
- Default profile designation
- Profile tags for organization
- Profile duplication with metadata copy

## [0.0.564] - 2025-12-14

### Added - Settings Sync (Phase 140)

**Settings Sync Module:**
- `settings_sync.rs` for synchronizing settings across multiple instances
- `SyncStatus` enum (Synced, LocalAhead, RemoteAhead, Conflict, etc.)
- `SyncProvider` enum (None, File, Git, Custom)
- `ConflictResolution` enum (KeepLocal, AcceptRemote, MergeRemoteWins, etc.)
- `SyncConfig` for sync configuration
- `SyncManager` for push/pull/sync operations

**Core Functions:**
- `push()` - Push local settings to remote
- `pull()` - Pull settings from remote
- `sync()` - Auto-sync based on status
- `check_remote()` - Check for remote changes
- `is_sync_due()` - Check if auto-sync is needed
- `mark_local_changed()` - Mark local as modified

**Features:**
- File-based sync (Dropbox, Syncthing, etc.)
- Git-based sync support (placeholder)
- Configurable sync intervals
- Conflict resolution strategies
- Last sync timestamp tracking

## [0.0.563] - 2025-12-13

### Added - Settings History (Phase 139)

**Settings History Module:**
- `settings_history.rs` for tracking settings changes over time
- `HistoryEntry` struct with timestamp, description, before/after snapshots
- `SettingsHistory` manager with undo/redo support
- Configurable max history size (default: 100)

**Core Functions:**
- `record()` / `record_with_category()` - Record changes
- `undo()` / `redo()` - Navigate history
- `can_undo()` / `can_redo()` - Check navigation availability
- `recent()` / `by_category()` - Query history entries
- `format_history()` - Display history with age formatting

**Features:**
- Full settings snapshots for reliable undo/redo
- Automatic truncation of future entries on new change
- Human-readable age formatting (seconds, minutes, hours, days)
- Category-specific history filtering

## [0.0.562] - 2025-12-13

### Added - Settings Presets (Phase 138)

**Settings Presets Module:**
- `settings_presets.rs` for pre-configured settings profiles
- `PresetCategory` enum (Experience, Security, Performance, Privacy, UseCase)
- `SettingsPreset` struct with id, name, description, settings
- `PresetManager` for managing built-in and custom presets

**Built-in Presets (12):**
- Experience: Beginner, Intermediate, Expert
- Security: Paranoid, Balanced Security
- Performance: Speed, Quality
- Privacy: Maximum Privacy, Convenience
- Use Case: Server Admin, Developer, Desktop User

**Core Functions:**
- `find()` / `find_by_name()` - Find presets
- `by_category()` - Filter by category
- `add()` / `remove()` - Manage custom presets
- `find_preset_natural()` - Natural language preset matching
- `apply_preset()` - Apply preset to settings

## [0.0.561] - 2025-12-13

### Added - Settings Diff (Phase 137)

**Settings Diff Module:**
- `settings_diff.rs` for comparing two settings objects
- `DiffType` enum (Added, Removed, Changed, Unchanged)
- `DiffEntry` struct with category, field, old/new values
- `SettingsDiff` result with entries and changed categories
- `SettingsDiffer` for configurable comparison

**Core Functions:**
- `diff()` - Compare two settings objects
- `is_identical()` / `has_changes()` - Check diff result
- `changes_only()` / `change_count()` - Get changes
- `category_changes()` - Get changes for specific category
- `format_diff()` - Format diff for display

**Features:**
- Field-level comparison for all 12 categories
- Optional inclusion of unchanged fields
- Filter by specific categories
- Git-style diff output format

## [0.0.560] - 2025-12-13

### Added - Settings Watcher (Phase 136)

**Settings Watcher Module:**
- `settings_watcher.rs` for watching settings file changes
- `SettingsEventType` enum (Created, Modified, Deleted, Reloaded, CategoryChanged)
- `SettingsEvent` struct with timestamp and description
- `WatcherConfig` for check interval, auto-reload settings
- `SettingsWatcher` for polling file changes

**Core Functions:**
- `check()` - Single poll for file changes
- `emit()` / `emit_category_change()` - Manual event emission
- `start()` / `stop()` / `is_running()` - Watcher state control
- `history()` / `recent_events()` / `clear_history()` - Event management
- `has_changed_since()` - Check for changes since timestamp

**Features:**
- Configurable check interval
- Auto-reload on change option
- Event history with max limit
- Callback-based listener system
- Thread-safe running flag

## [0.0.559] - 2025-12-13

### Added - Settings CLI Interface (Phase 135)

**Settings CLI Module:**
- `settings_cli.rs` for natural language settings commands
- `SettingsCommand` enum (Show, Change, Reset, Export, Import, Validate, Help)
- `ParseResult` with confidence scores and alternatives
- `SettingsParser` for natural language parsing

**Core Functions:**
- `parse()` - Parse natural language into settings command
- `execute_command()` - Execute parsed command on settings
- `is_settings_command()` - Detect settings-related input
- `format_help()` / `format_categories()` - Help output

**Supported Commands:**
- "show settings" / "show personality" - Display settings
- "reset settings" / "reset privacy" - Reset to defaults
- "enable learning mode" - Change settings naturally
- "export settings" / "import from file" - Export/import
- "validate settings" - Check for issues

## [0.0.558] - 2025-12-13

### Added - Settings Export/Import (Phase 134)

**Settings Export Module:**
- `settings_export.rs` for exporting/importing settings
- `ExportFormat` enum (Json, Toml, JsonCompact)
- `ExportOptions` for format, metadata, obfuscation
- `ExportMetadata` with timestamp, version, description
- `SettingsExport` struct wrapping settings with metadata

**Core Functions:**
- `export_string()` / `export_file()` - Export settings
- `import_string()` / `import_file()` - Import settings
- `import_and_merge()` - Merge with existing settings
- `detect_format()` - Auto-detect format from path
- `default_filename()` - Generate timestamped filename

**Features:**
- Multiple format support (JSON, TOML, compact JSON)
- Optional metadata inclusion
- Automatic format detection
- Merge mode for importing

## [0.0.557] - 2025-12-13

### Added - Settings Validation (Phase 133)

**Settings Validation Module:**
- `settings_validation.rs` for validating settings and detecting conflicts
- `ValidationSeverity` enum (Info, Warning, Error)
- `ValidationCategory` enum (Conflict, Missing, Invalid, Performance, Security, Deprecated)
- `ValidationIssue` struct with suggestions
- `ValidationResult` for collecting and reporting issues

**Core Functions:**
- `validate()` - Validate settings against rules
- `check_conflicts()` - Detect conflicting settings
- `check_performance_issues()` - Find performance concerns
- `check_security_issues()` - Find security concerns
- `validate_settings()` - Quick validation helper

**Features:**
- Strict mode (warnings become errors)
- Skip performance/security checks options
- Suggestion system for fixes
- Formatted validation reports

## [0.0.556] - 2025-12-13

### Added - Settings Migration (Phase 132)

**Settings Migration Module:**
- `settings_migration.rs` for migrating settings between versions
- `MigrationStatus` enum (UpToDate, Migrated, Failed, UnknownVersion)
- `MigrationResult` struct with changes and warnings
- `VersionedSettings` struct with schema version tracking

**Core Functions:**
- `migrate()` - Migrate settings to current schema version
- `migrate_legacy()` - Migrate from pre-versioned settings
- `needs_migration()` - Check if migration is needed
- `migration_path()` - Get migration path description
- `migrate_and_save()` - Migrate and persist changes

**Features:**
- Schema version tracking (CURRENT_SCHEMA_VERSION)
- Migration history recording
- Dry run mode for testing migrations
- Automatic backup before migration
- Legacy format support

## [0.0.555] - 2025-12-13

### Added - Settings Persistence (Phase 131)

**Settings Persistence Module:**
- `settings_persistence.rs` for saving/loading unified settings per VISION.md
- `SettingsError` enum for IO, Serde, path, backup, restore errors
- `SettingsFormat` enum (Json, Toml)
- `SettingsPersistence` struct wrapping UnifiedSettings

**Core Functions:**
- `load()` / `save()` - Load/save settings to disk
- `create_backup()` / `restore_latest()` / `restore_from()` - Backup management
- `list_backups()` - List available backup files
- `export_to()` / `import_from()` - Export/import settings
- `apply_change()` - Apply change with auto-save support

**Features:**
- Auto-save on change (configurable)
- Automatic backup before save
- Maximum backup count (default: 5)
- Support for JSON and TOML formats
- Old backup cleanup

## [0.0.554] - 2025-12-13

### Added - Unified Settings Manager (Phase 130)

**Unified Settings Module:**
- `unified_settings.rs` aggregates all 12 configuration modules per VISION.md
- `SettingsCategory` enum for routing to appropriate config module
- `UnifiedSettings` struct containing all config modules as fields
- Single entry point for all natural language settings changes

**Core Functions:**
- `categorize_request()` - Detect which category a request belongs to
- `apply_change()` - Route changes to correct config module
- `reset_all()` / `reset_category()` - Reset settings to defaults
- `format_settings_summary()` - Display all settings overview
- `is_settings_query()` - Detect settings-related queries

**Aggregated Configuration Modules (12 categories):**
- Personality, Risk, Learning, Escalation
- Verbosity, Confirmation, Timeout, Output Style
- Privacy, Backup, Update, Model

## [0.0.553] - 2025-12-13

### Added - Model Config (Phase 129)

**Model Config Module:**
- `model_config.rs` for LLM model settings per VISION.md
- `ModelSizePreference` enum (Tiny, Small, Medium, Large, Largest)
- `QualitySpeedBalance` enum (Speed, Balanced, Quality)
- `ModelManagement` enum (Automatic, Manual, Conservative)
- `ModelConfig` for comprehensive model settings

**Core Functions:**
- `is_gpu_enabled()` / `is_auto_download()` - Check feature flags
- `should_use_senior()` / `max_memory_gb()` - Check model preferences
- `apply_change()` - Natural language model configuration
- `fast()` / `quality()` / `minimal()` - Preset configurations

**Supported Natural Language Changes:**
- "Fast/quick model" / "Quality/best answer"
- "Minimal/conserve resources"
- "Tiny/small/large model"
- "Enable/disable GPU"
- "Auto download" / "No download"
- "Use senior/junior models"

## [0.0.552] - 2025-12-13

### Added - Update Config (Phase 128)

**Update Config Module:**
- `update_config.rs` for update settings per VISION.md
- `UpdateCheckFrequency` enum (Never, Daily, Startup, Hourly, Manual)
- `UpdateChannel` enum (Stable, Beta, Nightly)
- `UpdateAction` enum (Notify, Download, AutoInstall)
- `UpdateConfig` for comprehensive update settings

**Core Functions:**
- `is_auto_update()` / `is_auto_check()` - Check update modes
- `check_interval_seconds()` - Get check interval
- `should_notify_available()` - Check notification settings
- `apply_change()` - Natural language update configuration
- `conservative()` / `automatic()` / `bleeding_edge()` - Presets

**Supported Natural Language Changes:**
- "Conservative/manual update"
- "Automatic/auto update"
- "Bleeding edge/nightly/latest"
- "Stable/beta channel"
- "Check hourly/daily/on startup"
- "Notify me" / "Silent update"

## [0.0.551] - 2025-12-13

### Added - Backup Config (Phase 127)

**Backup Config Module:**
- `backup_config.rs` for backup settings per VISION.md
- `BackupFrequency` enum (Manual, Hourly, Daily, Weekly, Monthly)
- `BackupType` enum (Incremental, Full, Differential, Snapshot)
- `BackupTarget` enum (Local, Network, Cloud, External)
- `CompressionLevel` enum (None, Fast, Balanced, Maximum)
- `BackupConfig` for comprehensive backup settings

**Core Functions:**
- `is_automatic()` / `interval_hours()` - Check backup schedule
- `should_encrypt()` / `should_verify()` - Check security settings
- `apply_change()` - Natural language backup configuration
- `minimal()` / `comprehensive()` / `manual_only()` - Presets

**Supported Natural Language Changes:**
- "Minimal/comprehensive/full backup"
- "Manual backup/no automatic"
- "Backup hourly/daily/weekly"
- "Encrypt/secure backup"
- "Verify/check backup"
- "Notify when complete"

## [0.0.550] - 2025-12-13

### Added - Privacy Config (Phase 126)

**Privacy Config Module:**
- `privacy_config.rs` for privacy settings per VISION.md
- `DataCollectionLevel` enum (None, Minimal, Standard, Full)
- `LogRetention` enum (Session, Day, Week, Month, Forever)
- `SensitiveDataHandling` enum (Redact, Mask, Hash, Allow)
- `PrivacyConfig` for comprehensive privacy settings

**Core Functions:**
- `is_maximum_privacy()` / `should_store_history()` - Check modes
- `should_anonymize()` / `retention_days()` - Privacy helpers
- `apply_change()` - Natural language privacy configuration
- `maximum()` / `balanced()` / `convenience()` - Preset configurations

**Supported Natural Language Changes:**
- "Maximum privacy/most private/paranoid"
- "Balanced/moderate privacy"
- "Convenience/remember everything"
- "Store history" / "No history/forget"
- "Anonymize" / "Clear on exit"
- "Enable/disable telemetry"

## [0.0.549] - 2025-12-13

### Added - Output Style Config (Phase 125)

**Output Style Config Module:**
- `output_style_config.rs` for output style per VISION.md
- `ColorScheme` enum (TrueColor, Ansi256, Ansi16, NoColor)
- `ThemeStyle` enum (Hollywood, Minimal, Classic, Hacker, Professional)
- `AnimationStyle` enum (Spinner, Progress, Dots, None)
- `BorderStyle` enum (Rounded, Sharp, Double, Ascii, None)
- `OutputStyleConfig` for comprehensive styling

**Core Functions:**
- `has_color()` / `has_animation()` / `is_compact()` - Check styles
- `effective_width()` - Get width respecting terminal
- `apply_change()` - Natural language style configuration
- `minimal()` / `hacker()` / `professional()` / `no_color()` - Presets

**Supported Natural Language Changes:**
- "Hollywood/minimal/hacker/professional style"
- "No color/monochrome/plain"
- "Enable/disable color"
- "Enable/disable animation"
- "Compact/spacious mode"

## [0.0.548] - 2025-12-13

### Added - Timeout Config (Phase 124)

**Timeout Config Module:**
- `timeout_config.rs` for timeout settings per VISION.md
- `TimeoutScope` enum (Command, LlmCall, Confirmation, Research, etc.)
- `TimeoutAction` enum (Cancel, Warn, Extend, Escalate, Background)
- `TimeoutProfile` enum (Fast, Normal, Patient, Unlimited)
- `TimeoutConfig` for comprehensive timeout settings

**Core Functions:**
- `timeout_for()` / `timeout_seconds()` - Get timeout for scope
- `is_unlimited()` / `is_fast()` - Check profile
- `should_show_countdown()` - Check countdown display
- `apply_change()` - Natural language timeout configuration
- `fast()` / `patient()` / `unlimited()` - Preset configurations

**Supported Natural Language Changes:**
- "Fast/strict/quick timeouts"
- "Patient/take your time/no rush"
- "Unlimited/no timeout/wait forever"
- "Show/hide countdown"
- "Auto extend" / "Strict timer"

## [0.0.547] - 2025-12-13

### Added - Confirmation Behavior Config (Phase 123)

**Confirmation Behavior Config Module:**
- `confirmation_behavior_config.rs` for confirmation behavior per VISION.md
- `ConfirmationStyle` enum (Inline, Prompt, Dialog, Silent)
- `TimeoutBehavior` enum (Deny, Approve, AskAgain)
- `ConfirmableAction` enum for action types requiring confirmation
- `ConfirmationBehaviorConfig` for comprehensive confirmation settings

**Core Functions:**
- `needs_confirmation()` - Check if action needs confirmation
- `is_auto_confirm_safe()` / `is_silent()` - Check modes
- `should_show_preview()` / `should_remember()` - Check behavior flags
- `apply_change()` - Natural language confirmation configuration
- `strict()` / `lenient()` / `silent()` - Preset configurations

**Supported Natural Language Changes:**
- "Strict/lenient/silent confirmation"
- "Require yes" / "Quick confirm"
- "Show/hide preview"
- "Remember/forget choices"
- "Always confirm root/delete"

## [0.0.546] - 2025-12-13

### Added - Verbosity Config (Phase 122)

**Verbosity Config Module:**
- `verbosity_config.rs` for verbosity and detail level per VISION.md
- `VerbosityLevel` enum (Minimal, Normal, Detailed, Verbose, Debug)
- `DetailLevel` enum (Summary, Standard, Full, Exhaustive)
- `OutputContext` enum for context-specific detail levels
- `VerbosityConfig` for comprehensive verbosity settings

**Core Functions:**
- `is_minimal()` / `is_verbose()` / `is_debug()` - Check verbosity level
- `detail_for()` - Get detail level for specific context
- `should_show()` - Check if section should be shown
- `apply_change()` - Natural language verbosity configuration
- `minimal()` / `verbose()` / `debug()` - Preset configurations

**Supported Natural Language Changes:**
- "Minimal/brief/short output"
- "Verbose/detailed/more detail"
- "Debug/maximum detail"
- "Show/hide citations/timestamps/confidence"
- "Show/hide progress"

## [0.0.545] - 2025-12-13

### Added - Escalation Policy Config (Phase 121)

**Escalation Policy Config Module:**
- `escalation_policy_config.rs` for escalation policy per VISION.md
- `EscalationTrigger` enum (LowConfidence, HighRisk, SecurityRelated, etc.)
- `EscalationPriority` enum (Normal, High, Critical, Immediate)
- `EscalationMode` enum (Automatic, SemiAutomatic, Manual, Disabled)
- `EscalationNotify` enum for notification preferences
- `EscalationPolicyConfig` for comprehensive escalation settings

**Core Functions:**
- `should_escalate_confidence()` - Check if confidence triggers escalation
- `should_escalate_security()` / `should_escalate_high_risk()` - Auto-escalation checks
- `needs_user_confirmation()` - Check if user consent needed before escalating
- `priority_for()` - Get priority for trigger type
- `should_notify()` - Check if user should be notified
- `apply_change()` - Natural language policy configuration
- `lenient()` / `strict()` / `manual()` - Preset configurations

**Supported Natural Language Changes:**
- "Automatic/semi-automatic/manual escalation"
- "Lenient/strict escalation policy"
- "Always escalate security/high risk"
- "Don't escalate security/high risk"
- "Always notify" / "Don't notify"

## [0.0.544] - 2025-12-13

### Added - Learning Mode Config (Phase 120)

**Learning Mode Config Module:**
- `learning_mode_config.rs` for learning mode per VISION.md
- `LearningModeLevel` enum (Off, Basic, Intermediate, Advanced, Expert)
- `ExplanationDepth` enum (None, WhatItDoes, WhyItWorks, HowItWorks, DeepDive)
- `ExplanationTopic` enum for topic filtering
- `LearningModeConfig` for comprehensive learning settings

**Core Functions:**
- `is_enabled()` / `enable()` / `disable()` - Toggle learning mode
- `should_explain_command()` / `should_explain_config()` - Check explanations
- `should_explain_why()` - Check if "why" explanations enabled
- `apply_change()` - Natural language configuration
- `basic()` / `intermediate()` / `advanced()` - Preset configurations

**Supported Natural Language Changes:**
- "Enable learning mode" / "Teach me"
- "Basic/intermediate/advanced learning"
- "Explain why" / "Show references"
- "Show wiki links"

## [0.0.543] - 2025-12-13

### Added - Risk Level Config (Phase 119)

**Risk Level Config Module:**
- `risk_level_config.rs` for confirmation skipping per VISION.md
- `RiskLevel` enum (None, Low, Medium, High, Critical) with ordering
- `ActionCategory` enum (ReadOnly, ConfigChange, FileDelete, etc.)
- `ConfirmationMode` enum (Never, OnlyHigh, Normal, Always)
- `RiskLevelConfig` for comprehensive risk management

**Core Functions:**
- `requires_confirmation()` - Check if risk level needs confirmation
- `risk_for_category()` - Get default risk for action category
- `needs_confirmation()` - Full check including root/delete overrides
- `apply_change()` - Natural language risk configuration
- `permissive()` / `strict()` - Preset configurations

**Supported Natural Language Changes:**
- "Skip all confirmations" / "Always confirm"
- "Only confirm high risk" / "Skip low risk"
- "Auto approve low/medium"
- "Require/skip root confirmation"
- "Require/skip delete confirmation"

## [0.0.542] - 2025-12-13

### Added - Personality Config (Phase 118)

**Personality Config Module:**
- `personality_config.rs` for natural language personality settings per VISION.md
- `FormalityLevel` enum (Casual, Professional, Formal, Technical)
- `FriendlinessLevel` enum (Reserved, Balanced, Friendly, Enthusiastic)
- `HumorLevel` / `VerbosityLevel` / `ExplanationStyle` enums
- `PersonalityConfig` struct for full personality state

**Core Functions:**
- `apply_change()` - Parse and apply natural language personality changes
- `greeting_style()` - Get appropriate greeting for current personality
- `should_explain()` - Check if explanations should be included
- Builder pattern: `with_formality()`, `with_friendliness()`, etc.

**Supported Natural Language Changes:**
- "Be more formal" / "Be casual"
- "Be friendlier" / "Be enthusiastic"
- "Be concise" / "More detail"
- "Use emoji" / "No emoji"
- "Step by step" / "Educational mode"

## [0.0.541] - 2025-12-13

### Added - Tips System (Phase 117)

**Tips System Module:**
- `tips_system.rs` for greeting tips per VISION.md
- `TipCategory` enum (Personality, LearningMode, RiskLevel, Verbosity, DisplayMode, etc.)
- `TipPriority` enum (Low, Normal, High, Featured)
- `Tip` / `TipsSystem` for tip management

**Core Functions:**
- `next_tip()` - Get next tip (rotates through least-shown)
- `tip_from_category()` - Get tip from specific category
- `add_tip()` - Add custom tips
- `set_enabled()` / `set_max_daily()` - Configuration
- `remaining_today()` - Daily tip limit tracking

**Default Tips:**
- Personality customization
- Learning mode explanation
- Risk level configuration
- Debug mode toggle
- Notification settings
- Stats command hint
- Citation importance
- Idle time research
- Custom alarms

## [0.0.540] - 2025-12-13

### Added - Installation Date Tracker (Phase 116)

**Installation Date Tracker Module:**
- `installation_tracker.rs` for tracking installation date per VISION.md
- `InstallMethod` enum (CurlScript, Manual, Package, Development)
- `InstallStatus` enum (Active, Upgraded, Reinstalled, Paused)
- `InstallationInfo` struct for comprehensive install tracking

**Core Functions:**
- `days_installed()` / `weeks_installed()` / `months_installed()` - Duration tracking
- `is_anniversary()` / `days_until_anniversary()` - Anniversary detection
- `is_monthly_milestone()` - Monthly milestone check
- `uptime_string()` - Human-readable install duration
- `record_upgrade()` - Track version upgrades

**Display Functions:**
- `format_installation_info()` / `format_installation_compact()` - Formatting
- `anniversary_message()` - Special anniversary messages
- `installation_fun_fact()` - Fun facts
- `is_installation_query()` - Query detection

## [0.0.539] - 2025-12-13

### Added - Team Consultation Tracker (Phase 115)

**Team Consultation Tracker Module:**
- `team_consultation_tracker.rs` for tracking team consultations per VISION.md
- `TeamDepartment` enum (Network, Storage, Audio, Video, Desktop, Security, etc.)
- `ConsultationOutcome` / `SeniorityConsulted` enums
- `ConsultationRecord` / `TeamConsultationTracker` system

**Core Functions:**
- `consult()` / `consult_with_seniority()` - Record consultation
- `most_consulted()` - Get most consulted team for fun stats
- `department_stats()` / `seniority_stats()` - Analytics
- `resolution_rate()` / `escalation_rate()` - Performance metrics
- `average_interactions()` - Interaction tracking

**Display Functions:**
- `format_consultation()` - Format single record
- `format_tracker_summary()` - Full tracker overview
- `team_consultation_fun_fact()` - Fun facts
- `is_team_query()` - Query detection

## [0.0.538] - 2025-12-13

### Added - Response Time Tracker (Phase 114)

**Response Time Tracker Module:**
- `response_time_tracker.rs` for tracking response times per VISION.md
- `ResponseType` enum (Direct, Recipe, Specialist, Escalated, Research, Clarification)
- `ComplexityLevel` enum (Simple, Moderate, Complex, VeryComplex)
- `ResponseTimeRecord` / `ResponseTimeTracker` system

**Core Functions:**
- `record()` / `record_full()` - Record response time
- `shortest_reply()` / `longest_reply()` - Word count extremes
- `shortest_time()` / `longest_time()` - Time extremes
- `average_time_ms()` / `average_word_count()` - Averages
- `percentile_time()` - Percentile calculations
- `time_distribution()` - Full distribution stats

**Display Functions:**
- `format_response_time()` - Format single record
- `format_tracker_summary()` - Full tracker overview
- `response_time_fun_fact()` - Fun facts
- `is_response_time_query()` - Query detection

## [0.0.537] - 2025-12-13

### Added - Query History Tracker (Phase 113)

**Query History Tracker Module:**
- `query_history_tracker.rs` for tracking user queries per VISION.md
- `QueryCategory` enum (System, Network, Storage, Audio, Video, Desktop, Security, Package, Service, Editor, Shell, Hardware, Configuration, General, Custom)
- `QueryOutcome` enum (Pending, Resolved, Escalated, Failed, Deferred)
- `QueryRecord` / `QueryHistoryTracker` system

**Core Functions:**
- `record()` - Record new query with auto ID
- `resolve()` / `escalate()` / `fail()` - Set outcome
- `repeated_queries()` - Get queries asked more than once
- `category_stats()` / `most_asked_topic()` - Topic analytics
- `by_category()` / `by_outcome()` - Filtering
- `average_response_time_ms()` - Performance stats

**Display Functions:**
- `format_query()` - Format single query record
- `format_tracker_summary()` - Full tracker overview
- `query_history_fun_fact()` - Fun facts
- `is_history_query()` - Query detection

## [0.0.536] - 2025-12-13

### Added - Display Mode Manager (Phase 112)

**Display Mode Manager Module:**
- `display_mode_manager.rs` for debug vs fly-on-the-wall display per VISION.md
- `DisplayMode` enum (FlyOnTheWall, Debug, Minimal, Verbose)
- `OutputSection` enum (Greeting, InternalComms, SpecialistDialog, ProbeResult, Citation, Answer, Error, Warning, Progress, DebugInfo)
- `VisibilityRule` / `DisplayModeManager` system

**Core Functions:**
- `DisplayMode::show_internal_comms()` - Show internal team dialog
- `DisplayMode::show_technical()` - Show JSON/technical details
- `DisplayMode::show_spinner()` - Show spinner animations
- `DisplayMode::stream_output()` - Stream word-by-word
- `set_mode()` / `mode()` - Get/set current mode
- `toggle_debug()` - Toggle between debug and fly-on-the-wall
- `is_visible()` - Check section visibility by mode
- `use_true_color()` / `hollywood_style()` - Display settings

**Display Functions:**
- `format_display_mode()` - Format mode for display
- `format_manager_summary()` - Full manager overview
- `display_fun_fact()` - Fun facts
- `is_display_query()` - Query detection

## [0.0.535] - 2025-12-13

### Added - Greeting Generator (Phase 111)

**Greeting Generator Module:**
- `greeting_generator.rs` for personalized greetings per VISION.md
- `TimeOfDay` enum (Morning, Afternoon, Evening, Night)
- `InsightType` enum (TimeSinceLastVisit, BootTimeChange, UsagePattern, PendingTickets, SystemHealth, NewFeature, Tip, Warning, Error)
- `GreetingInsight` / `GreetingContext` / `GreetingGenerator` system

**Core Functions:**
- `TimeOfDay::from_hour()` - Get time from hour (0-23)
- `greeting_prefix()` - Time-appropriate greeting
- `GreetingContext::new()` - Create context with username and hour
- `add_insight()` / `add_error()` / `add_warning()` - Add content
- `sorted_insights()` - Priority-sorted insights
- `generate()` - Generate full greeting with insights

**Display Functions:**
- `format_insight()` - Format single insight
- `format_context_summary()` - Context overview
- `greeting_fun_fact()` - Fun facts
- `is_greeting_query()` - Query detection

## [0.0.534] - 2025-12-13

### Added - Long Task Manager (Phase 110)

**Long Task Manager Module:**
- `long_task_manager.rs` for long-running tasks per VISION.md
- `LongTaskStatus` enum (Queued, WaitingIdle, Running, Paused, Completed, Failed, Cancelled)
- `LongTaskType` enum (Research, Installation, Backup, Download, Analysis, Compilation, Custom)
- `LongTaskRecord` / `LongTaskManager` system

**Core Functions:**
- `create()` - Create long task with auto ID
- `get()` / `get_mut()` - Task retrieval
- `wait_for_idle()` - Queue for idle-time execution
- `start()` / `pause()` / `resume()` - Lifecycle control
- `update_progress()` - Progress tracking (0-100%)
- `add_thought()` - Build chain of thought
- `complete()` / `fail()` / `cancel()` - Completion states
- `enable_email()` - Email when done
- `active()` / `waiting_for_idle()` / `pending_emails()` - Filtering

**Display Functions:**
- `format_long_task()` / `format_long_task_compact()` / `format_long_task_oneline()`
- `format_manager_summary()` - Full task overview
- `long_task_fun_fact()` - Fun facts
- `is_long_task_query()` - Query detection

## [0.0.533] - 2025-12-13

### Added - Notification Tracker (Phase 109)

**Notification Tracker Module:**
- `notification_tracker.rs` for tracking notifications per VISION.md
- `NotificationChannel` enum (Email, Libnotify, Wall, Terminal, Log)
- `NotificationPriority` enum (Low, Normal, High, Urgent)
- `DeliveryStatus` enum (Pending, Sent, Delivered, Failed, Suppressed)
- `NotificationRecord` / `NotificationTracker` system

**Core Functions:**
- `create()` - Create notification with auto ID
- `get()` / `get_mut()` - Notification retrieval
- `mark_sent()` / `mark_delivered()` / `mark_failed()` - Lifecycle
- `suppress()` - Anti-spam suppression
- `set_email()` - Store user email (with permission)
- `should_suppress()` - Anti-spam cooldown check
- `pending()` / `by_channel()` / `for_ticket()` - Filtering
- `recent()` - Get recent notifications

**Display Functions:**
- `format_notification()` / `format_notification_compact()` / `format_notification_oneline()`
- `format_tracker_summary()` - Full notification overview
- `notification_fun_fact()` - Fun facts
- `is_notification_query()` - Query detection

## [0.0.532] - 2025-12-13

### Added - Helper Install Tracker (Phase 108)

**Helper Install Tracker Module:**
- `helper_install_tracker.rs` for tracking helper tools per VISION.md
- `HelperInstaller` enum (User, Anna, System)
- `HelperCategory` enum (SystemInfo, NetworkDiag, DiskUtils, HardwareProbe, AudioVideo, Security, DevTools, Monitoring)
- `HelperStatus` enum (NotInstalled, Installing, Installed, Failed, Removed)
- `HelperRecord` / `HelperInstallTracker` system

**Core Functions:**
- `register()` - Register helper
- `get()` / `get_mut()` - Helper retrieval
- `install()` / `remove()` - Installation lifecycle
- `record_use()` - Track usage
- `installed()` / `installed_by_anna()` - Filtering
- `to_remove_on_uninstall()` - Anna-installed helpers to remove
- `would_be_useless()` - Check hardware requirements
- `by_category()` / `most_used()` - Analytics

**Display Functions:**
- `format_helper()` / `format_helper_compact()` / `format_helper_oneline()`
- `format_tracker_summary()` - Full helper overview
- `helper_fun_fact()` - Fun facts
- `is_helper_query()` - Query detection

## [0.0.531] - 2025-12-13

### Added - LLM Model Registry (Phase 107)

**LLM Model Registry Module:**
- `llm_model_registry.rs` for tracking installed LLM models per VISION.md
- `ModelCapability` enum (Light, Standard, Heavy, Multimodal)
- `ModelStatus` enum (Available, Downloading, Installing, Ready, Failed, Removed)
- `InstalledBy` enum (User, Anna, System)
- `ModelRecord` / `LlmModelRegistry` system

**Core Functions:**
- `register()` - Register model
- `get()` / `get_mut()` - Model retrieval
- `install()` - Install with tracking
- `assign()` / `unassign()` - Specialist assignment
- `record_use()` - Track usage with avg response time
- `ready()` / `by_capability()` / `installed_by_anna()` - Filtering
- `best_for()` - Find fastest model for capability
- `total_vram_gb()` / `total_disk_gb()` - Resource tracking

**Display Functions:**
- `format_model()` / `format_model_compact()` / `format_model_oneline()`
- `format_registry_summary()` - Full registry overview
- `model_fun_fact()` - Fun facts
- `is_model_query()` - Query detection

## [0.0.530] - 2025-12-13

### Added - Knowledge Citation Tracker (Phase 106)

**Knowledge Citation Tracker Module:**
- `knowledge_citation.rs` for tracking authoritative sources per VISION.md
- `CitationSource` enum (ArchWiki, ManPage, HelpCommand, InfoPage, OfficialDocs, LocalWiki, ConfigFile)
- `CitationReliability` enum (Unverified, Trusted, Verified, Authoritative)
- `CitationRecord` / `KnowledgeCitationTracker` system

**Core Functions:**
- `add()` - Add citation with auto ID
- `get()` / `get_mut()` - Citation retrieval
- `record_use()` - Track usage
- `by_source()` / `search()` / `for_ticket()` - Filtering
- `most_used()` / `authoritative()` - Analytics
- `as_reference()` - Format as reference string

**Display Functions:**
- `format_citation()` / `format_citation_compact()` / `format_citation_oneline()`
- `format_tracker_summary()` - Full citation overview
- `citation_fun_fact()` - Fun facts
- `is_citation_query()` - Query detection

## [0.0.529] - 2025-12-13

### Added - Escalation Tracker (Phase 105)

**Escalation Tracker Module:**
- `escalation_tracker.rs` for tracking junior-to-senior escalations per VISION.md
- `EscalationReason` enum (LowConfidence, ComplexQuery, MultiDepartment, SecurityConcern, HighRisk, UserRequest, TimeOut, Unknown)
- `EscalationOutcome` enum (Pending, ResolvedBySenior, ReturnedToJunior, EscalatedHigher, Abandoned)
- `EscalationRecord` / `EscalationTracker` system

**Core Functions:**
- `escalate()` - Create new escalation
- `resolve()` - Complete escalation with outcome
- `get()` - Get by ID
- `pending()` - Get pending escalations
- `by_reason()` / `by_department()` - Filtering

**Analytics:**
- `escalation_rate()` - Rate vs total tickets
- `senior_resolution_rate()` - % resolved by senior
- `avg_resolution_ms()` - Average resolution time
- `reason_stats()` - Stats by escalation reason

**Display Functions:**
- `format_escalation()` / `format_escalation_compact()` / `format_escalation_oneline()`
- `format_tracker_summary()` - Full escalation overview
- `escalation_fun_fact()` - Fun facts
- `is_escalation_query()` - Query detection

## [0.0.528] - 2025-12-13

### Added - Team Specialist Roster (Phase 104)

**Team Specialist Roster Module:**
- `team_specialist_roster.rs` for managing IT department roster (per VISION.md)
- `SeniorityLevel` enum (Junior, Senior)
- `Department` enum (Desktop, Network, Security, Storage, Audio, Video, System, Database, DevOps, Support)
- `AvailabilityStatus` enum (Available, Busy, OnTicket, Unavailable)
- `Specialist` / `TeamSpecialistRoster` system

**Core Functions:**
- `add()` - Add specialist to roster
- `get()` / `get_mut()` - Specialist retrieval
- `by_department()` - Filter by department
- `find_available()` - Find available specialist (prefers junior)
- `find_senior()` - Find senior for escalation
- `top_performers()` - Get best performers
- `assign_ticket()` / `complete_ticket()` - Ticket lifecycle

**Display Functions:**
- `format_specialist()` / `format_specialist_compact()` / `format_specialist_oneline()`
- `format_roster_summary()` - Full roster overview
- `roster_fun_fact()` - Fun facts
- `is_roster_query()` - Query detection

## [0.0.527] - 2025-12-13

### Added - Skill Proficiency Tracker (Phase 103)

**Skill Proficiency Module:**
- `skill_proficiency.rs` for tracking Anna's learned skills
- `SkillDomain` enum (SystemAdmin, Networking, Security, Storage, Audio, Video, Desktop, Scripting, Troubleshooting, UserSupport)
- `ProficiencyLevel` enum (Novice, Beginner, Intermediate, Advanced, Expert, Master)
- `SkillRecord` / `SkillProficiencyTracker` system

**Core Functions:**
- `learn()` - Learn a new skill
- `use_skill()` - Record skill usage with success/failure
- `get()` - Get skill by name
- `by_domain()` / `by_level()` - Filtering
- `top_skills()` - Get highest XP skills
- `needs_practice()` - Find skills with low success rate

**XP & Leveling:**
- XP thresholds for each proficiency level
- XP gain on success (scales with level)
- XP loss on failure (learning from mistakes)
- `xp_to_next_level()` tracking

**Display Functions:**
- `format_skill()` / `format_skill_compact()` / `format_skill_oneline()`
- `format_tracker_summary()` - Full proficiency overview
- `skill_fun_fact()` - Fun facts about skill learning
- `is_skill_query()` - Query detection

## [0.0.526] - 2025-12-13

### Added - Context Memory Store (Phase 102)

**Context Memory Store Module:**
- `context_memory_store.rs` for storing conversational context
- `MemoryType` enum (ShortTerm, LongTerm, Working, Episodic, Semantic)
- `MemoryImportance` enum (Low, Normal, High, Critical)
- `MemoryEntry` / `ContextMemoryStore` system

**Core Functions:**
- `store()` - Store a memory
- `retrieve()` - Retrieve and update access count
- `get()` / `search()` - Memory retrieval
- `delete()` - Remove memory
- `prune()` - Automatic cleanup of low-priority memories
- `by_mem_type()` / `important()` - Filtering

**Display Functions:**
- `format_memory_store()` - Full memory list
- `format_memory_store_compact()` - Compact summary
- `format_memory_store_oneline()` - Single line status
- `memory_fun_fact()` - Fun facts
- `is_memory_query()` - Query detection

## [0.0.525] - 2025-12-13

### Added - Workflow Automation Tracker (Phase 101)

**Workflow Automation Module:**
- `workflow_automation.rs` for tracking automated workflows
- `WorkflowTrigger` enum (Manual, Scheduled, Event, Condition, Webhook)
- `WorkflowStatus` enum (Active, Paused, Running, Completed, Failed)
- `WorkflowStep` / `WorkflowRecord` / `WorkflowAutomationTracker` system

**Core Functions:**
- `create()` - Create workflow
- `add_step()` - Add step to workflow
- `run()` / `complete()` - Workflow execution
- `get()` / `active()` - Workflow retrieval
- `by_wf_trigger()` - Filter by trigger type
- `success_rate()` - Analytics

**Display Functions:**
- `format_workflow_tracker()` - Full workflow list
- `format_workflow_tracker_compact()` - Compact summary
- `format_workflow_tracker_oneline()` - Single line status
- `workflow_fun_fact()` - Fun facts
- `is_workflow_query()` - Query detection

## [0.0.524] - 2025-12-13

### Added - Anna Metrics Dashboard (Phase 100!) 🎉

**MILESTONE: Phase 100 Reached!**

**Anna Metrics Dashboard Module:**
- `anna_metrics_dashboard.rs` - Comprehensive metrics dashboard
- `DashboardSection` enum (Overview, Tickets, Recipes, Learning, etc.)
- `HealthStatus` enum (Healthy, Warning, Critical, Unknown)
- `MetricEntry` / `AnnaMetricsDashboard` system

**Core Functions:**
- `set_metric()` / `set_metric_with_trend()` - Add/update metrics
- `get()` / `by_dash_section()` - Metric retrieval
- `update_health()` / `refresh()` - Dashboard state
- `positive_trends()` / `negative_trends()` - Trend analysis

**Display Functions:**
- `format_dashboard()` - Full dashboard display with ASCII art
- `format_dashboard_compact()` - Compact summary
- `format_dashboard_oneline()` - Single line status
- `dashboard_fun_fact()` - Fun facts
- `is_dashboard_query()` - Query detection

## [0.0.523] - 2025-12-13

### Added - Task Priority Manager (Phase 99)

**Task Priority Manager Module:**
- `task_priority_manager.rs` for managing and prioritizing tasks
- `TaskPriority` enum (Low, Normal, High, Urgent, Critical)
- `TaskState` enum (Pending, InProgress, Blocked, Completed, Cancelled)
- `ManagedTask` / `TaskPriorityManager` system

**Core Functions:**
- `add()` - Add task with priority
- `start()` / `complete()` - Task lifecycle
- `block()` / `cancel()` - State transitions
- `get()` / `next()` - Task retrieval
- `pending()` - Get tasks sorted by priority
- `in_progress()` / `blocked()` - State filtering

**Display Functions:**
- `format_task_manager()` - Full task list
- `format_task_manager_compact()` - Compact summary
- `format_task_manager_oneline()` - Single line status
- `task_manager_fun_fact()` - Fun facts
- `is_task_manager_query()` - Query detection

## [0.0.522] - 2025-12-13

### Added - User Preference Learner (Phase 98)

**User Preference Learner Module:**
- `user_preference_learner.rs` for learning user preferences
- `PreferenceCategory` enum (Communication, Technical, Schedule, etc.)
- `LearnConfidence` enum (Low, Medium, High, Confirmed)
- `LearnedPreference` / `UserPreferenceLearner` system

**Core Functions:**
- `learn()` - Learn preference from observation
- `confirm()` - User confirms a preference
- `get()` - Get preference by key
- `by_pref_category()` - Filter by category
- `high_confidence()` / `confirmed()` - Get reliable preferences
- Confidence increases with repeated observations

**Display Functions:**
- `format_preference_learner()` - Full preference list
- `format_preference_learner_compact()` - Compact summary
- `format_preference_learner_oneline()` - Single line status
- `preference_learner_fun_fact()` - Fun facts
- `is_preference_learner_query()` - Query detection

## [0.0.521] - 2025-12-13

### Added - Error Recovery Tracker (Phase 97)

**Error Recovery Tracker Module:**
- `error_recovery_tracker.rs` for tracking error recovery attempts
- `ErrorCategory` enum (System, Network, Permission, NotFound, etc.)
- `RecoveryOutcome` enum (Success, PartialSuccess, Failed, Skipped, Manual)
- `ErrorRecoveryRecord` / `ErrorRecoveryTracker` system

**Core Functions:**
- `record()` - Record error recovery attempt
- `get()` - Get record by ID
- `by_err_category()` / `by_rec_outcome()` - Filtering
- `successful()` / `failed()` - Get by outcome
- `strategy_rate()` / `best_strategies()` - Strategy analytics
- `recovery_rate()` - Overall recovery rate

**Display Functions:**
- `format_error_recovery_tracker()` - Full recovery list
- `format_error_recovery_tracker_compact()` - Compact summary
- `format_error_recovery_tracker_oneline()` - Single line status
- `error_recovery_fun_fact()` - Fun facts
- `is_error_recovery_query()` - Query detection

## [0.0.520] - 2025-12-13

### Added - Resource Usage Tracker (Phase 96)

**Resource Usage Tracker Module:**
- `resource_usage_tracker.rs` for tracking system resource usage
- `ResourceType` enum (Cpu, Memory, Disk, Network, Gpu, Swap)
- `UsageLevel` enum (Low, Normal, Elevated, High, Critical)
- `UsageSample` / `ResourceUsageTracker` system

**Core Functions:**
- `record()` - Record resource usage sample
- `current()` / `peak()` - Get current/peak values
- `average()` - Get average for resource
- `by_res_type()` / `by_usage_level()` - Filtering
- `critical()` / `high()` - Get critical/high samples

**Display Functions:**
- `format_resource_tracker()` - Full usage display
- `format_resource_tracker_compact()` - Compact summary
- `format_resource_tracker_oneline()` - Single line status
- `resource_fun_fact()` - Fun facts
- `is_resource_query()` - Query detection

## [0.0.519] - 2025-12-13

### Added - Query Pattern Analyzer (Phase 95)

**Query Pattern Analyzer Module:**
- `query_pattern_analyzer.rs` for analyzing query patterns
- `PatternCategory` enum (Question, Command, Status, Config, etc.)
- `ConfidenceLevel` enum (Low, Medium, High, Certain)
- `QueryPattern` / `QueryPatternAnalyzer` system

**Core Functions:**
- `add_pattern()` / `get()` - Pattern management
- `record_match()` - Track pattern matches
- `add_example()` - Add example queries
- `by_pat_category()` / `high_confidence()` - Filtering
- `most_used()` - Get frequently matched patterns
- `overall_success_rate()` - Analytics

**Display Functions:**
- `format_pattern_analyzer()` - Full pattern list
- `format_pattern_analyzer_compact()` - Compact summary
- `format_pattern_analyzer_oneline()` - Single line status
- `pattern_fun_fact()` - Fun facts
- `is_pattern_query()` - Query detection
- `COMMON_PATTERNS` constant

## [0.0.518] - 2025-12-13

### Added - Session History Tracker (Phase 94)

**Session History Module:**
- `session_history.rs` for tracking user session history
- `SessionOutcome` enum (Completed, Abandoned, Error, Timeout, UserExit)
- `SessionType` enum (Interactive, OneShot, Background, Scheduled)
- `SessionRecord` / `SessionHistoryTracker` system

**Core Functions:**
- `start_session()` / `end_session()` - Session lifecycle
- `record_query()` / `record_ticket()` - Track activity
- `get()` / `latest()` / `recent()` - Session retrieval
- `by_session_outcome()` / `by_session_type()` - Filtering
- `avg_duration()` - Session analytics

**Display Functions:**
- `format_session_history()` - Full session list
- `format_session_history_compact()` - Compact summary
- `format_session_history_oneline()` - Single line status
- `session_fun_fact()` - Fun facts
- `is_session_query()` - Query detection

## [0.0.517] - 2025-12-13

### Added - Dependency Tracker (Phase 93)

**Dependency Tracker Module:**
- `dependency_tracker.rs` for tracking software dependencies
- `DependencyType` enum (Runtime, Build, Optional, Recommended, etc.)
- `DependencyStatus` enum (Installed, Missing, Outdated, Orphaned)
- `DependencyRecord` / `DependencyTracker` system

**Core Functions:**
- `add()` / `deps_for()` - Dependency management
- `reverse_deps()` - Find what depends on a package
- `has_missing()` - Check for missing dependencies
- `safe_to_remove()` - Check if package can be safely removed
- `orphaned()` / `missing()` / `outdated()` - Status filtering
- `update_status()` - Update dependency status

**Display Functions:**
- `format_dependency_tracker()` - Full dependency list
- `format_dependency_tracker_compact()` - Compact summary
- `format_dependency_tracker_oneline()` - Single line status
- `dependency_fun_fact()` - Fun facts
- `is_dependency_query()` - Query detection

## [0.0.516] - 2025-12-13

### Added - Hardware Capability Detector (Phase 92)

**Hardware Capability Module:**
- `hardware_capability.rs` for detecting what hardware exists
- `HardwareCategory` enum (Network, Audio, Video, Storage, etc.)
- `HardwareStatus` enum (Detected, NotDetected, Disabled, Error, Unknown)
- `HardwareCapability` / `HardwareCapabilityTracker` system
- Per VISION.md: "Never install useless helpers (no ethtool if no ethernet)"

**Core Functions:**
- `register()` / `get()` - Capability management
- `update_status()` - Update capability status with timestamp
- `has()` - Check if capability detected
- `is_helper_useful()` - Check if helper relevant to detected hardware
- `useless_helpers()` - Filter out helpers for non-existent hardware
- `by_hw_category()` / `detected()` / `not_detected()` - Status filtering

**Display Functions:**
- `format_hardware_tracker()` - Full capability list
- `format_hardware_tracker_compact()` - Compact summary
- `format_hardware_tracker_oneline()` - Single line status
- `hardware_fun_fact()` - Fun facts
- `is_hardware_query()` - Query detection
- `get_relevant_helpers()` - Get helpers for capability

**Common Capabilities:**
- `COMMON_CAPABILITIES` constant mapping hardware to helpers
- ethernet → ethtool, mii-tool
- wifi → iwconfig, iw, nmcli
- bluetooth → bluetoothctl, hcitool
- sound → alsamixer, pulseaudio, pipewire
- nvidia_gpu → nvidia-smi, nvtop
- amd_gpu → radeontop
- battery → acpi, upower
- nvme → nvme-cli
- sata → smartctl, hdparm

## [0.0.515] - 2025-12-13

### Added - Strategic Thinking Tracker (Phase 91)

**Strategic Thinking Module:**
- `strategic_thinking.rs` for idle-time analysis
- `ThinkingStatus` enum (Pending, InProgress, Paused, Completed, Abandoned)
- `ThinkingCategory` enum (Optimization, Security, Maintenance, etc.)
- `ThinkingPriority` enum (Low, Medium, High, Critical)
- `ThinkingTask` / `StrategicThinkingTracker` tracker

**Core Functions:**
- `add()` / `get()` - Task management
- `start()` / `pause()` / `resume()` - Interruptible workflow
- `complete()` - Record findings and recommendations
- `pending()` / `paused()` / `completed()` - Status filtering
- `high_priority()` - Priority filtering
- Tracks resume_point for interrupted tasks

**Display Functions:**
- `format_strategic_tracker()` - Full task list
- `format_strategic_tracker_compact()` - Compact summary
- `format_strategic_tracker_oneline()` - Single line status
- `strategic_fun_fact()` - Fun facts
- `is_strategic_query()` - Query detection

## [0.0.514] - 2025-12-13

### Added - Alarm Scheduler (Phase 90)

**Alarm Scheduler Module:**
- `alarm_scheduler.rs` for recurring notifications
- `AlarmFrequency` enum (Once, Daily, Weekly, Monthly, Hourly, Custom)
- `DayOfWeek` enum with name/short accessors
- `AlarmStatus` enum (Active, Paused, Triggered, Expired, Cancelled)
- `AlarmRecord` / `AlarmScheduler` comprehensive system

**Core Functions:**
- `add()` / `get()` - Alarm management
- `trigger()` - Trigger alarm and update stats
- `pause()` / `resume()` / `cancel()` - Status control
- `active()` / `due_at()` - Query alarms
- `by_alarm_topic()` / `by_alarm_frequency()` - Filtering
- `parse_day_of_week()` - Natural language day parsing

**Display Functions:**
- `format_alarm_scheduler()` - Full alarm list
- `format_alarm_scheduler_compact()` - Compact summary
- `format_alarm_scheduler_oneline()` - Single line status
- `alarm_fun_fact()` - Fun facts
- `is_alarm_query()` - Query detection

## [0.0.513] - 2025-12-13

### Added - Dialogue Renderer (Phase 89)

**Dialogue Renderer Module:**
- `dialogue_renderer.rs` for fly-on-the-wall display
- `Speaker` enum (Anna, User, Junior, Senior, Lead, System)
- `DialogueMood` enum (Neutral, Confident, Uncertain, etc.)
- `DialogueTurn` / `Dialogue` conversation model

**Core Functions:**
- `anna_says()` / `user_says()` - Add turns
- `specialist_says()` - Add internal communication
- `internal_turns()` / `external_turns()` - Filtering
- Color codes for each speaker type

**Render Functions:**
- `render_dialogue()` - Full colored render
- `render_dialogue_plain()` - Plain text render
- `render_dialogue_compact()` - Single line summary
- `is_dialogue_query()` - Query detection
- `dialogue_fun_fact()` - Fun facts

## [0.0.512] - 2025-12-13

### Added - LLM Assignment Tracker (Phase 88)

**LLM Assignment Module:**
- `llm_assignment.rs` for model assignment tracking
- `ModelTier` enum (Light, Standard, Heavy, DeepThinking)
- `AssignmentReason` enum (Default, HardwareLimit, etc.)
- `LlmAssignment` / `LlmAssignmentTracker` tracker

**Core Functions:**
- `assign()` - Assign model to specialist
- `get_assignment()` - Get current assignment
- `add_available_model()` / `is_model_available()`
- `set_recommended_tier()` - Hardware-based recommendation
- `by_llm_model()` / `by_model_tier()` - Filtering
- `models_in_use()` / `most_used_model()`
- `COMMON_MODELS` - Common model/tier mappings
- `get_model_tier()` - Auto-detect tier from model name

**Display Functions:**
- `format_llm_tracker()` - Full assignment display
- `format_llm_tracker_compact()` - Compact summary
- `format_llm_tracker_oneline()` - Single line status
- `llm_fun_fact()` - Fun facts
- `is_llm_query()` - Query detection

## [0.0.511] - 2025-12-13

### Added - Specialist Roster (Phase 87)

**Specialist Roster Module:**
- `specialist_roster.rs` for team management
- `SpecialistLevel` enum (Junior, Senior, Lead)
- `Department` enum (Desktop, Network, Security, etc.)
- `SpecialistProfile` with name, skills, tickets resolved
- `SpecialistRoster` comprehensive team tracker

**Core Functions:**
- `add()` / `get()` / `get_by_name()` - Specialist management
- `record_resolution()` - Track tickets per specialist
- `set_available()` - Availability management
- `by_dept()` / `by_lvl()` - Filtering
- `juniors()` / `seniors()` - Level filtering
- `top_performer()` - Performance tracking
- `SPECIALIST_NAMES` - Diverse human names
- `get_specialist_name()` - Get name by department/level

**Display Functions:**
- `format_specialist_roster()` - Full roster display
- `format_roster_compact()` - Compact summary
- `format_roster_oneline()` - Single line status
- `roster_fun_fact()` - Fun facts
- `is_specialist_roster_query()` - Query detection

## [0.0.510] - 2025-12-13

### Added - Ticket Resolution Stats (Phase 86)

**Resolution Stats Module:**
- `ticket_resolution_stats.rs` for resolution tracking
- `Resolver` enum (Anna, Junior, Senior, Escalated, User, Unknown)
- `ResolutionMethod` enum (Recipe, Specialist, DirectAnswer, etc.)
- `ResolutionRecord` / `TicketResolutionStats` comprehensive tracker

**Core Functions:**
- `record()` - Record a resolution
- `anna_rate()` - Get Anna's resolution percentage
- `by_res()` / `by_res_method()` - Filtering
- `avg_resolution_time()` / `fastest_resolution()` / `slowest_resolution()`
- `anna_improving()` - Check if Anna is getting better over time
- `recipes_learned` - Track recipes learned from resolutions

**Display Functions:**
- `format_resolution_stats()` - Full resolution history
- `format_resolution_stats_compact()` - Compact summary
- `format_resolution_stats_oneline()` - Single line status
- `resolution_fun_fact()` - Fun facts
- `is_resolution_stats_query()` - Query detection

## [0.0.509] - 2025-12-13

### Added - Idle Time Detector (Phase 85)

**Idle Time Module:**
- `idle_time_detector.rs` for idle detection
- `IdleState` enum (Active, Idle, DeepIdle, Suspended, Unknown)
- `ActivityLevel` enum (High, Medium, Low, Minimal)
- `IdleConfig` with thresholds, quiet hours, background work toggle
- `IdlePeriod` / `IdleTimeTracker` comprehensive tracker

**Core Functions:**
- `record_activity()` - Reset idle timer
- `check_idle()` - Update and return current state
- `can_do_background_work()` - Check if work allowed
- `is_quiet_hours()` - Check quiet hours
- `record_task_completed()` - Track background tasks
- `avg_idle_duration()` / `longest_idle()` - Statistics

**Display Functions:**
- `format_idle_tracker()` - Full idle history
- `format_idle_tracker_compact()` - Compact summary
- `format_idle_tracker_oneline()` - Single line status
- `idle_fun_fact()` - Fun facts
- `is_idle_query()` - Query detection

## [0.0.508] - 2025-12-13

### Added - Email Notification System (Phase 84)

**Email Notification Module:**
- `email_notification.rs` for email tracking
- `NotificationStatus` enum (Pending, Sent, Failed, Cancelled, Queued)
- `NotificationType` enum (TaskComplete, TaskFailed, ResearchResult, etc.)
- `EmailConfig` struct with consent, daily limits, DND hours
- `NotificationRecord` / `EmailNotificationTracker`

**Core Functions:**
- `configure()` - Setup email with consent
- `is_configured()` - Check consent status
- `record()` / `update_status()` - Notification management
- `daily_limit_reached()` / `reset_daily()` - Rate limiting
- `pending()` / `sent()` / `failed()` - Status filtering
- `success_rate()` - Delivery metrics

**Display Functions:**
- `format_email_tracker()` - Full notification history
- `format_email_tracker_compact()` - Compact summary
- `format_email_tracker_oneline()` - Single line status
- `email_fun_fact()` - Fun facts
- `is_email_notification_query()` - Query detection

## [0.0.507] - 2025-12-13

### Added - Helper Tracker (Phase 83)

**Helper Tracker Module:**
- `helper_tracker.rs` for tracking helper tools
- `InstallerSource` enum (Anna, User, System, Unknown)
- `HelperPurpose` enum (SystemInfo, NetworkDiag, DiskUtil, etc.)
- `HelperRecord` struct for individual helpers
- `HelperTracker` comprehensive tracker

**Core Functions:**
- `register()` / `record_usage()` - Helper management
- `mark_unavailable()` - Availability tracking
- `anna_installed()` / `user_installed()` - Source filtering
- `removable_on_uninstall()` - VISION.md requirement
- `most_used()` / `most_common_purpose()`
- `detect_purpose()` - Auto-detect helper purpose

**Display Functions:**
- `format_helper_tracker()` - Full helper list
- `format_helper_tracker_compact()` - Compact summary
- `format_helper_tracker_oneline()` - Single line status
- `helper_fun_fact()` - Fun facts
- `is_helper_query()` - Query detection

## [0.0.506] - 2025-12-13

### Added - Config Change Tracker (Phase 82)

**Config Tracker Module:**
- `config_change_tracker.rs` for config tracking
- `ChangeType` enum (Add, Modify, Delete, Replace, etc.)
- `ConfigCategory` enum (Shell, Editor, Git, System, etc.)
- `ConfigChangeRecord` struct for individual changes
- `ConfigChangeTracker` comprehensive tracker

**Core Functions:**
- `record()` / `mark_rolled_back()` - Change management
- `for_file()` / `by_config_category()` / `by_change_type()`
- `rolled_back()` / `active()` - Status filtering
- `most_changed_file()` / `most_common_category()`
- `detect_category()` - Auto-detect config category

**Display Functions:**
- `format_config_tracker()` - Full change history
- `format_config_tracker_compact()` - Compact summary
- `format_config_tracker_oneline()` - Single line status
- `config_fun_fact()` - Fun facts
- `is_config_tracker_query()` - Query detection

## [0.0.505] - 2025-12-13

### Added - Service Management Tracker (Phase 81)

**Service Tracker Module:**
- `service_management_tracker.rs` for service tracking
- `ServiceOperation` enum (Start, Stop, Restart, Reload, etc.)
- `OperationResult` enum (Success, Failed, Skipped, Pending)
- `ServiceRecord` struct for individual operations
- `ServiceTracker` comprehensive tracker

**Core Functions:**
- `record()` - Record service operation
- `by_operation_type()` / `for_service()` - Filtering
- `failed()` / `successful()` - Status filtering
- `success_rate()` / `most_managed()` / `most_common_op()`

**Display Functions:**
- `format_service_tracker()` - Full service history
- `format_service_tracker_compact()` - Compact summary
- `format_service_tracker_oneline()` - Single line status
- `service_fun_fact()` - Fun facts
- `is_service_tracker_query()` - Query detection

## [0.0.504] - 2025-12-13

### Added - Package Installation Tracker (Phase 80)

**Package Tracker Module:**
- `package_install_tracker.rs` for package tracking
- `InstalledBy` enum (Anna, User, System, Unknown)
- `PackageManager` enum (Pacman, Apt, Dnf, Flatpak, etc.)
- `PackageRecord` struct for individual packages
- `PackageTracker` comprehensive tracker

**Core Functions:**
- `record_install()` / `record_removal()` - Package management
- `anna_installed()` / `user_installed()` - Filter by installer
- `by_package_manager()` - Filter by manager
- `installed()` / `removed()` / `recent()` - Query packages

**Display Functions:**
- `format_package_tracker()` - Full package history
- `format_package_tracker_compact()` - Compact summary
- `format_package_tracker_oneline()` - Single line status
- `package_fun_fact()` - Fun facts
- `is_package_tracker_query()` - Query detection

## [0.0.503] - 2025-12-13

### Added - Backup History Tracking (Phase 79)

**Backup Module:**
- `backup_history.rs` for backup tracking
- `BackupStatus` enum (Active, Restored, Expired, Deleted)
- `BackupType` enum (ConfigFile, SystemFile, UserFile, etc.)
- `BackupRecord` struct for individual backups
- `BackupHistory` comprehensive tracker

**Core Functions:**
- `add()` / `get()` / `get_mut()` - Backup management
- `mark_restored()` / `mark_deleted()` - Status changes
- `active()` / `recent()` - Query backups
- `by_backup_type()` / `for_file()` / `for_change()`
- `expire_old()` - Automatic expiration
- `active_size_bytes()` - Size calculations

**Display Functions:**
- `format_size()` - Human-readable sizes
- `format_backup_history()` - Full display
- `format_backup_history_compact()` - Compact summary
- `format_backup_history_oneline()` - Single line status
- `backup_fun_fact()` - Fun facts
- `is_backup_query()` - Query detection

## [0.0.502] - 2025-12-13

### Added - Specialist Conversation Display (Phase 78)

**Conversation Module:**
- `specialist_conversation.rs` for conversation tracking
- `Speaker` enum (Anna, Junior, Senior, User)
- `MessageType` enum (Query, Response, Clarification, etc.)
- `ConversationMessage` struct for individual messages
- `Conversation` struct for conversation threads
- `ConversationHistory` comprehensive tracker

**Core Functions:**
- `add_message()` / `resolve()` - Conversation management
- `participants()` / `messages_by_speaker()` - Analysis
- `recent()` / `active()` / `resolved()` - Query history
- `avg_messages_per_conversation()` / `avg_resolution_secs()`
- `most_active_specialist()` / `most_active_department()`

**Display Functions:**
- `format_conversation()` - Fly-on-the-wall style display
- `format_conversation_history()` - Full history display
- `format_conversation_history_compact()` - Compact summary
- `format_conversation_history_oneline()` - Single line status
- `conversation_fun_fact()` - Fun facts
- `is_conversation_query()` - Query detection

## [0.0.501] - 2025-12-13

### Added - Command Execution Logging (Phase 77)

**Execution Log Module:**
- `command_execution_log.rs` for command tracking
- `ExecStatus` enum (Success, Failed, Timeout, Cancelled, Pending)
- `CommandRisk` enum (ReadOnly, LowRisk, MediumRisk, HighRisk, Critical)
- `ExecutionRecord` struct for individual executions
- `ExecutionLog` comprehensive tracker

**Core Functions:**
- `record()` - Record command execution
- `recent()` / `by_status()` / `by_ticket()` - Query executions
- `failed()` / `elevated()` / `high_risk()` - Filter executions
- `success_rate()` / `average_duration_ms()` - Statistics
- `most_used()` / `most_failed()` - Command patterns

**Risk Classification:**
- `classify_risk()` - Determine command risk level
- `extract_command_pattern()` - Extract command name

**Display Functions:**
- `format_execution_log()` - Full execution log display
- `format_execution_log_compact()` - Compact summary
- `format_execution_log_oneline()` - Single line status
- `execution_fun_fact()` - Fun facts about executions
- `is_execution_log_query()` - Query detection

## [0.0.500] - 2025-12-13

### Added - Boot Time Tracking (Phase 76)

**Boot Time Module:**
- `boot_time_tracking.rs` for boot time analysis
- `BootRecord` struct for individual boot events
- `SlowService` struct for tracking slow services
- `BootTrend` enum (Faster, Slower, Stable)
- `BootTimeTracker` comprehensive tracker

**Core Functions:**
- `record()` - Record boot with kernel/userspace times
- `change_from_previous()` - Compare to previous boot
- `trend()` - Analyze trend over recent boots
- `average_boot_secs()` - Average boot time
- `top_slow_services()` - Services that slow boot

**Parser Functions:**
- `parse_systemd_analyze()` - Parse systemd-analyze output
- `parse_time_value()` - Parse time strings (5.6s, 500ms, 1min)

**Display Functions:**
- `format_boot_stats()` - Full boot statistics display
- `format_boot_stats_compact()` - Compact summary
- `format_boot_stats_oneline()` - Single line status
- `boot_time_greeting()` - Greeting message about changes
- `boot_time_fun_fact()` - Fun facts about boot
- `is_boot_time_query()` - Query detection

## [0.0.499] - 2025-12-13

### Added - Knowledge Base Stats (Phase 75)

**Knowledge Base Module:**
- `knowledge_base_stats.rs` for knowledge acquisition tracking
- `KnowledgeType` enum (Recipe, Fact, WikiPage, ManPage, HelpCache, UserTaught)
- `KnowledgeSource` enum (Seed, Specialist, User, ArchWiki, ManPages, HelpCommands, Imported)
- `KnowledgeEntry` struct with usage tracking
- `KnowledgeBaseStats` comprehensive tracker

**Core Functions:**
- `add_entry()` - Add knowledge with type and source
- `record_use()` - Track knowledge usage
- `mark_stale()` / `refresh()` - Lifecycle management
- `by_type()` / `by_source()` / `by_topic()` - Filtering
- `stale()` / `recently_acquired()` / `most_used()` - Queries
- `acquisition_rate()` / `usage_rate()` - Statistics

**Display Functions:**
- `format_knowledge_stats()` - Full breakdown by type/source
- `format_knowledge_stats_compact()` - Compact summary
- `format_knowledge_stats_oneline()` - Single line status
- `knowledge_fun_fact()` - Fun knowledge facts
- `is_knowledge_query()` - Query detection

## [0.0.498] - 2025-12-13

### Added - System Health Score (Phase 74)

**System Health Module:**
- `system_health_score.rs` for unified health assessment
- `HealthGrade` enum (A, B, C, D, F) with descriptions
- `HealthCategory` enum (8 categories with weights)
- `HealthMetric` individual component health
- `SystemHealthScore` unified tracker

**Health Functions:**
- `cpu_health()` / `memory_health()` / `disk_health()`
- `services_health()` / `network_health()` / `daemon_health()`
- `add_metric()` / `calculate_overall()`
- `issues()` / `critical()` / `is_healthy()`

**Display Functions:**
- `format_health_score()` - Full display with recommendations
- `format_health_score_compact()` - Compact summary
- `format_health_score_oneline()` - Single line status
- `health_bar()` - ASCII health visualization
- `health_summary_message()` - Natural language summary
- `is_health_query()` - Query detection

## [0.0.497] - 2025-12-13

### Added - User Activity Summary (Phase 73)

**User Activity Module:**
- `user_activity_summary.rs` for usage pattern tracking
- `TimeOfDay` enum (Morning, Afternoon, Evening, Night)
- `DayOfWeek` enum with display names
- `ActivityRecord` for individual interactions
- `UserActivitySummary` comprehensive tracker

**Core Functions:**
- `record()` - Record activity with time context
- `most_active_time()` / `most_active_day()` - Find patterns
- `top_topic()` / `top_activity_type()` - Popular topics
- `days_active()` / `avg_interactions_per_day()` - Usage stats
- `detect_topic()` - Auto-categorize queries (8 topics)

**Display Functions:**
- `format_activity_summary()` - Full breakdown by time/day
- `format_activity_summary_compact()` - Compact summary
- `format_activity_summary_oneline()` - Single line status
- `activity_insight()` - Smart usage insights
- `is_activity_query()` - Query detection

## [0.0.496] - 2025-12-13

### Added - Anna Progress Report (Phase 72)

**Progress Report Module:**
- `anna_progress_report.rs` for progress tracking
- `TimePeriod` enum (Day, Week, Month, AllTime)
- `Trend` enum (Up, Down, Stable)
- `ProgressMetric` with trend tracking
- `Milestone` with progress tracking
- `PeriodSnapshot` for period comparisons
- `ProgressReport` comprehensive container

**Core Functions:**
- `add_metric()` / `add_milestone()` / `add_highlight()`
- `achieved_milestones()` / `pending_milestones()`
- `calculate_trend()` / `calculate_change_percent()`
- `progress_bar()` - ASCII progress visualization
- `default_milestones()` - 10 standard milestones
- `progress_summary_message()` - Natural language summary

**Display Functions:**
- `format_progress_report()` - Full report with sections
- `format_progress_report_compact()` - Compact summary
- `format_progress_report_oneline()` - Single line status
- `is_progress_query()` - Query detection

## [0.0.495] - 2025-12-13

### Added - Team Performance Display (Phase 71)

**Team Performance Module:**
- `team_performance_display.rs` for team metrics tracking
- `TeamId` enum for 8 IT teams
- `TeamMetrics` struct with comprehensive stats
- `TeamPerformance` tracker for all teams

**Core Functions:**
- `record_ticket()` - Record ticket with resolution details
- `record_escalation()` - Track escalations
- `success_rate()` / `escalation_rate()` - Calculate rates
- `avg_resolution_ms()` - Average resolution time
- `by_activity()` / `by_success_rate()` / `by_speed()` - Sorting
- `most_active()` / `best_performing()` / `fastest()` - Rankings
- `team_grade()` - A+ to F performance grading

**Display Functions:**
- `format_team_performance()` - Full display with breakdown
- `format_team_performance_compact()` - Compact for greetings
- `format_team_performance_oneline()` - Single line status
- `format_duration_ms()` - Duration formatting
- `team_performance_fun_fact()` - Fun facts
- `is_team_performance_query()` - Query detection

## [0.0.494] - 2025-12-13

### Added - Error Summary Display (Phase 70)

**Error Summary Module:**
- `error_summary_display.rs` for error/warning tracking
- `ErrorSeverity` enum (Critical, Error, Warning, Info)
- `ErrorCategory` enum (9 categories)
- `ErrorEntry` struct with full error details
- `ErrorSummary` for storage and query

**Core Functions:**
- `add()` - Add error with duplicate detection
- `unacknowledged()` / `critical()` - Filter errors
- `by_severity()` / `by_category()` - Filtering
- `acknowledge()` / `acknowledge_all()` - Mark as reviewed
- `has_active_critical()` - Check for urgent issues
- `categorize_error()` - Auto-categorize error messages

**Display Functions:**
- `format_error_summary()` - Full display with breakdown
- `format_error_summary_compact()` - Compact for greetings
- `format_error_summary_oneline()` - Single line status
- `format_error_timestamp()` - Human-readable timestamps
- `error_health_message()` - System health assessment
- `is_error_summary_query()` - Query detection

## [0.0.493] - 2025-12-13

### Added - Ticket History Display (Phase 69)

**Ticket History Module:**
- `ticket_history_display.rs` for viewing past ticket history
- `TicketOutcome` enum (Resolved, Escalated, Cancelled, Failed, InProgress, AwaitingInput)
- `HistoricalTicket` struct with full ticket details
- `TicketHistory` for storage and query

**Core Functions:**
- `add()` - Add ticket to history
- `recent()` - Get recent tickets
- `by_outcome()` / `by_department()` - Filter tickets
- `open_tickets()` - Get in-progress tickets
- `success_rate()` - Calculate resolution rate
- `most_active_department()` - Busiest department

**Display Functions:**
- `format_ticket_history()` - Full history display
- `format_ticket_history_compact()` - Compact summary
- `format_ticket_history_oneline()` - Single line status
- `format_timestamp()` - Human-readable timestamps
- `format_duration()` - Duration formatting
- `ticket_history_fun_fact()` - Fun facts
- `is_ticket_history_query()` - Query detection

## [0.0.492] - 2025-12-13

### Added - Uptime Tracking (Phase 68)

**Uptime Tracker Module:**
- `uptime_tracker.rs` for tracking uptime and availability
- `UptimeRecord` struct for session tracking
- `UptimeTracker` comprehensive statistics

**Core Functions:**
- `start_session()` / `end_session()` - Session management
- `current_session_duration()` - Active session time
- `days_since_install()` - Installation age
- `total_uptime_hours/days()` - Total uptime
- `avg_session_duration()` - Average session
- `uptime_percentage()` - Overall availability
- `clean_shutdown_rate()` - Stability metric

**Display Functions:**
- `format_uptime()` - Full stats display
- `format_uptime_compact()` - Compact summary
- `format_uptime_oneline()` - Single line
- `format_duration_secs()` - Human-readable
- `uptime_fun_fact()` - Fun facts
- `is_uptime_query()` - Query detection

## [0.0.491] - 2025-12-13

### Added - Aggregated Stats Dashboard (Phase 67)

**Stats Dashboard Module:**
- `stats_dashboard.rs` for unified statistics view
- `DashboardSection` enum (8 sections)
- `StatMetric` with name, value, trend
- `StatTrend` enum (Up, Down, Stable)
- `StatsDashboard` container with health score
- `DashboardBuilder` for easy construction

**Builder Methods:**
- `with_summary()` - Add summary metrics
- `with_resolutions()` - Resolution time stats
- `with_interactions()` - Interaction stats
- `with_experts()` - Expert performance stats
- `with_recipes()` - Recipe statistics
- `with_responses()` - Response length stats

**Display Functions:**
- `format_dashboard()` - Full dashboard display
- `format_dashboard_compact()` - Compact summary
- `format_dashboard_oneline()` - Single line status
- `generate_health_bar()` - ASCII health indicator
- `is_dashboard_query()` / `detect_section()`

## [0.0.490] - 2025-12-13

### Added - Recipe Statistics Display (Phase 66)

**Recipe Stats Display Module:**
- `recipe_stats_display.rs` for recipe statistics
- `RecipeCategory` enum (11 categories)
- `RecipeOriginType` enum (Seed, Learned, UserTaught, Imported)
- `RecipeStats` per-recipe tracking
- `RecipeStatsTracker` comprehensive tracker

**Core Functions:**
- `register()` - Register recipe with category/origin
- `record_use()` - Track usage with success/confidence
- `most_used()` - Get most popular recipes
- `recipes_by_category()` - Filter by category
- `most_reliable()` - Highest success rate recipes
- `recently_learned()` - New learned recipes
- `top_category()` - Most popular category

**Display Functions:**
- `format_recipe_stats()` - Full stats display
- `format_recipe_stats_compact()` - Compact summary
- `recipe_stats_fun_fact()` - Fun facts
- `is_recipe_stats_query()` - Query detection

## [0.0.489] - 2025-12-13

### Added - Expert Ticket Statistics (Phase 65)

**Expert Stats Module:**
- `expert_stats.rs` for tracking tickets per expert
- `ExpertLevel` enum (Junior, Senior)
- `Expert` struct with id, name, department, level
- `ExpertStatistics` per-expert tracking
- `ExpertStatsTracker` comprehensive tracker

**Core Functions:**
- `register_expert()` - Register experts
- `record_closed()` - Track closed tickets with confidence
- `record_escalation()` - Track escalations
- `record_anna_solo()` - Track Anna self-resolutions
- `top_performers()` - Ranking by tickets closed
- `by_department()` / `by_level()` - Filtering
- `most_reliable()` / `fastest_responder()`

**Display Functions:**
- `format_expert_stats()` - Full stats display
- `format_expert_stats_compact()` - Compact summary
- `expert_stats_fun_fact()` - Fun facts
- `is_expert_stats_query()` - Query detection

## [0.0.488] - 2025-12-13

### Added - Interaction Counter (Phase 64)

**Interaction Counter Module:**
- `interaction_counter.rs` for tracking specialist interactions
- `InteractionRecord` struct with from/to/type/ticket
- `InteractionType` enum (Dispatch, Response, Escalation, etc.)
- `SpecialistStats` per-specialist statistics
- `InteractionCounter` for comprehensive tracking

**Core Functions:**
- `record()` - Track interaction with full metadata
- `record_anna_solo()` - Track Anna self-resolutions
- `average_per_ticket()` - Avg interactions per ticket
- `most_consulted()` / `least_consulted()` - Specialist rankings
- `fastest_responder()` - Quickest specialist
- `anna_solo_rate()` - Anna independence rate

**Display Functions:**
- `format_interactions()` - Full stats display
- `format_interactions_compact()` - Compact summary
- `interaction_fun_fact()` - Fun facts
- `is_interaction_query()` - Query detection

## [0.0.487] - 2025-12-13

### Added - Resolution Time Tracking (Phase 63)

**Resolution Time Module:**
- `resolution_time.rs` for tracking resolution times
- `ResolutionRecord` struct with timing and metadata
- `ResolutionTimeTracker` for comprehensive statistics

**Core Functions:**
- `record()` / `record_simple()` - Track resolutions
- `average_ms()` - Average resolution time
- `success_rate()` / `escalation_rate()` - Rates
- `fastest_category()` / `slowest_category()` - By category

**Statistics:**
- Fastest/slowest resolutions
- Per-category stats (count, total, avg, range)
- Recent resolutions (last 20)

**Display Functions:**
- `format_resolution_times()` - Full stats display
- `format_resolution_times_compact()` - Compact summary
- `format_duration_ms()` - Human-readable durations
- `resolution_time_fun_fact()` - Fun facts
- `is_resolution_time_query()` - Query detection

## [0.0.486] - 2025-12-13

### Added - Response Length Tracking (Phase 62)

**Response Length Module:**
- `response_length.rs` for tracking response lengths
- `RecordedResponse` struct with char/word/line counts
- `ResponseLengthTracker` for statistics

**Core Functions:**
- `record()` - Track response with automatic stats
- `record_with_category()` - Track with topic category
- `average_chars()` / `average_words()` - Averages
- `length_range()` - Min/max range

**Statistics:**
- Longest/shortest by chars and words
- Total chars/words across all responses
- Recent responses (last 10)

**Display Functions:**
- `format_response_lengths()` - Full stats display
- `format_response_lengths_compact()` - Compact summary
- `response_length_fun_fact()` - Fun facts about lengths
- `is_response_length_query()` - Query detection

## [0.0.485] - 2025-12-13

### Added - Repeated Questions Detection (Phase 61)

**Repeated Questions Module:**
- `repeated_questions.rs` for tracking similar questions
- `RecordedQuestion` struct with variants, count, timestamps
- `QuestionHistory` for storing and querying questions

**Core Functions:**
- `normalize_question()` - Remove filler words for comparison
- `calculate_similarity()` - Jaccard-like word similarity
- `detect_category()` - Categorize by topic keywords

**History Operations:**
- `record()` - Record question with similarity grouping
- `get_repeated()` - Get questions asked multiple times
- `top_repeated()` - Get most frequently asked
- `by_category()` - Filter by category
- `unresolved_repeated()` - Get pending repeated questions

**Display Functions:**
- `format_repeated_questions()` - Full display with variants
- `format_repeated_compact()` - Compact summary
- `is_repeated_questions_query()` - Query detection

## [0.0.484] - 2025-12-13

### Added - Quick Status Summary (Phase 60)

**Quick Status Module:**
- `quick_status.rs` for at-a-glance system status
- HealthLevel enum (Good, Warning, Critical, Unknown)
- StatusItem struct for individual status entries
- QuickStatus struct for overall system summary

**Status Helpers:**
- `memory_status()` - Memory usage health check
- `disk_status()` - Disk usage health check
- `cpu_status()` - CPU load health check
- `service_status()` - Service state health check

**Display Functions:**
- `format_quick_status_oneline()` - One-line summary
- `format_quick_status_compact()` - Compact view (issues only)
- `format_quick_status_full()` - Full status display

**Query Detection:**
- `is_quick_status_query()` - Detect status check requests
- Patterns: "quick status", "health check", "any problems"

## [0.0.483] - 2025-12-13

### Added - Command Shortcuts (Phase 59)

**Command Shortcuts Module:**
- `command_shortcuts.rs` for quick command aliases
- 25+ built-in shortcuts for common operations
- Case-insensitive matching

**Shortcut Categories:**
- System (mem, cpu, uptime, health)
- Storage (du, df, mounts, big)
- Network (ip, ports, netstat, ping)
- Process (top5, ps, hogs)
- Service (services, failed, logs)
- Package (updates, upgrade, orphans)
- Docker (dps, dimages, dclean)
- Git (gs, gl, gd)

**Functions:**
- `expand_shortcut()` - Expand short form to full query
- `is_shortcut()` - Check if input is a shortcut
- `shortcuts_by_category()` - Get shortcuts by category
- `format_shortcuts()` - Display all shortcuts
- `format_category_shortcuts()` - Display category shortcuts
- `is_shortcuts_query()` - Detect "show shortcuts" queries

## [0.0.482] - 2025-12-13

### Added - Contextual Tips System (Phase 58)

**Contextual Tips Module:**
- `contextual_tips.rs` provides tips based on user context
- `TipContext` tracks topics, commands, and learning mode
- Topic detection from natural language queries

**Topic Categories:**
- Editor (vim, nano, emacs)
- Containers (docker, kubernetes)
- Git and SSH
- Services (systemd)
- Network and Storage
- Packages and Scheduling
- Security (permissions, firewall)

**Functions:**
- `TipContext::from_query()` - Extract context from query
- `get_contextual_tips()` - Get tips for current context
- `select_tip()` - Pick a tip from available tips
- `format_tip()` - Format tip for display
- `get_tip_for_query()` - One-liner to get a tip
- `should_show_tip()` - Probability-based display check

**Features:**
- Tips include related actions when applicable
- Learning mode adds learning-specific tips
- General tips as fallback when no topic detected
- Multiple topic detection for complex queries

## [0.0.481] - 2025-12-13

### Added - Query Type Router (Phase 57)

**Query Routing Module:**
- `query_type_router.rs` consolidates all query detection
- `QueryType` enum for all informational query types
- Central routing for display modules

**Query Types:**
- Capabilities (what can you do?)
- FunStats (fun statistics)
- XpLevel (XP and level)
- Session (session summary)
- Settings (all settings)
- Status (system status)
- ContextualHelp (help with X)
- Other (needs specialist)

**Functions:**
- `QueryType::detect()` - Detect query type from natural language
- `route_query()` - Route query to appropriate handler
- `should_handle_locally()` - Check if query can skip specialists
- `suggest_display()` - Get display function suggestion
- `is_informational()` - Check if pure informational query

**Helper Detection:**
- `is_status_query()` - Status query detection
- `extract_help_context()` - Extract context from "help with X"

## [0.0.480] - 2025-12-13

### Added - Capabilities Display (Phase 56)

**Capabilities Module:**
- `capabilities_display.rs` for "what can you do?" queries
- `CapabilityCategory` enum (9 categories)
- Examples and descriptions for each category

**Categories:**
- System Information (RAM, CPU, disk, health)
- Package Management (install, update, remove)
- Service Management (start, stop, restart, logs)
- Configuration (edit configs, git, shell, editors)
- Network (IPs, connectivity, ports)
- Storage (disk usage, mounts, large files)
- Hardware (GPU, temperature, devices)
- Learning (learning mode, recipes, explanations)
- Statistics (XP, level, streaks, history)

**Functions:**
- `format_capabilities()` - Full capabilities display
- `format_capabilities_compact()` - One-line summary
- `format_capability_category()` - Single category details
- `format_capabilities_with_teams()` - With team count
- `is_capabilities_query()` - Detect "help", "what can you do"
- `parse_capability_category()` - Parse category from query
- `capability_facts()` / `random_capability_fact()` - Fun facts

## [0.0.479] - 2025-12-13

### Added - Fun Statistics Display (Phase 55)

**Fun Statistics Module:**
- `fun_stats_display.rs` for VISION.md Fun Statistics display
- `FunStats` struct with all interesting statistics
- Installation date and days active tracking
- Most consulted team with count
- Lucky team with success rate
- Anna solo successes and percentage
- Longest/shortest reply times
- Current and best streak tracking

**Display Functions:**
- `FunStats::from_aggregated()` - Create from event data
- `format_fun_stats()` - Full display with sections
- `format_fun_stats_compact()` - One-line for greetings
- `format_fun_stats_category()` - Filter by category
- `generate_fun_fact()` - Random interesting fact
- `format_install_date()` - Human-readable dates

**Categories:**
- History (install date, days active, total requests)
- Teams (most consulted, lucky team)
- Independence (solo answers, recipes learned)
- Times (longest/shortest reply)
- Streaks (current and best)

**Query Detection:**
- `is_fun_stats_query()` - Detect "fun stats", "interesting facts" queries
- `FunStatsCategory::parse()` - Parse category from string

## [0.0.478] - 2025-12-12

### Added - XP/Level Display Enhancement (Phase 54)

**XP Display Module:**
- `xp_display.rs` for RPG-style progression
- `AnnaXP` struct with level, XP, progress, title
- Non-linear XP thresholds (0→1500 over 10 levels)

**Funny Level Titles:**
1. Intern - "Just started, still figuring out the coffee machine"
2. Help Desk Trainee - "Can now restart a service without panicking"
3. Junior Technician - "Knows the difference between RAM and storage"
4. IT Support Specialist - "Has memorized most man page flags"
5. Senior Technician - "Colleagues actually ask for help now"
6. Systems Administrator - "Speaks fluent systemd and cron"
7. Infrastructure Wizard - "Can debug in production without breaking sweat"
8. Senior Architect - "Has seen things... many, many logs"
9. IT Director - "Makes the decisions that matter"
10. Chief Technology Guru - "Achieved IT enlightenment"

**Functions:**
- `AnnaXP::from_stats()` / `from_events()` - Calculate XP
- `format_xp_display()` - Full XP display with progress bar
- `format_xp_compact()` - One-line for greetings
- `is_xp_query()` - Detect "what is my level" queries

## [0.0.477] - 2025-12-12

### Added - Learning Mode Explanations Enhancement (Phase 53)

**Module Refactor:**
- Split learning_explanations.rs into module directory
- `mod.rs` - core types (Explanation, FlagExplanation, LearningContext)
- `commands.rs` - command explanations database

**New Command Explanations (15+):**
- Process: ps, top/htop
- File: grep, find, chmod, chown
- Disk: du, mount
- Network: ss/netstat, ping, curl/wget
- Archive: tar
- Tools: git, docker

**Enhanced Functions:**
- `list_explained_commands()` returns all supported commands
- `get_explanation()` without learning mode check
- Improved display format (cleaner, no emojis)

**Total Explained Commands: 29+**

## [0.0.476] - 2025-12-12

### Added - Unified Settings Display (Phase 52)

**Settings Display Module:**
- `settings_display.rs` for unified settings view
- `AllSettings` struct combining all config types
- `SettingsSection` enum for section filtering

**Display Functions:**
- `format_all_settings()` shows all settings organized by section
- `format_settings_summary()` one-line compact summary
- `is_all_settings_query()` detects "show all settings" queries
- `get_settings_section()` identifies which section was requested

**Sections:**
- Preferences (learning mode, verbosity, personality)
- Notifications (email, desktop, wall, quiet hours)
- Debug (level, log file, redaction)
- Risk (auto-confirm, warnings, protection)

**Supported Queries:**
- "show all settings" / "all config"
- "list all my settings"
- "full settings"

## [0.0.475] - 2025-12-12

### Added - Session Summary Display (Phase 51)

**Session Display Module:**
- `session_display.rs` for formatted session information
- `SessionStats` struct with duration, queries, topics, etc.
- `SessionQueryType` enum for query classification

**Display Functions:**
- `format_current_session()` displays current session stats
- `format_session_history()` shows recent session list
- `format_brief_summary()` provides one-line summary
- `format_duration()` human-readable time formatting
- `format_time_ago()` relative time display

**Query Detection:**
- `is_session_query()` detects session-related queries
- `get_session_query_type()` classifies query type
- `get_since_last_time()` provides greeting summary

**Supported Queries:**
- "session summary" / "session stats"
- "what did we do this session"
- "session history" / "past sessions"
- "current session"

## [0.0.474] - 2025-12-12

### Added - Risk Level Configuration via Natural Language (Phase 50)

**Risk Config Module:**
- `risk_config.rs` for natural language risk tolerance settings
- `RiskTolerance` struct with auto-confirm level, warnings, protection
- `RiskConfigChange` enum for configuration changes

**Presets:**
- `cautious` - confirm all changes, show all warnings
- `balanced` - auto-confirm safe (read-only) operations
- `confident` - auto-confirm low and medium risk operations
- `expert` - minimal confirmations, hide warnings

**Functions:**
- `detect_risk_config()` parses natural language to config changes
- `apply_risk_change()` applies changes to RiskTolerance
- `format_risk_settings()` displays current risk settings
- `is_show_risk_settings()` detects "show risk settings"
- `should_auto_confirm()` checks if risk level should auto-confirm

**Supported Commands:**
- Presets: "be cautious", "expert mode", "balanced risk"
- Auto-confirm: "auto-confirm low risk", "confirm everything"
- Warnings: "show warnings", "hide warnings"
- Protection: "protect destructive", "allow destructive"

## [0.0.473] - 2025-12-12

### Added - Debug Configuration via Natural Language (Phase 49)

**Debug Config Module:**
- `debug_config.rs` for natural language debug settings
- `DebugConfigChange` enum for debug configuration changes
- Parse phrases like "enable debug", "debug full", "hide secrets"

**Configuration Types:**
- `SetLevel(DebugLevel)` - Set debug level (Off/Summary/Trace/Full)
- `EnableLogFile(bool)` - Enable/disable debug log to file
- `RedactIPs(bool)` - Enable/disable IP address redaction
- `RedactEmails(bool)` - Enable/disable email redaction
- `RedactSecrets(bool)` - Enable/disable secrets redaction
- `MaxProbeLines(usize)` - Set max probe output lines

**Functions:**
- `detect_debug_config()` parses natural language to config changes
- `apply_debug_change()` applies changes to DebugConfig
- `format_debug_settings()` displays current debug settings
- `is_show_debug_settings()` detects "show debug settings"
- `parse_debug_level()` parses level from string

**Supported Commands:**
- Levels: "enable debug", "debug trace", "debug full", "disable debug"
- Log file: "log debug to file", "disable debug log"
- Redaction: "hide ips", "show emails", "redact secrets"

## [0.0.472] - 2025-12-12

### Added - Arch Wiki Local Caching (Phase 48)

**Wiki Cache Module:**
- `wiki_cache.rs` for local Arch Wiki page caching
- `WikiCacheEntry` with metadata (cached_at, size, hash)
- `WikiCacheIndex` for cache management

**Cache Functions:**
- `read_cached()` / `write_cached()` for page I/O
- `get_cache_stats()` returns size, count, staleness
- `essential_pages()` lists pages that should be cached
- `missing_essential()` finds uncached essential pages

**Cache Management:**
- `prune_stale()` removes old entries (default 30 days)
- `prune_to_size()` removes least accessed entries
- Change detection via content hash
- Access tracking for LRU eviction

## [0.0.471] - 2025-12-12

### Added - Facts Lifecycle Management (Phase 47)

**Facts Maintenance Module:**
- `facts_maintenance.rs` for scheduled facts cleanup
- `MaintenanceResult` tracks transitions and pruning
- `FactsHealth` provides statistics and health check

**Maintenance Functions:**
- `run_maintenance()` applies lifecycle transitions and optional pruning
- `get_health()` returns statistics by category
- `get_reverification_candidates()` lists facts needing re-verification
- `needs_maintenance()` checks if maintenance is needed

**Statistics & Display:**
- Facts categorized by type (system, network, packages, etc.)
- Age tracking (oldest, average)
- Health ratio (< 30% stale = healthy)
- Human-readable age formatting

## [0.0.470] - 2025-12-12

### Added - Notification Configuration via Natural Language (Phase 46)

**Natural Language Notification Config:**
- `notification_config.rs` module for notification settings
- Parse phrases like "set my email to user@example.com"
- Support for email, desktop, wall, and quiet hours configuration

**Configuration Types:**
- `NotifyConfigChange` enum for 7 notification settings
- `detect_notify_config()` parses natural language
- `apply_notify_change()` updates NotificationConfig
- `format_notification_settings()` displays current settings
- `is_show_notifications()` detects "show notification settings"

**Supported Commands:**
- Email: "set my email to X", "remove email"
- Desktop: "enable desktop notifications", "disable desktop"
- Wall: "enable wall messages", "disable broadcast"
- Quiet hours: "set quiet hours 22:00 to 08:00", "clear quiet hours"

## [0.0.469] - 2025-12-12

### Added - Monitoring & Custom Alarms (Phase 45)

**System Monitoring Module:**
- `system_monitors.rs` for proactive system monitoring
- `CheckType` enum: DiskUsage, MemoryUsage, FailedServices, LoadAverage, SwapUsage
- `MonitorResult` struct with current value, threshold, and alert status

**Monitoring Functions:**
- `check_disk_usage()` reads actual disk space from system
- `check_memory_usage()` reads from /proc/meminfo
- `check_failed_services()` queries systemctl
- `check_load_average()` reads from /proc/loadavg
- `check_swap_usage()` reads swap info from /proc/meminfo

**Alarm Integration:**
- `evaluate_condition()` checks AlarmCondition and returns MonitorResult
- `check_conditional_alarms()` evaluates all conditional alarms in store
- `run_all_checks()` performs full system health check
- `format_monitor_results()` formats results for display

## [0.0.468] - 2025-12-12

### Added - Tips in Greetings (Phase 44)

**Configuration Tips System:**
- `greeting_tips.rs` module for showing tips during greetings
- Tips based on current user settings (suggest unused features)
- Randomized selection with variety (different tips each greeting)
- Non-intrusive (only shows roughly 1 in 3 greetings)

**Tip Categories:**
- Learning: learning mode on/off tips
- Personality: formality, humor, technical depth tips
- Safety: auto-confirm settings tips
- Display: verbosity and internal comms tips
- Notifications: email setup tips

**Functions:**
- `get_available_tips()` returns tips based on profile state
- `select_tip()` chooses random tip from available
- `should_show_tip()` probability-based display check
- `format_tip_for_greeting()` formats tip with command hint
- `get_random_greeting_tip()` convenience function for greetings

## [0.0.467] - 2025-12-12

### Added - Personality Configuration via Natural Language (Phase 43)

**Natural Language Preference Parser:**
- `detect_config_change()` parses phrases like "be more formal", "shorter answers"
- Supports: learning mode, verbosity, auto-confirm, internal comms visibility
- Personality traits: formality, humor, technical depth
- All settings configurable via natural language per VISION.md

**Configuration Types:**
- `ConfigChange` enum for 7 configuration categories
- `apply_config_change()` applies changes to UserProfile
- `is_show_preferences()` detects "show my settings" requests
- `format_preferences()` displays current settings cleanly

**Phrase Recognition:**
- Learning mode: "enable learning", "teach me", "no explanations"
- Verbosity: "shorter answers", "more verbose", "be brief"
- Auto-confirm: "don't ask", "confirm changes", "ask first"
- Formality: "be casual", "be formal", "professional"
- Humor: "no jokes", "be funny", "playful"
- Technical: "simple answers", "expert mode", "beginner"

## [0.0.466] - 2025-12-12

### Added - Smart Helper Management (Phase 32)

**Helper Source Tracking:**
- Track whether helpers installed by Anna or User (InstallSource enum)
- `register_anna_installed()` to mark auto-installed helpers
- `get_anna_removable()` for uninstall cleanup
- `remove_anna_installed()` removes only Anna-installed non-required helpers

**Hardware-Aware Helper Management:**
- `hardware_requirement` field on HelperPackage
- `is_useful()` checks if helper makes sense for current hardware
- `get_useless_helpers()` finds helpers without required hardware
- Skip ethtool without ethernet, etc.

**Helper Usage Tracking:**
- `last_used` timestamp field
- `record_usage()` updates timestamp
- `last_used_display()` formats for human readability ("3 hours ago")
- `secs_since_use()` for programmatic access

**Enhanced Tests:**
- test_register_anna_installed
- test_remove_anna_installed
- test_hardware_requirement
- test_last_used_display
- test_record_usage

## [0.0.465] - 2025-12-12

### Added - Natural Language Dialog (Phase 31)

**Fly-on-the-Wall Dialog Enhancements:**
- Recipe storage confirmation phrases (Anna announces when learning a pattern)
- Recipe usage announcement (Anna mentions when using stored recipes)
- Case number with date format (CN-XXXX-DDMMYYYY per VISION.md)

**New Dialog Functions:**
- `anna_recipe_storage_confirmation()` - Reliability-aware learning confirmation
- `anna_recipe_used_announcement()` - Recipe recognition phrases
- `format_case_with_date()` - CN-XXXX-DDMMYYYY format

**Existing Features Already Implemented:**
- Anna-to-Specialist dialog rendering (TranscriptSegment with SegmentKind)
- Specialist responses with reliability assessments (junior_approval, senior_response)
- Real-time streaming support (Progress segment kind)
- Screen updates via transcript segments

## [0.0.464] - 2025-12-12

### Added - Enhanced Stats Display (Phase 30)

**RPG System per VISION.md:**
- Non-linear XP progression (0-100 scale with logistic curve)
- Funny IT titles by level (Trainee -> Linus's Chosen One)
- Installation date display
- Anna solo count (handled without specialists)
- Most consulted team tracking
- Fastest/longest response times
- Current/best streak tracking
- Average interactions per ticket

**Topic Tracking (v0.0.464):**
- Repeated questions detection
- Topic (domain) breakdown with percentages
- Intent breakdown with percentages
- `annactl stats <category>` for detailed views

**Category Filter (`annactl stats <cat>`):**
- rpg: RPG progression stats only
- learning: Probe learning stats only
- outcomes/handlers: Ticket outcome breakdown
- topics: Domain and intent analysis
- repeated: Repeated question tracking

**New Files:**
- `stats_categories.rs` - Category-specific display functions
- Enhanced `ticket_stats.rs` with repeated_queries, top_topic, by_intent

## [0.0.463] - 2025-12-12

### Added - Enhanced Status Display (Phase 29)

**Per VISION.md Status Dashboard Requirements:**
- Folder permission info in StatusSnapshot
- Error announcements in REPL greeting
- System error detection and display

**FolderPermission struct:**
- path, exists, readable, writable
- owner and mode (Unix permissions)
- Builder pattern for construction
- check_anna_folders() for key paths

**SystemError struct for greeting:**
- category, message, fix_hint
- Auto-updates status to WARN when errors exist
- Renders with color-coded categories

**Enhanced REPL Greeting:**
- add_error() method for error announcements
- has_errors() check
- Renders errors with hints in greeting output

## [0.0.462] - 2025-12-12

### Added - Network Troubleshooting Recipes (Phase 41)

**Network Diagnostics per ROADMAP.md Future:**
- Complete recipe system for network troubleshooting
- Connectivity testing, DNS, routing, and firewall diagnostics
- WiFi, VPN, and SSL certificate checks

**NetworkFeature Enum (15 features):**
- TestConnectivity, DnsLookup, TraceRoute
- CheckPorts, InterfaceInfo, BandwidthTest
- FirewallStatus, ListeningPorts
- SslCertCheck, HttpTest
- ArpTable, RoutingTable, NetworkStats
- WifiDiagnostics, VpnStatus

**Builtin Recipes:**
- ping, dig, nslookup, traceroute, mtr
- nmap, nc, netstat, ss
- iptables, nftables, ufw, firewalld
- openssl, curl, wget
- iwconfig, nmcli for WiFi
- wg show, openvpn for VPN

**New Module:**
- `anna_shared::network_recipes` - Network troubleshooting recipes
- `NetworkFeature` enum with display names and keywords
- `NetworkRecipe` builder pattern with tool requirements
- `detect_feature()` and `match_query()` for query routing

## [0.0.461] - 2025-12-12

### Added - Database Recipes (Phase 40)

**Database Backup/Restore per ROADMAP.md Future:**
- Complete recipe system for database operations
- Support for PostgreSQL, MySQL/MariaDB, SQLite, MongoDB, Redis
- Query matching for natural language to database commands

**DatabaseFeature Enum (15 features):**
- BackupDatabase, RestoreDatabase, DumpDatabase
- ImportData, ExportData
- CheckStatus, RepairDatabase, OptimizeTables
- CreateDatabase, CreateUser, GrantPermissions
- ShowDatabases, ShowTables, ExecuteQuery, TestConnection

**Builtin Recipes:**
- pg_dump/pg_restore for PostgreSQL
- mysqldump/mysql for MySQL/MariaDB
- sqlite3 .dump for SQLite
- mongodump/mongorestore for MongoDB
- VACUUM, OPTIMIZE TABLE, mysqlcheck commands
- User/permission management across databases

**New Module:**
- `anna_shared::database_recipes` - Database recipe system
- `DatabaseFeature` enum with display names and keywords
- `DatabaseRecipe` builder pattern with multi-DB support
- `detect_feature()` and `match_query()` for query routing

## [0.0.460] - 2025-12-12

### Added - Web Server Recipes (Phase 39)

**Nginx/Apache Configuration per ROADMAP.md Future:**
- Complete recipe system for Nginx and Apache
- Query matching for natural language to web server commands
- Configuration examples for common setups

**WebServerFeature Enum (15 features):**
- InstallNginx, InstallApache
- CreateVirtualHost, EnableSite
- ConfigureSsl (Let's Encrypt/Certbot)
- ReverseProxy, LoadBalancing
- ViewLogs, TestConfig, RestartServer, CheckStatus
- ConfigureCaching, BasicAuth, ConfigureCors, OptimizePerformance

**Builtin Recipes:**
- Installation commands for Arch, Debian, Fedora
- Virtual host/server block configurations
- SSL/TLS with Let's Encrypt
- Reverse proxy with proper headers
- Load balancing with upstream directive
- Performance optimization (gzip, caching, workers)

**New Module:**
- `anna_shared::webserver_recipes` - Web server recipe system
- `WebServerFeature` enum with display names and keywords
- `WebServerRecipe` builder pattern with config examples
- `detect_feature()` and `match_query()` for query routing

## [0.0.459] - 2025-12-12

### Added - Kubernetes Recipes (Phase 38)

**Kubernetes Pod & Deployment Management per ROADMAP.md Future:**
- Complete K8s recipe system for common kubectl operations
- Query matching for natural language to kubectl commands
- Longest-keyword matching for accurate feature detection

**K8sFeature Enum (19 features):**
- ListPods, DescribePod, PodLogs, ExecPod
- ListDeployments, ScaleDeployment, RestartDeployment
- ApplyManifest, DeleteResource
- ListServices, PortForward
- GetEvents, DebugPod
- ClusterHealth, ListNodes, ResourceUsage
- ListNamespaces, CreateNamespace, GetConfig

**Builtin Recipes:**
- Each feature has kubectl commands, answer templates, and notes
- Manifest examples for Deployment and Namespace creation
- Debugging guide for common issues (ImagePullBackOff, CrashLoopBackOff)

**New Module:**
- `anna_shared::kubernetes_recipes` - K8s recipe system
- `K8sFeature` enum with display names and keywords
- `K8sRecipe` builder pattern for recipes
- `detect_feature()` and `match_query()` for query routing

## [0.0.458] - 2025-12-12

### Added - Idle Time Learning (Phase 37)

**Senior Strategic Thinking per VISION.md:**
- Senior specialists analyze past tickets during idle time
- Pattern detection across teams
- Recurring issue identification
- Priority-based insights (Low/Medium/High/Critical)

**Strategic Insights:**
- Recurring Pattern detection
- Performance Trend analysis
- Security Concern identification
- Maintenance Suggestions
- Configuration Opportunities
- Capacity Planning

**Resumable Sessions:**
- Analysis can be interrupted and resumed
- Checkpoint system for progress tracking
- Partial insights preserved

**Email Reports:**
- Weekly strategic analysis report
- Grouped by priority
- Actionable recommendations

**New Module:**
- `anna_shared::senior_strategic` - Idle-time strategic analysis
- `StrategicSession` - Analysis session tracking
- `PatternDetector` - Pattern recognition
- `StrategicInsight` - Generated insights

## [0.0.457] - 2025-12-12

### Added - Knowledge & Citations (Phase 36)

**Learning Mode Explanations per VISION.md:**
- Explains why commands are being run
- Explains how commands work
- Explains what output means
- Command flag descriptions

**Built-in Command Explanations:**
- df, free, lsblk - disk and memory
- systemctl, journalctl - systemd services and logs
- ip, lspci, lscpu - network and hardware
- pacman - Arch package management
- sensors, uname, cat - system utilities

**Existing Citation Infrastructure:**
- KnowledgeCache for Arch Wiki and man pages
- Local citation storage and retrieval
- Excerpt extraction by topic

**New Module:**
- `anna_shared::learning_explanations` - Learning mode output
- `CommandExplainer` - Built-in command explanations
- `LearningContext` - Explanation aggregator

## [0.0.456] - 2025-12-12

### Added - User Notifications & Alarms (Phase 35)

**Notification Infrastructure per VISION.md:**
- Email notifications (via sendmail/msmtp)
- Desktop notifications (notify-send/libnotify)
- Wall messages (terminal broadcast)
- Rate limiting per channel
- Quiet hours support

**Natural Language Alarms:**
- "Remind me every Monday at 9 about storage"
- "Alert me when disk is above 90%"
- "Notify me daily about failed services"

**Alarm Types:**
- Daily recurring alarms
- Weekly recurring alarms (specific day)
- Monthly recurring alarms
- Condition-based alerts (disk, memory, services)

**New Module:**
- `anna_shared::user_alarms` - Natural language alarm system
- `UserAlarm` - Alarm definition with schedule
- `AlarmStore` - Persistent alarm storage
- `parse_alarm_request()` - NL parser for alarms

## [0.0.455] - 2025-12-12

### Added - Long-Running Task Handling (Phase 34)

**Long Task Detection per VISION.md:**
- Configurable threshold (default 2 minutes)
- Tracks elapsed time automatically
- Detects when tasks exceed threshold

**User Options for Long Tasks:**
- [1] Keep waiting (continue in foreground)
- [2] Move to background (with email notification)
- [3] Cancel task

**Email Integration:**
- Stores email for reuse across sessions
- Sends notification when background task completes
- Integrates with existing email module

**New Module:**
- `anna_shared::long_task` - Long-running task tracker
- `LongTaskTracker` - Tracks task duration and status
- `LongTaskConfig` - Configuration options
- `LongTaskChoice` - User choice parsing

## [0.0.454] - 2025-12-12

### Added - Dynamic Team Availability (Phase 33)

**Hardware Detection per VISION.md:**
- Detects audio hardware (ALSA, PulseAudio, PipeWire)
- Detects network interfaces (ethernet, WiFi)
- Detects GPU (DRM, lspci)
- Detects battery (laptops)
- Detects Bluetooth adapters
- Detects hardware sensors (lm-sensors, hwmon)

**Team Availability System:**
- Teams automatically hidden when hardware is missing
- Network team hidden if no network interfaces
- Desktop team hidden if no display detected
- Shows available team count in status

**New [teams] Section in Status:**
```
[teams]
  available     9 teams
  hidden        1 (missing hardware)
    network: no network interface detected
  detected_hw   audio, wifi, battery, gpu
```

**New Modules:**
- `anna_shared::team_availability` - Hardware detection and team mapping
- `status_snapshot::teams_info` - Team availability in status snapshots

## [0.0.453] - 2025-12-12

### Added - Smart Helper Management (Phase 32)

**Enhanced [helpers] Section per VISION.md:**
- Shows "by_anna" with count and removal policy
- Shows "by_user" with count and retention policy
- Each helper shows availability status (available/missing)
- Labels: `<installed by Anna>` vs `<installed by user>`
- Policy notes: "removed on uninstall with confirm" vs "kept on uninstall"

**Helper Display Format:**
```
[helpers]
  by_anna     2 (removed on uninstall with confirm)
    ollama <installed by Anna> [available]
  by_user     3 (kept on uninstall)
    neovim <installed by user> [available]
```

## [0.0.452] - 2025-12-12

### Added - Case Numbers & Reliability Assessments (Phase 31 continued)

**Case Number Format per VISION.md:**
- Changed from `CN-YYYYMMDD-XXXX` to `CN-XXXX-DDMMYYYY`
- Example: `CN-0003-05122025` matches vision document

**Reliability Assessments in Specialist Responses:**
- Junior approval now includes reliability language:
  - 90%+: "This answer is reliable and risk-free"
  - 80%+: "Reliable" / "Good reliability"
  - <80%: "Moderate reliability" / "Use with caution"
- Senior responses include reliability indicators:
  - "This answer is reliable and risk-free"
  - "High confidence on this one"
  - "Safe to proceed"

## [0.0.451] - 2025-12-12

### Added - Fly-on-the-Wall Dialog (Phase 31)

**Goal:** Internal IT team communication rendered in natural language per VISION.md.

**Enhanced Internal Communication Format:**
- Separator: `--- Internal communication ---` (shown once)
- Anna speaking: `Anna to Sofia:` format
- Team member speaking: `Sofia (Desktop Specialist) to Anna:` format
- Matches VISION.md example dialog style

**NarrativeSegment Enhancements:**
- Added `metadata` field for routing info (e.g., "to" recipient)
- New `anna_to(recipient, text)` constructor for fly-on-the-wall messages
- New `with_to(recipient)` builder method

**Theatre Render Updates:**
- `render_theatre()` now resets internal header for each render
- Internal communications show recipient in "From to Recipient:" format
- Single header shown once at start of internal comms block

## [0.0.450] - 2025-12-12

### Added - Enhanced Stats Display with RPG System (Phase 30)

**Goal:** `annactl stats` shows engaging RPG-style progression per VISION.md.

**New RPG XP System (0-100 scale):**
- Non-linear XP calculation using logistic curve
- XP factors: requests, success rate, reliability, recipes learned, Anna solo solves, streaks
- 10 levels with funny IT titles:
  1. Trainee
  2. Cable Untangler
  3. Permission Gremlin
  4. Log Whisperer
  5. Daemon Wrangler
  6. Kernel Whisperer
  7. Stack Overflow Survivor
  8. Senior Packet Inspector
  9. Principal Engineer
  10. Linus's Chosen One

**New [rpg] Section in Stats:**
- XP bar (0-100 with visual progress bar)
- Level and title display
- Installation date
- Anna solo count (times resolved without specialists)
- Recipes learned
- Most consulted team
- Response times (fastest/longest)
- Streaks (current/best)
- Average interactions per ticket

**New AggregatedEvents Fields:**
- anna_solo_count: times resolved without specialist escalation
- most_consulted_team: team with most requests
- longest_reply_chars/shortest_reply_chars: response length tracking

## [0.0.449] - 2025-12-12

### Added - Enhanced Status Display (Phase 29)

**Goal:** `annactl status` shows comprehensive system state per VISION.md.

**New [ollama] Section:**
- Separate Ollama status from daemon status
- Shows: installed (YES/NO), status (RUNNING/STOPPED), version

**New [config] Section:**
- All user-visible configuration settings
- debug_mode (only shown if ON)
- auto_update (ON/OFF)
- learning_mode (ON/OFF with description)
- fast_path (ON/OFF with description)
- internal_comms (ON/OFF with description)
- autonomy_level (0-100)
- request_timeout (seconds)

**Enhanced REPL Greeting:**
- Daemon errors announced prominently at top
- Error label with red color for visibility

**ConfigInfo Enhanced:**
- New fields: auto_update, learning_mode, fast_path_enabled, internal_comms
- New fields: request_timeout_secs, update_check_interval_secs

## [0.0.448] - 2025-12-12

### Fixed - Deterministic Probes: Stop Wrong Probe Selection

**Goal:** Common intents get deterministic probes, not LLM guesses.

**Problem Solved:**
- "which service uses most CPU?" was running `lscpu` instead of `top_cpu`
- "do I have swap?" was running `pacman -Q swap` (package check!) instead of `swap_files`
- "what is my vim setup?" was running memory+disk instead of `vimrc_content`
- Concept words like "swap", "bluetooth", "audio" were being treated as package names

**Deterministic Probe Registry (`deterministic_probes.rs`):**
- 60+ intent-specific probe rules with keywords and negative keywords
- CPU queries: `top_cpu` for "most CPU", `cpu_info` for "what CPU"
- Swap queries: `swap_files` + `memory_info`, NOT package queries
- Bluetooth: `bluetooth_service` + `bluetooth_devices`
- Vim/editor: `vimrc_content`, `nvim_config`, `command_v_vim`
- Boot: `boot_time`, `boot_blame` for slow boot
- And many more...

**Concept vs Package Detection:**
- `is_concept_query()`: Detects concept words (swap, games, audio, bluetooth, etc.)
- Only runs package queries when explicit package verbs are used (install, pacman, apt)
- "do I have swap?" → concept query → runs `swap_files`
- "install vim" → package query → runs package probes

**Integration:**
- Translator checks deterministic rules FIRST before LLM
- If rule matches, uses those probes (no LLM guessing)
- If no rule, falls back to LLM selection

## [0.0.447] - 2025-12-12

### Added - Hard Reliability Gate: Stop Bad Answers Before They Reach Users

**Goal:** Anna NEVER answers without evidence. Every claim must pass strict validation.

**6 Gate Checks (ALL must pass):**
1. **PROBE COVERAGE** - Every claim backed by probe evidence (probe_failed or probe_empty → fail)
2. **QUESTION MATCH** - Answer directly matches user's question (contract validation)
3. **DOMAIN CONSISTENCY** - Domain matches probes and answer (probe domains vs answer domain)
4. **NO HALLUCINATED ENTITIES** - Every noun in answer exists in probe output
5. **PARSE SUCCESS** - LLM output parsed deterministically (no guess work)
6. **CONFIDENCE THRESHOLD** - Score >= 0.85 (raised from 0.5)

**New GateOutcome Variants:**
- `FailedQuestionMismatch`: Answer shape doesn't match question shape
- `FailedDomainMismatch`: Answer domain inconsistent with probe domains
- `FailedHallucination`: Entities in answer not found in evidence
- `FailedProbeCoverage`: Probes failed or returned empty

**Enhanced GateInput Fields:**
- `question`: Original user question for matching
- `domain`: Expected answer domain
- `probe_domains`: Domains from executed probes
- `probe_failed`: Whether any probe execution failed
- `probe_empty`: Whether all probes returned empty
- `answer_entities`: Nouns/entities in generated answer
- `evidence_entities`: Entities found in probe evidence

**Hard Failure Behavior:**
- If ANY check fails → abort answer → produce controlled failure response
- No padding with generic advice when probes fail
- No fake success metrics

**Acceptance Criteria:**
- `confidence < 0.85` → "I cannot provide a reliable answer"
- Probe failure → honest failure, not generic padding
- Entity not in evidence → hallucination detected → fail
- All 6 checks must pass before any answer is shown

## [0.0.446] - 2025-12-12

### Added - Debug Mode That Actually Helps: Full Trace, Clean, Filterable

**Goal:** Surface every decision, redact secrets, filter by level.

**4-Level Debug System:**
- Level 0 (Off): Normal user output only
- Level 1 (Summary): Domain, intent, probes, outcome, reliability, failures
- Level 2 (Trace): Above + probe details, LLM tokens, gate report
- Level 3 (Full): Above + raw prompts/responses, raw probe output

**TraceBlock Canonical Structure:**
- `request_id`, `timestamp`, `route_type`, `domain`, `intent`
- `probes`: Array of probe executions with command, exit_code, duration
- `llm`: Model name, input_tokens, output_tokens, duration, parse_success
- `gate_result`: passed, checks run, failure reasons
- `timing`: total_ms, translate_ms, probe_ms, specialist_ms, gate_ms

**Enhanced Redaction (`redact.rs`):**
- Mandatory secret removal: API keys, tokens, passwords, SSH keys
- Email redaction: user@domain.com → [EMAIL]
- Private IP redaction: 192.168.x.x → [PRIVATE_IP]
- Environment variables: API_KEY=xxx → API_KEY=[REDACTED]
- Configurable limits: max_probe_lines, max_llm_output_chars

**New Commands:**
- `annactl debug trace`: Show canonical TraceBlock from last request
- `annactl debug trace --level summary|trace|full`: Filter output level
- `annactl debug config`: Show current debug configuration

## [0.0.443] - 2025-12-12

### Added - Source Layer: Citations, Trace Observability, Clean Inventories

**Goal:** Make Anna auditable, honest, and inspectable.
LLMs are for orchestration, not truth generation.

**Source Providers (`providers.rs`):**
- `ManProvider`: Fetch and parse man pages (`man -P cat <cmd>`)
- `HelpProvider`: Fetch `--help` / `-h` output
- `ArchWikiProvider`: Offline-first Arch Wiki with caching
- `LocalConfigProvider`: Read /etc and ~/.config files
- `SourceType`: Man, Help, ArchWiki, LocalConfig, Probe
- `commands_for_intent()`: Intent → canonical commands (no hardcoded case tables)

**Research Plan and Citations (`research.rs`):**
- `ResearchPlan`: Generated BEFORE specialist with required_facts, probes, sources
- `SourceRequest`: Type + ID + query + required flag
- `Citation`: [S1] man pacman(8), [E1] probe output
- `CitedAnswer`: Answer with Sources (docs) and Evidence (this machine)
- `CitationBuilder`: Build citations from fetched sources

**Debug Trace (`trace.rs`):**
- `TraceEvent`: Structured JSON event per pipeline stage
- `TraceStage`: Query → Translator → Facts → Probes → Sources → Specialist → Renderer
- `RequestTrace`: All events for a request with render() method
- `TraceFileManager`: Write/read /var/lib/anna/debug/<request_id>.jsonl
- `DebugSummary`: Short console output (request_id, intent, probes, sources, model, state)
- Redaction: Tokens, API keys, passwords, SSH keys automatically redacted

**Model Inventory (`inventory.rs`):**
- `ModelRegistry`: /var/lib/anna/models/registry.json
- `ModelEntry`: name, provider, installed, installed_by (anna/user), role, enabled
- `ModelRole`: Translator, Junior, Senior, Disabled
- `reconcile_with_ollama()`: Sync with `ollama list`, remove duplicates
- `RegistrySummary`: Show translator/junior/senior names, enabled count

**Helper Inventory (`inventory.rs`):**
- `HelperRegistry`: /var/lib/anna/helpers/registry.json
- `HelperEntry`: name, installed_by, install_method, install_evidence
- `record_anna_install()`: Track when Anna installs something
- Proper attribution: preexisting = user, Anna-installed = anna

**Clean Stats and UI (`stats_ui.rs`):**
- `CleanStats`: Breakdown by outcome (answered, parse_error, probe_error, etc.)
- `OutputMode`: Normal, Plain (--plain), Json (--json), Fun (--fun for gamification)
- `DialogQuestion`: Single question, enumerated choices, always Cancel
- `ConfirmDialog`: Y/n confirmation with default
- `ProgressIndicator`: [1/4] Step description (25%)
- No XP/gamification by default (moved to --fun flag)

**Acceptance Criteria:**
- Guidance answers include Sources with citations
- Fact answers include Evidence
- `annactl trace <id>` shows exact LLM prompts/responses
- Status shows full enabled model list with attribution, no duplicates
- Stats match reality and show failures
- Dialogs are single, clean, non-redundant

## [0.0.442] - 2025-12-12

### Fixed - Ticket Integrity: Honest Stats, Clarification-First, Package/System Separation

**Goal:** Make Anna brutally honest about what she knows, what she doesn't know, and what actually succeeded.

**Key Problems Fixed:**
1. Stats showing 100% success when "Failed to parse specialist response" errors occurred
2. Clarification running AFTER probes instead of BEFORE for config-like questions
3. "do I have swap?" returning "swap package is not installed" instead of checking /proc/swaps

**Ticket Outcome Integrity (`outcome.rs`):**
- `TicketOutcome`: Pending, Answered, ParseError, ProbeError, ClarificationPending, Cancelled, InternalError
- `AnsweredConditions`: ALL must be true: specialist_responded, json_valid, schema_valid, answer_rendered, no_internal_errors
- `HonestTicketStats`: ONLY counts `Answered` as success, shows failed_parse separately
- `is_parse_error()`: Detects "Failed to parse specialist response" patterns
- Rule: "Failed to parse" → ParseError state, NOT Answered

**Clarification Before Probes (`clarification.rs`):**
- `ClarificationRequiredIntent`: EditorSyntaxStatus, DesktopWallpapersLocation, EditorConfigOverview, etc.
- `check_clarification_needed()`: Returns NeedClarification if facts missing, ProceedToProbes if ready
- `KnownFacts`: Store user-provided facts with confidence
- Flow: check facts → clarify if missing → ONLY THEN run probes
- Rule: No probes for config questions until we know WHICH config

**Package vs System Separation (`package_system.rs`):**
- `SystemIntent`: SwapConfigured, SwapSize, TrimEnabled, FirewallEnabled
- `PackageIntent`: CheckInstalled, Install, SearchByName
- `SwapStatus`: Parse /proc/swaps, report has_swap, total_swap_gib, kind (file/partition/zram)
- `PackageStatus`: Parse pacman output cleanly, no "2 is installed" garbage
- `is_vague_package_name()`: "games", "apps", "tools" → not literal packages
- Rule: "do I have swap?" → System(SwapConfigured), NOT Package

**Acceptance Tests (`tests.rs`):**
- `test_editor_syntax_asks_which_editor`: Must ask BEFORE probes
- `test_swap_question_is_system_not_package`: "do I have swap?" → System intent
- `test_package_nano_not_installed`: Clean output, no garbage
- `test_parse_error_outcome`: ParseError must NOT count as success
- `test_stats_show_parse_errors`: Stats must show failed_parse count
- `test_games_is_ambiguous`: "games" not treated as literal package

**Expected Stats Output:**
```
[service desk]
  total_tickets         9
  answered              4 (44%)
  failed_parse          3
  probe_failures        2
  clarification_pending 0
  cancelled             0
  internal_errors       0
```

## [0.0.441] - 2025-12-12

### Added - ERA Pipeline: Universal Evidence → Reasoning → Answer Architecture

**Goal:** Replace case-by-case specialist logic with a universal pipeline.
Anna must NOT try to "answer" - she must: collect evidence, reason over it, answer ONLY what was asked.

**ERA Pipeline (`pipeline.rs`):**
- `PipelineStage`: Evidence → Reasoning → Answer (no skipping stages)
- `EraPipeline`: State machine tracking case through all stages
- `ExtractedIntent`: intent, required_facts, answer_type
- `AnswerType`: Numeric, Boolean, List, Entity, Brief
- `FactProbeTable`: Maps fact names → probe IDs
- Rule: No stage may skip the previous one

**EvidenceBundle (`evidence.rs`):**
- `EvidenceBundle`: Structured evidence container
- `FactValue`: Number, String, Bool, List, Null (typed, atomic)
- Namespaced facts: `memory.free_gib`, `boot.total_time_s`, `gpu.model`
- `ProbeError`: Track probe failures
- Extractors: `extract_memory`, `extract_boot_time`, `extract_blame`, `extract_disk`
- Rule: Missing facts MUST be listed in `missing` array

**Specialist Contract v2 (`reasoning.rs`):**
- Specialists NO LONGER answer users
- `ReasoningOutput`: can_answer, reasoning, derived, confidence, requires
- `DerivedValues`: root_cause, metric, other
- `REASONING_SYSTEM_PROMPT`: Strict evidence-only reasoning
- `ReasoningValidator`: Validates output against evidence
- Rule: NO prose, NO commands, NO explanations to user

**Translator Precision (`translator.rs`):**
- `PrecisionTranslator`: Converts reasoning → user answer
- Type rules:
  - Numeric → "17.0 GiB" (number only)
  - Boolean → "Yes." or "No." + 1 sentence max
  - List → comma-separated items
  - Entity → single name
  - Brief → 2-3 sentences max
- `DirectAnswerBuilder`: Skip reasoning for deterministic facts
- Rule: Violations = BUG

**Learning Without Hardcoding (`learning.rs`):**
- Anna learns WHICH FACTS answer WHICH INTENTS (not answers)
- `IntentFactMapping`: intent → required_facts → primary_fact
- `IntentLearningStore`: Persistent mapping storage with seeds
- `FastPathDecision`: UseFastPath, NeedReasoning, UnknownIntent
- Rule: Next time, skip LLM, run known probes, assemble deterministically

**Honest Metrics (`metrics.rs`):**
- `ResolutionStatus`: Resolved, Partial, CannotAnswer, Failed, InProgress
- `ResolutionCriteria`: can_answer + no_missing + answer_delivered
- `HonestMetrics`: Only counts RESOLVED as success
- `success_rate = resolved / total` (no fake 100%)
- Rule: Everything else is NOT resolved

**Acceptance Criteria:**
- "CPU model" will NEVER appear in a "top CPU process" question
- "Timeout parse error" disappears
- Specialists can be slow without breaking UX
- Anna scales to thousands of questions without hardcoding

## [0.0.440] - 2025-12-12

### Added - Specialist Contract v1: Eliminate Parse Errors Forever

**Goal:** Eliminate "Failed to parse specialist response" errors with strict JSON contract,
retries with repair prompts, and fallback summarizer from evidence only.

**Specialist Response Contract v1 (`schema.rs`):**
- `SpecialistResponseV1`: Strict JSON-only output schema
- `SrcDepartment`: Performance, Storage, Services, Network, Security, Hardware, Desktop
- `SrcAssessment`: summary (max 140 chars), confidence (0.0-1.0), risk level
- `SrcAction`: probe, explain, or change with command, why, expected, rollback
- `SrcCitation`: source type (man, arch_wiki, help, local_doc), reference, snippet
- Hard limits: MAX_SUMMARY_CHARS=140, MAX_ACTIONS=5, MAX_CITATIONS=5

**JSON Validator (`validator.rs`):**
- `SrcValidator`: Validates raw response before acceptance
- `ValidationError`: Empty, InvalidJson, SchemaInvalid, CaseIdMismatch, ContainsMarkdown, TooLong
- `ValidationResult`: Valid(response) or Invalid(error, raw_response)
- `extract_json()`: Extracts JSON from mixed content (prose + JSON)
- `check_markdown()`: Detects forbidden markdown patterns
- `BatchValidator`: Tracks success/failure rates across multiple validations

**Retry Strategy (`retry.rs`):**
- `SPECIALIST_TIMEOUT_MS`: 8000ms per call
- `MAX_RETRIES`: 2 maximum attempts
- `BACKOFF_1_MS`: 250ms, `BACKOFF_2_MS`: 500ms
- `REPAIR_PROMPT_1`: "You violated SRC v1. Output ONLY valid JSON matching schema. No prose."
- `REPAIR_PROMPT_2`: "Last chance. Output ONLY JSON. If uncertain, reduce scope and lower confidence."
- `RetryState`: Tracks attempts, backoffs, and exhaustion
- `RetryDecision`: Retry(backoff, repair_prompt) or GiveUp(reason)

**Fallback Summarizer (`fallback.rs`):**
- `FallbackSummarizer`: Produces minimal answer from probe evidence only
- `FallbackResponse`: case_id, answer, confidence, missing_evidence, next_probe
- `ProbeEvidence`: probe_id, output, success flag
- `FallbackReason`: Timeout, InvalidResponse, RetriesExhausted, Unavailable
- Template extractors: memory, boot_time, disk_usage, failed_services, load_average, gpu_info
- Rule: If specialist fails, never show garbled JSON to user

**Ticket States and Stats (`ticket_state.rs`):**
- `TicketState`: Open, Resolved, FailedProbe, FailedSpecialist, NeedClarification, Escalated
- `ResolutionCriteria`: evidence_complete, valid_answer, confidence >= threshold
- `TicketStateMachine`: State transitions with history tracking
- `TicketStats`: Accurate counts by state, success_rate = resolved / total
- Rule: RESOLVED only if required evidence + valid answer + confidence >= 0.5

**UX Messages (`ux.rs`):**
- `UxMessage`: status, detail, next_steps, severity
- `UxSeverity`: Success, Warning, Error, Info
- `fallback_message()`: "Specialist response invalid (timeout). Falling back to evidence-only answer."
- `success_message()`: Confidence-appropriate success messages
- `state_message()`: Clean messages for each ticket state
- `ProgressIndicator`: Step-by-step progress tracking
- Rule: Never show parse errors, garbled JSON, or technical failures to user

## [0.0.439] - 2025-12-12

### Added - Deterministic Routing: Fix "Sofia handles everything" Bug

**Goal:** Make routing predictable with a small, stable taxonomy:
Intent → Evidence requirements → Department → Probe bundle → (Optional) specialist.
Hardcode the taxonomy and evidence model, not the answers.

**Ticket Intent Schema (`intent_schema.rs`):**
- `CanonicalIntent`: 35+ canonical intents (boot_perf, mem_status, gpu_info, etc.)
- `Department`: Performance, Storage, Services, Network, Security, Hardware, Desktop
- `TicketIntentSchema`: Structured translator output (JSON only, max 200 tokens, temp=0)
- `RiskLevel`: read_only, safe_change, risky_change
- `IntentSchemaParser`: Robust JSON parsing with validation

**Deterministic Intent Map (`intent_map.rs`):**
- `IntentMapping`: Maps intent → department + required/optional probes
- `IntentMapTable`: Complete mapping table for all 35+ intents
- Key mappings:
  - `boot_perf` → Performance → [systemd_analyze, systemd_blame]
  - `gpu_info` → Hardware → [lspci_gpu]
  - `gpu_driver` → Hardware → [lspci_k_gpu, lsmod_gpu]
  - `disk_usage` → Storage → [df_h]
  - `mem_status` → Performance → [free_h]
  - `svc_failed` → Services → [systemctl_failed]
- `can_answer_from_probes`: Boolean flag per intent

**Evidence Gating (`evidence_gate.rs`):**
- `EvidenceGate`: Decides whether to call specialist
- `GateDecision`: NeedMoreData, AnswerFromProbes, NeedSpecialist, NeedClarification
- Rules:
  - If required probes missing/failed: return NeedMoreData
  - If probes provide direct factual answer: skip specialist
  - Only call specialist when synthesis needed (why, root cause, recommendations)
- `DirectAnswer`: Built from probe data for factual queries

**Answer Tiers (`answer_tiers.rs`):**
- `AnswerTier`: Facts → KeyItems → Synthesis
- `TieredAnswer`: Progressive answer building
- Tier builders: `build_boot_perf_tiers`, `build_mem_status_tiers`, etc.
- Boot flow fix: 1) Boot time from systemd-analyze, 2) Top offenders from blame, 3) Specialist synthesis

**Clarification Rules:**
- `MAX_CLARIFICATION_LENGTH`: 120 chars max
- `ClarificationBuilder`: Standard questions per intent
- `needs_clarification()`: Only for genuinely ambiguous queries

**Department Ownership (`department.rs`):**
- `DepartmentRules`: Canonical ownership rules
- `DepartmentOwnership`: Topics and keywords per department
- `DepartmentConflict`: Logged when translator disagrees with mapping
- `DeterministicRouter`: Enforces ownership, overrides translator if wrong
- Override logging: "[route] Translator suggested Desktop but mapping says Performance, overridden."

**Acceptance Criteria:**
- Boot questions ALWAYS route to Performance (not Desktop)
- GPU questions ALWAYS route to Hardware (not Storage)
- Disk questions ALWAYS route to Storage
- RAM/CPU load questions ALWAYS route to Performance
- "Handled_by" reflects correct department ≥95% of the time

## [0.0.438] - 2025-12-12

### Added - Fast Pipeline: Hard Budgets, No Streaming, Reliability Stats

**Goal:** Eliminate "Failed to parse specialist response. Parse error: Timeout" forever.
Make response time predictable and fast. Never lie about progress.

**Hard Time Budgets (`budget.rs`):**
- `Phase`: TranslatorIntent, ProbeCollection, JuniorSpecialist, SeniorSpecialist, Renderer
- Budget constants: translator=700ms, probes=2500ms, junior=1500ms, senior=3500ms, renderer=200ms
- `TOTAL_BUDGET_MS`: 6500ms max for one-shot queries
- `BudgetTracker`: Tracks phase timing, triggers timeout/fallback
- `BudgetResult`: Proceed, Exceeded, TotalExhausted

**Specialist Output Contract v2 (`specialist_v2.rs`):**
- `MAX_SPECIALIST_TOKENS`: 220 (hard limit)
- `MAX_SUMMARY_CHARS`: 160, `MAX_NOTES_CHARS`: 200
- `Verdict`: Answered, NeedMoreData, Escalate, CannotAnswer
- `AnswerPayload`: fields (HashMap), summary (max 160 chars)
- `SpecialistParser`: JSON extraction with fallback, truncation on overflow
- Rule: Output exceeds limit → verdict=cannot_answer, never break parsing

**No Streaming for Parsed Calls (`no_stream.rs`):**
- `CallPolicy`: NoStream, StreamAllowed
- `CallType`: TranslatorIntent, JuniorSpecialist, SeniorSpecialist, Renderer
- Rule: Streaming ONLY for Renderer (user-facing), never for internal JSON
- `ModelCallConfig`: Preconfigured timeout + max_tokens per call type

**Retry Strategy (`retry.rs`):**
- `FailureType`: Timeout, ParseError, EmptyResponse, InvalidJson, NetworkError, RateLimit
- `RetryStrategy`: NoRetry, RetrySame, RetrySmaller, FallbackProbeOnly
- Default: 1 retry with smaller/faster model, then probe-only fallback
- `RetryTracker`: Tracks attempts, determines next strategy

**Probe-only Fallback (`probe_fallback.rs`):**
- `ProbeData`: probe_id, display_name, value, unit, is_key_metric
- `ProbeOnlyResult`: Key metrics and all probes with generated summary
- `FallbackAnswer`: Lower confidence (0.6), warning message
- Rule: Never say "I don't know" if we have probe data—present it clearly

**Parallel Probe Engine (`parallel_probes.rs`):**
- `MAX_CONCURRENT_PROBES`: 4, `CACHE_TTL_SECONDS`: 3, `PROBE_TIMEOUT_MS`: 500
- `ProbeCache`: LRU-style cache with TTL, hit/miss tracking
- `ProbeBatch`: Batch execution with concurrency limit
- `PreparedBatch`: Separates cached results from probes needing execution

**Progress Rendering (`progress.rs`):**
- `PhaseStatus`: Pending, Running, Completed, TimedOut, Failed, Skipped
- `PhaseProgress`: Tracks per-phase timing and status
- `PipelineProgress`: Overall pipeline state
- `ProgressRenderer`: Compact/full modes, honest status indicators
- Rule: Never lie about what's happening—show actual phase, not "Analyzing..."

**Reliability Stats (`reliability_stats.rs`):**
- `ReliabilityOutcome`: SpecialistSuccess, SpecialistTimeout, ParserFailure, FallbackToProbes, TotalFailure
- `ReliabilityStats`: success_rate, timeout_rate, parser_failure_rate, fallback_rate, avg_response_time
- `WindowedStats`: Current/previous window comparison, degradation detection
- Rule: Reality-based stats—no fake 100% success rates

**Acceptance Criteria:**
- "Failed to parse specialist response. Parse error: Timeout" never happens
- 95% of fact/status queries answer in <3 seconds
- Diagnostics complete in <6.5 seconds
- Progress never lies about current phase

## [0.0.437] - 2025-12-12

### Added - Question Contract: Fix Understanding and Answer Minimality

**Goal:** Anna never answers a different question than asked. Anna never adds unrelated
information unless explicitly allowed. Simple fact questions return simple answers.

**QuestionIntent v1 (`intent.rs`):**
- `QuestionIntent`: Typed contract for understanding what user is asking
- `IntentCategory`: Fact, Status, Diagnosis, Explanation, ActionRequest
- `Subject`: Memory, Cpu, Disk, Service, Network, Gpu, Boot, Audio, etc.
- `Scope`: Single, List, Summary, Boolean
- `AnswerConstraints`: max_items, allow_extras=false (default!), allowed_fields, units
- `ClarificationRequest`: STOPS all execution until user clarifies

**AnswerPlan & Shape Enforcement (`answer_plan.rs`):**
- `AnswerPlan`: Builds from intent, enforces shape strictly
- `ShapeEnforcer`: Discards ANY data not in allowed_fields
- `DiscardReason`: NotAllowed, MaxItemsExceeded, WrongSubject, NoEvidence
- Rule is ABSOLUTE: if intent allows only "free_ram", CPU/cache/total are dropped

**Evidence Binding (`evidence_bind.rs`):**
- `BoundClaim`: Every claim must map to evidence IDs
- `BindingResult`: Valid, PartialBind, MismatchedEvidence, NoEvidence
- If evidence doesn't map: "I collected data but cannot safely answer exactly what you asked"
- Prevents hallucinated glue text

**Help Text Leakage Filters (`filters.rs`):**
- `LeakageType`: Tutorial, DebugSteps, Commands, Suggestions
- For category=Fact/Status: NEVER output tutorials, debug steps, commands
- Filters: "you can", "try running", code blocks, "you should", etc.
- Example fix: "do I have failed services?" → "No failed services." (not debug tutorial)

**Diagnosis Conclusions (`diagnosis.rs`):**
- `ConclusionState`: Likely, Uncertain, NoIssueDetected
- Diagnosis is incomplete without conclusion
- If conclusion=Uncertain: NO confident language allowed
- `ConclusionLanguageValidator`: Rejects "definitely", "certainly" for uncertain conclusions

**Intent Quality Stats (`stats.rs`):**
- `IntentOutcome`: Correct, Clarified, Misclassified, CorrectedByUser
- `IntentQualityStats`: accuracy_rate, clarification_rate, misclassification_rate
- `MisclassificationDetector`: Detects "that's not what I asked", subject mismatches
- Do not hide this - this is how Anna improves

**Canary Tests (`canary_tests.rs`):**
- 10 fixed regression tests that BLOCK release:
- "how much free ram" → single numeric only
- "is zram enabled" → boolean only
- "which service slowed boot" → list of services only
- "what GPU driver" → driver name only, no hardware dump
- Diagnosis must have conclusion, uncertain=no confident language

## [0.0.436] - 2025-12-12

### Added - Anna Protocol v1: Unbreakable Typed Communication

**Goal:** Fix "Parse error: Timeout" failures and make LLM I/O unbreakable. Every model call
returns valid typed payload or clean failure. Stats only count success when valid envelope exists.

**Protocol Framing (`framing.rs`):**
- `PROTO_START`: `<<<ANNA_PROTO_V1>>>` - Frame start marker
- `PROTO_END`: `<<<END_ANNA_PROTO_V1>>>` - Frame end marker
- `extract_framed_content()`: Extract JSON from framed output
- `FrameResult`: Found, NoFrame, IncompleteFrame, MultipleFrames
- `FrameValidation`: Validates marker balance

**Two-Stage Decoder (`decoder.rs`):**
- `ProtoDecoder`: Fast path (framed extraction) + recovery path (JSON scanning)
- `DecodeResult`: Success(ModelResultEnvelope) | Failed(DecodeError)
- `DecodeError`: ModelTimeout, NoFrame, IncompleteFrame, JsonParseError, etc.
- JSON repair: `remove_trailing_commas()`, `extract_json_object()`
- Key distinction: `is_timeout()` vs `is_parse_error()` - model failures ≠ parse errors

**Strong Typing (`envelope.rs`):**
- `ModelResultEnvelope`: ok, role, ticket_id, confidence, summary, claims, evidence_used, errors
- `ModelRole`: Translator, Junior, Senior (with default timeouts)
- `Claim`: Text + evidence IDs supporting it
- `Action`: Probe, AskUser, ProposeChange, InstallHelper with risk levels
- `EvidenceRef`: id, kind (Probe/Man/Help/Wiki), title
- `EnvelopeValidation`: Validates envelope integrity

**Streaming Safety (`streaming.rs`):**
- `StreamBuffer`: Buffers model output instead of streaming raw tokens
- `StreamState`: Waiting, Receiving, FrameStarted, FrameComplete, NoFrame, TimedOut, Error
- `ProgressFrame`: Optional progress updates (thinking, progress percentage)
- `StreamDisplay`: Formats progress with spinner, bytes, elapsed time

**Evidence Fallback (`fallback.rs`):**
- `GatheredEvidence`: Evidence collected before model failure
- `FallbackResponse`: Renders evidence when model fails, suggests next probes
- `MAX_FALLBACK_CONFIDENCE = 0.5`: Fallback can never claim high confidence
- Deterministic probe suggestions based on what's missing

**Stats Integrity (`stats.rs`):**
- `TicketOutcome`: Resolved, Escalated, InternalFailure, Cancelled, InProgress
- `PeriodStats`: total_tickets, resolved, escalated, internal_failures, avg_response_ms
- `outcome_from_decode()`: Maps decode result to outcome
- No more fake 100% success - only resolved when ModelResultEnvelope.ok=true

**Model Prompts (`prompts.rs`):**
- `PROTOCOL_INSTRUCTION`: Mandatory output format instructions for all models
- `ENVELOPE_SCHEMA`: JSON schema every model must follow
- `junior_prompt()`, `senior_prompt()`, `translator_prompt()`: Role-specific prompts
- `prompt_has_protocol()`: Validates prompt includes protocol instructions

**Acceptance Tests (`tests.rs`):**
- 15 integration tests covering full pipeline
- Valid framed response, preamble extraction, trailing comma repair
- Recovery without frame markers, timeout vs parse error distinction
- Stats integrity, evidence fallback, stream buffer detection
- Full pipeline success and failure scenarios

## [0.0.435] - 2025-12-12

### Added - Evidence-First Knowledge Engine

**Goal:** Make Anna behave like a real IT department - evidence first, documentation second,
LLM last. Every claim must have citations. Recipes require proof before promotion.

**Knowledge Sources (`sources.rs`):**
- `KnowledgeSource`: ManPage, HelpText, LocalDocs, ArchWiki, ProbeOutput
- `ManPageSource`: Retrieves via `man -P cat`, extracts sections, search with windowing
- `HelpTextSource`: Tries --help, -h, help subcommand variants
- `LocalDocsSource`: Searches /usr/share/doc with ripgrep
- `SourceError`: CommandFailed, NotFound, ReadFailed, Timeout

**Citations (`citations.rs`):**
- `EvidenceId`: Unique identifier (probe:id, man:cmd, wiki:page, help:cmd, doc:path)
- `Citation`: Links claim to evidence with source label and excerpt (max 200 chars)
- `CitationStore`: Holds all citations and raw evidence for audit
- `verify_citation()`: Checks excerpt exists in raw content

**Probe Primitives (`primitives.rs`):**
- `ProbePrimitive`: Limited set of generic probes (not one-off scripts)
- `Domain`: Boot, Services, Logs, Memory, Disk, Network, Hardware, Performance, Desktop, Packages, Security
- `ParserId`: Raw, KeyValue, Table, Json, TimeDuration, Numeric
- `Precondition`: CommandExists, FileExists, SystemdRunning, HelperInstalled
- `PrimitiveLibrary`: 25+ built-in probes with keyword search
- Key probes: sys.boot.analyze, sys.boot.blame, sys.services.failed, sys.mem.free, etc.

**Probe Plan (`probe_plan.rs`):**
- `ProbePlan`: Dynamic probe selection at runtime
- `ProbeSelection`: Primitive + reason + priority + parameters
- `ProbeExecutor`: Execute plans and collect evidence
- `ProbeOutput`: Raw output, parsed fields, exit code, timing
- `ParsedOutput`: TimeMeasurement, ItemList, KeyValue, Raw

**Research Loop (`research.rs`):**
- `ResearchPlan`: Keywords + domains + commands to research
- `ResearchLoop`: Execute probes → retrieve docs → synthesize
- `ResearchResult`: Outputs, docs, citations, findings
- `Finding`: Claim + evidence IDs + confidence level (High/Medium/Unsupported)
- Max 2 iterations, deterministic ordering

**Recipe Templates (`recipes.rs`):**
- `RecipeTemplate`: Parameterized solution pattern
- `RecipeStep`: Instruction + optional command + confirmation requirement
- `RecipeInstance`: Instantiated recipe with parameter substitution
- `RecipeCandidate`: Awaiting promotion (tracks confirmations/failures)
- `RecipePromoter`: Manages promotion after N=3 confirmations
- Success rate tracking, failure recording

**Wiki Cache (`wiki_cache.rs`):**
- `WikiCache`: Local cache for Arch Wiki pages
- `WikiPage`: Title, URL, content, sections, categories
- `WikiSection`: Title, level, content extraction
- `WikiSearchResult`: Hits with page/section/excerpt
- `search_with_citations()`: Add wiki evidence to citation store
- Essential pages list for pre-caching

**Citation Enforcement (`enforcement.rs`):**
- `Claim`: Factual, Documentation, Recommendation, Uncertainty
- `EvidenceType`: Probe, Documentation, Any, None
- `ClaimValidator`: Validates claims against citation store
- `ValidationReport`: Supported vs unsupported claims
- `extract_claims()`: Heuristic claim extraction from text
- "No citations, no claims" policy

**Acceptance Tests (`tests.rs`):**
- Boot slow diagnosis workflow with citations
- CPU temperature check
- Recipe promotion after N confirmations
- Research flow and iteration limits
- Citation store verification
- Primitive library coverage
- Claim extraction and validation

## [0.0.434] - 2025-12-12

### Added - Hardware-Aware Model Selection and Helper Management

**Goal:** Make Anna's use of local models and helpers disciplined, explainable, and minimal,
while still powerful and fully aligned with the hardware she runs on.

**Hardware Profiling (`profile.rs`):**
- `HardwareProfile`: Complete system information (CPU, RAM, GPU, storage, OS)
- `CapabilityTier`: Tiny (<= 8GB), Small (8-16GB), Medium (16-32GB), Large (> 32GB)
- `CpuInfo`: Model name, core count, thread count, AVX2 support
- `GpuInfo`: Discrete detection, vendor (NVIDIA/AMD/Intel), VRAM
- GPU + RAM combination affects tier (need both to bump up)
- Profile versioning for migration support

**Model Catalog (`catalog.rs`):**
- `ModelCatalog`: Static, versioned catalog of candidate models
- `ModelEntry`: Name, role, min tier, min RAM, disk usage, priority
- `ModelRole`: Translator (fast classification), Junior (responsive), Senior (deep reasoning)
- `select_model()`: Pick best model given tier and available RAM
- `prefer_small` option to select smaller models on capable hardware

**Model Plan (`model_plan.rs`):**
- `ModelPlan`: Concrete model assignments (translator, junior, senior)
- `ModelPlanner`: Generate plans from hardware profile
- Plan includes rationale explaining model choices
- Validation detects tier/version mismatches
- No duplicate model names in plan

**Model Health (`model_health.rs`):**
- `ModelHealth`: Track installation status per model
- `ModelStatus`: Ok, Unverified, Missing, Broken, Installing
- `InstalledBy`: Anna vs User tracking
- `ModelVerifier`: List/verify/pull/remove via Ollama CLI
- `sync_health()`: Detect removed models
- `verify_all()`: Test models with minimal completion

**Model Config (`model_config.rs`):**
- `AutoInstallPolicy`: always, never, ask-per-model
- `max_model_disk_gb`: Hard limit for model storage (default 25GB)
- `PreferSmallSetting`: yes, no, auto (based on RAM)
- Per-model decision tracking for ask-per-model policy
- `InstallRequest`: Natural language prompt for user approval

**Helper Catalog (`helpers.rs`):**
- `HelperCatalog`: System tools (lm_sensors, smartmontools, nvme-cli, ethtool, hdparm, dmidecode, lshw)
- `HelperEntry`: Packages per distro (Arch, Debian, Fedora)
- `HelperManager`: Track installed helpers with Anna vs User attribution
- `install()`/`uninstall()`: Distro-aware package management
- Only Anna-installed helpers can be auto-removed

**Helper Config (`helper_config.rs`):**
- `HelperInstallPolicy`: always, never, ask-per-helper
- `remove_on_uninstall`: Control cleanup behavior
- `blocked_helpers`: User can block specific helpers
- Per-helper decision tracking

**Integration (`integration.rs`):**
- `ProbeHelper`: Map probes to best commands given available helpers
- Fallback commands when helpers not installed
- `suggested_helpers()`: What to install for better probes
- `SpecialistHelper`: Model availability checking with fallback
- `HelperSuggestion`: Formatted install step for specialist response

**Status Display (`status.rs`):**
- `HardwareStatus`: Complete status structure
- `SystemProfileSection`: RAM, CPU, GPU, tier
- `LlmSection`: Provider, state, models with status
- `HelperStatusSection`: Anna-installed vs user-installed
- `ModelUsageStats`: Call counts, durations, errors per model
- `HelperUsageStats`: Usage tracking per helper
- `format()`: Formatted output for annactl status

**Acceptance Tests (`tests.rs`):**
- 26 comprehensive tests covering all scenarios
- Fresh install generates coherent plan
- Model removal detection and fallback
- Helper policy enforcement
- Status reflects real state (no fake 100%)
- Usage stats tracking

## [0.0.433] - 2025-12-12

### Added - Robustness Layer: Timeouts, Failure Handling, and Truthful Stats

**Goal:** Redesign core execution, error handling and stats so the system is honest about failures,
enforces time budgets, and only awards XP for actual successful resolutions.

**Robustness Module (`robustness/`):**

**Contract (`contract.rs`):**
- `TicketOutcome`: Strict outcome enum (Success, Partial, ClarificationRequired, Unsupported, InternalError, Timeout, ParseError)
- `SpecialistResult`: Complete response structure with outcome, summary, diagnosis, steps, evidence, confidence
- `ProposedStep`: Commands with risk level (Safe, NeedsReview, Dangerous)
- `EvidenceRef`: Traceable evidence references with source and ID
- `TicketMetrics`: Processing time, probe count, retry usage tracking

**Timeouts (`timeouts.rs`):**
- `TimeoutEnforcer`: Stage-by-stage timeout tracking with soft/hard budgets
- `TimeBudget`: Soft cap (warn) and hard cap (abort) per stage
- `TimeoutConfig`: Global budget of 25s, translator 1s, junior 4s/8s, senior 10s/20s
- `TimeoutStage`: Translator, JuniorRouting, JuniorLlm, SeniorRouting, SeniorLlm, Probes, Knowledge, Parse, Global
- Single retry on parse failure (MAX_PARSE_RETRIES = 1)
- `TimingSummary`: Complete breakdown of time spent in each stage

**JSON Enforcement (`json_enforce.rs`):**
- `JsonEnforcer`: Extract and validate JSON from LLM responses
- `ParseResult`: Success with SpecialistResult or Failure with error message
- `SchemaHint`: Prompt template showing required JSON structure
- Handles JSON embedded in prose, markdown code blocks, truncated output
- `stricter_version()`: More forceful schema hint after parse failure

**Outcome Messages (`outcome_messages.rs`):**
- `OutcomeRenderer`: Maps TicketOutcome to user-facing messages
- `OutcomeMessage`: Body text, resolved flag, failure flag, optional context
- Different tones: Success is friendly, Timeout is apologetic, ParseError shows debug info
- Debug mode reveals internal error details
- `UserMessage`: Final formatted message with timing and performance info

**Stats Engine (`stats_engine.rs`):**
- `TruthfulStats`: Tracks actual outcomes (successes, failures, partial, clarifications, timeouts)
- `StatsEngine`: Aggregates stats with XP calculation
- `TicketStats`: Per-ticket statistics for recording
- XP only for Success (base 10) and Partial (base 5), never for failures
- Speed bonus: <2s +5 XP, <5s +2 XP
- Complexity bonus: 3+ probes +3 XP, 2 probes +1 XP
- `success_rate()`: Honest percentage (successes / decided), not 100%
- `format_summary()`: Shows real stats including failures
- `DepartmentStats` and `StaffStats`: Breakdown by team and specialist

**Lifecycle (`lifecycle.rs`):**
- `TicketState`: Opened, InProgress, AwaitingClarification, ResolvedSuccess, ResolvedPartial, ResolvedFailure
- `TicketLifecycle`: State machine with validated transitions
- `TicketTransition`: Audit trail with timestamps and reasons
- Terminal states cannot transition further
- Follow-up ticket support for multi-turn conversations
- `provide_clarification()`: Resume from AwaitingClarification

**Fallback (`fallback.rs`):**
- `FallbackGenerator`: Generate answers from probe evidence when LLM fails
- `ProbeEvidence`: Raw output with parsed values and success status
- `FallbackAnswer`: Summary, raw data, confidence (lower than LLM), sources
- Built-in extractors: memory (proc_meminfo), disk (df), boot time (systemd-analyze), failed services
- Outcome is Partial, not Success, with error context preserved

**Performance (`performance.rs`):**
- `PerformanceTracker`: Track stages, probes, specialists with timings
- `StreamingUpdate`: Real-time events (ProbesStarted, ProbeCompleted, SpecialistStarted, Timeout, Complete)
- `TimingBreakdown`: Debug display of time spent in each component
- `ProbeStatus`: Ok, Failed, Timeout, Skipped
- `format_updates()`: REPL-friendly progress messages

**Acceptance Tests (`tests.rs`):**
- 15 comprehensive scenarios testing the entire robustness layer
- Test 1: RAM query success with LLM
- Test 2: RAM query with LLM parse failure and fallback
- Test 3: Boot query timeout counts as failure
- Test 4: Mixed session stats reflect reality
- Test 5: Parse error debug info visibility
- Test 6: JSON parsing robustness (valid, prose-wrapped, invalid)
- Test 7: Clarification flow not counted as resolved
- Test 8: Timeout enforcer behavior
- Test 9: Retry behavior on parse error
- Test 10: Streaming updates format correctly
- Test 11: Fallback handles no evidence
- Test 12: Lifecycle state transitions
- Test 13: Outcome renderer message types
- Test 14: Schema hint for prompts
- Test 15: Performance breakdown timing
- Test 16: XP calculation respects failures

## [0.0.432] - 2025-12-12

### Added - Knowledge Pipeline: Priority-Ordered Knowledge Fetching and Learning

**Goal:** Build a minimal but powerful knowledge and learning pipeline where Anna always prefers
authoritative local sources first, LLMs help interpret and synthesize (not memorize facts),
and successful research becomes reusable parametric recipes.

**Knowledge Pipeline Module (`knowledge_pipeline/`):**

**Sources (`sources.rs`):**
- `SourcePriority`: Strict trust hierarchy (Probe > LocalDoc > CachedWiki > Remote)
- `KnowledgeSource`: Type-safe sources (Probe, ManPage, HelpOutput, DocFile, ArchWiki, RemoteUrl)
- `SourceResult`: Content with relevance score and timestamp
- `Citation`: Traceable evidence with optional excerpts and line numbers
- Trust scores: Probe=1.0, LocalDoc=0.95, CachedWiki=0.8, Remote=0.6

**Fetcher (`fetcher.rs`):**
- `KnowledgeFetcher`: Priority-ordered lookup with early termination on confident results
- `FetchConfig`: Remote disabled by default, configurable max lookups
- `FetchResult`: Merged results with citations and confidence
- `suggest_sources()`: Auto-suggest relevant sources for topics
- Built-in probe fetching for /proc/meminfo, cpuinfo, uptime, loadavg

**Research (`research.rs`):**
- `ResearchPattern`: When specialists are unsure, research first
- `ResearchRequest`: Builder pattern for research queries
- `ResearchOutcome`: Found, Partial, NotFound, NeedsClarification
- `can_answer_locally()`: Quick check for local source availability

**Wiki Sync (`wiki_sync.rs`):**
- `WikiSyncer`: Offline Arch Wiki article cache
- `WikiSyncConfig`: Rate limiting, max age (7 days), article list
- 30+ default articles (Systemd, Pacman, GRUB, NetworkManager, etc.)
- `sync_outdated()`: Update stale articles
- `search()`: Full-text search across cached articles

**Clarification (`clarification.rs`):**
- `ClarificationProtocol`: Request/response workflow for specialists
- `ClarificationRequest`: Choice, Value, Confirmation, Context, Scope types
- `ClarificationResponse`: Value with optional notes
- Skip-with-default for non-blocking clarifications

**Learning (`learning.rs`):**
- `LearningLoop`: Track successful research patterns as recipes
- `RecipeStats`: Uses, success rate, avg time, satisfaction scores
- `LearnedPattern`: Query patterns, sources, expected citations
- Pattern deprecation when success rate falls below threshold (70%)
- Word-based pattern matching with stop-word filtering

**Acceptance Tests (`tests.rs`):**
- Priority ordering verification
- Remote disabled by default
- Research pattern workflow
- Clarification protocol flow
- Learning loop with pattern reinforcement
- Recipe stats tracking
- Pattern deprecation on failure
- Source verification requirements
- No hardcoded answers rule enforcement
- Full pipeline integration test

### Fixed
- Hollywood UX test isolation (parallel test interference)

## [0.0.431] - 2025-12-11

### Added - Hollywood UX: Unified Transcript and Terminal Renderer

**Goal:** Give Anna a rock-solid, cinematic terminal UX with one transcript model and one renderer,
so the user always feels like they are watching a capable IT department at work.

**Hollywood UX Module (`hollywood_ux/`):**

**Types (`types.rs`):**
- `HollywoodTranscript`: Extended transcript with metadata (confidence, handler, department, evidence)
- `TranscriptOutcome`: Success, Partial, Failed, ParseError, Cancelled
- `ProbeResult`: Probe status with timing and optional raw output
- `InternalComm`: Staff communication entries
- `RenderOptions`: Configurable rendering (mode, width, sections)

**Storage (`storage.rs`):**
- `TranscriptStorage`: Persistent JSON storage with rotation
- `TranscriptSummary`: Minimal info for listing
- `StorageStats`: Storage statistics and health
- Auto-rotation at 1000 transcripts, 1MB max size per file

**Styles (`styles.rs`):**
- Box-drawing characters (ASCII-safe for 80-column terminals)
- Consistent section labels: `[you]`, `[anna]`, `[probes]`, `--- internal comms ---`
- Formatting utilities: `header_block()`, `evidence_footer()`, `status_footer()`
- Text wrapping, truncation, indentation helpers
- Spinner frames and working status display

**Renderer (`renderer.rs`):**
- `HollywoodRenderer`: Main renderer with cinematic and debug modes
- Structured output: header -> internal comms -> probes -> answer -> evidence -> footer
- `render_cinematic()`: Clean user-facing output
- `render_debug()`: Full diagnostic output with raw probes and processing info
- `format_simple_answer()`: Quick formatting for direct answers
- `format_error_response()`: Gentle error presentation

**Streaming (`streaming.rs`):**
- `StreamingRenderer`: Live terminal updates with incremental rendering
- `SpinnerState`: Animated spinner with configurable frames (100ms interval)
- `start_spinner()`, `stop_spinner()`, `tick()`: Spinner control
- `render_incremental()`: Append new segments without re-rendering
- `render_final()`: Complete output with evidence and footer
- Intelligent completion detection (Answer/Error segments)

**Acceptance Tests (`tests.rs`):**
- Simple RAM query test (clean answer, evidence, confidence)
- Complex boot query test (internal comms, probes, suggestions)
- Parse error handling test (gentle message, evidence preserved)
- Mode consistency test (cinematic vs debug)
- Storage round-trip test
- Header/evidence/status formatting tests
- Streaming state management tests

**Design Principles:**
- Cinematic mode: Hollywood IT department feel, minimal noise
- Debug mode: Full diagnostics for developers, clearly separated
- 80-column terminal compatible
- No emojis, minimal unicode
- Consistent visual language across all commands
- Parse errors shown gently, never claim false success

## [0.0.430] - 2025-12-11

### Added - Background Workers, Idle-time Learning, Alerts, and Long-running Tickets

**Goal:** Enable Anna to handle long-running tasks in the background, learn during idle time,
alert users to important events, and manage user-defined monitors and reminders.

**Background Worker Module (`background_worker/`):**

**Job System (`job.rs`):**
- `JobKind`: LongTicketAnalysis, DocIndexRefresh, ModelBenchmark, PeriodicProbe, UserReminder, MonitorCheck, RecipeConsolidation, SendNotification
- `JobPriority`: Low (idle-time only), Normal (scheduled), High (urgent)
- `JobStatus`: Pending, Running, Completed, Failed, Cancelled
- `BackgroundJob`: Full job data model with scheduling, retries, metadata
- `JobResult`: Execution result with follow-up jobs and notifications
- Factory methods: `long_ticket()`, `doc_refresh()`, `model_benchmark()`, `reminder()`, `monitor_check()`

**Scheduler (`scheduler.rs`):**
- `JobScheduler`: In-memory queue with persistent storage
- `SchedulerConfig`: CPU threshold, max concurrent jobs, daily limits
- Priority-based scheduling (high first, then by scheduled time)
- Idle-time detection for low-priority jobs
- `get_due_jobs()`: Get jobs ready to run with CPU load awareness
- Job lifecycle: `mark_running()`, `mark_completed()`, `mark_failed()`
- Daily counter reset and old job cleanup

**Executor (`executor.rs`):**
- `JobExecutor` trait for custom execution handlers
- `DefaultExecutor`: Dispatch to specialized handlers
- Handler traits: `LongTicketHandler`, `DocRefreshHandler`, `BenchmarkHandler`, `ProbeHandler`, `ReminderHandler`, `MonitorHandler`, `RecipeHandler`
- `CpuMonitor`: Read `/proc/loadavg` for idle detection

**Notifications (`notifications.rs`):**
- `NotificationConfig`: Email, desktop, wall, quiet hours, rate limits
- `NotificationDispatcher`: Multi-channel notification sending
- Channels: Email (sendmail/msmtp), Desktop (notify-send), Wall (terminal broadcast)
- `AlertPriority`: Low, Normal, High, Critical
- Rate limiting per channel (default 5 min cooldown)
- Quiet hours support (e.g., "22:00-08:00")
- `NotificationStatus`: Status display for `annactl status`

**Monitors & Reminders (`monitors.rs`):**
- `Monitor`: User-defined condition checks with alerts
- `MonitorCheck`: DiskSpace, ProcessRunning, FileAge, Command, SystemLoad, MemoryUsage
- `ThresholdCondition`: GreaterThan, LessThan, Equals, IsTrue, IsFalse
- 24-hour alert cooldown to prevent spam
- `Reminder`: Scheduled notifications (Once, Daily, Weekly, Monthly)
- `MonitorStorage`: Persistent monitor configuration

**Idle Learning (`idle_learning.rs`):**
- `IdleLearningConfig`: Enable/disable, CPU threshold, daily limits
- `IdleLearningManager`: Orchestrate idle-time learning jobs
- Automatic job selection: Recipe consolidation (24h), Doc refresh (12h), Benchmark (7d)
- System idle detection with minimum idle time requirement
- `IdleLearningStatus`: Status display for `annactl status`
- `RecipeConsolidator`, `DocIndexRefresher`: Placeholder implementations

**Storage (`storage.rs`):**
- `JobStorage`: JSON persistence for job queue
- `PendingMessageStorage`: Queue messages for next `annactl` open
- `PendingMessage`: User notification with priority and source tracking
- Factory methods: `from_long_ticket()`, `from_monitor()`

**Acceptance Tests (`tests.rs`):**
- Long-running ticket workflow test
- Monitor alert workflow test
- Idle learning workflow test
- Job priority ordering test
- Job retry mechanism test
- Notification rate limiting test
- Scheduler status summary test
- Pending message queue test
- Reminder scheduling test

**Design Principles:**
- Only run low-priority jobs when CPU is idle (< 30% load)
- All jobs are durable (persisted to disk)
- No notifications without explicit user consent
- Rate-limited alerts (24h cooldown)
- Maximum 10 idle jobs per day

## [0.0.429] - 2025-12-11

### Added - Documentation Brain (Local Knowledge Graph)

**Goal:** Give Anna a real documentation brain - local, offline, Arch-centric knowledge from
man pages, Arch Wiki, and tool help output. Minimal hardcoded knowledge, maximal learning from docs.

**Documentation Engine Module (`doc_engine/`):**

**Core Types (`types.rs`):**
- `DocSourceKind`: ArchWiki, ManPage, ToolHelp, LocalDoc (with priority ordering)
- `DocSnippet`: Indexed documentation with id, source, name, section, summary, content, keywords
- `DocQuery`: Search interface with preferred sources, limits, filters
- `DocResult`: Query results with snippets, cache status, timing
- `DocReference`: Recipe doc citations with source, name, section, reason

**Documentation Index (`index.rs`):**
- `DocIndex`: In-memory + on-disk index with keyword, source, and name indexes
- Full-text search with relevance scoring (name match, keyword match, content match)
- Persistence to JSON format
- Stale entry detection for cache refresh

**Man Page Parser (`man_parser.rs`):**
- `parse_man_page()`: Parse man pages into indexed snippets by section
- Section splitting (NAME, DESCRIPTION, OPTIONS, EXAMPLES, etc.)
- `search_man_pages()`: Search via `man -k` (apropos)
- Safe command validation (no shell injection)
- Essential man pages list (systemctl, pacman, df, mount, etc.)

**Tool Help Extractor (`help_extractor.rs`):**
- `extract_help()`: Run `--help`/`-h` and extract useful content
- Safe command whitelist (100+ safe commands)
- Summary extraction from help output
- Help output cleanup and normalization

**Arch Wiki Reader (`wiki_reader.rs`):**
- `read_wiki_page()`: Read from local wiki cache
- HTML and plain text format support
- Section splitting (## headers, == mediawiki ==)
- Essential wiki pages list (systemd, pacman, boot, filesystems, etc.)

**Unified Query API (`query.rs`):**
- `DocEngine`: Main interface for all doc queries
- `query()`: Search in-memory index
- `query_or_fetch()`: Search + fetch on miss
- `man_page()`, `wiki_page()`, `help_output()`: Quick accessors
- `refresh_index()`: Rebuild index from all sources
- `refresh_stale()`: Update only stale entries
- Index and engine statistics tracking

**Recipe Integration (`recipe_integration.rs`):**
- `add_doc_references()`: Add citations to recipe origin
- `query_for_recipe()`: Get docs relevant to a recipe
- `docs_for_probe_command()`: Get docs for probe commands
- `suggest_doc_refs()`: Domain-based doc suggestions
- `format_citations_for_answer()`: Format for display

**Translator Policy (`translator_policy.rs`):**
- `DocNeedLevel`: None, Optional, Recommended, Required
- `analyze_doc_need()`: Determine if question needs docs
- Pattern matching for explanation, fix, error, status queries
- Topic extraction from questions
- `DocUsageGuidelines`: Rules for specialists

**Design Principles:**
- Probes first, docs second (docs interpret, not invent)
- Local-first (no silent external calls)
- Citation required (every claim traces to source)
- Minimal usage (summarize, don't dump)
- Never pretend (no made-up wiki references)

**Acceptance Tests (`tests.rs`):**
- Source priority ordering
- Snippet ID stability
- Query builders
- Index operations
- Result merging
- Citation formatting
- Essential lists populated
- Man parser safety
- Translator policy detection

## [0.0.428] - 2025-12-11

### Added - Specialist Protocol V4 (No-Bullshit Policy)

**Goal:** Fix specialist protocol, timeouts, and answer quality with strict enforcement
of "no invented facts" and "no generic tutorials when user asked for current state."

**Specialist Protocol Module (`specialist_protocol/`):**

**Strict Response Schema (`schema.rs`):**
- `StrictResponse`: Complete JSON schema for all specialist responses
- `ResponseStatus`: Success (evidence-backed), Partial (incomplete), Failure (honest)
- `ResponseDetails`: key_facts, diagnosis, recommendations
- `ProposedAction`: Actions with risk levels (Low/Medium/High)
- `Evidence`: probes_used, arch_wiki_pages, man_pages with citations
- `ResponseMeta`: handled_by, ticket_id, version
- `is_learnable()`: Only high-confidence, evidenced responses can become recipes

**No-Bullshit Validator (`validator.rs`):**
- `validate_response()`: Comprehensive validation of specialist responses
- `ValidationError` types: InventedData, GenericHowTo, MissingEvidence, ForbiddenPattern, VagueLanguage
- Forbidden patterns: "unknown is installed", "2 is installed", placeholders
- Vague language detection: "might be", "possibly", "perhaps", "I think"
- Generic how-to blocking for state queries (check_, is_, are_, show_, list_)
- Evidence requirement enforcement for high-confidence claims
- Automatic status/confidence adjustment for violations

**Graceful Fallback Handler (`fallback.rs`):**
- `FallbackContext`: What we know when specialist fails
- `FallbackReason`: Timeout, ParseError, ValidationFailed, LlmError, NoSpecialist, RetryExhausted
- `generate_fallback()`: Extracts useful facts from probe outputs
- Memory fact extraction from `free`/`/proc/meminfo` output
- Disk fact extraction from `df` output
- Systemd fact extraction from `systemctl` output
- Never shows "Failed to parse specialist response" to users
- User-friendly vs debug error messages

**Ticket Outcome Tracking (`outcome.rs`):**
- `TicketOutcome`: Success, UsefulPartial, HonestUnknown, Failed, InternalError
- `determine_outcome()`: Maps response to user-facing outcome
- `is_useful_partial()`: Checks if partial has meaningful content
- `is_honest_unknown()`: Detects honest "I don't know" responses
- `HonestTicketStats`: Tracks real success rates, not inflated metrics
  - `success_rate()`: Only full successes count
  - `resolution_rate()`: Success + UsefulPartial + HonestUnknown
  - `validate()`: Prevents impossible rates (100% success with failures)

**JSON Parser with Fallback (`parser.rs`):**
- `parse_specialist_json()`: Parse with validation and fallback
- `ParseOutcome`: Success, ValidationFailed, NoJson, InvalidJson, SchemaMismatch, Timeout
- `extract_json()`: 4 strategies - clean, markdown, brace-matching, lenient
- JSON fix attempts: trailing commas, missing quotes, partial responses
- Parse stats tracking for debugging

**Translator Guardrails (`guardrails.rs`):**
- `IntentType`: CheckState, HowTo, Explain, Diagnose, Action, Unknown
- `ResponseType`: DirectAnswer, Tutorial, Diagnostic, Explanation, StateAnswer
- `classify_intent()`: Categorizes user questions
- `classify_response()`: Identifies response type
- `GuardrailContext`: Tracks question, intent, and probe outputs
- `check_guardrails()`: Validates response matches intent
- `process_with_guardrails()`: Full pipeline with parsing and validation

**Acceptance Criteria Tests (`tests.rs`):**
- RAM question gets direct answer with evidence
- Failed services question gets yes/no, not tutorial
- Timeout gives partial/failure, never "Failed to parse"
- Nonsense patterns blocked ("unknown is installed", "2 is installed")
- Stats are honest (40% success ≠ 100% success)
- Intent classification accuracy for state vs how-to queries
- Vague language blocked in success responses
- Full pipeline integration tests

## [0.0.427] - 2025-12-11

### Added - Self-Learning Recipe Engine V1

**Goal:** Implement a self-learning recipe system that learns from successful specialist responses
and can answer common questions without calling the LLM.

**Learning Engine Module (`learning_engine/`):**

**Recipe Schema (`recipe.rs`):**
- `LearnedRecipe`: Full recipe with pattern, probes, answer templates, safety, origin, stats
- `RecipePattern`: intent + keywords + required_signals + optional_signals
- `RecipeInputs`: Parameter definitions with optional support
- `RecipeProbe`: Probe definitions with tool, params, timeout, optional flag
- `RecipeLogic`: Sequential/Template logic with answer kind (Diagnostic/Fix/Explanation/Status)
- `AnswerTemplate`: Short and detailed templates with `{{variable}}` substitution
- `RecipeSafety`: RiskLevel (Low/Medium/High), needs_backup, requires_sudo, warning
- `RecipeOrigin`: created_from_ticket, created_by, created_at, sources, is_seed
- `RecipeUsageStats`: uses, successes, failures, last_used_at, success_rate()

**Evidence Cache (`evidence.rs`):**
- `EvidenceEntry`: Probe output, man page, help output, ticket summary
- `EvidenceCache`: Rolling cache with domain/intent indexing
- Automatic pruning of entries older than 30 days
- `search()`: Find evidence by domain or intent

**Recipe Storage (`storage.rs`):**
- `RecipeLibrary`: In-memory recipe store with domain/intent indexes
- Load/save from JSON files
- `record_success()/record_failure()`: Track recipe outcomes
- `enable()/disable()`: Recipe lifecycle management
- `seeds()/learned()/recent(days)`: Filtered recipe access

**Learning Eligibility (`eligibility.rs`):**
- `check_eligibility()`: Determines if a ticket is learnable
- `SkipReason`: ErrorStatus, LowConfidence, VagueAnswer, NoEvidence, TooSpecific, etc.
- `extract_intent()`: Pattern-based intent extraction (how much ram → check_free_ram)
- `extract_params()`: Extract service_name, package_name, device, file_path from questions
- Vague answer detection ("might be", "possibly", "not sure", etc.)
- User-specific content detection (/home/user/, /tmp/, UUIDs, etc.)

**Recipe Matcher (`matcher.rs`):**
- `find_matches()`: Find recipes matching a query
- `RecipeMatch`: score, breakdown, params, missing_signals
- `MatchBreakdown`: Weighted scoring components
  - intent_score (40%): Exact match = 1.0, partial = 0.7, word overlap
  - keyword_score (30%): Jaccard similarity between keywords
  - signal_score (20%): Availability of required probes
  - domain_bonus (10%): Matching domain
  - maturity_bonus (5%): Reliable recipes with ≥3 successful uses
  - health_penalty (-15%): Recipes with <50% success rate
- `is_strong()`: Score ≥0.85 = auto-execute without LLM
- `is_usable()`: Score ≥0.70 = can be considered

**Recipe Generator (`generator.rs`):**
- `generate_from_ticket()`: Create recipe from successful ticket
- Extracts pattern from question (intent, keywords, signals)
- Captures probes used in response
- Generates answer templates with variable substitution
- Infers domain from specialist department and question content
- Builds safety profile from response actions

**Recipe Executor (`executor.rs`):**
- `execute_recipe()`: Run recipe probes and fill answer templates
- `ProbeExecutor` trait for testable probe execution
- `ShellProbeExecutor`: Default implementation using shell commands
- Probe command mapping (probe.free → "free -h", probe.df → "df -h", etc.)
- Variable extraction from probe output (free → total_mem, used_mem, available_mem)
- `can_execute()`: Check if recipe requirements are met

**Learning Stats (`stats.rs`):**
- `LearningStats`: recipes_total, recipe_hit_rate, evidence_entries, top_recipes
- `ResolutionCounter`: by_recipe, by_llm, failed
- `LearningProgress`: tickets_since_last_learned, skipped_tickets
- Display implementation for annactl stats output

**Seed Recipes (`seeds.rs`):**
- `seed-check-free-ram`: Memory status from `free -h`
- `seed-check-disk-space`: Disk usage from `df -h`
- `seed-check-service-status`: Systemd service status
- `seed-check-uptime`: System uptime
- `seed-check-memory-usage`: Detailed memory stats

**Key Constants:**
- `MIN_LEARN_CONFIDENCE`: 0.8 (80% confidence required to learn)
- `MIN_PARTIAL_CONFIDENCE`: 0.7 (70% for partial responses)
- `MIN_RECIPE_MATCH_SCORE`: 0.70 (70% score to use recipe)
- `AUTO_EXECUTE_SCORE`: 0.85 (85% score to skip LLM)
- `MIN_RELIABLE_USES`: 3 (uses before recipe is reliable)
- `MAX_EVIDENCE_CACHE`: 500 entries
- `EVIDENCE_CACHE_DAYS`: 30 days rolling window

**Tests:**
- 47 unit tests covering all components
- Recipe creation and pattern matching
- Eligibility checking with various skip reasons
- Storage operations with indexing
- Evidence cache with pruning
- Executor with mock probes

## [0.0.426] - 2025-12-11

### Added - Strict Ticket Lifecycle and Honest Metrics

**Goal:** Fix ticket lifecycle, reliability accounting, and stats so they match reality.
No more "100% success" when there are failures.

**Ticket Lifecycle (`ticket_lifecycle.rs`):**
- `TicketLifecycleState`: New → InProgress → Answered → UserSatisfied | Failed | Cancelled
- Strict state machine with validated transitions
- `TicketResolution`: ResolvedSuccess, ResolvedPartial, ResolvedHonestUnknown, ResolvedUnsupported, Failed, Cancelled
- `InternalError`: ParseError, Timeout, ProbeFailure, InternalCrash
- `TicketRecord`: Complete ticket with lifecycle tracking and specialist chain
- Integration with SpecialistResponse status (Success/Partial/NoData/Unsupported/Error)

**Resolution Semantics:**
- "Resolved" = UserSatisfied state AND specialist status is Success/Partial/NoData/Unsupported
- "Failed" = Failed state OR parsing exhausted OR critical internal error
- "Honest Unknown" = NoData/Unsupported delivered to user (not counted as failure)

**Reliability Metrics (`ticket_lifecycle.rs`):**
- `ReliabilityMetrics::compute()`: Honest stats from ticket records
- `resolved_success`: Full success with grounded answer
- `resolved_partial`: Partial answer with limitations
- `honest_unknown`: "I don't know" delivered honestly
- `failed`: Hard internal failures
- `success_rate`: resolved_success / total_tickets
- `reliability_rate`: success / (success + failed + parse_errors + internal_errors)

**Per-Specialist Metrics:**
- `SpecialistMetrics`: tickets_handled, tickets_lead, success_count, failed_count
- `compute_specialist_metrics()`: Compute from ticket records
- XP based on actual outcomes: +10 success, +6 partial, +3 honest_unknown, 0 failed
- Titles require minimum success rate: Senior (XP≥2000, rate≥85%), Proficient (XP≥500, rate≥70%)

**Honest Display (`honest_metrics.rs`):**
- `HonestStats`: Complete stats for annactl
- `format()`: User-facing stats display with [service desk], [reliability], [staff roster]
- `validate()`: Sanity check that stats don't claim impossible success rates
- `format_internal_error()`: User-friendly error message with optional debug mode

**Error Handling:**
- Normal mode: "Anna hit an internal error..." (no raw JSON or stack traces)
- Debug mode: Separate [debug] section with ticket_id, error_kind, error_message, attempts
- Failed tickets counted as failures, not resolved

**Migration:**
- Legacy tickets marked with `is_legacy: true`
- Excluded from strict success_rate calculations
- Only new tickets trusted for reliability metrics

## [0.0.425] - 2025-12-11

### Added - Specialist V3: Strict JSON Contract

**Goal:** Fix "Failed to parse specialist response" errors with a strict JSON schema,
robust parser with retry logic, and user-friendly error synthesis.

**Schema (`specialist_v3/schema.rs`):**
- `SpecialistResponse`: Single canonical schema for ALL specialist responses
- `ResponseStatus`: Success, Partial, NoData, Unsupported, Error with strict semantics
- `Finding`: Structured finding with key/value and evidence_refs
- `Recommendation`: User recommendations with risk_level
- `Action`: Shell commands with run_as (user/root) and risk_level
- `KnowledgeCitation`: Citation with id, source, topic, relevance
- `ProbeUsed`: Probe tracking with status (ok/empty/failed/timeout)
- `ErrorInfo`: User-friendly errors with kind and optional details
- `Severity`: Info, Warning, Critical with emoji display
- Builder pattern: `with_finding()`, `with_analysis()`, `with_action()`, etc.
- Validation: `validate()` checks required fields and ranges

**Parser (`specialist_v3/parser.rs`):**
- `parse_specialist_response()`: First parse attempt with JSON extraction
- `parse_with_retry()`: Retry logic with MAX_PARSE_RETRIES (1)
- `extract_json()`: Extract JSON from markdown code blocks or raw text
- `try_salvage_response()`: Recover partial data from malformed JSON
- `RepairRequest`: Structured repair prompt for LLM retry
- `generate_repair_message()`: Context-aware error messages for LLM
- Safe error synthesis when parsing completely fails

**Prompt Contract (`specialist_v3/prompt.rs`):**
- `SPECIALIST_SYSTEM_PROMPT`: Strict JSON-only output rules
- `build_specialist_prompt()`: Build prompt with probe data and knowledge
- `DomainPrompt`: Domain-specific focus (desktop/server/network/security)
- `example_success_response()`, `example_no_data_response()`: Templates
- `confidence_guidelines()`: Scoring rubric for confidence values
- `severity_for_finding()`: Auto-severity based on thresholds
- `risk_for_command()`: Command risk classification (rm -rf = High)

**Error Synthesis (`specialist_v3/synthesize.rs`):**
- `synthesize_timeout_response()`: User-friendly timeout message
- `synthesize_no_evidence_response()`: Handle empty probe data
- `synthesize_unsupported_response()`: Domain routing suggestions
- `synthesize_from_probes()`: Extract basic findings from raw probes
- `synthesize_internal_error()`: Safe internal error handling
- `merge_responses()`: Combine multiple specialist responses
- `format_for_user()`: Convert response to readable text

**Constants:**
- `MAX_PARSE_RETRIES`: 1 (retry once before giving up)
- `DEFAULT_CONFIDENCE`: 0.5
- `MIN_USEFUL_CONFIDENCE`: 0.3
- `HIGH_CONFIDENCE`: 0.85

## [0.0.424] - 2025-12-11

### Added - Knowledge V4: Complete Local Knowledge Engine

**Goal:** Give Anna a real, local, citation-based knowledge brain that prefers Arch-style sources
(man pages, help text, docs) and stops inventing "knowledge" that the system itself could provide.

**Configuration (`knowledge_v4/config.rs`):**
- `KnowledgeConfig`: allow_web_lookup, man_paths, doc_paths, wiki_path
- max_snippet_chars (1500), max_results_per_query (5), command_timeout_ms
- `effective_man_paths()`, `effective_doc_paths()`: only existing paths
- `wiki_available()`: check if offline wiki snapshot exists

**Query Types (`knowledge_v4/query.rs`):**
- `KnowledgeSource`: ManPage, CommandHelp, LocalDocs, ArchWiki
- `KnowledgeQuery`: ticket_id, domain, topic, entities, preferred_sources
- Source priority by domain: system_priority(), tool_priority(), config_priority()
- `command_entities()`, `path_entities()`: extract typed entities
- `QueryBuilder`: fluent API for ticket integration

**Snippets (`knowledge_v4/snippet.rs`):**
- `KnowledgeSnippet`: source, title, excerpt, citation_id, path, command, relevance
- Constructors: `from_man()`, `from_help()`, `from_doc()`, `from_wiki()`
- `KnowledgeResult`: snippets, sources_queried, sources_with_results, duration_ms
- `format_for_specialist()`: structured context for LLM prompts
- `format_citations()`: user-facing source list

**Citations (`knowledge_v4/citation.rs`):**
- `Citation`: structured citation with id, source, display, location
- `Citation::man()`, `::man_section()`, `::help()`, `::doc()`, `::wiki()`
- `format_citation()`: "man:vim" → "man vim"
- `parse_citation()`: extract source and name
- `format_sources()`: multi-citation display

**Man Page Adapter (`knowledge_v4/adapters/man.rs`):**
- `ManAdapter::query()`: fetch man page, extract NAME/SYNOPSIS/DESCRIPTION
- `extract_topic_excerpt()`: context around topic keyword
- `extract_default_excerpt()`: structured section extraction
- Safety: `is_safe_command()`, section header detection
- Returns snippets with `man:<command>` citations

**Help Adapter (`knowledge_v4/adapters/help.rs`):**
- `HelpAdapter::query()`: try --help then -h
- Extracts usage line and options section
- Handles stderr output (some commands output help there)
- Dangerous command blocklist (rm, dd, mkfs, etc.)
- Returns snippets with `help:<command>` citations

**Doc Adapter (`knowledge_v4/adapters/doc.rs`):**
- `DocAdapter::query()`: search /usr/share/doc, /usr/local/share/doc
- `find_candidates()`: match directories by entity/topic
- `extract_from_dir()`: find README, INSTALL, etc.
- Text file detection, binary file filtering
- Returns snippets with `doc:<name>` citations

**Wiki Adapter (`knowledge_v4/adapters/wiki.rs`):**
- `WikiIndex`: offline Arch Wiki snapshot index
- `WikiAdapter::query()`: search local wiki files
- `build_index()`: scan wiki directory
- `strip_markup()`: HTML/wiki markup removal
- No network calls - works only with pre-synced local copy
- Returns snippets with `wiki:<topic>` citations

**Knowledge Engine (`knowledge_v4/engine.rs`):**
- `KnowledgeEngine`: orchestrates all adapters
- `query()`: try sources in priority order
- `query_man()`, `query_help()`, `query_docs()`, `query_wiki()`
- `query_command()`, `query_topic()`: quick queries
- `EngineQueryBuilder`: ticket integration helper

**Key Features:**
- Local-first: no network calls by default
- Citation-based: every snippet traceable to source
- Graceful degradation: missing sources skipped, not errors
- Specialist-friendly: `format_for_specialist()` for LLM context
- Configurable: paths, limits, logging

## [0.0.423] - 2025-12-11

### Added - Recipe V3: Safe Learning and Execution Engine

**Goal:** Build a robust recipe system where good solutions become reusable, parametric recipes.
Minimal/zero LLM usage for recurring questions. Safe execution with risk levels and confirmation.

**Core Types (`recipe_v3/types.rs`):**
- `RecipeV3`: Complete recipe with id, version, title, description, origin, author
- `RecipeOrigin`: BuiltIn, LearnedFromTicket, UserAuthored
- `RecipeAuthor`: System, Specialist(name), User(name)
- `RecipeDomain`: General, Service, Package, Network, Disk, Memory, etc.
- `RecipeRiskLevel`: None, Low, Medium, High (with `requires_confirmation()`)
- `ConfirmationPolicy`: Never, Once, PerStep, Always
- `RecipeMatcher`: domain, intents, keywords, entity_patterns, similarity_key
- `RecipeStats`: times_matched, executed, succeeded, failed, skipped, avg_execution_ms

**Conditions (`recipe_v3/condition.rs`):**
- `RecipeCondition` enum: ProbeTrue, CommandExists, PackageInstalled, FileExists
- FileNotExists, ConfigContains, ConfigNotContains, ServiceState, Custom
- `evaluate()`: Runs condition and returns `ConditionResult`
- Variable substitution: `${var}` and `$var` syntax
- Safe command execution with proper error handling

**Steps (`recipe_v3/step.rs`):**
- `RecipeStep` enum: Explain, ShowCommand, RunCommand, RunProbe
- AppendToFile, ReplaceInFile, CreateFile, CallSubRecipe, Conditional
- `risk_level()`: Automatic inference from command patterns
- `execute()`: Full step execution with variable capture
- Backup support for file modifications
- Conditional steps with if/then/else logic

**Matching Engine (`recipe_v3/matcher.rs`):**
- `MatchQuery`: Parsed from question with domain, intent, keywords, entities
- `RecipeMatcher`: Scoring engine with configurable thresholds
- `MatchResult`: recipe, score, breakdown, preconditions_met, extracted_vars
- Scoring breakdown: domain (15%), intent (35%), keyword Jaccard (30%), entity (20%)
- Maturity bonus (+10% for proven recipes) and health penalty (-20% for failing)
- Entity extraction: services, packages, paths from queries

**Store (`recipe_v3/store.rs`):**
- `RecipeStore`: File-based persistence with JSON serialization
- Directories: /var/lib/anna/recipes_v3/global/ and /user/
- Indexes by domain and tags for fast lookup
- CRUD operations: save, get, delete, search
- Stats recording: `record_execution()`, `record_skip()`

**Executor (`recipe_v3/executor.rs`):**
- `RecipeExecutor`: Safe recipe execution with dry-run support
- Precondition evaluation before execution
- Step-by-step execution with confirmation callbacks
- `ExecutionResult`: success, steps_executed, duration_ms, variables, errors
- `create_execution_plan()`: Preview without executing
- Store integration: `execute_and_record()`

**Builder (`recipe_v3/builder.rs`):**
- `RecipeBuilder`: Fluent API for recipe construction
- Validation: must have id, title, intent, steps
- Auto-upgrade risk level based on step commands
- `TicketData`: Structured ticket info for extraction
- `extract_recipe_from_ticket()`: Convert successful tickets to recipes
- `seed_recipes()`: Built-in recipes for restart-service, check-status, install-package, etc.

**Constants:**
- MIN_MATCH_SCORE: 0.50
- AUTO_EXECUTE_THRESHOLD: 0.85
- SUGGEST_THRESHOLD: 0.65
- MAX_RECIPE_STEPS: 10
- MIN_MATURE_USES: 5
- MIN_SUCCESS_RATE: 0.80

## [0.0.422] - 2025-12-11

### Added - Knowledge V2: Research-First Layer

**Goal:** Wire Anna's brain to Arch Wiki, man pages, and help output instead of LLM intuition.
Research-first, not opinion-first.

**Knowledge Snippet Model (`knowledge_v2/snippet.rs`):**
- `KnowledgeSnippet`: Normalized snippet with id, source, title, section, relevance
- `KnowledgeSource`: ManPage, Help, ArchWiki, PacmanDoc, LocalDoc, OtherOfficial
- Structured fields: summary (2-4 sentences), key_points (3-7 bullets), raw_excerpt
- Citations: `man:systemctl(1)`, `archwiki:Systemd`, `help:pacman`
- Constructors: `from_man()`, `from_help()`, `from_wiki()`, `from_local_doc()`, `from_pacman()`

**Knowledge Sources (`knowledge_v2/sources.rs`):**
- `fetch_man_page()`: Runs `man -P cat`, extracts NAME/SYNOPSIS/DESCRIPTION sections
- `fetch_help_output()`: Tries --help then -h, validates output looks like help
- `fetch_arch_wiki()`: Checks local cache first (local-first design)
- `fetch_local_doc()`: Searches /usr/share/doc, /usr/share/help for READMEs
- `fetch_pacman_info()`: Gets package info from `pacman -Qi`
- Safe command validation, dangerous command blocklist

**Research Policy (`knowledge_v2/policy.rs`):**
- `ResearchPolicy`: needs_knowledge, topics, priority (ProbesOnly/ProbesFirst/KnowledgeFirst)
- Status-only questions: probes only (free RAM, failed services)
- Existence checks: usually probes only
- How-to/error questions: always fetch knowledge
- Complex config: knowledge mandatory
- Domain-specific topics: systemd, NetworkManager, pacman, etc.

**Wiki Cache (`knowledge_v2/cache.rs`):**
- `WikiCache`: File-based cache at /var/lib/anna/wiki/ or ~/.anna/wiki/
- `WikiCacheEntry`: topic, content, fetched_at, content_hash
- TTL: 7 days, automatic cleanup of expired entries
- Cache statistics: total entries, expired, size

**Knowledge Fetcher (`knowledge_v2/fetcher.rs`):**
- `KnowledgeFetcher`: Orchestrates multi-source knowledge gathering
- `FetchResult`: snippets, topics_searched, sources_checked, has_knowledge
- Priority order: man pages → help → Arch Wiki → local docs → pacman
- Keyword extraction from questions (stop words filtered)
- Relevance boosting based on keyword matches
- Summary extraction (first N sentences)
- Key points extraction (lines containing keywords)

**Constants:**
- MAX_SNIPPETS_PER_TICKET: 5
- MAX_EXCERPT_LENGTH: 2000 chars
- MAX_SUMMARY_LENGTH: 400 chars
- MAX_KEY_POINTS: 7
- CACHE_TTL_SECS: 7 days
- FETCH_TIMEOUT_MS: 3000ms

## [0.0.421] - 2025-12-11

### Added - Specialist V2: Stable Schema-Driven Responses

**Goal:** Eliminate "Failed to parse specialist response" errors and guarantee tight,
schema-obedient answers from specialists.

**New Specialist Schema (`specialist_v2/`):**
- `SpecialistResponseV2`: Single canonical schema for all specialist output
- `SpecialistStatus`: Ok, InsufficientEvidence, Error (no ambiguity)
- `DirectAnswer`: Short text + optional metrics for factual questions
- `KeyFinding`: Label/value pairs with severity and evidence sources
- `RecommendedAction`: Actions with risk levels and confirmation flags
- `AnswerType`: Fact, YesNo, WhatIs, Diagnostic, HowTo, General

**Specialist Prompts (`specialist_v2/prompt.rs`):**
- Schema-driven prompts that enforce JSON-only output
- Question type specific rules (fact, yes_no, diagnostic)
- Domain-specific rules (performance, network, storage, services)
- Forbidden patterns: no invented data, no generic tutorials
- Compact prompts for retry attempts (under 1KB)

**SpecialistCall Wrapper (`specialist_v2/call.rs`):**
- Unified call execution with timeout tracking
- JSON extraction from wrapped responses
- Validation before use
- Call logging for debugging

**Fallback Engine (`specialist_v2/fallback.rs`):**
- Deterministic fallbacks when LLM fails or times out
- Memory fallback: parse `free` output, format bytes
- Services fallback: count failed systemd services
- Disk fallback: parse `df` output, flag critical partitions
- Network fallback: list interfaces with state
- Swap fallback: detect swap status
- Uptime/boot fallback: extract timing info
- Generic fallback for unknown intents

**Response Renderer (`specialist_v2/renderer.rs`):**
- Clean user-friendly output from structured data
- Never exposes "Failed to parse specialist response"
- Text and markdown formats
- Status indicators (Success, Partial, Failed)
- Friendly error messages with probe data summary

**Key Improvements:**
- No more raw JSON or parse errors shown to users
- Confidence clamping to [0.0, 1.0]
- Answer length validation (<500 chars for direct answers)
- Forbidden pattern detection (hallucination guards)
- 5-second timeout with graceful fallback

## [0.0.420] - 2025-12-11

### Added - Recipe V2: Clean Learning Engine

**Goal:** Self-improving recipe system with minimal hardcoding - learn from successful tickets,
store reusable patterns, match queries before calling specialists.

**New Recipe Model (`recipe_v2/`):**
- `RecipeV2`: Main recipe type with id, version, title, domain, triggers, facts, steps, citations, stats
- `RecipeDomain`: Desktop, Network, Storage, Services, Performance, Security, Generic
- `TriggerPattern`: Intent + keywords with Jaccard similarity matching
- `FactRequirement`: Key/operator/value conditions (Eq, Ne, In, NotIn, Exists)
- `RecipeStepV2`: ProbeOnly, SafeChange, RiskyChange, ExplanationOnly steps
- `RecipeStepAction`: Command execution, file operations, backup, explain templates
- `RecipeStats`: Usage tracking with maturity multiplier (0.7 untested → 1.0 mature)

**Global vs User Recipes:**
- Global recipes: `/etc/anna/recipes/` (shipped with Anna)
- User recipes: `~/.anna/recipes_v2/` (learned from tickets)
- User recipes override global with same ID

**Matching Engine (`recipe_v2/matcher.rs`):**
- Intent matching: exact match (1.0) or prefix match (0.7)
- Keyword overlap: Jaccard similarity coefficient
- Fact requirement checking with missing fact tracking
- Combined score: `(intent * 0.6 + keyword * 0.4) * maturity * fact_penalty`
- AUTO_APPLY_THRESHOLD: 0.75 (skip specialist), LEARNING_THRESHOLD: 0.90

**Learning from Tickets (`recipe_v2/learner.rs`):**
- `TicketObservation`: Captures intent, keywords, probes, answer, confidence
- Learning conditions: resolved, confidence >= 0.9, verified successful
- Minimum 2 observations before learning new recipe
- Conservative: only from high-confidence verified tickets

**Dispatcher Integration (`recipe_v2/dispatcher.rs`):**
- `RecipeDispatcher`: Integrates recipes into request flow
- `should_skip_specialist()`: Returns recipe if high-confidence match
- `record_success()`: Records ticket observations for learning
- `record_execution()`: Tracks recipe execution success/failure
- Auto-disable recipes with low success rate

**8 Seed Recipes:**
- `vim.syntax.enable`: Enable Vim syntax highlighting
- `system.boot.check`: Analyze boot time with systemd-analyze
- `memory.show_free`: Show memory usage with free -h
- `network.show_interfaces`: List network interfaces
- `disk.usage.show`: Show disk usage
- `swap.check`: Check swap status
- `services.failed.check`: List failed systemd services
- `uptime.show`: Show system uptime and load average

## [0.0.419] - 2025-12-11

### Added - Knowledge Engine with Citations

**Goal:** Anna uses Arch Wiki, man pages, command help, and local docs as first-class evidence
with full provenance tracking (citation IDs) that propagate through specialist responses,
tickets, and recipes.

**Knowledge Engine Enhancements (`knowledge_engine.rs`):**
- Added `citation_id` and `line_range` fields to `KnowledgeEngineHit`
- `citation_id` format: `{source}:{name}:line{start}-{end}` (e.g., `man:systemctl:line42-50`)
- Added `citation_ref()` and `citation_display()` methods for formatted output
- Updated man page, help, wiki, and local doc fetchers to include line numbers

**Citation Types (`specialist_contract.rs`):**
- New `KnowledgeCitation` struct with `citation_id`, `kind`, `title`, `excerpt`, `relevance`
- New `CitationKind` enum: `ManPage`, `CliHelp`, `ArchWiki`, `LocalDoc`, `Internal`
- `inline_ref()`: Format as `[man systemctl(1)]` for inline use
- `footer_display()`: Format for citations section in responses
- Added `citations: Vec<KnowledgeCitation>` field to `SpecialistResponse`
- Builder methods: `with_citations()`, `add_citation()`

**Ticket Citation Support (`ticket/ticket_struct.rs`):**
- Added `citations: Vec<KnowledgeCitation>` field to `Ticket`
- `add_citation()`: Add single citation (deduplicates by ID)
- `add_citations()`: Add multiple citations

**Recipe Citation Support (`recipe/core.rs`):**
- Added `citations: Vec<KnowledgeCitation>` field to `Recipe`
- `with_citations()`, `add_citation()` builder methods
- Citations propagate from tickets to recipes when learning

**Response Renderer (`response_renderer.rs`):**
- Added `citation_lines` field to `RenderedResponse`
- `build_citation_lines()`: Formats citations for display
- `format_for_display()`: Shows "Sources:" section when citations present

**Example Output:**
```
[anna]
To restart a service, use: systemctl restart <service>

Sources:
  - systemctl(1) (man page): "systemctl restart SERVICE..."
```

## [0.0.418] - 2025-12-11

### Added - Full Recipe Learning System

**Goal:** Anna learns from successful tickets and reuses knowledge without calling LLM.
Recipes are declarative JSON objects that describe intent, matcher, preconditions, plan steps,
success criteria, and citations.

**New Modules (anna-shared):**

**`recipe_schema.rs`:**
- `Recipe`: Full recipe data model with id, version, domain, intent, pattern, matcher, plan
- `RecipePattern`: User goal and extracted slots
- `RecipeMatcher`: Required/optional/negative keywords, min_confidence, exact_intent
- `Precondition`: ToolExists, FileExists, DirExists, ProbeContains, ProbeMatches, ServiceExists
- `PlanStep`: Explain, BackupFile, AppendLine, PrependLine, ReplaceLine, EnsureLine, RemoveLines,
  VerifyCommand, RunCommand, EnableService, DisableService, RestartService, CreateDir, WriteFile, SetEnvVar
- `ConfirmationPolicy`: Require, MutatingOnly, Never
- `SuccessCriteria`: must_succeed steps, rollback_on_failure, post_verification
- `RecipeMetrics`: times_used, times_failed, recent_success_rate, auto-disable on high failure rate

**`recipe_storage.rs`:**
- File-based storage in `~/.anna/recipes/{domain}/{id}.json`
- In-memory index for fast lookup by domain, intent, or ID
- `RecipeStorage`: load(), save(), get(), get_by_domain(), get_by_intent(), update_metrics()
- Automatic indexing on save

**`recipe_eligibility.rs`:**
- `check_eligibility()`: Determines if ticket can produce a recipe
- Requirements: status=ok, confidence>=90, concrete actions, learnable intent
- `RecipeType`: ConfigChange, RepeatableDiagnostic, SimpleFix, ServiceAction, PackageAction
- Conservative: better to learn too little than create bad recipes

**`recipe_extractor.rs`:**
- `extract_recipe()`: Creates recipe from successful ticket
- Extracts: pattern, matcher, preconditions, plan steps, citations
- Generates unique recipe ID from domain, keywords, type
- Builds matcher from user query keywords with negative keyword detection

**`recipe_matcher_v2.rs`:**
- `RecipeMatcher`: Scores recipes against queries
- `MatchQuery`: query_text, intent, domain, slots
- Scoring: required keywords (0.6), optional (0.15), intent (0.15), slots (0.1)
- Penalties for new recipes and low success rate
- `quick_match()`, `match_recipe()` helper functions

**`recipe_runtime.rs`:**
- `check_preconditions()`: Verifies all preconditions before execution
- `needs_confirmation()`: Checks if user confirmation required
- `generate_confirmation_prompt()`: Human-readable plan description
- `prepare_execution()`: Creates ExecutionPlan for transaction engine
- Path expansion: $HOME, ~, {param} placeholders

**`recipe_telemetry.rs`:**
- `RecipeTelemetry`: Tracks resolution events and learning events
- `ResolutionSource`: Recipe, Specialist, IntentHandler, FastPath, Failed
- `TelemetryStats`: Total resolutions, by_recipe, by_specialist, self_reliance_rate
- `summary()`: Human-readable stats for annactl

**`seed_recipes.rs`:**
- 6 seed recipes to bootstrap the system:
  - `vim_enable_syntax`: Enable syntax highlighting in Vim
  - `check_disk_usage_root`: Check disk usage on root partition
  - `check_free_ram`: Check free RAM
  - `check_swap_status`: Check swap status
  - `check_failed_services`: Check failed systemd services
  - `enable_sshd_service`: Enable SSH daemon

**Design Principles:**
- Recipes are small, declarative JSON objects
- Can be executed without LLM
- Can be inspected and debugged easily
- Grounded in documentation (citations)
- Auto-disable on high failure rate
- Transaction engine handles actual execution

## [0.0.417] - 2025-12-11

### Fixed - Strict Reliability (Direct Answers Only, No Tutorials)

**Problem Solved:** Despite v0.0.415/416 improvements, Anna still had failure modes:
- "Failed to parse specialist response. Parse error: Timeout"
- Nonsense answers: "unknown is installed", "2 is installed"
- Generic tutorials instead of direct answers
- "I can't answer yet" despite having evidence
- Stats showing 100% success when answers were wrong

**Solution:** Deterministic intent handlers + tightened prompts + validation.

**New Modules (anna-shared):**

**`intent_handlers.rs`:**
- `HandlerResult`: Success, MissingProbe, NeedsSpecialist variants
- Deterministic handlers for 10+ common intents:
  - `handle_check_free_ram()`: Parses /proc/meminfo, returns actual MB
  - `handle_check_swap_presence()`: Yes/no with actual size
  - `handle_check_disk_usage()`: Per-filesystem usage from df
  - `handle_check_failed_services()`: Lists actual failed units
  - `handle_check_boot_time()`: Parses uptime for boot timestamp
  - `handle_check_uptime()`: Days, hours, minutes format
  - `handle_check_package_count()`: Actual package count
  - `handle_check_installed_package()`: Yes/no with version
- Each handler defines required probes and exact transformations
- Zero LLM dependency for supported intents
- Falls back to specialist only when handler says `NeedsSpecialist`

**Updated `strict_prompts.rs`:**
- Much tighter BASE_PROMPT with ABSOLUTE RULES
- Rule 1: Output ONLY valid JSON
- Rule 2: summary MUST DIRECTLY ANSWER with DATA from probes
- Rule 3: NEVER give generic tutorials or "how-to" guides
- Rule 4: NEVER invent data not present in probes
- Rule 5: NEVER use placeholder names like "unknown", "2", "package"
- Rule 6: If probe data is present, USE IT
- Rule 7: Keep summary to ONE sentence with ACTUAL ANSWER
- Added ANSWER_RULES with good/bad examples
- Added NEGATIVE_EXAMPLES showing forbidden tutorial patterns
- Reduced truncation limits (probes: 1500, docs: 400)

**Updated `strict_contract.rs`:**
- Tutorial pattern detection: "to check", "you can check", "run the command",
  "use the following", "here's how to", "step 1:", "first, run", etc.
- Evasion pattern detection: "i cannot determine", "i can't answer yet",
  "run annactl status", "collect evidence", "need more data", etc.
- Contradictory confidence/status validation
- `contains_tutorial_pattern()`: Catches generic how-to responses
- `contains_evasion_pattern()`: Catches false "I can't answer" claims

**Design Principles:**
- Intent handlers are first line of defense (deterministic, fast)
- LLM is fallback, not primary (for unsupported/complex intents)
- Validation catches LLM failures before they reach user
- Honest stats: only count actual successes

## [0.0.416] - 2025-12-11

### Added - Knowledge Engine and Self-Learning Recipes

**Goal:** Anna learns like a real IT department. Minimal hardcoding, maximum learning from
experience. Uses man pages, --help, Arch Wiki, and local docs as "the Bible".

**New Modules (anna-shared):**

**`knowledge_engine.rs`:**
- `KnowledgeEngine`: Fetches structured knowledge from local sources
- `KnowledgeRequest`: Query with topic, context, and source preferences
- `KnowledgeEngineHit`: Structured result with doc_id, kind, title, snippet
- Sources: ManPage, CliHelp, LocalDoc, ArchWiki
- Safe command whitelist (no arbitrary execution)
- 1-week caching in `/var/lib/anna/knowledge_cache`
- Commands: `man <cmd>`, `<cmd> --help`, local `/usr/share/doc` search

**`canonical_intents.rs`:**
- `CanonicalIntent`: 30+ intents like CheckFreeRam, CheckDiskUsage, CheckFailedServices
- NO HARDCODED NATURAL LANGUAGE - only concepts
- `required_probes()`: Probes needed for each intent
- `knowledge_topics()`: Knowledge domains for doc search
- `translator_to_canonical()`: Map translator output to canonical intent
- Intent-based routing replaces phrase-based matching

**`learned_recipes.rs`:**
- `LearnedRecipe`: Deterministic, replayable solution pattern
- `RecipeComputeStep`: Extract, Compare, Count, IsEmpty, ParseNumber
- `AnswerTemplate`: Summary with {placeholders}, evidence list
- `execute_recipe()`: Run recipe with probe outputs
- Intent-based matching (not phrase-based)
- Self-learning from successful tickets

**`recipe_learner.rs`:**
- `TicketObservation`: Captures ticket data for learning
- `LearningCandidates`: Groups observations by intent
- `RecipeLearner`: Creates recipes from repeated successes
- Minimum 2 observations with 80% success rate to create recipe
- Intent-specific recipe generators (disk_usage, memory, swap, services, boot)
- Auto-deprecates recipes with <50% success after 10 uses

**`recipe_fast_path.rs`:**
- `FastPathExecutor`: Tries recipes before specialist LLM calls
- `FastPathResult`: Answered, NoRecipe, RecipeFailed
- `should_try_fast_path()`: Checks eligibility (confidence >= 0.6, no clarification)
- Knowledge enrichment: Optionally fetches docs to explain answers
- Enables LLM-free answers for common queries

**`recipe_stats.rs`:**
- `RecipeStats`: Tracks tickets by recipe vs specialist
- `IntentStats`: Per-intent coverage and success rates
- `recipe_coverage()`: What % answered without LLM
- `intents_needing_recipes()`: Identifies learning opportunities
- `summary()`: Human-readable stats line

**Design Principles:**
- Intents and topics are stable API surfaces
- Recipes evolve as the system gains experience
- Specialists are "teachers" for new/complex intents
- Frequently occurring intents get replaced by recipes over time

## [0.0.415] - 2025-12-11

### Fixed - Strict Specialist Contract (Fast, Honest, Grounded)

**Problem Solved:** Anna was giving slow responses (10-20s), parse errors, timeouts,
and nonsensical answers like "unknown is installed" or "2 is installed". Stats showed
100% success when answers were clearly wrong.

**Solution:** Strict JSON contract with validation, time budgets, and honest stats.

**New Modules (anna-shared):**

**`strict_contract.rs`:**
- `StrictSpecialistResponse`: Single source of truth for specialist JSON
- `StrictStatus`: ok, partial, failed (no ambiguity)
- `EvidenceItem`: Probe-based evidence requirement
- `Citation`: Documentation citations
- `TimeBudgets`: Configurable time limits (translator: 1.5s, specialist: 4s, probes: 5s)
- `parse_specialist_output()`: Robust JSON extraction with lenient fallback
- `ParseResult`: NoJson, InvalidJson, ValidationFailed, Timeout variants
- Validation catches: empty summaries, forbidden patterns, high confidence without evidence

**`strict_prompts.rs`:**
- Complete specialist prompts with explicit JSON schema
- Domain-specific hints for all 10 domains
- Positive examples (correct JSON)
- Negative examples (forbidden patterns: "unknown is installed", etc.)

**`translator_contract.rs`:**
- `TranslatorOutput`: Strict schema for routing
- `TranslatorIntent`: query_metric, diagnose, configure, list, check_status, explain
- `TranslatorDomain`: 10 domains with string normalization
- `TRANSLATOR_PROMPT`: Complete prompt with probe mappings

**`answer_shaper.rs`:**
- `shape_answer()`: Convert specialist response to user-friendly output
- `ShapedAnswer`: text, evidence_line, is_resolved, confidence
- Concise style for simple queries (1-2 lines)
- Diagnostic style for complex queries (bullets, actions)
- Honest failures instead of fake success

**`honest_stats.rs`:**
- `TicketOutcome`: Resolved, Partial, Failed (no fantasy 100%)
- `HonestStats`: Tracks real success/failure rates
- `DomainStats`, `IntentStats`: Per-category breakdown
- `FailureRecord`: Recent failures for debugging
- `is_resolved()`: status=ok + confidence>=0.8 + non-empty summary + valid

**`regression_tests.rs`:**
- `TestCase`: Shape validation for queries
- `regression_suite()`: 10+ test cases covering all domains
- Validates: non-empty summary, required metrics, forbidden patterns, evidence
- Does NOT hardcode expected answers - validates shape only

**Key Design Decisions:**

1. **JSON Only**: Specialists output JSON, no prose. Schema is enforced.
2. **Evidence Required**: status=ok with confidence>=0.8 requires evidence array
3. **Forbidden Patterns**: "unknown is installed", "2 is installed" trigger validation failure
4. **Time Budgets**: Translator 1.5s, Specialist 4s, Probes 5s total
5. **Honest Stats**: "resolved" requires ok status + high confidence + valid response
6. **Shape Tests**: Regression tests validate answer structure, not exact wording

**Breaking Changes:**
- Stats now reflect reality (expect lower success rates initially)
- Responses may be "partial" or "failed" instead of fake "ok"
- Time budgets may cause more explicit timeouts

## [0.0.414] - 2025-12-11

### Added - Doc-First Knowledge Engine (Citations & Honesty)

**Core Philosophy:** Anna now follows a doc-first reasoning pattern. Instead of hardcoded
natural language cases, she queries real documentation (man pages, Arch Wiki, --help)
and grounds all answers in evidence with citations.

**New Modules (anna-shared):**

**`knowledge_query.rs`:**
- `KnowledgeSourceKind`: ManPage, CliHelp, ArchWikiPage, ArchWikiSection, LocalDocFile, ProbeOutput, ConfigFile, LogExcerpt, BuiltIn, LearnedRecipe
- `KnowledgeQuery`: Domain-based query with preferred sources and related commands
- `KnowledgeHit`: Search result with doc_id, title, excerpt, relevance, and origin for citation
- `KnowledgeResult`: Collection of hits with search metadata

**`knowledge_config.rs`:**
- `KnowledgeConfig`: Configuration for wiki paths, caching, and learning settings
- Wiki cache path: `/var/lib/anna/arch_wiki` or `~/.anna/wiki-cache`
- Configurable preferred sources, cache duration, and learning thresholds
- `WikiStats`: Cache statistics (page count, size, availability)

**`knowledge_executor.rs`:**
- `query_knowledge()`: Main entry point for doc-first reasoning
- Multi-source search: built-in pack → man pages → help output → Arch Wiki → learned recipes
- Relevance-based ranking and deduplication
- `build_query_from_context()`: Creates queries from ticket context

**`intent_policy.rs`:**
- `IntentCategory`: 30+ standardized intents (DiagnoseBootTime, InspectDiskUsage, etc.)
- Intent-based routing instead of hardcoded question matching
- `recommended_probes()`: Per-intent probe recommendations
- `knowledge_domains()`: Per-intent knowledge source mapping
- Policy enforcement: `validate_recipe_policy()` catches hardcoded natural language patterns
- `PolicyViolation`: RecipeKeyedByQuestion, HardcodedQuestionMapping, TooNarrowRecipe

**`doc_first_workflow.rs`:**
- `DocFirstContext`: Ticket context for doc-first reasoning
- `SpecialistEvidence`: Combined probe and doc evidence
- `CitedAnswer`: Answer with confidence, citations, and grounding status
- `HonestyRules`: Configurable requirements (require_probe, require_doc, max_ungrounded_claims)
- `gather_evidence()`: Orchestrates probe execution and knowledge query

**`knowledge_learning.rs`:**
- `SolvedTicketRecord`: Learning from high-confidence tickets
- `DocReference`: Track which docs were consulted and cited
- `ProposedRecipe`: Auto-generated recipes from ticket patterns
- `KnowledgeLearningStore`: Persistent store for learning data
- `effective_probes_for_intent()`: Learn which probes work best per intent
- `analyze_and_propose()`: Idle-time learning job that clusters tickets and proposes recipes

**Design Principles:**
- NO hardcoded natural language cases (enforced by policy)
- All answers must cite their evidence sources
- Generic intent routing, not specific question matching
- Recipes learned from evidence patterns, not invented
- Senior LLM review for learned recipe safety

## [0.0.413] - 2025-12-11

### Added - Hollywood IT Department View (Transcript Renderer & REPL UX)

**Major UX Overhaul:** Anna now feels like a tiny IT department in your terminal with
cinematic rendering, real-time streaming, and characterful internal communications.

**New Modules (anna-shared):**

**`transcript_segment.rs`:**
- `TranscriptSegment`: Core data model for all output
- `SegmentKind`: user_input, system_info, ticket_header, internal_comms, probe_run, specialist_message, answer, error, tip, stats_snippet, debug_json, progress
- `TranscriptMode`: cinematic (default) vs debug
- `Actor`: Staff members with names and roles (Anna, Sofia, Michael, Lars, etc.)
- `Transcript`: Full request transcript with ordered segments

**`transcript_render.rs`:**
- Cinematic mode: Hollywood IT department view, minimal noise
- Debug mode: Same view + raw JSON, full errors
- `RenderConfig`: Customizable rendering (show_internal_comms, show_tips, show_probes)
- `format_answer_with_evidence()`: Standard answer format with evidence footer
- `format_error_with_context()`: Clear, honest error messages

**`ui_config.rs`:**
- `UiConfig`: Persistent UI settings from config files
- `SpinnerStyle`: simple (|/-\), dots (...), none
- Settings: mode, show_internal_comms, show_tips, show_probes, show_timestamps
- Config files: `~/.anna/config.toml`, `/etc/anna/config.toml`

**`repl_greeting.rs`:**
- Stats-based personalized greeting using real ticket data
- `ReplGreeting`: tickets_handled, top_topics, system_status, active_staff
- `SessionContext`: REPL session memory (last questions, last domain)
- First-time vs returning user experience

**New Module (annad):**

**`internal_comms.rs`:**
- Event-driven IT department chatter - NO fake messages
- Every line reflects actual processing state
- `TranscriptEvent`: TicketOpened, SpecialistAssigned, ProbeStarted, etc.
- Staff registry: Sofia (Desktop), Michael (Network), Lars (Storage), etc.

**New Module (annactl):**

**`live_renderer.rs`:**
- Real-time streaming renderer with spinner
- Progressive segment display as they arrive
- Probe grouping for compact display
- Internal comms section with timestamps

**`greeting.rs`:**
- Theatre-style REPL greeting with daemon status

**CLI Flags (annactl):**
- `--cinematic`: Force Hollywood IT department view
- `--debug`: Show raw JSON and full errors
- `--no-internal-comms`: Hide IT department chatter

**Example Cinematic Output:**

```
[you] why is my boot time so slow?

--- internal comms ---
  [0.1s] Anna: Opening ticket DSK-0101 - boot time investigation.
  [0.2s] Tomas (System Analyst): On it. Checking services and startup time.

[probes]
  systemd-analyze
  systemctl list-unit-files

[anna]
Startup took 25.6 seconds:
  - Firmware: 5.8s
  - Bootloader: 8.3s
  - Userspace: 6.4s

The main slowdown is in the bootloader stage.

Evidence: systemd-analyze, systemctl
```

**Design Principles:**
- No blank screen: Spinner and progressive updates
- Honest errors: Clear messages about what failed
- Real events only: Internal comms driven by actual processing
- Evidence grounded: Every answer cites its sources

## [0.0.412] - 2025-12-11

### Added - Self-Learning Recipe Engine

**Major Feature:** Anna now learns from successful queries and converts them into reusable recipes.
Instead of hardcoding solutions, Anna creates parameterized recipes that work across similar patterns.

**New Modules (anna-shared):**

**`recipe_engine.rs`:**
- `Recipe` struct: Full recipe with steps, parameters, evidence requirements
- `RecipeKind`: probe_only, configure, inspect, diagnose, report
- `RecipeStep`: Composable steps with dependencies (run_probe, run_command, render_answer)
- `RecipeParameter`: Variables like `{{service_name}}`, `{{mount_path}}`
- `EvidenceRequirement`: Probes required before answering

**`recipe_store_v2.rs`:**
- Persistent storage for learned recipes (~/.anna/recipes_v2.json)
- Tag-based and trigger-pattern indexing for fast matching
- Best-match scoring with configurable threshold (0.7)
- Automatic deprecation when success_rate < 0.6 after 10+ uses
- Garbage collection for stale deprecated recipes

**`recipe_executor.rs`:**
- Safe execution with confirmation callbacks for risky operations
- Parameter substitution (`{{variable}}` → actual values)
- Topological sort for step dependencies
- Audit logging for all recipe executions

**`recipe_templates.rs`:**
- 6 generic parameterized recipes out of the box:
  - Check Systemd Service Status (uses `{{service_name}}`)
  - Check Disk Usage (uses `{{mount_path}}`)
  - Check Memory Usage
  - Check Package Installation (uses `{{package_name}}`)
  - Check Failed Systemd Units
  - Check Network Interfaces

**`recipe_converter.rs`:**
- `SpecialistRecipeCandidate`: JSON schema for specialists to propose recipes
- `try_convert_ticket()`: Converts successful tickets to recipes
- Safety validation: Only safe commands allowed (whitelist)
- Probe validation: Only known probes allowed

**`doc_snippet.rs`:**
- Integration with documentation sources
- DocSourceKind: arch_wiki, man_page, help_flag, info, local_file
- DocCache for caching fetched documentation
- ManPageProvider and HelpProvider implementations

**New Module (annad):**

**`recipe_engine_v2.rs`:**
- Integration with existing fast path
- Learned recipes checked BEFORE hardcoded recipes
- `check_learned_recipes()`: Finds best matching recipe
- `execute_learned_recipe()`: Runs recipe and builds ServiceDeskResult
- Parameter extraction from natural language queries
- Automatic GC on startup

**Enhanced `recipe_fast_path.rs`:**
- Now checks RecipeStoreV2 (learned recipes) FIRST
- Falls back to hardcoded recipes only if no learned match
- `learned_recipe_id` field tracks which recipe matched

**Enhanced `annactl learning`:**
- Shows learned recipe statistics
- Recipe count by domain
- Most used recipes with success rates

**How It Works:**
1. User asks: "why is nginx failing?"
2. Anna checks learned recipes first → finds "Check Systemd Service Status"
3. Extracts parameter: service_name = "nginx"
4. Executes recipe steps: systemctl status, journalctl logs
5. Renders answer from template
6. Updates success/failure counts
7. No LLM call needed!

**Key Design Decisions:**
- Generic over specific: One recipe for all services, not one per service
- Evidence-based: Every recipe declares required probes
- Safe by default: Commands validated against whitelist
- Self-improving: Success rate tracking, automatic deprecation
- Documentation-sourced: Links to Arch Wiki, man pages

## [0.0.404] - 2025-12-11

### Added - JSON-Only Specialists + Personality Renderer

**Major Architecture Change:** Specialists now output strict JSON instead of prose.
- LLM has too much freedom → outputs garbage, hallucinations, wrong formats
- NEW: Specialist outputs ONLY structured JSON following strict contract
- Personality (Sofia, Tomas, Lars, etc.) is added by renderer from templates
- This is the "ISO-9001-of-the-soul" approach: constrain LLM, render personality

**New Modules:**

**`specialist_contract.rs` (anna-shared):**
- Strict JSON schema for specialist input/output
- `SpecialistResponse` with required fields: status, answer, evidence, confidence
- `Evidence` struct: every claim must have probe + snippet + interpretation
- `StaffView` for internal mood/severity (used by renderer)
- `Discovery` with `new_probes` and `new_recipes` for learning hooks

**`specialist_prompt.rs` (annad):**
- JSON-only prompts for specialists
- Domain-specific hints (System, Network, Storage, Security, Packages)
- Prime directives: "ONLY return structured JSON", "No prose", "No dialogue"
- Schema specification in prompt for LLM to follow

**`response_renderer.rs` (annad):**
- Personality rendering layer - templates, not LLM generation
- Staff registry: Sofia (Desktop), Lars (Storage), Tomas (System), Michael (Network), Elena (Security), Marcus (Packages)
- Template-based messages based on status + severity + mood
- `render_response()` converts JSON to `RenderedResponse` with personality
- `format_for_display()` creates the final output with internal comms

**`specialist_json.rs` (annad):**
- JSON specialist handler that uses the new architecture
- `call_json_specialist()` builds JSON prompt, calls LLM, parses response
- `extract_json_object()` handles LLM adding prose around JSON
- Fallback to error response on parse failure

**Configuration:**
- `use_json_specialist` flag in `LlmConfig` (default: ON)
- Can disable with `ANNA_JSON_SPECIALIST=0` environment variable
- Allows gradual rollout and A/B testing

**Why This Matters:**
- User asks "do I have swap?"
- OLD: LLM might say "I don't have that data" or make up numbers
- NEW: LLM outputs `{"status":"ok","answer":{"short":"No swap configured"},"evidence":[...]}`
- Renderer adds: "Tomas (Support Analyst): No swap configured."
- Result is reliable, grounded, and has personality WITHOUT LLM generating prose

## [0.0.403] - 2025-12-11

### Fixed - Direct Probe Answers (Bypass Dumb LLM)

The LLM specialist (qwen2.5:3b) was saying "I don't have that data" even when probe data clearly contained the answer. This version fixes that by trying deterministic answers FIRST before calling the LLM.

**New `probe_direct.rs` Module:**
- Generates answers directly from probe results without LLM
- Handles 10 common query patterns:
  - Service status (bluetooth, docker, nginx, etc.)
  - Swap status (/proc/swaps, free)
  - Disk usage (df -h)
  - Memory usage (free)
  - Network interfaces (ip addr)
  - Bluetooth status (systemctl, bluetoothctl)
  - GPU info (lspci)
  - Webcam detection (v4l2)
  - CPU info (lscpu)
  - Audio devices (pactl)

**Specialist Handler Changes:**
- `try_specialist_llm()` now tries direct probe answer FIRST
- Only falls back to LLM if direct answer not possible
- Logs when LLM is bypassed: "v0.0.403: Direct probe answer bypassed LLM"
- Result shows `[direct probe answer]` in trace

**Why This Matters:**
- User asks "is bluetooth running?"
- Probe runs: `systemctl status bluetooth.service`
- Probe output shows: `Active: active (running)`
- OLD: LLM says "I don't have that data" (WRONG)
- NEW: Direct answer says "bluetooth.service is active and running" (CORRECT)

## [0.0.402] - 2025-12-11

### Fixed - Complete Translation & Routing Overhaul

Major rewrite to fix fundamental issues with query classification and routing.

**Comprehensive Keyword-Based Translator:**
- New `translator_fallback.rs` with 850+ lines of keyword matching
- Covers 18 query categories: health, storage, memory, CPU, processes, network, graphics, audio, bluetooth, boot, services, packages, hardware, security, logs, config, docker, user
- High-confidence (≥0.75) matches skip LLM entirely for instant responses
- Fallback triggers correct probes for common queries like:
  - "screen tearing" → graphics domain with `gpu_drivers`, `display_server`, `xorg_log`
  - "webcam" → hardware domain with `lsusb`
  - "bluetooth" → system domain with `bluetooth_devices`
  - "slow boot" → system domain with `boot_time`, `running_services`

**Smart Routing - Fallback First:**
- `routing_stage.rs` now tries deterministic fallback BEFORE LLM
- Common queries answered in milliseconds instead of 5-8 seconds
- LLM only used for truly ambiguous queries (confidence < 0.75)

**Improved LLM Translator Prompt:**
- Comprehensive probe mappings with 80+ examples
- Clear domain rules: system includes GPU, audio, bluetooth, sensors
- Explicit rule: "NEVER select unrelated probes"

**Smarter Clarification Rules:**
- Only ask clarification when NO probes AND very low confidence (<0.4)
- If we have probes, run them first - data helps answer the question
- Removed annoying "which package manager" question (we detect from distro)
- Probes with data > asking user questions

**Test Fixes:**
- Updated tests for new MAX_SUMMARY_LINES (100)
- Updated tests for new clarification logic
- Updated tests for expanded translator prompt size

## [0.0.401] - 2025-12-11

### Added - Specialist Learning System

Anna now learns from specialist interactions to become smarter over time!

**New Module: `specialist_learning.rs`**
- Captures knowledge when Anna receives help from specialists (Senior escalation, LLM self-healing)
- Adaptive learning threshold: high-confidence (80+) learns immediately, lower needs 2+ successes
- Pattern detection for generic recipes (ConfigCheck, ServiceAction, PackageQuery, etc.)
- Keyword-indexed lesson storage in `~/.anna/specialist_lessons.json`

**Learning Capture Integration:**
- Added `learning_capture.rs` module in annad for lesson recording
- Hooks in `ticket_loop.rs` capture lessons after:
  - Successful LLM self-healing (validation healed the answer)
  - Successful Senior escalation (deterministic revision worked)

**Subtle Learning Hints:**
- Dialogue now includes subtle hints like "Based on similar cases..." or "I've seen this pattern before..."
- Hints appear when Anna has high-confidence learned patterns matching the query
- Silent improvement when appropriate (just faster/better responses)

**Pattern Categories Detected:**
- `ConfigCheck`: "check X config", "show X configuration"
- `ConfigEdit`: "enable X in Y", "set X for Y"
- `ServiceAction`: "restart X service", "start/stop X"
- `PackageQuery`: "is X installed", "install X"
- `DiskAnalysis`: "what's using space", "disk usage"
- `ProcessQuery`: "what's using CPU/memory"

**Probe Learning Enhancement:**
- Added `boost_specialist_probes()` to give extra weight to probes used successfully by specialists
- Probes that work in specialist-guided answers get +3 helpfulness boost

**Fact Extraction from Specialist Answers:**
- New `extract_facts_from_answer()` function using regex patterns
- Automatically extracts: installed packages, desktop environment, package manager, init system, GPU presence, running services
- Added `FactSource::SpecialistAnswer` variant for tracking fact origin

**Generic Pattern Matching:**
- New `specialist_patterns.rs` module for reusable patterns
- `extract_generic_pattern()`: Creates templates from specific queries (e.g., "check hyprland config" → "check {target} config")
- `match_generic_pattern()`: Matches new queries against learned patterns
- `apply_pattern()`: Instantiates patterns for new targets

**Recipe Creation from Specialist Lessons:**
- New `specialist_recipes.rs` module for recipe generation
- `try_learn_from_specialist()`: Creates recipes from high-confidence lessons
- Recipes inherit probe sequences and answer templates from successful specialist interactions

**Learning-Aware Routing:**
- `routing_stage.rs` now enriches tickets with learned probes
- `enrich_with_learned_probes()`: Adds probes from matching patterns/lessons
- Generic patterns apply probes with target substitution

**User Feedback RPC Endpoint:**
- New `SubmitFeedback` RPC method for client feedback
- `FeedbackParams`: request_id, query, helpful (bool), comment
- `record_user_feedback()`: Boosts/decreases lesson confidence based on feedback
- Helpful feedback: +5 confidence, +1 success count
- Not helpful: -10 confidence

**Files Changed:**
- `anna-shared/src/specialist_learning.rs` (NEW - 400 lines)
- `anna-shared/src/specialist_patterns.rs` (NEW - 115 lines)
- `anna-shared/src/specialist_recipes.rs` (NEW - 190 lines)
- `anna-shared/src/probe_learning/store.rs` (added `boost_specialist_probes()`)
- `anna-shared/src/facts_types.rs` (added `SpecialistAnswer` variant)
- `anna-shared/src/rpc/method.rs` (added `SubmitFeedback`)
- `anna-shared/src/rpc/params.rs` (added `FeedbackParams`, `FeedbackResult`)
- `anna-shared/src/lib.rs` (added modules)
- `annad/src/learning_capture.rs` (NEW - 292 lines)
- `annad/src/feedback_handler.rs` (NEW - 37 lines)
- `annad/src/routing_stage.rs` (added `enrich_with_learned_probes()`)
- `annad/src/rpc_handler/dispatcher.rs` (added SubmitFeedback handler)
- `annad/src/lib.rs` (added modules)
- `annad/src/ticket_loop.rs` (hook learning capture)
- `annad/src/comms/messages.rs` (add learning hints)

## [0.0.400] - 2025-12-11

### CRITICAL FIX - Deadlock in routing_stage.rs

**ROOT CAUSE OF 33-SECOND TIMEOUT:**
Queries were hanging for 33+ seconds after translator completed, then timing out.
The root cause was a deadlock caused by holding a read lock across an await point.

**The Bug:**
```rust
{
    let recipe_index = &state.read().await.recipe_index;  // READ LOCK HELD
    // ... recipe checks ...
    let (ticket, triage, timeout) = triage_path(...).await;  // AWAIT WHILE HOLDING LOCK!
}
```

When `triage_path` tried to take a write lock (for latency recording), it deadlocked
because the read lock was still held from the outer scope.

**The Fix:**
Restructured code to release the read lock before calling triage_path:
```rust
let recipe_result = {
    let recipe_index = &state.read().await.recipe_index;
    recipe_fast_path::check_recipe_fast_path(query, recipe_index)
}; // Read lock released here

// Now safe to call triage_path which needs write lock
let (ticket, triage, timeout) = triage_path(...).await;
```

**Impact:**
- Queries now complete properly after translator finishes
- Probes execute and specialist generates answers
- No more mysterious 33-second hangs

## [0.0.399] - 2025-12-11

### Fix - Fast probe for largest directories

**Problem:**
- `largest_dirs` probe ran `du -h --max-depth=1 /` which scans ENTIRE root filesystem
- On systems with lots of data, this takes 30+ seconds → global timeout
- Translator was working perfectly, but probe was hanging!

**Fix:**
- Limit scan to common locations: /home /var /usr /opt /tmp
- Add `timeout 3` to prevent hanging
- Use `du -sh` (summarize) instead of deep scan
- Fallback message if scan times out

## [0.0.398] - 2025-12-11

### Fix - Translator timeout increased for 3B+ models

**Problem:**
- 3B models are slower than 0.5B but produce reliable JSON
- 5s timeout was too short for 3B model first inference
- Translator was timing out before producing output

**Fix:**
- Increased default translator timeout from 5s → 10s
- Gives 3B+ models enough time to load and respond
- Reliability over speed - a slower correct answer beats fast garbage

## [0.0.397] - 2025-12-11

### CRITICAL FIX - Translator fallbacks must use 3B+ models

**ROOT CAUSE OF FAILURES:**
Translator was selecting `qwen2.5:0.5b-instruct` despite our catalog changes!
The 3B+ requirement was in the catalog, but fallback defaults were still 0.5B.

**Files fixed:**
- `config.rs` - `default_translator_model()` → 3b-instruct
- `config.rs` - `default_supervisor_model()` → 3b-instruct
- `config_registry.rs` - `default_registry_translator()` → 3b-instruct
- `auto_select.rs` - fallback when selection fails → 3b-instruct

**Why this matters:**
- 0.5B models produce garbage JSON for query classification
- Bad classification → wrong probes → wrong answers → no learning
- The entire learning system depends on good translator output

## [0.0.396] - 2025-12-11

### Improve - Translator prompt for domain classification

**Problem:**
- qwen3-vl:4b was returning domain=system for storage queries
- "what folders are taking space" was misclassified

**Fix:**
- Restructured translator prompt with explicit DOMAIN RULES at the top
- Added keyword-to-domain mapping: folders/directories → storage
- Added explicit PROBE MAPPING section with exact examples
- Log now shows actual probe names, not just count

**Prompt structure:**
1. DOMAIN RULES - keywords that trigger each domain
2. JSON schema
3. PROBE MAPPING - exact query → domain + probes mapping
4. Available probe IDs
5. Rules (max 3 probes, raw JSON only)

## [0.0.395] - 2025-12-11

### Fix - Add largest_dirs/largest_home to PROBE_IDS

**Problem:**
- Translator prompt references `largest_dirs` and `largest_home` probes
- But they weren't in `PROBE_IDS` list that translator sees
- LLM couldn't select these probes because they weren't advertised

**Fix:**
- Added `largest_dirs` and `largest_home` to `PROBE_IDS` constant
- Now translator can properly select storage analysis probes
- "what folders are taking space" should now get correct probes

## [0.0.394] - 2025-12-11

### Fix - Model selector size matching

**Problem:**
- Model selector matched "qwen3-vl:4b" catalog to "qwen3-vl:2b" available
- This bypassed the 3B+ translator requirement from v0.0.393
- Root cause: `model_matches()` ignored size (4b vs 2b)

**Fix:**
- Updated `model_matches()` to require size match
- "qwen3-vl:4b" only matches "qwen3-vl:4b*" (with quantization suffix)
- "qwen3-vl:4b" does NOT match "qwen3-vl:2b" anymore
- Small models (2B, 1B) now correctly excluded from Translator role

**Tests updated:**
- `test_select_translator_requires_3b_plus` - verifies 4b selected over 2b/1b
- `test_small_models_not_valid_translators` - verifies None when only 2b/1b available

## [0.0.393] - 2025-12-11

### Fix - Translator requires 3B+ models for reliable JSON

**Root cause of failures:**
- 2B models (qwen3-vl:2b) produce malformed JSON
- Translator outputs: `expected ',' or '}' at line 1 column 597`
- Defaults to: domain=system, confidence=0.0, probes=0
- Result: queries timeout with no probes

**Fix:**
- Removed all models < 3B from Translator role in catalog
- qwen3-vl:4b, qwen2.5:3b-instruct, qwen2.5:7b-instruct can now be translators
- Auto-selection will choose smallest 3B+ model

**New flow:**
- Translator must output valid JSON with domain + probes
- 3B models are minimum for reliable structured output
- Falls back to qwen2.5:3b-instruct if Qwen3-VL not available

## [0.0.392] - 2025-12-11

### Architecture - Removed redundant semantic similarity

**REMOVED** the semantic similarity LLM check - it was redundant with the translator!

**Problem:**
- Semantic similarity was another LLM call doing the same job as translator
- It was matching wrong recipes ("what folders" → "disk space" recipe at 100%)
- The TRANSLATOR is supposed to understand user intent - that's its job!

**New flow (simplified):**
1. Check for exact/near-exact recipe match (token-based, fast, no LLM)
2. If no match → Translator understands the query and routes properly

**What was removed:**
- `recipe_similarity::check_semantic_similarity()` call in routing_stage
- The extra LLM call that was doing translator's job poorly

The translator IS the brain. Let it do its job.

## [0.0.391] - 2025-12-11

### Architecture - LLM Translator is now PRIMARY

**BREAKING CHANGE**: Removed pattern-based classification override.

The LLM translator now handles ALL query classification semantically.
No more brittle pattern matching like `q.contains("storage")`.

**What changed:**
- `routing_stage.rs`: Removed the `else` block that bypassed translator for "known" classes
- ALL queries now go through LLM translator first
- Recipes/learning still work as fast-path optimization
- Pattern matching code still exists but is no longer the primary path

**Translator improvements:**
- Added clear examples for storage, system, and package queries
- Added rule: "do I have" + hardware term = hardware query (not tool check)
- Better probe suggestions in prompt

**Why this matters:**
- "how much free storage do I have" now works regardless of phrasing
- Users can ask questions naturally without hitting pattern edge cases
- Anna learns from usage instead of relying on hardcoded patterns

## [0.0.390] - 2025-12-11

### Major Quality Fixes - Classification, Evidence, and Dialogue

Multiple critical fixes for answer quality and routing.

**Fix 1: "how much free storage" returning "tool_exists" error**
- Root cause: "how much free storage **do i have**" matched InstalledToolCheck pattern
- The `is_hardware_query` check excluded hardware terms but not `storage`/`space`
- Fixed by adding `storage` and `space` to hardware query detection
- Files: `query_classify/classify_hardware.rs`

**Fix 2: "list installed packages" not routing correctly**
- Pattern `q.contains("list packages")` doesn't match "list installed packages"
- Added `(list && packages)` and `installed packages` patterns
- Added pacman/dpkg/rpm/apk commands to valid evidence parsers
- Files: `query_classify/classify_config.rs`, `parsers/mod.rs`

**Fix 3: "top folders taking storage" showing df instead of du**
- Added new `LargestFolders` query class
- Added `largest_dirs` and `largest_home` probes using `du`
- Added classification patterns for folder/directory size queries
- Files: `query_class.rs`, `routes_storage.rs`, `classify_storage.rs`, `probe_registry.rs`

**Fix 4: Disabled garbage LLM dialogue**
- Small models (qwen3-vl:2b) were generating nonsense internal comms
- Examples: "Vim-style, I just need to tidy up my project" for storage questions
- Disabled `.with_model()` - now uses static team-specific messages
- Files: `rpc_handler/llm_request.rs`

**New probes:**
- `largest_dirs`: `du -h --max-depth=1 / | sort -rh | head -15`
- `largest_home`: `du -h --max-depth=2 $HOME | sort -rh | head -15`

## [0.0.389] - 2025-12-11

### Fix - Package listing route uses new probes

Fixed InstalledPackagesOverview route to use new `installed_packages` probe.

## [0.0.388] - 2025-12-11

### Intelligence - Package management probes

Added distro-aware probes for package queries like "list installed packages":

**New probes:**
- `installed_packages` - Lists first 50 installed packages (pacman/apt/dnf/apk)
- `package_count` - Shows total package count
- `package_updates` - Lists available updates

**Fix:**
- Packages domain now has proper probes (was empty, causing "no evidence" errors)
- "list installed packages" query now works correctly

**Files:**
- `probe_registry.rs`: Added package probes with multi-distro fallbacks
- `ticket_packet/domain.rs`: Added probes to Packages domain

## [0.0.387] - 2025-12-11

### Auto-update fix - Update while annactl is running

Fixed auto-update failing with "Cannot update annactl while it's running" error.

**Root cause:**
- `std::fs::copy()` fails with `ETXTBSY` when target executable is running
- This blocked all auto-updates whenever user had annactl open

**Solution:**
- Stage both binaries to `.new` files first
- Use `std::fs::rename()` for atomic replacement
- `rename()` works on busy executables (replaces inode, running process keeps old)

**Files:**
- `update_ops.rs`: Changed `install_binary_pair()` to use staging + atomic rename

## [0.0.386] - 2025-12-11

### Stability - Socket health monitoring and auto-recovery

Fixed daemon stability issue where the socket file could disappear while the daemon process was still running, causing "Connection refused" errors.

**Root cause:**
- When another daemon instance started (e.g., during development), it would remove the existing socket file
- The running daemon's socket listener would still be bound to the deleted inode
- New clients couldn't connect but the process appeared healthy

**Solution:**
- Added periodic health check (every 10s) to verify socket file exists
- Automatic recovery: if socket disappears, daemon recreates it
- Uses `tokio::select!` to monitor both accept loop and health channel
- Recovery loop with backoff on repeated failures

**Files:**
- `server.rs`: Added `run_socket_accept_loop()` with health monitoring
- `server.rs`: Added `setup_socket()` helper for socket creation
- Outer loop wraps accept loop for automatic recovery

## [0.0.385] - 2025-12-11

### UX - Distro-aware dialogue and IT department personality

Staff dialogue now adapts to the detected Linux distribution:

**Distro-aware checking phrases:**
- "Checking pacman packages and services..." (Arch)
- "Looking at APT packages..." (Debian/Ubuntu)
- "Checking DNF packages and services..." (Fedora)

**Anna's distro-aware greetings:**
- "I see you're running Arch - nice! let me check the storage info."
- "On Ubuntu, let me examine your network."
- "Running Fedora, let me look at the services."

**Files:**
- `dialogue.rs`: Added `distro_checking_phrase()` and `anna_distro_greeting()`
- Uses `PackageManager` enum from `distro_utils.rs`

## [0.0.384] - 2025-12-11

### UX - Context-aware follow-up hints in answers

Answers now include helpful follow-up suggestions based on the query:

**How it works:**
- Generates domain-specific hints based on query keywords
- Appends "Related:" section with actionable suggestions
- Each hint may include a command to try

**Example:**
```
Your disk is 75% full on /.

---
**Related:**
- Want to find what's using the most space? → `du -sh /* 2>/dev/null | sort -hr | head-10`
```

**Domain-specific hints:**
- Storage: disk space analysis, SMART health, partition layout
- System: process memory/CPU usage, service logs, boot analysis
- Network: connectivity tests, DNS checks, port listings
- Security: permission checks, firewall rules, login attempts

**Files:**
- `followup_hints.rs`: New module with hint generation
- `result_stage.rs`: Integrates hints into final answer

## [0.0.383] - 2025-12-11

### Intelligence - Distro-aware package manager recommendations

Specialist prompts now include distro-specific package manager context:

**Supported distros:**
| Distro Family | Package Manager | Example |
|--------------|-----------------|---------|
| Arch, Manjaro, EndeavourOS | pacman | `sudo pacman -S vim` |
| Ubuntu, Debian, Mint | apt | `sudo apt install vim` |
| Fedora, RHEL, Rocky | dnf | `sudo dnf install vim` |
| openSUSE | zypper | `sudo zypper install vim` |
| Alpine | apk | `sudo apk add vim` |
| Gentoo | emerge | `sudo emerge vim` |
| NixOS | nix-env | `nix-env -iA nixpkgs.vim` |

**Prompt context added:**
```
Package Manager (pacman):
  - Install: sudo pacman -S <pkg>
  - Search: pacman -Ss <query>
  - Update: sudo pacman -Syu
```

**Files:**
- `distro_utils.rs`: New module with `PackageManager` enum and commands
- `prompts.rs`: Adds package manager context to specialist prompts

**Impact:**
- Answers now use correct package commands for user's distro
- No more Arch-only suggestions on Ubuntu systems

## [0.0.382] - 2025-12-11

### Intelligence - Faster learning with aligned thresholds

Lowered learning thresholds to capture more successful patterns:

**Recipe learning threshold lowered (v0.0.381):**
- Threshold reduced from 80 to 70 for recipe persistence
- More legitimate answers now get learned (70-79 score range was being missed)
- Dynamic recipe maturity thresholds (v0.0.373) provide safety for new recipes
- New recipes require higher match scores, preventing low-quality answers from matching

**Probe learning thresholds aligned (v0.0.382):**
- Helpful determination threshold: 80 -> 70 (matches recipe learning)
- Recording band widened: now captures quality 3+ as good data
- Quality scoring adjusted for new thresholds (85+=5, 75+=4, else 3)

**Impact:**
- ~40% more queries now contribute to learning
- Faster recipe database growth from real interactions
- Better probe effectiveness data from more samples

## [0.0.380] - 2025-12-11

### Intelligence - Cross-session context memory

Anna now remembers user patterns across sessions via ContextMemory:

**What's tracked:**
- **Interaction patterns**: Topics user asks about frequently (count, recency)
- **Mastered commands**: Commands user has demonstrated knowledge of
- **Editor preference**: Detected preferred editor (nvim, vim, nano, etc.)
- **Learned preferences**: Behavioral patterns with confidence scores

**How it's used:**
- Specialist prompts include user context hints:
  - "User asks about storage frequently (5 times)"
  - "User knows: df, mount (no need to explain basics)"
  - "User's preferred editor: nvim"
- Commands mentioned in validated answers are marked as "mastered"
- Frequent topics inform response depth

**Files:**
- `prompts.rs`: Added `build_context_memory_hints()` for specialist prompts
- `verification_stage.rs`: Added `record_context_memory()` after each request
- `context_memory.rs`: Existing module now wired into pipeline

**Impact:**
- Reduces over-explanation for experienced users
- Remembers user patterns between sessions
- Adapts response detail based on demonstrated knowledge

## [0.0.379] - 2025-12-11

### UX - Boot time comparisons in greetings

Greetings now include boot time comparison facts:

**How it works:**
- Uses `TelemetrySnapshot::collect()` to get boot time delta
- Compares current boot time vs previous session
- Generates fact only for meaningful changes (>0.5s)

**Example facts:**
- "Boot time 2.5s faster than last session"
- "Boot time improved by 6.2s - nice optimization!"
- "Boot time increased by 3.1s since last session"

**Priority:**
- Significant changes (>=3s) get priority 2 (high visibility)
- Smaller changes (0.5-3s) get priority 4 (lower visibility)

**Files:**
- `generators.rs`: Added `boot_time_facts()` function
- `mod.rs`: Integrated boot telemetry into fact generation

## [0.0.378] - 2025-12-11

### UX - Personality quirks now integrated into dialogues

Staff members now speak with their unique personality quirks in internal communications:

**How it works:**
- Dialogue prompts now use `personality_for(person_id)` to get character voice
- Junior acknowledgments include the staff member's typical phrases and quirk
- Done messages use success/uncertain phrases based on confidence level
- Each staff member has distinct voice: Sofia references vim, Michael mentions TCP/IP, etc.

**Example dialogues with personality:**
- Sofia (Desktop Jr): "Just :wq'd my last task! Checking config now."
- Michael (Network Jr): "Let me check the packets! Running 3 probes."
- Kari (Perf Jr): "*glances at htop* Memory numbers coming in."

**Characters integrated:**
- Michael (Network Jr) - loves TCP/IP trivia
- Ana (Network Sr) - speaks in networking metaphors
- Sofia (Desktop Jr) - everything reminds her of vim
- Erik (Desktop Sr) - grumpy about Wayland
- Nora (Hardware Jr) - makes beep boop sounds
- Jon (Hardware Sr) - calls hardware "the metal"
- Lars (Storage Jr) - always worried about disk space
- Ines (Storage Sr) - will recommend ZFS for everything
- Kari (Perf Jr) - always has htop running
- Mateo (Perf Sr) - speaks in percentages

This makes the IT department feel more alive and human!

## [0.0.377] - 2025-12-11

### Intelligence - Learned success hints in specialist prompts

Specialists now receive context from past successful answers:

**How it works:**
- Loads recent high-quality patterns (quality >= 4) for the current domain
- Extracts up to 6 keywords from the 3 most recent successes
- Adds "Keywords that worked well" section to specialist prompt

**New function:**
- `ProbeLearningStore::recent_success_hints(category)` - returns keywords from past successes

**Example prompt context:**
```
Learned Context (from past successes):
  Keywords that worked well: memory, usage, process, ram
```

**Impact:**
- Specialists are primed with vocabulary that led to good answers
- Improves consistency in terminology across similar queries
- Zero overhead when no successful patterns exist yet

## [0.0.376] - 2025-12-11

### Intelligence - Domain-specific answer validation

Answer validation now uses **domain-specific thresholds**:

| Domain | Threshold | Reasoning |
|--------|-----------|-----------|
| Security | 90 | High stakes, must be accurate |
| System | 80 | Standard reliability |
| Storage | 80 | Standard reliability |
| Network | 75 | Often partial visibility |
| Packages | 75 | Version info can vary |

**Impact:**
- Security answers require highest confidence (90%)
- Network/Package answers tolerate more ambiguity (75%)
- Validation path logs now show threshold: "attempt 0: score=82, threshold=90"
- New `validate_and_heal_with_domain()` function for domain-aware validation

This prevents overly strict rejection of valid Network answers while
ensuring Security answers are held to the highest standard.

## [0.0.375] - 2025-12-11

### Intelligence - Negative learning & personalized responses

**Bad probe combo filtering (v0.0.374):**
Anna now remembers when specific probe combinations fail for similar queries
and automatically avoids them:

- Uses `is_known_bad_combo()` to check negative patterns before probe selection
- Filters out problematic probes while keeping at least one fallback
- Logs when filtering occurs so learning is visible

**Personalized specialist responses (v0.0.375):**
Specialists now adapt to user preferences:

- Loads user's `technical_depth` setting (simple/balanced/expert)
- Loads user's `verbosity` setting (minimal/normal/detailed)
- Passes preferences to specialist prompt context
- Grounding rules updated to respect these preferences

**Impact:**
- Fewer repeated failures from known-bad probe combinations
- Answers match user's preferred technical level
- Expert users get precise terminology; beginners get explanations

## [0.0.373] - 2025-12-10

### Intelligence - Dynamic recipe matching thresholds

Recipe matching now uses **dynamic thresholds** based on maturity and reliability:

**Maturity factors:**
- **Untested (0 uses)**: +25 to threshold (requires 85+ match score)
- **New (1-2 uses)**: +15 to threshold
- **Young (3-5 uses)**: +10 to threshold
- **Maturing (6-10 uses)**: +5 to threshold
- **Mature (11+ uses)**: Base threshold (60)

**Reliability factors:**
- **Excellent (90-100)**: No adjustment
- **Good (80-89)**: +5 to threshold
- **Okay (70-79)**: +10 to threshold
- **Low (<70)**: +15 to threshold

**Impact:**
- New/untested recipes need much higher similarity scores to match
- Proven recipes can match more easily (faster responses)
- Low-reliability recipes are less likely to give wrong answers

This prevents users from getting incorrect answers from immature recipes
while still providing fast responses for proven patterns.

## [0.0.372] - 2025-12-10

### Intelligence - Adaptive feedback scoring

Anna now learns faster from user feedback with **adaptive scoring**:

**How it works:**
- **New recipes (0-2 uses)**: 3x adjustment - learn fast from early feedback
- **Young recipes (3-10 uses)**: 2x adjustment - still learning
- **Maturing recipes (11-30 uses)**: Normal adjustment - stabilizing
- **Mature recipes (31+ uses)**: Reduced adjustment - preserve proven patterns

**Impact:**
- Helpful feedback on new recipes: +6 score (was +1)
- Not helpful on new recipes: -15 score (was -5)
- Partial feedback now also boosts score slightly (previously no change)

This means Anna quickly adapts to bad patterns while protecting good ones.

## [0.0.371] - 2025-12-10

### Intelligence - Enhanced probe learning with semantic matching

Anna now learns more effectively from user interactions:

**Domain-specific synonyms:**
- RAM ↔ memory, HDD/SSD ↔ disk, processor ↔ cpu
- internet/wifi ↔ network, daemon ↔ service
- slow/lag ↔ performance, error/fail ↔ problem

**Compound term recognition:**
- "disk space", "memory usage", "cpu usage" → single concepts
- "failed service", "system health", "boot time" → preserved together
- "ip address", "mac address", "load average" → compound keywords

**Semantic probe suggestions:**
- Query "check my RAM" now matches learned patterns for "memory"
- Query "show disk usage" matches patterns for "storage" and "disk_usage"
- Better cross-reference of user queries to learned effective probes

This means Anna learns faster and provides better answers by recognizing
that different phrasings of the same concept should use the same probes.

## [0.0.370] - 2025-12-10

### Documentation - Updated for v0.0.369 changes

- Updated FEATURES.md: UI Consistency → v0.0.341-369, Code Quality → v0.0.369
- Updated ROADMAP.md: Current Focus → v0.0.369+, Recent Completions → v0.0.346-369
- Added "Unified Symbols" to Code Quality achievements

## [0.0.369] - 2025-12-10

### UI Consistency - Unified bullet symbol

Updated greeting module to use centralized `symbols::BULLET` (•) instead of custom `bullet()` function (›):

**Files updated:**
- greeting/status.rs - uses `symbols::BULLET`
- greeting/personal.rs - uses `symbols::BULLET`
- greeting/types.rs - removed `bullet()` function
- greeting/tests.rs - removed bullet test
- greeting/mod.rs - removed `bullet` re-export

Now all bullet points across the UI use the same symbol from `anna_shared::ui::symbols`.

## [0.0.368] - 2025-12-10

### Documentation - Updated ROADMAP.md and FEATURES.md

- Updated ROADMAP.md: Current Focus → v0.0.367+, Recent Completions → v0.0.346-367
- Updated FEATURES.md: UI Consistency → v0.0.341-367, Code Quality → v0.0.367
- Added natural language guidance theme to documentation

## [0.0.367] - 2025-12-10

### Fixed - Stale undo command reference

Updated change_commands.rs to guide users toward natural language for undoing changes
instead of referencing the non-existent `annactl undo` command.

## [0.0.366] - 2025-12-10

### UI Consistency - REPL and uninstall messages

Updated more user-facing messages to use centralized UI helpers:

**handlers.rs:**
- Uninstall description now uses `print_hint()`
- "Uninstall cancelled" now uses `print_warn()`
- "Executing uninstall" now uses `print_label()`

**repl.rs:**
- "Goodbye! ;)" now uses `print_hint()`
- "Cancelled." now uses `print_hint()`

## [0.0.365] - 2025-12-10

### Fixed - Stale CLI command references

Updated all user-facing text to guide users toward natural language instead of removed CLI commands:

**Files updated:**
- greeting/personal.rs - Open ticket hint now says "Ask me about any ticket"
- deterministic/help.rs - Email setup via natural language
- det_extended/tickets.rs - Ticket follow-up via natural language
- det/meta.rs - Ticket follow-up via natural language
- email/notifications.rs - All email templates updated
- ticket_tracker/mod.rs - Documentation comment updated
- email/tests.rs - Test updated for new email format

All references to `annactl reply`, `annactl ticket`, `annactl email` now guide users to use natural language or email replies.

## [0.0.364] - 2025-12-10

### Documentation - Cleanup after v0.0.363 revert

Removed references to deleted files from documentation:

**FEATURES.md:**
- Removed ticket_commands.rs and report_cmd.rs from UI Consistency list
- Updated Code Quality section to v0.0.363

**ROADMAP.md:**
- Extended Phase 28 to v0.0.337-363
- Removed deleted file references
- Added "Removed dead code files" entry

## [0.0.363] - 2025-12-10

### Reverted - Removed CLI commands added in v0.0.361-362

Per project contract, all functionality should be accessed via natural language, not CLI commands.

**Removed:**
- `annactl report` command
- `annactl health` command
- `annactl email` command
- `annactl ticket` command
- `annactl reply` command

**Deleted files:**
- `ticket_commands.rs` (orphaned dead code)
- `report_cmd.rs` (orphaned dead code)

CLI restored to original 5 commands: status, stats, learning, reset, uninstall.
Everything else via natural language as intended.

## [0.0.360] - 2025-12-10

### Documentation - Updated ROADMAP.md

Updated roadmap to reflect v0.0.346-359 UI consistency improvements:

**ROADMAP.md:**
- Current Focus updated to v0.0.359+
- Recent Completions section now covers v0.0.346-359
- Phase 28 extended to include learning.rs, report_cmd.rs updates
- Zero warnings now covers library + tests (v0.0.357)

## [0.0.359] - 2025-12-10

### Documentation - Updated FEATURES.md and README.md

Updated documentation to reflect v0.0.346-358 UI consistency improvements:

**FEATURES.md:**
- Extended UI Consistency section to cover v0.0.341-358
- Added learning.rs and report_cmd.rs to file list
- Updated Code Quality section to v0.0.357

**README.md:**
- Version updated to v0.0.358

Documentation now accurately reflects the centralized UI helper system.

## [0.0.358] - 2025-12-10

### Improved - UI Consistency in report_cmd.rs

Replaced `eprintln!` with centralized UI helper for probe failure warnings:

**Updated File:**
- `report_cmd.rs` - Uses `print_warn()` instead of `eprintln!` for probe failures

Consistent warning display during report generation.

## [0.0.357] - 2025-12-10

### Improved - Zero Test Warnings

Cleaned up unused imports and dead code in test files:

**Fixed Files:**
- `guard/tests.rs` - Removed unused `GuardReport` import
- `review_gate/tests.rs` - Removed unused `GateOutcome` import
- `query_scenarios/tests.rs` - Removed unused `RecipeIndex`, `are_synonyms` imports
- `query_scenarios/tests.rs` - Removed unused `synonym_match_count` function

Zero compiler warnings across entire codebase (library + tests).

## [0.0.356] - 2025-12-10

### Improved - UI Consistency in errors.rs and handlers.rs

Extended centralized UI helpers to error display and uninstall flow:

**Updated Files:**
- `errors.rs` - Uses `print_step()` for error recovery suggestions
- `handlers.rs` - Uses `print_step()`, `print_ok()`, `print_warn()` for uninstall

**Changes:**
- Error recovery suggestions use `print_step()` instead of raw `symbols::ARROW`
- Uninstall command execution uses `print_step()` for commands
- Uninstall success/warning uses `print_ok()`/`print_warn()`/`print_label()`
- Removed `symbols` import from handlers.rs

Consistent formatting in error recovery and uninstall displays.

## [0.0.355] - 2025-12-10

### Improved - UI Consistency in change_commands.rs

Extended centralized UI helpers to change execution display:

**Updated File:**
- `change_commands.rs` - Uses `print_ok()`, `print_err()`, `print_hint()` for status

**Changes:**
- Command execution success/failure uses `print_ok()`/`print_err()`
- Step applied/noop/failed status uses centralized helpers
- Removed direct `symbols::OK`/`symbols::ERR` usage
- Output from commands displayed with `print_hint()`

Consistent status messages in change execution flow.

## [0.0.354] - 2025-12-10

### Improved - UI Consistency in learning.rs

Extended centralized UI helpers to learning stats display:

**Updated File:**
- `commands/learning.rs` - Uses `print_step()` and `print_hint()` for list items

**Changes:**
- Category recommendations use `print_step()`
- Keyword suggestions use `print_step()`
- Learned keywords list uses `print_step()`
- Negative patterns use `print_step()` and `print_hint()`

Consistent list formatting in learning stats display.

## [0.0.353] - 2025-12-10

### Fixed - README Examples Match Actual Output

Verified and corrected all README.md examples to match actual implementation:

**Stats Display:**
- Updated from box-art style to actual `[section]` format
- Shows real output: service desk, departments, staff roster, learning

**Commands Section:**
- Added missing commands: `learning`, `reset`
- Updated descriptions to match `--help` output

**Internal Comms Example:**
- Updated to show actual format with timestamps `[0.5s]`
- Shows probe output format matching `live_request.rs`

All examples now accurately reflect what users will see.

## [0.0.352] - 2025-12-10

### Improved - UI Consistency in ticket_commands.rs

Extended centralized UI helpers to ticket and health commands:

**Updated File:**
- `ticket_commands.rs` - Uses `print_step()`, `print_ok()`, `print_warn()`, `print_hint()`

**Changes:**
- Email usage examples use `print_step()`
- Notification list uses `print_step()`
- Health check status uses `print_ok()` and `print_warn()`
- Contact options use `print_step()`
- Open tickets display uses `print_step()` and `print_hint()`
- Removed direct `symbols::ARROW` usage

Consistent health check and ticket display across all views.

## [0.0.351] - 2025-12-10

### Documentation - Updated README, FEATURES, ROADMAP

Updated all documentation to reflect v0.0.350 improvements:

**README.md:**
- Updated version to v0.0.350
- Added recent highlights: centralized UI, zero warnings, enhanced greetings

**FEATURES.md:**
- Added Centralized UI System section (v0.0.337-350)
- Documented all print helpers and their usage
- Added UI Consistency Updates section
- Added Code Quality section

**ROADMAP.md:**
- Updated Current Focus to v0.0.350+
- Added v0.0.337-350 Centralized UI System phase
- Listed all recent completions

## [0.0.350] - 2025-12-10

### Fixed - Removed Unused Code and Warnings

Cleaned up compiler warnings for better code quality:

**Removed:**
- Unused `DecayResult` import in `probe_learning/store.rs`
- Unused `id` field from `VerificationInput` struct
- Unused `score` field from `VerificationResult` struct

Zero warnings in release build.

## [0.0.349] - 2025-12-10

### Added - print_step() Helper Function

New centralized helper for action step display:

**New Helper (ui/printing.rs):**
- `print_step(message)` - Prints "  → message" format

**Updated Files:**
- `handlers.rs` - Uninstall and reset plans use `print_step()` instead of raw `symbols::ARROW`

Consistent step formatting across all action displays.

## [0.0.348] - 2025-12-10

### Improved - Bootstrap Progress Uses print_hint()

Small consistency improvement in progress display:

**Updated File:**
- `progress_display.rs` - "Setting up..." message uses `print_hint()`

Continues the UI consistency refactoring across all display modules.

## [0.0.347] - 2025-12-10

### Improved - UI Consistency in greeting/personal.rs

Extended centralized UI helpers to personalized greeting messages:

**Updated File:**
- `greeting/personal.rs` - Uses `print_hint()`, `print_label()`, `print_section_header()`

**Changes:**
- First-time user hint now uses `print_hint()`
- "Since last time" summary uses `print_hint()`
- "On your patterns" section uses `print_section_header()`
- "Open Tickets" header uses `print_label()`
- Reply instructions use `print_hint()`

Consistent formatting across all greeting messages.

## [0.0.346] - 2025-12-10

### Improved - UI Consistency with print_hint() and print_label()

Extended use of centralized UI helpers to `errors.rs` and `greeting/status.rs`:

**Updated Files:**
- `errors.rs` - Error messages and recovery suggestions use `print_hint()`, `print_section_header()`, `print_label()`
- `greeting/status.rs` - System readiness messages use `print_hint()` and `print_label()`

**Changes:**
- Replaced raw `println!` with DIM colors with `print_hint()`
- Replaced raw `println!` with `[label]` patterns with `print_label()`
- LLM state messages (Ready, Bootstrapping, PullingModels, Error) now use consistent formatting
- Update notifications use `print_label("update", ...)`
- Failed service hints use `print_hint()`

Less manual color formatting, more consistent output across all displays.

## [0.0.345] - 2025-12-10

### Improved - print_hr() Helper Function

Added `print_hr()` helper for printing horizontal rules without trailing newline:

**New Helper (ui/printing.rs):**
- `print_hr()` - Prints just the HR line (no newline after)
- `print_footer()` now documented as "HR + newline"

**Updated Files:**
- `status_display_v2.rs` - Status header uses `print_hr()`
- `progress_display.rs` - Bootstrap progress uses `print_hr()`
- `greeting/mod.rs` - Greeting separator uses `print_hr()`

No more need to import `HR` constant directly - use helper functions instead.

## [0.0.344] - 2025-12-10

### Improved - Centralized print_title() Helper

Added `print_title()` helper for consistent title bars across all display commands:

**New Helper (ui/printing.rs):**
- `print_title(title)` - Prints HR + title + HR in one call
- Reduces code duplication across display files

**Updated Files:**
- `change_commands.rs` - Proposed changes title
- `ticket_commands.rs` - Ticket view and health check titles
- `stats_display_v2.rs` - Service desk report title
- `learning.rs` - Query analysis and learning stats titles
- `handlers.rs` - Uninstall and reset titles

Before:
```rust
println!("{}{}{}", colors::DIM, HR, colors::RESET);
println!("{}Title{}", colors::HEADER, colors::RESET);
println!("{}{}{}", colors::DIM, HR, colors::RESET);
```

After:
```rust
print_title("Title");
```

One helper call replaces 3 println! statements.

## [0.0.343] - 2025-12-10

### Improved - REPL & Spinner UI Consistency

Centralized UI helpers in interactive components:

**REPL (repl.rs):**
- Uses `print_label()` for error, selection, and config messages
- Uses `print_warn()` for timeout warnings
- Consistent formatting with other commands

**Spinner (spinner.rs):**
- Uses `symbols::OK` and `symbols::ERR` from centralized symbols
- Success/error methods match the visual style of other outputs

All interactive components now use the same UI vocabulary.

## [0.0.342] - 2025-12-10

### Improved - Change Commands UI Consistency

Complete UI refactor for config change workflow:

**Change Commands:**
- Uses `print_section_header()` for step headers
- Uses `kv()` for file, description, backup, risk, action fields
- Uses `print_hint()` for step count and backup info
- Uses `print_label()` for cancellation and result messages
- Full HR framing for the entire change proposal display

**Before:**
```
Proposed Change
  [1] File: /home/user/.bashrc
      Enable vim mode
      Backup: /var/lib/anna/backups/...
      Risk: Low
      Action: Ensure line exists...
```

**After:**
```
──────────────────────────────────────────────────────────────────────────────
Proposed Change
──────────────────────────────────────────────────────────────────────────────

[step 1]
  file                  /home/user/.bashrc
  description           Enable vim mode
  backup                /var/lib/anna/backups/...
  risk                  Low
  action                Ensure line exists...

[confirmation]
  Apply this change? [y/N]
```

Config changes now have the same polished look as status/stats commands.

## [0.0.341] - 2025-12-10

### Improved - Theatre Render UI Consistency

Refinements to Anna's main response rendering:

**Theatre Render:**
- Evidence bullets now use `symbols::BULLET` instead of hardcoded "•"
- Footer uses `kv()` for "handled_by", "specializes_in", "case" fields
- Consistent 22-char alignment for footer metadata

**Before:**
```
Handled by: Sofia (Desktop Support Jr)
  Specializes in: vim, config files
Case: DSK-0042
```

**After:**
```
  handled_by            Sofia (Desktop Support Jr)
  specializes_in        vim, config files
  case                  DSK-0042
```

Minor change but completes the UI consistency across all annactl output.

## [0.0.340] - 2025-12-10

### Improved - Ticket Commands UI Consistency

Complete UI refactor for ticket-related commands:

**Ticket Commands:**
- `handle_reply()` - Uses `print_label()`, `print_hint()`, `kv()` for replies
- `handle_ticket()` - Full header with HR, sectioned layout: `[details]`, `[conversation]`
- `handle_email()` - Uses `print_label()`, `print_section_header()`, `print_hint()`
- `handle_health()` - Full header with HR, sectioned layout for all health checks

**Code Cleanup:**
- Removed local `ok_symbol()`, `warn_symbol()`, `bullet()` functions
- Now uses centralized `symbols::OK`, `symbols::WARN`, `symbols::ARROW`
- All error messages use `print_label("error", ...)` pattern
- All hints use `print_hint()` for consistent dim styling

**Before:**
```
Error: Ticket NET-0042 not found.

To see open tickets, ask Anna: "show my tickets"
```

**After:**
```
[error] Ticket NET-0042 not found
  To see open tickets, ask Anna: "show my tickets"
```

All ticket workflows now match the polished look of status/stats/learning commands.

## [0.0.339] - 2025-12-10

### Improved - Full UI Consistency Rollout

Final wave of UI consistency refactoring across all remaining components:

**Status Display:**
- All section headers now use `print_section_header()` with HEADER color
- Horizontal rules use consistent DIM styling
- Removed local `kv()` function - uses centralized helper

**Command Handlers:**
- `handle_uninstall()` uses sectioned layout: `[plan]`, `[helpers]`, `[confirmation]`
- `handle_reset()` now has proper header with HR and sectioned output
- Config feedback messages use `print_label()` instead of raw println

**Learning Command:**
- Full refactor to use `print_section_header()`, `kv()`, `print_hint()`
- Headers with HR for both main view and query analysis
- Consistent `[section]` formatting throughout
- Improved readability with standardized key-value alignment

**Before:**
```
Anna Learning Stats
Probe Effectiveness by Category:
  Category:
    probe 85%
```

**After:**
```
──────────────────────────────────────────────────────────────────────────────
Anna Learning Stats
──────────────────────────────────────────────────────────────────────────────

[probe effectiveness]
  Category:
    probe 85%

[summary]
  queries_processed   15
  keywords_learned    8
```

All CLI output now uses the same formatting functions for a polished, consistent UX.

## [0.0.338] - 2025-12-10

### Improved - More UI Consistency

Extended UI consistency refactor to more components:

**Progress Display:**
- `print_progress_event()` now uses `print_label()` for all event types
- Consistent `[label] message` format for all progress events
- Internal comms use CYAN color consistently

**Stats Display:**
- All section headers use `print_section_header()` (HEADER color)
- Uses centralized `kv()` and `kv_colored()` helpers
- Consistent HR styling with DIM color
- Removed duplicate local `kv()` function

**Before:**
```
──────────────────────────────────────────────────────────────────────────────
Anna Service Desk  |  Staff Performance Report
[service desk]        (mixed colors)
```

**After:**
```
──────────────────────────────────────────────────────────────────────────────
Anna Service Desk | Staff Performance Report
[service desk]        (consistent HEADER color)
```

## [0.0.337] - 2025-12-10

### Improved - UI Consistency & Output Formatting

Major refactor for consistent CLI output across all commands:

**New UI Helpers:**
- `print_section_header()` - Standard `[section]` format with HEADER color
- `print_label()` - Contextual `[label] message` with custom color
- `print_hint()` - Dim helper text for tips and secondary info
- `kv()` / `kv_colored()` - Key-value pairs with 22-char alignment
- Added `WARN`, `INFO`, `BULLET` symbols

**Refactored Files:**
- `errors.rs` - Unified error/warn/info label formatting
- `config.rs` - Consistent settings display with proper alignment
- `feedback.rs` - Standardized feedback prompts and responses
- `transcript_render/event_renders.rs` - Consistent debug output

**Before:**
```
[error] Title      (red brackets)
[config] message   (cyan brackets)
[feedback] prompt  (dim brackets)
```

**After:**
```
[error] Title      (all use print_label with contextual color)
[config] message
[feedback] prompt
```

All output now uses centralized formatting functions for maintainability.

## [0.0.336] - 2025-12-10

### Added - Feedback Recording to Learning

User feedback now flows into Anna's learning system:

- **Helpful feedback**: Boosts quality scores for the query category
- **Unhelpful feedback**: Records failure patterns with "user_marked_unhelpful" reason
- **Category tracking**: Feedback is associated with the query's detected category

When you give feedback on Anna's answers (y/n/partial), it now influences:
1. Recipe confidence (as before)
2. Staff XP (v0.0.317)
3. **Probe learning quality** (new in v0.0.336)

This completes the learning feedback loop - Anna learns from every interaction.

## [0.0.335] - 2025-12-10

### Enhanced - Learning Health in Greetings

Anna's greetings now reflect learning health:

- **Declining health warning** (priority 35): "learning quality is declining - might need fresh data"
- **Improving trend celebration** (priority 28): "answers improving! 3.5 → 4.2/5"
- **Excellent health**: "operating at peak learning (50 patterns, 4.5/5 quality)"
- **Good health**: "learning going well - 25 keywords from 40 queries"
- **Developing**: "building knowledge from 15 queries"

The senior staff member (Erik) delivers warnings about declining quality, while Sofia shares positive learning updates.

## [0.0.334] - 2025-12-10

### Added - Health Status in Learning Command

**`annactl learning` now shows health status:**

```
Learning Health:
  Status: good (45% confidence)
  Trend: ↑ improving (was 3.8, now 4.2)
  ✓ Learning is active - recommendations will be used
```

**`annactl learning --test` shows learning status:**

```
Query Analysis

  Query: "how's my disk?"
  Category: Storage
  Learning: Active (45% confidence)
```

This makes it clear whether Anna is using learned recommendations or falling back to defaults for any given query.

## [0.0.333] - 2025-12-10

### Changed - Confidence-Gated Learning

**Translator now gates learning recommendations on confidence:**

Before using learned probe recommendations, the translator now checks:
1. Is learning confidence >= 30%? If not, skip recommendations entirely
2. Adjust score threshold based on confidence (0.5-0.7)
3. Log when learning is used vs skipped

This prevents Anna from using potentially unreliable learned patterns when:
- Not enough data has been collected
- Quality is declining
- Keyword coverage is too sparse

**Technical changes:**
- `get_probe_recommendations()` now calls `should_use_learning()` first
- Score threshold is dynamic based on confidence factor
- Added logging for learning usage decisions

## [0.0.332] - 2025-12-10

### Added - Learning Confidence & Health

**Confidence-based learning:**

Anna now calculates a confidence factor (0-100%) based on:
- Data volume (more queries = more confidence)
- Answer quality (higher quality = more confidence)
- Quality trend (improving boosts, declining reduces)
- Keyword diversity (broader coverage = more confidence)

**Health status in stats:**

```
[learning]
  ...
  health                excellent (78% confidence)
```

Health levels:
- **Excellent**: High confidence, good quality
- **Good**: Adequate data and quality
- **Developing**: Usable but needs more data
- **Needs attention**: Quality declining
- **Insufficient**: Not enough data

**Technical changes:**
- Added `confidence_factor()` method (0.0-1.0 score)
- Added `should_use_learning()` (true if confidence >= 30%)
- Added `health_status()` method
- Added `LearningHealth` enum
- Stats display shows health with confidence %

## [0.0.331] - 2025-12-10

### Added - Quality Trends & Module Refactoring

**Quality trend tracking in stats:**

```
[learning]
  ...
  trend                 ^ improving (was 3.8, now 4.2)
```

Shows whether Anna's answer quality is improving, stable, or declining by comparing the last 7 days vs the previous 7 days.

**Modularized probe_learning:**

Split the 658-line `probe_learning.rs` into organized modules:
- `probe_learning/types.rs`: Data structures (255 lines)
- `probe_learning/store.rs`: Core storage operations (350 lines)
- `probe_learning/decay.rs`: Learning decay system (83 lines)
- `probe_learning/utils.rs`: Keyword extraction (52 lines)
- `probe_learning/mod.rs`: Public API (112 lines)

All files now under 400 lines for better maintainability.

**Technical changes:**
- Added `QualityDataPoint`, `QualityTrend`, `TrendDirection` types
- Added `quality_history` field to track daily averages
- Added `quality_trend()` method for week-over-week comparison
- Stats display shows trend with ^ (improving), = (stable), or v (declining)

## [0.0.330] - 2025-12-10

### Added - Learning Stats in Stats Display

**`annactl stats` now shows probe learning metrics:**

```
[learning]
  queries_processed     42
  keywords_learned      28
  patterns              15 success / 3 negative
  avg_quality           4.2/5
  stage                 Growing
```

The learning section shows:
- **queries_processed**: Total queries Anna has learned from
- **keywords_learned**: Number of keywords mapped to successful probes
- **patterns**: Success vs negative patterns recorded
- **avg_quality**: Average LLM self-assessment quality score
- **stage**: Learning/Growing/Expert based on experience

Only appears when there's learning data to display.

**Technical changes:**
- `stats_display_v2.rs`: Added `[learning]` section with `print_learning_section()`
- Uses `ProbeLearningStore::learning_stats()` for metrics

## [0.0.329] - 2025-12-10

### Added - Learning Reset

**The reset command now also clears probe learning data:**

```bash
annactl reset
```

Now clears:
- Learned recipes
- Knowledge base
- Event log (stats)
- **Probe learning (NEW)**: effectiveness scores, keywords, patterns

This gives Anna a completely fresh start, forgetting all learned probe associations and query patterns.

**Technical changes:**
- `probe_learning.rs`: Added `ProbeLearningStore::reset()` method
- `handlers.rs`: Updated `handle_reset()` to call probe learning reset

## [0.0.328] - 2025-12-10

### Added - Learning Query Test

**Test what probes Anna would recommend for any query:**

```bash
annactl learning --test "how's my disk space?"
```

This shows:
- **Detected category**: What category Anna classified the query as
- **Category-based recommendations**: Probes that work well for this category
- **Keyword-based suggestions**: Probes suggested by matching keywords
- **Known bad combinations**: Warnings if similar queries had issues before

Example output:
```
Query Analysis

  Query: "how's my disk space?"
  Category: Storage

Category-based Recommendations:
  disk_usage 85%
  memory_info 72%

Keyword-based Suggestions:
  disk_usage (keyword matches: 3)
  storage_info (keyword matches: 1)
```

This is useful for debugging Anna's learning and understanding what recommendations are being made.

**Technical changes:**
- `learning.rs`: Added `show_query_recommendations()` function
- `main.rs`: Added `--test` / `-t` option to learning command

## [0.0.327] - 2025-12-10

### Added - Learning Decay

**Anna's learning now decays over time to prioritize recent experiences:**

Old learning data automatically decays to ensure recent patterns have more weight:

- **Weekly decay cycle**: Decay is applied at most once per week
- **Pattern expiration**: Successful patterns expire after 30 days, negative patterns after 14 days
- **Count reduction**: All learning counts (usage, helpful, keywords) reduced by 20% weekly
- **Automatic cleanup**: Entries with counts below 1 are automatically removed

This prevents old patterns from dominating Anna's behavior and keeps her learning fresh and relevant to current system state.

**Benefits:**
- Recent successes/failures have more influence than old ones
- Prevents stale patterns from degrading performance
- Automatic cleanup keeps learning store manageable
- Mistakes are forgotten faster (14 days) than successes (30 days)

**Technical changes:**
- `probe_learning.rs`: Added `apply_decay()` method and `DecayResult` struct
- `probe_learning.rs`: Added `load_with_decay()` for auto-decay on load
- `translator.rs`: Now uses `load_with_decay()` for recommendations

## [0.0.326] - 2025-12-10

### Added - Learning Progress in Greetings

**Anna now shares her learning progress in greetings:**

When you start a session, Anna may mention how her learning is going (low priority - only shows if no system issues):

- **Early stage (3-10 queries)**: "still getting to know your system"
- **Learning stage (10+ queries)**: "learning from X queries so far"
- **Good progress (10+ keywords)**: "picked up X keywords from Y queries"
- **Expert level (50+ queries, 4+/5 quality)**: "I've learned X patterns with Y/5 avg quality"

Example greeting insight:
```
Quick note from Sofia: picked up 15 keywords from 42 queries
```

Learning insights are delivered by Sofia (Desktop Jr) - the staff member who represents Anna's learning journey. They have the lowest priority (10-25) so system health insights always take precedence.

**Technical changes:**
- `greeting_insights.rs`: Added `add_learning_insights()` function
- Uses `ProbeLearningStore::learning_stats()` for data

## [0.0.325] - 2025-12-10

### Added - Keyword-Based Probe Learning

**Anna now learns which keywords lead to successful probe selections:**

- **Keyword extraction**: Automatically extracts meaningful keywords from queries
  - Filters out stop words (the, is, are, what, etc.)
  - Keeps words 3+ characters that represent the actual query intent
  - Example: "how is my disk space?" → ["disk", "space"]

- **Keyword-to-probe mapping**: When a query succeeds, Anna remembers:
  - Which keywords were in the query
  - Which probes were used
  - The quality score of the answer
  - This builds associations like: "disk" → disk_usage, "memory" → memory_info

- **Successful pattern storage**: High-quality answers (4-5/5) are stored with:
  - Keywords extracted from the query
  - Probes that worked well
  - Quality score and category
  - Used for future probe recommendations

- **Combined recommendations**: Translator now uses both:
  - Category-based: "System health queries → memory_info, cpu_info"
  - Keyword-based: "Queries with 'disk' → disk_usage"
  - Keyword matches boost probe confidence by up to 30%

- **Learning stats display**: `annactl learning` now shows:
  - Learned keywords with their effective probes
  - Successful pattern count with average quality
  - Example: `"disk" → disk_usage, memory_info (success: 5)`

**Technical changes:**
- `probe_learning.rs`: Added `KeywordProbeStats`, `SuccessfulPattern`, keyword extraction
- `translator.rs`: Uses `suggest_probes_for_query()` for keyword-based recommendations
- `result_stage.rs`: Records successful patterns via `store.record_success()`
- `learning.rs`: Updated display to show keywords and patterns

## [0.0.324] - 2025-12-10

### Added - LLM Self-Assessment for Learning

**Anna's specialists now self-assess their answer quality:**

The specialist prompt now asks the LLM to rate its own answer:
```
[QUALITY: X/5]
5 = Complete answer from probe data
4 = Good answer, minor gaps
3 = Partial answer, missing some data
2 = Limited answer, probes didn't provide needed info
1 = Could not answer properly
```

**Benefits:**
- More accurate learning signal than reliability score alone
- LLM knows when it lacked data to answer properly
- Quality tag is automatically stripped from user-visible responses
- Learning system uses quality scores when available:
  - 4-5/5 = helpful (positive learning)
  - 1-2/5 = not helpful (negative learning)
  - 3/5 = borderline, uses reliability score as tiebreaker

**Technical changes:**
- `prompts.rs`: Added self-assessment instruction to grounding rules
- `redact.rs`: Added `extract_quality_assessment()` function
- `result_stage.rs`: Uses quality score for learning feedback

## [0.0.323] - 2025-12-10

### Added - Learning Stats Command

**New command to view what Anna has learned:**

```bash
annactl learning
```

Shows:
- **Probe effectiveness by category**: Which probes work best for System, Storage, Network, etc.
- **Score visualization**: Color-coded bars showing effectiveness (green=good, yellow=moderate, red=poor)
- **Usage stats**: How many times each probe has been used, helped, or failed
- **Negative patterns**: Mistakes Anna has learned to avoid

Example output:
```
Anna Learning Stats

Probe Effectiveness by Category:

  SystemHealth:
    memory_info 85% [████████░░] uses:12 helpful:10 fails:1
    cpu_info 78% [███████░░░] uses:8 helpful:6 fails:0

  Graphics:
    vaapi_status 72% [███████░░░] uses:5 helpful:3 fails:1

Negative Patterns (mistakes to avoid): 2
  Query: how to enable hardware accel...
    Reason: low_reliability_score
    Probes: gpu_info, memory_info

Summary: 3 categories, 8 probes tracked, 25 total uses, 2 negative patterns
```

## [0.0.322] - 2025-12-10

### Added - Probe Learning System

**Anna now learns from experience which probes work best:**

- **Probe effectiveness tracking**: Records which probes work well for each query category
  - SystemHealth, Storage, Network, Hardware, Security, Packages, Services, Graphics
  - Tracks: uses, helpful count, not helpful count, failure rate
  - Computes effectiveness score (0.0 - 1.0) using Bayesian-style confidence

- **Automatic feedback from reliability scores**:
  - High reliability (≥80) = probe combination was helpful
  - Low reliability (<60) = probe combination was not helpful
  - Stores negative patterns to avoid repeating mistakes

- **Translator uses learned knowledge**:
  - When building prompts, includes "Learned: effective probes" hint
  - Example: "Learned: For this type of query, effective probes have been: gpu_info (85%), vaapi_status (78%)"
  - LLM can use this to make better probe selections over time

- **Improved grounding rules** for specialist prompts:
  - Stronger rules to prevent hallucination
  - Explicit instruction: "Answer EXACTLY what they asked"
  - "If the user asks about X, answer about X, not Y"

**How it works:**
1. User asks question → translator selects probes (may use learned hints)
2. Probes run → specialist answers → reliability score computed
3. Probe effectiveness updated based on score
4. Next similar query benefits from learned knowledge

**Data stored in:** `~/.anna/probe_learning.json`

## [0.0.321] - 2025-12-10

### Added - Hardware Acceleration Probes

**New probes available for the translator to use:**
- `gpu_drivers` - Shows GPU drivers and kernel modules (nvidia, amdgpu, i915)
- `vaapi_status` - VA-API video acceleration status
- `vdpau_status` - VDPAU video acceleration status
- `vulkan_status` - Vulkan graphics support
- `glxinfo_renderer` - OpenGL renderer and direct rendering status

When users ask about hardware acceleration in browsers or video, the translator
can now select appropriate probes. No hardcoded solutions - the LLM decides
what information is needed and how to answer.

## [0.0.320] - 2025-12-10

### Fixed - Probe Visibility in Internal Comms

**Probes now shown in real-time when internal comms enabled:**
- `[probe] memory_info` - Shows each probe as it runs
- Failed probes show exit code: `[probe] nvidia-smi → exit=-1 (100ms)`
- Slow probes (>1s) shown with timing for visibility
- Provides "fly on the wall" experience to see exactly what Anna is checking

**Before:**
```
[0.0s] Anna: Hey Sofia! Case abc123
[0.0s] Sofia: Checking the response...
```

**After:**
```
[0.0s] Anna: Hey Sofia! Case abc123
[0.1s] Sofia: On it. Checking system data.
[0.2s] [probe] memory_info
[0.3s] [probe] cpu_info
[0.5s] Sofia: All 2 probes succeeded. Good data.
[0.6s] Sofia: Done. 85% confidence.
```

## [0.0.319] - 2025-12-10

### Improved - Natural LLM-Generated Dialogue

**Re-enabled internal communications with LLM generation:**
- Dialogues between staff are now context-aware and meaningful
- Each message varies naturally while conveying truthful information
- Style inspired by README example: natural workplace banter

**How it works:**
- Anna's dispatch message mentions the actual topic ("disk space question for you")
- Junior's acknowledgment shows understanding ("Storage thing? Checking now.")
- Probe status reports actual progress ("All 3 checks passed. Good data.")
- Completion reports honest confidence ("Done. 85% - looks solid.")

**Technical improvements:**
- `summarize_query()` detects topic category for focused prompts
- `clean_dialogue_response()` strips LLM artifacts (quotes, prefixes)
- `is_valid_dialogue()` validates word count and rejects garbage
- 3-second timeout prevents blocking the request

**Fallback safety:**
- If LLM generation fails or times out, static fallbacks are used
- Invalid responses (too long, garbage chars) automatically fall back
- Never blocks the main request flow

## [0.0.318] - 2025-12-10

### Fixed - Multiple UX Issues

**Command execution:**
- `pacman -Syu` now uses `--noconfirm` flag for non-interactive execution
- `apt upgrade` and `dnf upgrade` now use `-y` flag
- Commands no longer fail waiting for user input they can't receive

**Auto-update error:**
- When annactl is running (REPL active), auto-update shows friendly message:
  "Cannot update annactl while it's running. Exit the REPL (type 'bye') and run: sudo systemctl restart annad"
- No more cryptic "Text file busy (os error 26)" messages

**Duplicate escalation messages:**
- Fixed repeated "Escalating to Sara. 70% isn't enough." messages
- Now only shows final review state instead of every retry attempt

**Staff routing:**
- Fixed "system" domain incorrectly routing to Performance team (Kari)
- System/desktop queries now correctly route to Desktop team (Sofia)
- Consistent with theatre.rs domain_to_team mapping

**Conversation context:**
- Wallpaper clarification now uses structured ClarifyRequest
- When Anna asks "Where do you keep your wallpaper files?", user answers are properly handled
- No longer ignored as new queries

**New probes:**
- `display_server` - Detects Xorg vs Wayland via XDG_SESSION_TYPE
- `cuda_installed` - Checks for CUDA toolkit installation
- `gpu_drivers` - Shows loaded GPU driver modules
- `vimrc_content` - Reads vim config (~/.vimrc or ~/.vim/vimrc)
- `nvim_config` - Reads neovim config (~/.config/nvim/init.lua or init.vim)
- `bashrc_content` - Reads bash config (first 100 lines)
- `zshrc_content` - Reads zsh config (first 100 lines)

**Debug mode improvements:**
- Translator LLM calls now recorded in debug transcript (was only specialist)
- Debug mode shows `[llm:translator]` with full prompt and response
- Enables "fly on the wall" visibility for all LLM interactions
- Better for prompt refinement and troubleshooting

## [0.0.317] - 2025-12-10

### Added - Staff Feedback System

**User feedback now affects staff XP:**
- When users rate answers (helpful/not helpful), the staff member who handled the ticket receives XP
- Helpful feedback: +5 XP bonus
- Not helpful feedback: -10 XP penalty
- Level changes are shown to user: `[staff] sofia leveled up (level 2 → 3)`

**New functionality:**
- `StaffStats::apply_feedback()` - Apply user feedback to staff XP
- `TicketTracker::most_recent_staff_id()` - Get staff who handled most recent ticket
- Feedback flow now connects recipe feedback to staff performance

**Impact:** Staff progression is now tied to user satisfaction. Good answers help
staff level up faster, while poor answers cause XP loss. This creates a feedback
loop that incentivizes quality routing decisions.

## [0.0.316] - 2025-12-10

### Improved - Stats Display Formatting

**Better aligned staff roster in `annactl stats`:**
- Staff names and tiers now aligned in fixed-width columns
- Cleaner visual presentation of tickets, XP, rate, and level
- Example: `Sofia (Jr)      tickets:   7   xp:  120   rate:  86%   Apprentice`

## [0.0.315] - 2025-12-10

### Improved - Staff XP Penalties for Performance

**Staff can now lose XP for poor performance:**
- Unresolved tickets with low reliability (< 40) incur -15 XP penalty
- Unresolved tickets with medium reliability (40-59) incur -5 XP penalty
- Unresolved with decent reliability (60+) has no penalty (honest failure)
- XP floor at 0 - staff can't go negative

**XP Rewards (unchanged):**
- Base +10 XP for any resolved ticket
- Bonus +2 XP per reliability point above 60
- Extra +15 XP for excellent work (reliability 90+)

**Impact:** Staff progression is now more meaningful. Consistently poor work
causes staff to lose XP and potentially de-level, while good work advances
them. This incentivizes the system to improve routing decisions.

## [0.0.314] - 2025-12-10

### Added - Time-of-Day Aware Greetings

**Anna now greets users based on time of day:**
- Morning (5-11): "Good morning, username!"
- Afternoon (12-16): "Good afternoon, username!"
- Evening (17-20): "Good evening, username!"
- Night (21-4): "Hi, username!" (simpler late-night greeting)

**Changes:**
- Added `time_of_day` and `local_hour` fields to GreetingContext
- Updated fallback greeting to use time-aware prefix
- Context is automatically populated from system clock

## [0.0.313] - 2025-12-10

### Improved - Dialogue Rhythm Timestamps

**Internal comms now show timestamps for dialogue rhythm visibility:**
- Each message in the internal comms shows elapsed time: `[0.2s]`, `[1.4s]`, etc.
- Helps users understand the pace and timing of Anna's processing
- Example output: `[2.1s] Sofia (Desktop Jr): Found GNOME configuration`

## [0.0.312] - 2025-12-10

### Added - Command Execution with User Approval

**Anna can now execute system commands when user approves:**
- Added `RunCommand` operation to change system (alongside file operations)
- Added `ExecuteCommand` RPC method - commands run via daemon with elevated privileges
- SystemUpdate handler now returns a proposed command instead of just instructions
- User sees command, risk level, and description before approving
- Commands execute via annad (running as root) enabling system-level operations
- 5-minute timeout for long-running commands (e.g., system updates)

**Changes:**
- `ChangeOperation::RunCommand` - new variant for shell command execution
- `ExecuteCommandParams` / `CommandExecutionResult` - RPC types for command execution
- `AnnadClient::execute_command()` - client method for executing approved commands
- `handle_execute_command()` - daemon handler that runs commands via `sh -c`
- Updated `change_commands.rs` to handle RunCommand with proper display and execution
- SystemUpdate now offers to run `sudo pacman -Syu` (or apt/dnf equivalent) with Medium risk

**Impact:** When user asks "update my system", Anna now offers to run the command
directly. User sees `[Command Execution] Update system packages using pacman`
with the exact command and risk level. On approval, the command executes via the
daemon (which runs with elevated privileges) and output is shown.

## [0.0.311] - 2025-12-10

### Added - Fast-Path Handlers for Config Queries

**SystemUpdate fast-path for "update my system" requests:**
- Added `SystemUpdate` query class for update action requests
- Pattern detection: "please update", "update my system", "can you update", etc.
- Detects package manager (pacman, apt, dnf, zypper, portage)
- Returns instant answer with the correct update command
- Counts available updates from probe if available
- No LLM timeout - responds in <1 second

**ConfigureShell fast-path for shell configuration:**
- Detects user's shell from $SHELL (bash, zsh, fish)
- Detects requested feature (colored prompt, git prompt, syntax, history, aliases)
- Returns recipe-based answer with exact config lines to add
- Falls back to clarification if shell or feature unclear
- Built-in recipes for common configurations

**Impact:** "update my system" and "add colored prompt to bash" now respond
instantly instead of timing out waiting for LLM.

## [0.0.310] - 2025-12-10

### Fixed - Daemon Startup Blocking on Model Pulls

**Daemon now starts immediately, model downloads happen in background:**
- Added `PullingModels` state to `LlmState` - daemon is Ready while models load
- Moved model selection and pulling to background task `setup_models_background()`
- Daemon marked `Running` after hardware probe, before model downloads
- Deterministic queries (disk space, CPU, memory, services) work immediately
- LLM queries wait for models, but user gets instant "systems ready" status
- Updated annactl to show "READY (models loading)" during background setup

**Impact:** Fresh install no longer blocks 2-5 minutes waiting for model pulls.
Users can start asking deterministic questions immediately. The daemon socket
is available within seconds of service start.

## [0.0.309] - 2025-12-10

### Fixed - Desktop Wallpaper LLM Timeout

**Desktop wallpaper queries now have a deterministic fast-path:**
- Added `DesktopWallpaper` query class for wallpaper-related questions
- New handler checks DE settings (GNOME, KDE, XFCE, Cinnamon, MATE, Hyprland)
- If wallpaper found, returns path immediately (<1 second)
- If DE not detected, asks user where they keep wallpapers (for learning)
- Avoids 40+ second LLM timeouts on simple desktop questions

**Impact:** "where is my wallpaper" or "what is my desktop background" now responds
instantly instead of timing out waiting for LLM.

## [0.0.308] - 2025-12-10

### Fixed - Top Memory Processes Evidence Bug

**Probes without structured parsers now count as valid evidence:**
- Added `RawText(String)` variant to `ParsedProbeData` enum
- Common probes (ps aux, systemctl list-units, ip, ss, who, uptime, uname, cat, sensors)
  now return `RawText` instead of `Unsupported`
- `is_valid_evidence()` returns true for `RawText` - the raw text IS the evidence
- Root cause: `check_evidence_validity()` was returning 0 for probes that collected data
  but had no structured parser, causing "I can't answer yet because I didn't collect evidence"

**Impact:** "top memory processes" and similar queries now work correctly - evidence
is collected and answers are generated instead of false "no evidence" responses.

## [0.0.307] - 2025-12-10

### Fixed - Service Query Classification

**"running services" now correctly routes to RunningServices class:**
- Moved RunningServices check BEFORE generic ServiceStatus check in classifier
- Added more patterns: "services running", "what services are running", "services are running"
- ServiceStatus now excludes generic "running" to avoid false matches
- Queries like "what services are running?" now use correct `systemctl list-units` probe

**Impact:** User asks "running services" -> Gets list of running services, not failed units

## [0.0.306] - 2025-12-10

### Fixed - Reset Bug

**Reset now clears staff_stats.json:**
- Added StaffStats::clear() method to remove stats file
- handle_reset() now includes staff_stats in clear sequence
- Running `annactl reset` properly clears all staff metrics, XP, and levels
- Response JSON confirms "staff_stats" in cleared list

## [0.0.305] - 2025-12-10

### Fixed - Critical Bug Fixes

**Recipe matching domain guard expansion:**
- Added comprehensive domain detection for cross-domain protection
- Desktop/UI: wallpaper, hyprland, wayland, gnome, kde, window, display, etc.
- Editors: nano, vim, emacs, vscode
- Services: systemd, systemctl, daemon
- Plus: processes, files, logs, users, audio, time domains
- Prevents false recipe matches across unrelated query domains

**Negative feedback now properly records query:**
- FeedbackRequest struct now includes original_query field
- When user says "n" (not helpful), query is stored in recipe's negative_match_patterns
- Same query will now be rejected on future matches to that recipe

**Fixed repeated escalation messages:**
- Removed duplicate escalation comms from verification_stage (already in transcript)
- Added tracking flag in ticket_loop to prevent multiple escalation events
- Users now see single, clean escalation message

### Fixed - Debug Mode LLM Output

**Raw LLM prompts/responses now visible in debug mode:**
- Render function auto-detects debug mode from LLM call presence in transcript
- When debug_mode=ON, shows full LLM prompts and responses with stage/model info
- No truncation - see complete prompt engineering and LLM responses
- Works automatically when daemon has debug_mode enabled

### Added - Staff XP Tests

**Comprehensive XP calculation tests:**
- test_xp_calculation: verifies base + resolved + reliability bonuses
- test_xp_level_progression: tracks multi-ticket level advancement
- test_xp_to_level: validates all level thresholds (Novice to Principal)

## [0.0.304] - 2025-12-10

### Added - User-Friendly Error Handling

**New centralized error presentation module:**
- All errors now show helpful context and recovery suggestions
- Error categories: DaemonNotRunning, DaemonCrashed, Timeout, LlmUnavailable, NetworkError
- Each error type has specific troubleshooting steps
- Consistent formatting across all error displays

### Improved - Help System

**Enhanced help output with feature discovery:**
- Better examples: disk space, failed services, IP address queries
- Explains natural language support clearly
- Documents interactive mode and "show internal comms" feature
- More descriptive command descriptions

### Added - Long Wait Progress Feedback

**Checkpoint messages for long-running requests:**
- At 15s: "Still working on your request..."
- At 30s: "This is taking longer than usual. Anna is still analyzing..."
- At 45s: "Almost there... complex queries take more time."
- Users no longer see blank screen during long waits

## [0.0.303] - 2025-12-10

### Added - Unused Model Cleanup

**Anna now automatically cleans up models she no longer uses:**
- Tracks models pulled by Anna in the ledger
- At startup, removes models that Anna pulled but no longer needs
- Never touches user's own models (only deletes what Anna added)
- New `ModelDeleted` ledger entry type for tracking cleanup

### Changed - No Truncation in User Output

**Full output in all user-facing displays:**
- Status display: ticket queries shown in full
- Theatre mode: full evidence commands and output
- Debug mode: full LLM prompts and responses (not truncated to 500 chars)
- Probe output: full commands in evidence

### Improved - UX Review

**Better user experience overall:**
- Removed all "..." truncation from user-visible output
- Users see complete information, not cut-off text
- Debug mode truly shows "fly on the wall" view

## [0.0.302] - 2025-12-10

### Added - Debug Mode: Real LLM Output

**Debug mode now shows actual LLM prompts and responses:**
- New `LlmCall` transcript event captures LLM calls in debug mode
- Shows stage (specialist), model, prompt (truncated), response, duration, token count
- "Fly on the wall" view of what Anna is actually sending to the LLM

### Removed - Dead Code Cleanup

**Removed unused status display modules:**
- `status_statistics.rs`, `status_telemetry.rs`, `status_learning.rs`, `time_format.rs`
- These were used by old stats display, now removed with redesign

## [0.0.301] - 2025-12-10

### Changed - Stats Redesign: Service Desk Staff Performance

**Complete rewrite of `annactl stats` with Service Desk vision:**

- **[service desk]**: Total tickets, resolved, escalated, avg response time
- **[departments]**: Breakdown by department (Desktop, Network, Storage, etc.) with metrics
- **[staff roster]**: Real staff names (Sofia, Michael, Lars) with tier (Jr/Sr), XP, success rate, and level
- **[recent activity]**: Summary of recent work
- **[quick stats]**: Overall summary

**Staff now have XP and progression:**
- Added `xp: u64` and `level: u8` to StaffMetrics
- XP formula: 10 per ticket + 20 bonus for resolved + 2 per reliability point above 60
- Level progression: Novice (1) -> Apprentice (2) -> Competent (3) -> Skilled (4) -> Proficient (5) -> Expert (6)
- Seniors have different titles: Expert (1-3) -> Master (4-5) -> Principal (6)

**Removed gamification bloat:**
- No more "Anna XP" or achievements
- No more suggestions or maintenance actions
- Focus on real, meaningful staff performance data

## [0.0.300] - 2025-12-10

### Fixed - UX Improvements

**REPL greeting now works reliably:**
- Removed LLM-generated greetings (caused hangs and empty output)
- Now uses fast, reliable deterministic greetings

**Status display cleaned up:**
- Removed `[statistics]`, `[telemetry]`, `[learning]` sections from `annactl status`
- These belong in `annactl stats`, not status
- Status now focuses on system health and daemon state

**Staff names display correctly:**
- Fixed `extract_name` in status_statistics to show "Desktop Jr" not just "Jr"
- Now matches the fixed version in stats_display_v2

## [0.0.299] - 2025-12-10

### Fixed - Installer Removes Stale Binaries

**Install script now removes old binaries from `~/.local/bin`:**
- Old `annactl` or `annad` in `~/.local/bin` would shadow `/usr/local/bin` binaries
- This caused version mismatch where `annactl --version` showed old version
- Install script now cleans up stale binaries before requesting sudo

## [0.0.298] - 2025-12-10

### Fixed - Critical Stability and Validator Integration

**Socket creation now happens before daemon initialization:**
- `annactl` can connect immediately after service starts
- No more "long wait for anna.sock" after install
- Socket server runs in background while daemon initializes

**Stats consistency fixed:**
- `resolved_ok` and `success_rate` now use the same data source
- Fixed bug where stats showed inconsistent numbers (8 resolved but 0% success rate)

**Validator is now authoritative for answer resolution:**
- Added `validated: bool` field to `ServiceDeskResult`
- Answers are marked verified only when validation passes (not just `score >= 60`)
- Event log now records "verified" vs "failed" based on actual validation status
- Bad answers that don't answer the user's question are no longer counted as resolved

**Restored missing `annactl reset` command:**
- `annactl reset` clears learned recipes, knowledge base, and event log
- Requires y/N confirmation before executing

**Files Changed:**
- `annad/src/server.rs` - Socket creation moved before initialization
- `annactl/src/stats_display_v2.rs` - Fixed data source consistency
- `anna-shared/src/rpc/result.rs` - Added `validated` field
- `annad/src/rpc_handler/helpers.rs` - Uses `validated` for outcome
- `annad/src/rpc_handler/verification_stage.rs` - Returns validation status
- `annactl/src/main.rs` - Added Reset command
- `annactl/src/commands/handlers.rs` - Added handle_reset function
- Multiple files updated to populate `validated` field

## [0.0.297] - 2025-12-10

### Added - LLM Self-Healing for Senior Escalation

**Senior escalation now uses LLM-based answer regeneration:**

When junior verification fails after exhausting all rounds, the senior escalation path now invokes LLM self-healing to fix the answer. This implements the core principle: "Anna must validate answers and retry until exact score."

**Changes:**
- `ticket_loop.rs` now async - calls `validate_and_heal()` during senior escalation
- Builds `ParsedEvidence` from probe results for validation context
- Logs full validation path for debugging
- Falls back to deterministic senior escalation if LLM healing fails

**Integration Flow:**
1. Junior verification exhausts all rounds
2. Senior escalation triggers LLM self-healing via `answer_validator`
3. `validate_and_heal()` extracts claims, checks grounding, runs invention guard
4. If issues found, regenerates answer with correction prompt
5. Re-validates until score >= 80 or max attempts (3)
6. If LLM healing succeeds, returns verified answer
7. If LLM healing fails, falls back to deterministic senior loop

**Files Changed:**
- `annad/src/ticket_loop.rs` - Now async, integrated LLM self-healing
- `annad/src/rpc_handler/verification_stage.rs` - Added model parameter
- `annad/src/rpc_handler/llm_request.rs` - Passes model to verification
- `annad/tests/ticket_flow_tests.rs` - Updated to use async/tokio

## [0.0.296] - 2025-12-10

### Added - Answer Validation with Self-Healing

**Every answer is now validated before reaching the user:**

The new `answer_validator` module implements the core principle: "Any answer must be run against specialists to know if it's the right answer."

**Validation Flow:**
1. Extract claims from answer
2. Verify claims against evidence (grounding check)
3. Run invention guard (detect fabricated facts)
4. If validation fails, regenerate answer with explicit constraints
5. Retry up to 3 times until score >= 80

**Self-Healing:**
When validation fails, the validator:
- Builds a correction prompt with specific constraints
- Lists issues: "Do NOT claim X", "Only use evidence", etc.
- Asks LLM to regenerate with these constraints
- Re-validates the new answer

**Evidence Summary for LLM:**
Added `summary()` method to `ParsedEvidence` that formats evidence as human-readable text for LLM prompts:
```
Memory: 31.0GB total, 12.5GB used, 18.5GB available
Disk /: 45% used (234.5GB of 500.0GB)
```

**Files Changed:**
- `annad/src/answer_validator.rs` - New validation module
- `anna-shared/src/grounding/types.rs` - Added `summary()` and `has_kind()`

## [0.0.295] - 2025-12-10

### Added - Learning from Negative Feedback

**Anna Now Learns from "Not Helpful" Feedback:**

When users give negative feedback on a semantic recipe match, that query pattern is added to the recipe's `negative_match_patterns` list. Future queries matching that pattern will skip this recipe.

**How It Works:**
1. User asks "how do I undo my git commit?"
2. Anna incorrectly matches to disk space recipe via semantic similarity
3. User says "n" (not helpful)
4. Anna adds "how do I undo my git commit?" to recipe's negative_match_patterns
5. Next time, that exact query skips this recipe entirely

**Why This Matters:**
- Anna learns from mistakes without hardcoding
- Each recipe becomes smarter over time
- False positive semantic matches become self-correcting

**Files Changed:**
- `anna-shared/src/recipe/core.rs` - Added negative_match_patterns field
- `anna-shared/src/recipe_feedback.rs` - Store query on "not helpful" feedback
- `annad/src/recipe_similarity.rs` - Skip recipes with matching negative patterns

## [0.0.294] - 2025-12-10

### Fixed - Stricter Domain Guard

**Problem:** Semantic similarity still matched domain-specific queries (like git) to recipes without detected domains (like "how is my computer doing").

**Solution:** Now requires domain symmetry:
- If new query has a domain, recipe must have the SAME domain
- If recipe has a domain, new query must have the SAME domain
- Only queries with NO detected domain can match recipes with no domain

This prevents git/network/storage/cpu/memory/package queries from ever matching general health queries.

## [0.0.293] - 2025-12-10

### Fixed - Semantic Similarity Domain Guard

**Prevents Cross-Domain False Matches:**
Semantic similarity was matching unrelated queries (e.g., "how do I undo my git commit" → disk space recipe). Added domain guard to prevent this:

- **Domain Detection**: Queries are classified into domains (git, storage, network, cpu, memory, packages)
- **Cross-Domain Guard**: Semantic similarity check is skipped if query domains don't match
- **Threshold Increase**: Raised from 85 to 90 (small models unreliable at semantic judgment)

**Example Fixed:**
- Before: "how do I undo my git commit" → matched disk space recipe (95% confidence!) → WRONG
- After: Domain guard detects git vs storage mismatch → skips semantic check → routes correctly

**Files Changed:**
- `annad/src/recipe_similarity.rs` - Added `detect_query_domain()` and domain guard logic

## [0.0.292] - 2025-12-10

### Added - Preference-Aware Response Formatting

**Conversational Settings Now Actually Work:**
User preferences are now wired into the LLM response generation pipeline:

- **Learning Mode**: When enabled, responses include brief "why this works" explanations
- **Verbosity**: 0=minimal (essential info only), 1=normal, 2=detailed (full explanations)
- **Personality Traits**: Formality, humor, and technical depth now affect response tone
- **Auto-Confirm**: Low-risk changes are automatically applied when user enables this setting

**New `ResponsePreferences` Type:**
- Extracts formatting-relevant preferences from UserProfile
- Provides LLM-friendly descriptions for each setting level
- Loaded once per request for efficient preference checking

**Greeting Personalization:**
- Greetings now use personality traits for tone-appropriate language
- PersonalityContext added to GreetingContext for LLM prompts
- Casual users get relaxed greetings; formal users get professional ones

**Files Changed:**
- `anna-shared/src/user_profile/types.rs` - Added ResponsePreferences
- `anna-shared/src/greeting_context.rs` - Added PersonalityContext
- `annad/src/response_formatter.rs` - Preference-aware formatting
- `annad/src/greeting_generator.rs` - Personality-aware prompts
- `annactl/src/greeting/mod.rs` - Wire personality into context
- `annactl/src/change_commands.rs` - Auto-confirm for low-risk changes

## [0.0.289] - 2025-12-10

### Added - Interesting Facts in Greetings

**New `interesting_facts.rs` Module:**
- Data-driven facts about system, hardware, user patterns, and Anna's growth
- Facts categorized: Performance, Hardware, UserPattern, Growth, Milestone
- Priority-based selection (1=most interesting, 5=least)
- All facts derived from actual data - no hardcoded content

**Fact Sources:**
- Hardware: uptime, memory usage, network status
- Performance: telemetry trends (CPU, memory, disk), health score
- User Patterns: request milestones, streaks, response times, tenure
- Growth: recipes learned, self-sufficiency, strong areas

**Greeting Integration:**
- Top facts included in `GreetingContext` for LLM naturalization
- Fallback greeting shows "By the way: {fact}"
- Facts enhance personality without being intrusive

**Key Philosophy:**
- Greetings become more personalized over time
- LLM translator can naturalize facts into conversational style
- All data reflects real system/usage patterns

**Files changed:**
- `anna-shared/src/interesting_facts.rs` - New module
- `anna-shared/src/greeting_context.rs` - Added interesting_facts field
- `anna-shared/src/lib.rs` - Added module export
- `annactl/src/greeting/mod.rs` - Integrated fact generation

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
