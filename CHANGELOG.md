# Changelog

All notable changes to Anna will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.16...HEAD
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
