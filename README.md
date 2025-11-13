# Anna Assistant

**Your Knowledgeable Arch Linux Caretaker**

Anna is a local system and desktop caretaker for Arch Linux. She continuously analyzes your machine - hardware, software, services, and configuration - and helps you fix and improve everything in the simplest possible way.

**Current Version:** 4.2.0-beta.1

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

### Daily Use

```bash
# Every morning
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

## How Anna Helps

### 1. Detects Real Problems

Anna checks for:
- **Disk Space** - Package cache, logs, downloads consuming space
- **Service Failures** - systemd services not running
- **Misconfigurations** - TLP not enabled, Bluetooth not started
- **Missing Firmware** - Hardware not functioning optimally
- **Package Updates** - Security fixes available

### 2. Explains in Plain English

No jargon. No acronyms. Just clear explanations:
- "Your disk is 96% full. Package cache can free 30GB."
- "TLP is installed but not enabled. Your battery life could be better."
- "3 updates available including a kernel security fix."

### 3. Provides Clear Fixes

Every issue comes with:
- **Exact command to run** - Copy-paste ready
- **Plain English explanation** - What it does and why
- **Arch Wiki reference** - Direct link to relevant section
- **Risk level** - Safe, Low, Medium, or High
- **Estimated impact** - "Frees 30GB", "Improves battery life"

### 4. Actually Executes Fixes

`annactl repair` doesn't just show recommendations - it:
1. Installs missing tools if needed (e.g., pacman-contrib)
2. Executes the fix with confirmation
3. Reports what was done and the result

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
