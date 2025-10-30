# Getting Started with Anna Assistant

Welcome! Anna is an intelligent Linux system assistant that helps you manage, monitor, and optimize your Arch Linux system.

## Quick Start

### Prerequisites

- Arch Linux (or Arch-based distribution)
- Rust toolchain (`rustup`)
- systemd
- sudo access (Anna will ask politely when needed)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/anna-assistant/anna.git
cd anna-assistant
```

2. Run the installer (as your regular user):
```bash
./scripts/install.sh
```

The installer will:
- Build the binaries
- Install to `/usr/local/bin`
- Create the `anna` system group and add you to it
- Set up runtime directories
- Install and start the systemd service
- Verify everything is working

**Note**: You may need to log out and back in for group changes to take effect.

### Verify Installation

Check that Anna is running:
```bash
annactl status
```

You should see:
- Daemon status: `active`
- Version: `0.9.6-alpha.6`
- Socket path: `/run/anna/annad.sock`

## Core Commands

### Status & Health

```bash
# View system status
annactl status

# Check Anna's own health
annactl doctor check

# Automatically fix any issues
annactl doctor repair

# View system profile
annactl profile show

# Run health checks with JSON output
annactl profile checks --json
```

### Configuration Management

```bash
# List all configuration
annactl config list

# Get a specific value
annactl config get ui.emojis

# Set a value
annactl config set ui.emojis on

# Reset to defaults
annactl config reset
```

### Telemetry

```bash
# View current metrics snapshot
annactl telemetry snapshot

# View historical trends
annactl telemetry trends --hours 24

# Enable telemetry collection
annactl telemetry enable
```

### Personas

Anna can adapt her communication style:

```bash
# List available personas
annactl persona list

# Set a persona
annactl persona set ops

# See why current persona is active
annactl persona why
```

Available personas:
- `default` - Friendly and helpful
- `ops` - Concise, technical
- `minimal` - Bare essentials

### Policies

```bash
# List loaded policies
annactl policy list

# Reload policies from disk
annactl policy reload

# Test policy evaluation
annactl policy eval "telemetry.cpu_usage > 80"
```

## Understanding the System

### Architecture

Anna consists of two main components:

1. **annad** (daemon) - Runs as root via systemd
   - Collects telemetry every 60 seconds
   - Evaluates policies
   - Monitors system health
   - Provides RPC interface via Unix socket

2. **annactl** (CLI) - Run by users (in `anna` group)
   - Communicates with daemon via socket
   - Provides human-friendly output
   - Can run some commands without daemon (doctor, profile)

### Directory Structure

```
/etc/anna/              # Configuration and policies
  config.toml           # System configuration
  version               # Installed version
  policies.d/           # Policy YAML files
  personas.d/           # Persona definitions

/var/lib/anna/          # Persistent data
  telemetry.db          # SQLite database with metrics
  backups/              # Installation backups

/var/log/anna/          # Logs
  install.log           # Installation history
  doctor.log            # Repair operations
  autonomy.log          # Autonomy level changes

/run/anna/              # Runtime files
  annad.sock            # Unix socket (0660 root:anna)
```

### Health Checks

Anna performs 11 health checks when you run `annactl profile checks`:

**Core System**:
- CPU & Memory - Load, core count, RAM
- CPU Governor - Power management mode
- GPU Driver - Vendor detection and driver status
- Hardware Video Acceleration (VA-API)
- Audio Server - PulseAudio/PipeWire detection
- Network Interfaces - Active interface count
- Boot Performance - Startup time analysis
- Display Server - X11/Wayland session type

**Maintenance**:
- SSD TRIM - Weekly maintenance timer
- Firmware Updates - fwupd check
- System Logs - Persistent journald

Each check provides:
- Status: PASS/WARN/ERROR/INFO
- Clear message
- Remediation hint (if applicable)

## Common Tasks

### After Installation

1. Check system profile:
```bash
annactl profile show
```

2. Run health checks:
```bash
annactl profile checks
```

3. Enable telemetry (optional):
```bash
annactl telemetry enable
```

4. Follow daemon logs:
```bash
journalctl -u annad -f
```

### Troubleshooting

If something isn't working:

1. Check Anna's own health:
```bash
annactl doctor check
```

2. Let Anna fix herself:
```bash
annactl doctor repair
```

3. Check daemon logs:
```bash
journalctl -u annad -n 50
```

4. Restart the daemon:
```bash
sudo systemctl restart annad
annactl status
```

### Upgrading

When a new version is released:

```bash
cd anna-assistant
git pull
./scripts/install.sh
```

The installer will:
- Detect your existing installation
- Prompt for confirmation
- Create a backup
- Upgrade binaries
- Restart daemon
- Verify health

Backups are stored in `/var/lib/anna/backups/`.

### Uninstalling

To remove Anna:

```bash
sudo systemctl stop annad
sudo systemctl disable annad
sudo rm /usr/local/bin/annad /usr/local/bin/annactl
sudo rm -rf /etc/anna /var/lib/anna /var/log/anna /run/anna
sudo rm /etc/systemd/system/annad.service
sudo systemctl daemon-reload
```

To remove the group:
```bash
sudo gpasswd -d $USER anna
sudo groupdel anna
```

## Advanced Usage

### Autonomy Levels

Anna has two autonomy levels:

- **Low** (default) - Recommends actions, asks before executing
- **High** (opt-in) - Can perform repairs automatically

```bash
# View current level
annactl autonomy get

# Set level (prompts for confirmation)
annactl autonomy set high
```

All autonomy changes are logged to `/var/log/anna/autonomy.log`.

### Policy Authoring

Policies are YAML files in `/etc/anna/policies.d/`:

```yaml
# /etc/anna/policies.d/custom.yaml

- when: "telemetry.cpu_usage > 90"
  then: "alert"
  message: "High CPU usage detected"
  enabled: true

- when: "telemetry.mem_usage > 95"
  then: "alert"
  message: "Critical memory pressure"
  enabled: true
```

Reload after editing:
```bash
annactl policy reload
```

### Telemetry Analysis

Query telemetry data:

```bash
# Current snapshot
annactl telemetry snapshot

# Trends over time
annactl telemetry trends --hours 24

# Export to JSON
annactl telemetry snapshot --json > metrics.json
```

The telemetry database is SQLite, so you can query it directly:
```bash
sqlite3 /var/lib/anna/telemetry.db "SELECT * FROM telemetry ORDER BY id DESC LIMIT 10"
```

## Performance Expectations

- **Daemon idle CPU**: < 1% (watchdog alerts if > 5%)
- **Memory footprint**: ~20-50 MB
- **Telemetry collection**: Every 60 seconds
- **Database size**: ~1 MB per week of telemetry

## Next Steps

- Explore `annactl --help` for all commands
- Read the [CHANGELOG](../CHANGELOG.md) for version history
- Check system logs with `journalctl -u annad`
- Run health checks regularly with `annactl profile checks`

## Getting Help

- Check logs: `journalctl -u annad -n 100`
- Self-diagnose: `annactl doctor check`
- GitHub Issues: https://github.com/anna-assistant/anna/issues

## Development

To contribute or test changes:

```bash
# Build in debug mode
cargo build

# Run tests
cargo test

# Run end-to-end tests
./tests/e2e_basic.sh

# Test installer without installing
bash -n scripts/install.sh
```

---

**Version**: 0.9.6-alpha.6
**Status**: Alpha - core functionality working, polish in progress
**Support**: Arch Linux (primary), other systemd distros (experimental)
