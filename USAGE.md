# Anna Usage Guide

**Version:** v0.3.155
**Status:** Production Ready

---

## Quick Reference

### Pattern Library (Instant Answers)

**Vim Configuration**
```bash
annactl "enable syntax highlighting on vim"
annactl "show line numbers in vim"
```

**Git Setup**
```bash
annactl "initialize a git repository"
annactl "how to setup git"
```

**SSH Keys**
```bash
annactl "generate SSH keys"
annactl "create SSH key pair"
```

**File Permissions**
```bash
annactl "chmod 755 permissions"
annactl "make file executable"
```

**Service Management**
```bash
annactl "start a service with systemctl"
annactl "restart service"
annactl "enable service on boot"
```

**File Operations**
```bash
annactl "find files by name"
annactl "search text in files with grep"
annactl "extract tar.gz archive"
```

**Network Troubleshooting**
```bash
annactl "troubleshoot network connection"
annactl "check network connectivity"
```

**Disk Management**
```bash
annactl "check disk usage and space"
annactl "find large files"
```

**Process Management**
```bash
annactl "kill a process"
annactl "list running processes"
```

**System Logs**
```bash
annactl "view system logs with journalctl"
annactl "check logs"
```

**User Management**
```bash
annactl "add a user"
annactl "create new user account"
```

**Package Management**
```bash
annactl "install package with pacman"
annactl "update system"
annactl "search for package"
```

---

## System Investigation (Real Data)

**Hardware Information**
```bash
annactl "what is my CPU model?"
# → Intel(R) Core(TM) i9-14900HX

annactl "how much RAM do I have?"
# → 31.0 GB total, 5.9 GB used (19%)

annactl "list my disks"
# → Actual disk list with sizes
```

**System Status**
```bash
annactl "check disk usage"
# → 27% used - 255 GB / 952 GB

annactl "show memory usage"
# → Real memory stats

annactl "what services are running?"
# → List of active systemd services
```

**Network Status**
```bash
annactl "what is my IP address?"
# → Actual IP from ip addr

annactl "am I connected to wifi?"
# → Yes/no with interface details
```

---

## Historical Analysis (Telemetry)

**Hardware Changes**
```bash
annactl "has my hardware changed recently?"
# → Compares current vs 30-day snapshot

annactl "has my RAM been upgraded?"
# → Analyzes hardware history
```

**Package History**
```bash
annactl "what packages were installed this week?"
# → Queries pacman log

annactl "when was firefox last updated?"
# → Package update history
```

**System Trends**
```bash
annactl "will I run out of disk space?"
# → Predicts based on usage trend

annactl "is my boot time getting slower?"
# → Boot time trend analysis
```

---

## Configuration Changes

**Display Management**
```bash
annactl "enable GDM auto-login"
# → Creates action plan, asks for confirmation

annactl "disable laptop lid close suspend"
# → Modifies logind.conf with backup
```

**Bootloader**
```bash
annactl "set default boot timeout to 3 seconds"
# → Updates GRUB config

annactl "add kernel parameter quiet"
# → Modifies GRUB cmdline
```

---

## Package Management

**Install/Remove**
```bash
annactl "install neovim"
# → Uses pacman -S

annactl "install yay"
# → Installs AUR helper

annactl "remove old kernels"
# → Lists and removes safely
```

**Updates**
```bash
annactl "update all packages"
# → pacman -Syu

annactl "update all packages including AUR"
# → yay -Syu

annactl "what updates are available?"
# → Shows pending updates
```

**Maintenance**
```bash
annactl "clean package cache"
# → paccache -rk1

annactl "remove orphan packages"
# → pacman -Rns $(pacman -Qdtq)
```

---

## Troubleshooting

**Errors**
```bash
annactl "pacman says database is locked"
# → Pattern match: check process, remove lock

annactl "disk is full"
# → Shows cleanup strategies

annactl "wifi not working"
# → Network troubleshooting guide
```

**Performance**
```bash
annactl "why is my system slow?"
# → Investigates CPU/RAM/disk

annactl "what's using the most CPU?"
# → Lists top processes

annactl "what's using the most memory?"
# → Memory usage analysis
```

---

## Multi-Interface Usage

### CLI
```bash
# Single query
annactl "question"

# Interactive mode
annactl

# Daemon status
annactl status
```

### Telegram
Message your bot naturally:
```
enable syntax highlighting on vim
what is my CPU model?
check disk usage
update all packages
```

Same capabilities as CLI, same answers.

---

## Pattern Library Details

**28 Total Patterns:**

**Error Patterns (12):**
- pacman-db-lock
- disk-full
- permission-denied
- service-failed
- wifi-not-working
- audio-not-working
- system-freeze
- boot-grub
- nvidia-driver
- ssh-refused
- docker-permission
- dns-resolution

**Config Patterns (3):**
- vim-syntax-highlighting
- vim-line-numbers
- bash-alias

**Task Patterns (13):**
- git-init
- ssh-keygen
- chmod-permissions
- systemctl-service
- find-files
- grep-search
- tar-operations
- network-troubleshooting
- disk-usage
- process-management
- log-viewing
- user-management
- pacman-usage

**Recognition:** Keyword-based matching (0.85-0.95 confidence)
**Response Time:** <1ms (instant, no LLM needed)
**Fallback:** If pattern doesn't match, uses Ralph loop with LLM

---

## Understanding Responses

**Pattern Match (Instant)**
```
Based on available information, [answer here]

(0 iterations)
```
No LLM used, instant pattern library response.

**System Investigation (3-5s)**
```
Based on the provided system information, [answer here]

Evidence:
  [Probe: lscpu | grep Model (exit 0)]
  [Doc: [Arch Wiki: ...]]

(1-3 iterations)
```
LLM used with actual command execution and evidence tracking.

**Action Plan (Requires Confirmation)**
```
I can help with this. Here's what I'll do:

1. Create backup of /etc/gdm/custom.conf
2. Add AutomaticLoginEnable=true
3. Add AutomaticLogin=username
4. Restart GDM

Proceed? (yes/no)
```
Requires user confirmation before execution.

---

## Best Practices

**Pattern Library**
- Use natural language matching patterns
- "enable vim syntax" triggers vim-syntax-highlighting
- "troubleshoot network" triggers network-troubleshooting
- Faster than LLM queries (<1ms vs 3-5s)

**System Investigation**
- Be specific: "what is my CPU" better than "CPU info"
- Anna will execute commands and return real data
- Evidence shows what commands were run

**Configuration Changes**
- Always review action plans before confirming
- Backups are automatic, but review what changes
- Test non-destructive commands first

**Historical Queries**
- Use "has X changed?" for telemetry comparison
- Anna tracks 30 days of snapshots
- Good for: hardware, packages, performance trends

---

## Advanced Usage

**Multi-Domain Queries**
```bash
annactl "check wifi and disk space"
# → Parallel investigation (network + storage agents)
```

**Complex Reasoning**
```bash
annactl "why is my boot slow and how to fix it?"
# → Multi-step investigation + recommendations
```

**Prediction**
```bash
annactl "will I run out of disk space soon?"
# → Trend analysis with timeline prediction
```

---

## Troubleshooting Anna

**First Query Slow**
- LLM warming up (normal)
- Subsequent queries faster
- Pattern matches always instant

**Ollama Timeout**
- Heavy system load
- Circuit breaker activates
- Retry automatically with exponential backoff
- Pattern library still works

**Pattern Not Matching**
- Use more specific keywords
- Check pattern library coverage
- Falls back to LLM investigation (still works)

**Wrong Answer**
- Report via GitHub issues
- Include query and response
- Helps improve pattern library

---

## Monitoring

**Check Pattern Hit Rate**
```bash
journalctl -u annad | grep "Pattern match:" | tail -20
```

**Check Response Times**
```bash
journalctl -u annad | grep "iterations"
```

**Check for Errors**
```bash
journalctl -u annad | grep -i error | tail -20
```

---

## Tips

1. **Pattern queries are instant** - Use them for quick reference
2. **Be specific** - "what is my CPU model?" better than "CPU?"
3. **Natural language** - No special syntax needed
4. **Review before confirming** - Action plans show what will happen
5. **Use Telegram** - Same capabilities, convenient for remote access
6. **Check telemetry** - "has X changed?" uses 30-day history
7. **Report issues** - Help improve Anna

---

## Examples by Task

**Daily Tasks**
```bash
annactl "check system health"
annactl "update packages"
annactl "clean cache"
```

**Troubleshooting**
```bash
annactl "why is X not working?"
annactl "diagnose slow boot"
annactl "check for errors"
```

**Configuration**
```bash
annactl "enable auto-login"
annactl "setup SSH keys"
annactl "configure firewall"
```

**Learning**
```bash
annactl "how to setup git?"
annactl "explain systemd timers"
annactl "show pacman commands"
```

---

## Getting Help

**Pattern Library:** Try common phrases - if it matches, instant answer
**System Investigation:** Be specific about what you want to know
**Configuration:** Review action plans carefully before confirming
**Errors:** Include full query and response when reporting

**Support:**
- GitHub Issues: https://github.com/jjgarcianorway/anna-assistant/issues
- Check logs: `journalctl -u annad`
- Test with simple queries first

---

## Summary

**Anna combines:**
- **Pattern library** - 28 instant answers (<1ms)
- **System investigation** - Real command execution with data
- **Multi-agent** - Specialized domain experts working together
- **Telemetry** - 30-day historical analysis
- **Safety** - Confirmation + backups + rollback

**Result:** Fast, accurate, system-aware Linux administration assistant.
