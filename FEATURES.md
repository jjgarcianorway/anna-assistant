# Anna Features

v0.3.155 - Production Ready

## Pattern Library (Instant Answers)

28 common scenarios return answers in <1ms without LLM overhead.

**Error Patterns (12):**
- pacman-db-lock, disk-full, permission-denied, service-failed
- wifi-not-working, audio-not-working, system-freeze, boot-grub
- nvidia-driver, ssh-refused, docker-permission, dns-resolution

**Config Patterns (3):**
- vim-syntax-highlighting, vim-line-numbers, bash-alias

**Task Patterns (13):**
- git-init, ssh-keygen, chmod-permissions, systemctl-service
- find-files, grep-search, tar-operations, network-troubleshooting
- disk-usage, process-management, log-viewing, user-management, pacman-usage

**Instant Response:** Keyword matching (0.85-0.95 confidence) triggers immediate answers with no LLM call.

## Intelligent Reasoning (Ralph Loop)

For complex queries beyond pattern matching:

**Multi-Step Investigation:**
- Iterative command execution with real data
- Evidence tracking (probe results, wiki citations)
- Self-verification (LLM checks if data sufficient)
- Up to N iterations until answer is complete

**Grounded Answers:**
- All answers based on actual command output
- No invented facts or speculation
- Citations from Arch Wiki, man pages, --help

## Real System Access

Anna executes commands and returns actual system data:

**System Investigation:**
- Hardware info (CPU, RAM, disks, network)
- Service status (systemd units, processes)
- Resource usage (disk, memory, CPU, network)
- Configuration state (files, settings)

**30-Day Telemetry:**
- Hardware snapshots (detect upgrades/changes)
- Package history (installs, updates, removals)
- Performance baselines (boot time, resource usage)
- Trend analysis (capacity planning, predictions)

## Multi-Agent Orchestration

**8 Domains, 16 Specialists:**
- Desktop (junior/senior)
- Network (junior/senior)
- Storage (junior/senior)
- Security (junior/senior)
- Services (junior/senior)
- Performance (junior/senior)
- Hardware (junior/senior)
- Packages (junior/senior)

**Intelligence:**
- Domain routing based on keywords
- Escalation to seniors for complex issues
- Parallel investigation for multi-domain queries
- Result synthesis and learning

## Predictive ML

**Trend Analysis:**
- Disk full prediction (linear regression)
- Boot time degradation alerts
- Memory leak detection
- Anomaly detection (CPU, RAM, disk, network baselines)

**Capacity Planning:**
- Resource forecast projections
- Timeline predictions ("disk full in 45 days")
- Historical comparison

## Action Execution

**Template Plans:**
- GDM auto-login
- Disable lid close suspend
- Display resolution settings

**Safety:**
- User confirmation before execution
- Automatic backups before modifications
- Rollback support on failure
- Idempotency checks (skip if already configured)

**Privilege Escalation:**
- pkexec for automatic sudo
- No manual command prompts
- Auto-recovery from permission failures

## Self-Healing

Auto-recovery for infrastructure failures:
- Daemon connection (auto-restart via systemctl)
- Ollama service (auto-start with pkexec)
- Permissions (auto-add user to anna group)
- Models (retry with backoff)
- Wiki (retry initialization)

No manual recovery commands shown to users.

## Multi-Interface

**CLI:**
```bash
annactl "question"     # Natural language query
annactl                # Interactive session
annactl status         # Daemon health
```

**Telegram Bot (Optional):**
- Same capabilities as CLI
- Remote system administration
- Natural language queries
- Real-time streaming responses

## Local Privacy

- 100% local execution
- No cloud APIs
- No telemetry sent anywhere
- Ollama for LLM inference (qwen2.5)
- All state in system directories

## Production Quality

**Test Coverage:**
- 100% comparison test success rate (10/10)
- 28 pattern library tests
- 757/761 integration tests passing (99.5%)
- System investigation verified with real execution

**Reliability:**
- Graceful degradation under load
- Circuit breakers and retry logic
- Session isolation (multi-user safe)
- Auto-update from GitHub releases

## System Integration

**Paths:**
- `/etc/anna/` - Configuration
- `/var/lib/anna/` - State, telemetry, backups
- `/run/anna/` - Runtime socket

**Permissions:**
- Group-based (anna group)
- Systemd service
- Auto-update support
