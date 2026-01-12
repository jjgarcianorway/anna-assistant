# Anna Roadmap

## Current Version: v0.0.991

## Status Summary

Anna has completed extensive infrastructure with 1700+ instant-answer patterns across 42 categories.
See `docs/ROADMAP_ARCHIVE.md` for completed phase details.

---

## Recently Implemented (v0.0.900+)

### Pattern Matching System
- [x] 1700+ instant-answer patterns for common Linux questions
- [x] 42 pattern categories (pacman, systemd, network, security, hardware, gaming, etc.)
- [x] Fuzzy matching with typo tolerance
- [x] Synonym expansion for natural language variations
- [x] Pre-execution of diagnostic commands for grounded answers

### Proactive Monitoring (Learning System)
- [x] Hardware baseline tracking (USB/PCI devices)
- [x] Config file hash monitoring (/etc/ssh/sshd_config, /etc/sudoers, etc.)
- [x] Performance baseline learning (memory, CPU, boot time)
- [x] I/O rate tracking with delta calculation (disk/network KB/s)
- [x] Package change detection from pacman.log
- [x] Anomaly detection (>3x average triggers alert)

### Security Investigation
- [x] "who accessed my system" patterns
- [x] Login history analysis (last, lastlog, lastb)
- [x] SSH attempt monitoring
- [x] Failed login detection

---

## Not Yet Implemented

### High Priority - Natural Language Everything

#### Proactive Alerts (Background Daemon)
- [ ] Periodic health checks (every 5-10 min)
- [ ] Natural language notifications: "Your disk is 91% full"
- [ ] Security alerts: "3 failed login attempts from unknown IP"
- [ ] Performance alerts: "Boot time increased after yesterday's update"

#### Automatic Problem Solving
- [ ] Offer to fix known issues automatically
- [ ] "pacman lock detected, removing it now"
- [ ] "Your user lost wheel group access, adding you back"
- [ ] Confirmation before destructive actions

#### Context Memory Across Sessions
- [ ] Remember previous conversations
- [ ] "Last time this happened, it was Firefox. Checking..."
- [ ] Learn user patterns and preferences
- [ ] Suggest actions based on history

#### Rollback via Conversation
- [ ] "Undo what you just did"
- [ ] "Revert the changes from yesterday"
- [ ] Track all changes Anna makes
- [ ] Snapshot system state before risky operations

### Medium Priority

#### Multi-System Awareness
- [ ] "Check why my server is slow" (knows which server from context)
- [ ] SSH into remote systems for diagnostics
- [ ] Unified view across multiple machines

#### Smarter Learning
- [ ] Learn from user corrections
- [ ] Track which answers were helpful
- [ ] Suggest common workflows
- [ ] "You usually run git status after this"

---

## Development Guidelines

1. **All files must be under 400 lines**
2. No hardcoded specific cases (only reusable patterns)
3. Every change creates backup
4. Auto-update must always work
5. curl install must always work
6. **Pure natural language** - no extra commands or syntax
