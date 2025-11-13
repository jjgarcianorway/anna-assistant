# Anna Assistant

**Your Knowledgeable Arch Linux Caretaker**

Anna is a local system and desktop caretaker for Arch Linux. She continuously analyzes your machine - hardware, software, services, and configuration - and helps you fix and improve everything in the simplest possible way.

**Current Version:** 4.4.0-beta.1

---

## What Anna Does

Anna silently watches your Arch machine, spots problems before they get bad, and either fixes them or tells you exactly what to do.

**Core Capabilities:**
- **Morning Health Check** - Two-second answer to "Is my system OK?"
- **Automatic Problem Detection** - Disk space, service failures, misconfigurations, missing firmware
- **Interactive Repairs** - Guided fixing with clear explanations and Arch Wiki references
- **Zero Ceremony** - Most users only need `annactl daily`

**What Anna is NOT:**
- ❌ Not a monitoring platform (she's for your local machine)
- ❌ Not an AI chatbot (she doesn't have conversations)
- ❌ Not a remote management server (she runs locally)

---

## Quick Start

### Installation

One command installs everything:

```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sh
```

That's it. Anna is now watching your system.

### First Run

The first time you interact with Anna, she'll automatically scan your system:

```bash
annactl daily

# Anna detects this is first run and shows:
# 👋 Welcome to Anna!
#
# Looks like this is the first time I see this machine.
# I will run a deeper scan once and then remember the results.
#
# Running first system scan...
#
# [Shows prioritized issues and recommendations]
```

Anna checks for:
- **Disk space**: Critical/warning levels, package cache, logs
- **Failed systemd services**: Units not running properly
- **Pacman health**: Stale database locks (>1 hour)
- **Laptop power**: TLP installation and configuration
- **GPU drivers**: NVIDIA GPUs without loaded drivers
- **Journal errors**: High error volume (>50 errors per boot)
- **Zombie processes**: Accumulating defunct processes
- **Orphaned packages**: Unused dependencies (>10 packages)
- **Core dumps**: Old crash dumps consuming disk space

### Daily Use

After first run, Anna is lightning fast:

```bash
# Every morning (takes ~2 seconds)
annactl daily

# If something is wrong
sudo annactl repair

# That's all most people need
```

### Example Session

```bash
$ annactl daily
╔══════════════════════════════════════════════════════════╗
║ 🔴 Daily System Check - 2025-11-13 12:27 ║
╚══════════════════════════════════════════════════════════╝

Health: 2 ok, 0 warnings, 1 failures

Disk: 96.5% used (24.8GB / 802.1GB total)

📊 Issues Detected:

  ❌ disk-space: Issue detected

📁 Disk Space Analysis:

  Your disk is 96.5% full. Here's what's using space:

  ⬇️ Downloads                71.0GB  /home/user/Downloads
  📦 Packages                 29.9GB  /var/cache/pacman/pkg

🎯 Recommended Actions:

1. Clean package cache
   $ sudo paccache -rk1
   📖 Keeps only the latest version of each package
   💾 Impact: Frees 29.9GB
   🔗 Arch Wiki: https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache
   Risk: ✅ Safe

💡 Next Steps:
   Run the disk space cleanup commands above (start with the safest)

$ sudo annactl repair
════════════════════════════════════════════════════════════
🔧 SYSTEM REPAIR
════════════════════════════════════════════════════════════

Anna will attempt to fix detected issues automatically.
Only low-risk actions will be performed.

⚠️  Actions may modify system state!

Proceed with repair? [y/N]: y

🔧 EXECUTING REPAIRS

✅ disk-space
  Action: cleanup_disk_space
  Details: Installed pacman-contrib; paccache -rk1: removed 847 packages (29.4GB)
  Source: [archwiki:System_maintenance#Clean_the_filesystem]

────────────────────────────────────────────────────────────
Summary: 1 succeeded, 0 failed
```

---

## Commands

Anna has a simple command surface. Most people only need 3 commands:

```bash
annactl daily       # Quick morning health check
annactl status      # Detailed system status
sudo annactl repair # Fix detected issues
```

### Advanced Commands

For power users and troubleshooting, Anna has additional commands:

```bash
annactl --help --all   # Show all commands including advanced
```

This includes:
- `health` - Detailed health probe execution
- `metrics` - System metrics display
- `profile` - Environment and hardware analysis
- `doctor` - Comprehensive diagnostics
- `learn` - View learning patterns
- `predict` - Show predictive analysis
- `upgrade` - Update Anna herself
- `ping` - Test daemon connection

---

## What Anna Detects

Anna's caretaker brain performs these checks on every run:

### Disk Space Analysis
- **Critical (>95% full)**: Immediate action required, system at risk
- **Warning (>90% full)**: Package cache cleanup recommended
- **Info (>80% full)**: Proactive space management suggested
- **Recommendation**: `paccache -rk1` to free package cache space

### Failed Systemd Services
- Detects services in failed or degraded state
- Shows which services and what failed
- **Repair**: Attempts to restart failed services

### Pacman Database Health
- Detects stale lock files (>1 hour old)
- Prevents package operation failures
- **Repair**: Safely removes stale locks after verification

### Laptop Power Management
- Auto-detects laptops via battery presence
- Checks if TLP is installed and enabled
- **Warning**: TLP installed but not enabled
- **Info**: TLP not installed, battery life could improve
- **Repair**: Enables TLP service

### GPU Driver Status
- Detects NVIDIA GPUs via `lspci`
- Checks if driver kernel module is loaded
- **Warning**: GPU present but driver not loaded
- **Recommendation**: Install nvidia and nvidia-utils packages

### Journal Error Volume
- Counts error-level entries in current boot journal
- **Critical (>200 errors)**: System has serious issues
- **Warning (>50 errors)**: Configuration or hardware problems
- **Repair**: Cleans old journal entries (vacuum to 7 days)

### Zombie Processes
- Scans `/proc` for processes in zombie state
- **Warning (>10 zombies)**: Parent processes not cleaning up
- **Info (>0 zombies)**: Minor process management issue
- **Note**: Zombies can't be killed directly - parent must reap them

### Orphaned Packages
- Finds packages no longer required by any installed package
- **Warning (>50 orphans)**: Significant disk space waste
- **Info (>10 orphans)**: Cleanup recommended
- **Repair**: Removes orphaned packages with `pacman -Rns`

### Core Dump Accumulation
- Checks `/var/lib/systemd/coredump` for crash dumps
- Identifies dumps older than 30 days
- **Warning (>1GB)**: Significant disk space consumed
- **Info (>10 files, >5 old)**: Old dumps can be cleaned
- **Repair**: Vacuums core dumps with `coredumpctl`

---

Every issue comes with:
- **Severity level**: Critical, Warning, or Info
- **Plain English explanation**: What's wrong and why it matters
- **Specific action**: Exact command to run
- **Arch Wiki reference**: Direct link to official documentation
- **Estimated impact**: What you'll gain (disk space, performance, etc.)
- **Repair action**: Can be fixed automatically via `annactl repair`

---

## Documentation

- **[Product Vision](docs/PRODUCT_VISION.md)** - What Anna is and why
- **[User Guide](docs/USER_GUIDE.md)** - Detailed usage flows and scenarios
- **[Changelog](CHANGELOG.md)** - Version history and updates

For historical and internal documentation, see `docs/archive/`.

---

## Philosophy

> Anna is the knowledgeable sysadmin friend who silently watches your Arch machine, spots problems before they get bad, and either fixes them or tells you exactly what to do - with as little ceremony as possible.

**Principles:**
1. **Few commands, high value** - 80% of value from 3-4 commands
2. **Two-second answer** - `annactl daily` tells you if you're OK
3. **Plain English** - No internal jargon or phase numbers
4. **Opinionated** - Clear recommendations based on Arch Wiki best practices

---

## Contributing

Anna follows a strict product guardrail:

**Any new feature must answer:**
1. What specific problem on the user's machine does it detect or fix?
2. How does it appear to the user through `daily`, `status`, `repair`, or `init`?

If you can't answer both, don't build it.

See `docs/PRODUCT_VISION.md` for the full vision.

---

## License

GPL-3.0-or-later

---

## Links

- **GitHub**: https://github.com/jjgarcianorway/anna-assistant
- **Issues**: https://github.com/jjgarcianorway/anna-assistant/issues
- **Releases**: https://github.com/jjgarcianorway/anna-assistant/releases

---

**Stop worrying about your system. Let Anna watch it for you.**
