# Anna v6.0 Epistemology

**The Foundation: What Anna Knows and How She Knows It**

---

## Core Principle

**Anna only reports what she can verify from real system sources.**

If Anna cannot point to the exact command or file that produced a piece of information, she does not report it.

---

## The Three Domains

Anna operates in exactly three domains. No more.

### 1. PACKAGES

**Source of Truth**: `pacman`

| Data Point | Command | Example |
|------------|---------|---------|
| Installed packages | `pacman -Q` | `nano 8.2-1` |
| Package details | `pacman -Qi <pkg>` | Description, size, deps |
| Package files | `pacman -Ql <pkg>` | Config paths in `/etc/` |
| Explicit vs dep | `pacman -Qe` / `pacman -Qd` | User installed vs auto |
| AUR packages | `pacman -Qm` | Foreign packages |

**What Anna tracks per package:**
```
- name: string (from pacman -Q)
- version: string (from pacman -Q)
- description: string (from pacman -Qi)
- installed_reason: "explicit" | "dependency" (from pacman -Qe/Qd)
- source: "official" | "aur" (from pacman -Qm)
- config_files: [paths] (from pacman -Ql filtered to /etc/)
- install_date: timestamp (from pacman -Qi)
- size: bytes (from pacman -Qi)
```

**Counts:**
- `pacman -Q | wc -l` = total installed packages
- `pacman -Qe | wc -l` = explicitly installed
- `pacman -Qd | wc -l` = dependencies
- `pacman -Qm | wc -l` = AUR/foreign

---

### 2. COMMANDS

**Source of Truth**: `$PATH` + `which` + `man`

| Data Point | Command | Example |
|------------|---------|---------|
| All binaries | `ls` each dir in `$PATH` | `/usr/bin/vim` |
| Binary exists | `which <cmd>` | `/usr/bin/vim` |
| Man summary | `man -f <cmd>` | `vim - Vi IMproved` |
| Help text | `<cmd> --help 2>&1 \| head -5` | Usage line |

**What Anna tracks per command:**
```
- name: string (binary name)
- path: string (from which)
- description: string (from man -f, fallback to --help first line)
- package: string | null (from pacman -Qo)
```

**Counts:**
- Count unique binaries across all `$PATH` directories
- This is the ONLY valid denominator for "commands"

---

### 3. SERVICES

**Source of Truth**: `systemctl`

| Data Point | Command | Example |
|------------|---------|---------|
| All units | `systemctl list-unit-files --type=service` | 260 units |
| Unit state | `systemctl is-active <unit>` | active/inactive |
| Unit enabled | `systemctl is-enabled <unit>` | enabled/disabled |
| Failed units | `systemctl --failed` | List of failures |

**What Anna tracks per service:**
```
- name: string (unit name)
- state: "active" | "inactive" | "failed"
- enabled: "enabled" | "disabled" | "static" | "masked"
- description: string (from systemctl show -p Description)
```

**Counts:**
- `systemctl list-unit-files --type=service | wc -l` = total services
- `systemctl list-units --type=service --state=active | wc -l` = running
- `systemctl --failed --type=service | wc -l` = failed

---

## Categories

Anna uses a FIXED taxonomy based on Arch Wiki categories. No invention.

| Category | Detection Method |
|----------|------------------|
| `editor` | Package in `vim`, `neovim`, `emacs`, `nano`, `helix`, `kate`, `gedit` |
| `shell` | Package provides shell in `/etc/shells` |
| `terminal` | Package in `alacritty`, `kitty`, `foot`, `wezterm`, `konsole`, `gnome-terminal` |
| `browser` | Package in `firefox`, `chromium`, `brave`, `librewolf`, `qutebrowser` |
| `compositor` | Package in `hyprland`, `sway`, `wayfire`, `river`, `dwl` |
| `service` | Has systemd unit file |
| `tool` | Default for anything else |

**Rule**: If unsure, category is `tool`. Never invent categories.

---

## Descriptions

Anna gets descriptions from these sources IN ORDER:

1. `pacman -Qi <pkg>` → Description field
2. `man -f <cmd>` → Man page summary
3. `<cmd> --help 2>&1 | head -1` → First line of help
4. Empty string (not "Unknown" or invented text)

**Never invent descriptions.**

---

## Error Intelligence

**Source of Truth**: `journalctl`

```bash
# Errors in last 24h
journalctl --since "24 hours ago" -p err -o json

# Warnings in last 24h
journalctl --since "24 hours ago" -p warning -o json
```

**What Anna tracks per error:**
```
- timestamp: u64
- unit: string (SYSLOG_IDENTIFIER or _SYSTEMD_UNIT)
- priority: 0-7 (0=emerg, 3=err, 4=warning, 6=info)
- message: string
```

**Aggregation:**
- Group by unit
- Count per severity level
- Show most recent message as sample
- Deduplicate identical messages

---

## Metrics

### Valid Metrics

| Metric | Numerator | Denominator | Source |
|--------|-----------|-------------|--------|
| Packages | `pacman -Q \| wc -l` | N/A | pacman |
| Explicit | `pacman -Qe \| wc -l` | total packages | pacman |
| AUR | `pacman -Qm \| wc -l` | total packages | pacman |
| Commands | count of PATH binaries | N/A | ls + which |
| Services | `systemctl list-unit-files` | N/A | systemctl |
| Running | active services | total services | systemctl |
| Failed | failed services | total services | systemctl |

### Invalid Metrics (REMOVE)

- "Quality: X%" - meaningless
- "Coverage: X%" - meaningless
- "Objects: N" - undefined domain
- "Total: N" without context - meaningless
- "Descriptions: X/Y" - Y is not meaningful

---

## Display Rules

### `annactl status`

```
[DAEMON]
  Status:     running (up 2h 15m)

[PACKAGES]
  Installed:  1,234
  Explicit:   456 (37%)
  AUR:        23 (2%)

[SERVICES]
  Total:      260
  Running:    45
  Failed:     0

[ERRORS] (24h)
  Errors:     12
  Warnings:   34
```

### `annactl knowledge`

```
[PACKAGES]
  Installed:  1,234
  Explicit:   456
  AUR:        23

[COMMANDS]
  In PATH:    2,685

[BY CATEGORY]
  Editors:    3 (vim, neovim, nano)
  Shells:     2 (bash, zsh)
  Terminals:  1 (alacritty)
  Browsers:   2 (firefox, brave)
```

### `annactl knowledge <name>`

```
[IDENTITY]
  Name:        vim
  Category:    editor

[PACKAGE]
  Version:     9.1.0-1
  Source:      official (extra)
  Installed:   explicit
  Size:        3.2 MB
  Date:        2024-11-15

[PATHS]
  Binary:      /usr/bin/vim
  Config:      /etc/vimrc

[DESCRIPTION]
  Vi IMproved, a programmer's text editor
  (source: pacman -Qi)
```

---

## What Anna Does NOT Do

1. **Invent numbers** - Every number has a source command
2. **Invent descriptions** - Empty is better than hallucinated
3. **Invent categories** - Fixed taxonomy only
4. **Show percentages without context** - Always show X/Y
5. **Claim "coverage"** - There is no meaningful coverage metric
6. **Claim "quality"** - There is no meaningful quality metric

---

## Implementation Checklist

- [ ] Remove `KnowledgeObject` generic model
- [ ] Create `Package`, `Command`, `Service` specific structs
- [ ] Implement real `pacman` queries
- [ ] Implement real `$PATH` scanning
- [ ] Implement real `systemctl` queries
- [ ] Implement real `journalctl` error parsing
- [ ] Remove invented categories
- [ ] Remove quality/coverage metrics
- [ ] Fix all display commands
