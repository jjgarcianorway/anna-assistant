# Security Policy

## Overview

Anna Assistant is a local-first system administrator for Arch Linux. Security is built on three principles:

1. **Local-first**: All data stays on your machine unless you explicitly configure otherwise
2. **Least privilege**: The daemon runs with minimal necessary permissions
3. **Explicit approval**: No system changes without your explicit command

This document describes Anna's security model, how to report vulnerabilities, and recommended hardening practices.

---

## Reporting Security Vulnerabilities

If you discover a security vulnerability in Anna Assistant, please report it privately:

**Email**: jjgarcianorway@gmail.com
**Subject**: [SECURITY] Anna Assistant Vulnerability Report

### What to Include

- **Description**: Clear explanation of the vulnerability
- **Impact**: What an attacker could do
- **Reproduction steps**: How to reproduce the issue
- **Environment**: Anna version, Arch Linux version, relevant hardware/software
- **Suggested fix** (optional): If you have ideas for remediation

### Response Timeline

- **Acknowledgment**: Within 48 hours of report
- **Assessment**: Vulnerability validated within 7 days
- **Fix**: Critical issues fixed and released within 7-14 days
- **Disclosure**: Coordinated public disclosure 90 days after fix release or when patch is widely deployed

---

## Data & Privacy Model

### What Anna Stores Locally

Anna keeps all data on your machine by default:

**Historian Database** (`/var/lib/anna/historian.db`):
- System metrics snapshots (CPU, memory, disk, boot events)
- Service health history
- OOM kill events and process statistics
- No personally identifying information beyond hostname

**Query Logs** (`/var/log/anna/`):
- User queries to `annactl`
- Command executions and results
- Success/failure status and timing
- Logged in structured format for diagnostics

**State Files** (`/var/lib/anna/`):
- Context database for conversation history
- Daemon state and configuration
- Health check results and reports

### What Anna Sends Over the Network

By default, Anna only makes network requests for:

1. **LLM Backend** (if configured):
   - Sends queries to the configured endpoint (default: `localhost:11434` for Ollama)
   - Query text and system context may be sent to the LLM
   - **No automatic "phone home"** - only to your configured LLM endpoint

2. **GitHub Release Checks** (manual installations only):
   - Checks GitHub API for new releases every 24 hours
   - Sends no user data, only receives release information
   - AUR installations disable this automatically

3. **Package manager operations** (when you explicitly request them):
   - Standard pacman/yay network access for package queries and updates

### Privacy Guarantees

- ✅ All telemetry stays local unless you configure a remote LLM
- ✅ No analytics or tracking sent to developers
- ✅ Historian database is SQLite on disk, no cloud sync
- ✅ Logs are readable by the `anna` group for diagnostics
- ✅ You can inspect `/var/log/anna/` at any time to see what's logged

---

## Secrets Management Policy

### Repository Policy

The Anna Assistant repository enforces strict secrets hygiene:

**Never commit:**
- API keys or tokens
- Private keys (`*.key`)
- Certificates (`*.pem`, `*.crt`, `*.csr`)
- Certificate serials (`*.srl`)
- Configuration files containing secrets

**Gitignore rules** (`.gitignore`):
```
testnet/config/tls/
testnet/certs/
**/*.key
**/*.pem
**/*.srl
**/*.crt
**/*.csr
.env
.env.local
```

### User Secret Management

If you configure Anna to use a remote LLM endpoint requiring authentication:

**Store API keys in:**
- Environment variables (e.g., `OPENAI_API_KEY`)
- Local configuration files with restricted permissions:
  ```bash
  sudo chown root:anna /etc/anna/llm.toml
  sudo chmod 640 /etc/anna/llm.toml
  ```

**Never:**
- Commit API keys to version control
- Store keys in world-readable files
- Share keys in issues or pull requests

---

## Runtime Permissions and Systemd Hardening

### Current Service Configuration

The `annad.service` systemd unit implements security hardening:

**User and Group Isolation:**
```ini
User=root  # Currently root for compatibility
Group=anna
SupplementaryGroups=
```

**Capability Restrictions:**
```ini
CapabilityBoundingSet=CAP_DAC_OVERRIDE CAP_CHOWN CAP_FOWNER CAP_SYS_ADMIN CAP_SYS_RESOURCE
AmbientCapabilities=
NoNewPrivileges=true
```

**Filesystem Protection:**
```ini
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/log/anna /var/lib/anna /run/anna /usr/local/bin
PrivateTmp=true
```

**Kernel Protection:**
```ini
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectKernelLogs=yes
ProtectControlGroups=yes
RestrictRealtime=yes
RestrictNamespaces=yes
LockPersonality=yes
MemoryDenyWriteExecute=yes
```

**Network Restrictions:**
```ini
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
# Note: IPAddressDeny removed due to WiFi driver compatibility
```

**System Call Filtering:**
```ini
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @obsolete @resources
SystemCallArchitectures=native
SystemCallErrorNumber=EPERM
```

**Device Access:**
```ini
DevicePolicy=closed
DeviceAllow=/dev/null rw
DeviceAllow=/dev/zero rw
DeviceAllow=/dev/urandom r
```

### File System Permissions

Anna uses the `anna` group for access control:

**Configuration** (`/etc/anna/`):
- Owner: `root:anna`
- Permissions: `755` (directory), `644` (files)
- Only root can modify, anna group can read

**Logs** (`/var/log/anna/`):
- Owner: `root:anna`
- Permissions: `750` (directory), `640` (files)
- Anna group can read for diagnostics

**Socket** (`/run/anna/anna.sock`):
- Owner: `root:anna`
- Permissions: `660`
- Only anna group members can connect

**State/Database** (`/var/lib/anna/`):
- Owner: `root:anna`
- Permissions: `770` (directory), `640` (files)
- Anna group has write access for reports

**Auto-update Path** (`/usr/local/bin/`):
- Required for self-update capability
- Only manual installations update binaries here
- AUR installations are managed by pacman

---

## Command Risk Levels and Action Safety

Anna's Planner → Executor → Interpreter architecture enforces safety at execution time:

### Safety Levels

**Read-Only (Safe)**:
- Inspection queries: "do I have steam installed?"
- Hardware queries: "does my CPU support AVX2?"
- System status: CPU usage, memory, service health
- **Risk**: None - no system modifications
- **Requirement**: User in `anna` group

**Configuration/Package Changes (Requires Approval)**:
- Package installation/removal
- Service start/stop/restart
- Configuration file modifications
- **Risk**: Medium - changes system state
- **Requirement**: Explicit user command + sudo when needed
- **Safety**: Commands shown for review before execution

**High-Risk Operations (Manual Only)**:
- Destructive filesystem operations (rm, dd)
- Kernel parameter changes
- System-wide configuration
- **Risk**: High - potential data loss or system instability
- **Safety**: Executor blocks by default, requires explicit override

### Executor Safety Rules

The executor (`anna_common/src/executor_core.rs`) validates all commands before execution:

- ✅ Package queries (pacman -Q, pacman -Qq)
- ✅ System inspection (lscpu, lsblk, systemctl status)
- ✅ Safe grep/awk/sed read operations
- ❌ Destructive commands (rm -rf, dd, mkfs)
- ❌ Privilege escalation attempts
- ❌ Arbitrary command execution without validation

### Future: Rollback and Backups

Current version does not include automatic rollback for configuration changes. For now:
- **Package operations**: Use pacman's cache (`/var/cache/pacman/pkg/`)
- **Configuration**: Manually back up `/etc/` before changes
- **State**: Anna keeps history in historian database

Future versions may include structured rollback helpers.

---

## Network Access and Auto-Update

### Network Calls

Anna makes network requests only for:

1. **LLM Endpoint** (when enabled):
   - Default: `http://localhost:11434` (Ollama)
   - Configurable to OpenAI-compatible endpoints
   - Only sends queries you explicitly ask

2. **GitHub Release Checks** (manual installations):
   - Checks `https://api.github.com/repos/jjgarcianorway/anna-assistant/releases`
   - Every 24 hours for update availability
   - Logs results, never auto-installs

3. **Package Managers**:
   - Standard pacman/AUR mirrors when you request updates
   - Respects your pacman configuration

### Auto-Update Security

**Manual installations** support self-update via `annactl upgrade`:

**Security precautions:**
- ✅ SHA256 checksum verification of downloaded binaries
- ✅ Atomic binary replacement (no partial installs)
- ✅ Backup of previous version to `/var/lib/anna/backup/`
- ✅ Root requirement (`sudo annactl upgrade`)
- ✅ HTTPS for all GitHub API and download URLs
- ✅ 10-second timeout on network operations

**AUR/Pacman installations:**
- Auto-update **disabled** - respects package manager
- Updates via `yay -Syu` or `pacman -Syu`

---

## Logging, Audit, and Monitoring

### What Gets Logged

**Daemon logs** (`/var/log/anna/` and `journalctl -u annad`):
- Daemon startup/shutdown events
- Mode changes and critical errors
- System health check results
- Network operations (LLM calls, update checks)

**Query logs** (`/var/log/anna/ctl.jsonl`):
- User commands to annactl
- Query text and responses
- Execution timing and success/failure

**Historian database** (`/var/lib/anna/historian.db`):
- Time-series system metrics (CPU, memory, disk)
- Service health status changes
- Boot events and OOM kills

### Log Security

- Logs are local-only (no automatic forwarding)
- Sensitive values (API keys, tokens) should not be logged
- Log files readable by `anna` group for diagnostics
- Rotate logs with `logrotate` to manage size

### Monitoring

If you set up Prometheus/Grafana for Anna metrics:

**Bind to localhost only:**
```bash
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3000

# Access remotely via SSH tunnel (NEVER expose ports):
ssh -L 9090:localhost:9090 user@host
ssh -L 3000:localhost:3000 user@host
```

---

## User Hardening Checklist

After installation, verify these security practices:

### Service Security
- [ ] Daemon running as `root` user with `anna` group (current design)
- [ ] Your user added to `anna` group: `sudo usermod -aG anna $USER`
- [ ] Socket permissions correct: `ls -la /run/anna/anna.sock` → `srw-rw---- root anna`
- [ ] Systemd hardening directives applied (see `annad.service`)

### Filesystem Security
- [ ] `/etc/anna/` owned by root, `644` permissions
- [ ] `/var/log/anna/` has `750` permissions, `anna` group readable
- [ ] `/var/lib/anna/` has `770` permissions, `anna` group writable
- [ ] `/run/anna/` has `770` permissions for socket

### LLM Security
- [ ] LLM endpoint configured (default: `localhost:11434`)
- [ ] If using remote LLM, API key stored securely in `/etc/anna/` with `640` permissions
- [ ] Remote LLM endpoint uses HTTPS

### Network Security
- [ ] Monitoring (if installed) bound to localhost only
- [ ] SSH tunnels configured for remote access
- [ ] Firewall rules restrict external access

### Operational Security
- [ ] Regular updates: `sudo annactl upgrade` (manual) or `yay -Syu` (AUR)
- [ ] Review logs periodically: `journalctl -u annad`
- [ ] Monitor anna group membership: `getent group anna`

---

## Recommended Additional Hardening

These are **recommendations**, not currently enforced by default:

### Systemd Unit Hardening

Consider adding to `/etc/systemd/system/annad.service.d/override.conf`:

```ini
[Service]
# Run as dedicated user (requires user creation)
User=annad
Group=anna

# Stricter syscall filtering
SystemCallFilter=@system-service @file-system @network-io
SystemCallFilter=~@privileged @resources @obsolete @debug

# Disable network if using local-only LLM
# (Uncomment only if Ollama is local and no GitHub checks needed)
# PrivateNetwork=yes

# Restrict writable paths further
ReadWritePaths=/var/log/anna /var/lib/anna /run/anna
# Remove /usr/local/bin if not using manual updates
```

Apply with:
```bash
sudo systemctl daemon-reload
sudo systemctl restart annad
```

### Firewall Rules

If exposing monitoring tools remotely:
```bash
# Allow only from specific IPs
sudo ufw allow from 192.168.1.0/24 to any port 9090
sudo ufw allow from 192.168.1.0/24 to any port 3000

# Default deny
sudo ufw default deny incoming
```

### AppArmor Profile

Consider creating an AppArmor profile for additional MAC-level protection (not provided by default).

---

## Responsible Disclosure

We follow coordinated disclosure practices:

1. **Report received** → Acknowledge within 48 hours
2. **Validation** → Confirm vulnerability within 7 days
3. **Fix developed** → Test and review patch
4. **Release** → Publish fix with security advisory
5. **Public disclosure** → 90 days after fix release, or when widely deployed

**For contributors**: If GitGuardian or another tool alerts you about a committed secret:
1. **Stop**: Do not push more commits
2. **Revert**: Use `git reset` to remove the commit locally
3. **Notify**: Email security contact immediately
4. **Rotate**: If the secret was real (not test), rotate it

---

## References

- [Anna Assistant GitHub Repository](https://github.com/jjgarcianorway/anna-assistant)
- [Arch Linux Security Guidelines](https://wiki.archlinux.org/title/Security)
- [OWASP Key Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html)
- [systemd Security Hardening](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)

---

**Last Updated**: 2025-11-25 (v6.42.0 - Security documentation rewritten to match current architecture)
**Maintained By**: Anna Assistant Security Team
