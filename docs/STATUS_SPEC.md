# STATUS_SPEC.md - annactl status Contract

Version: 1.0 (Phase 21, v0.3.54)

## Overview

`annactl status` displays a deterministic, movie-clean dashboard.
This spec defines the exact output contract.

## Section Order (MANDATORY)

Sections appear in this exact order. Empty sections are omitted silently.

1. VERSION
2. UPDATES
3. SERVICE
4. PERMISSIONS
5. CONFIG
6. HELPERS
7. MODELS

## Indicators

Use only these indicators (per UX_SPEC.md):
- `[OK]` - success/healthy (green)
- `[!]` - warning (yellow)
- `[X]` - error/failure (red)

No icons. No emojis.

## Field Formatting Rules

- Labels: lowercase, 13 chars padded, colon suffix
- Values: immediately after label
- Unknown values: use "unknown" (not "n/a", not "-")
- Disabled values: use "disabled" (not "off", not "n/a")
- Empty lists: omit section entirely

## Section 1: VERSION

```
VERSION
  annactl:      0.3.54
  annad:        0.3.54
  available:    0.3.54 [OK] | unknown
  consistency:  [OK] | [X] mismatch
```

Rules:
- annactl: from CARGO_PKG_VERSION
- annad: from daemon status response
- available: from GitHub release cache, or "unknown" if never fetched
- consistency: [OK] if annactl == annad, else [X] with "mismatch"

## Section 2: UPDATES

```
UPDATES
  interval:     60s | disabled
  last_check:   38s ago | never
  last_result:  [OK] | [X] FAILED
  next_check:   22s | disabled
```

Rules:
- interval: seconds or "disabled" if 0
- last_check: relative time or "never"
- last_result: [OK]/[X] prefix required
- next_check: relative time or "disabled"

## Section 3: SERVICE

```
SERVICE
  daemon:       [OK] running | [X] not running
  socket:       /run/anna/anna.sock
  socket_mode:  0660 anna:anna | [X] permission error
  last_error:   none | <single line error>
```

Rules:
- daemon: [OK]/[X] prefix, then status
- socket: path only
- socket_mode: numeric mode + owner:group, or [X] if wrong
- last_error: "none" if no errors, else single line (truncated at 60 chars)

## Section 4: PERMISSIONS

```
PERMISSIONS
  /etc/anna:      root:anna 755
  /var/lib/anna:  root:anna 750
  /run/anna:      root:anna 750
  /var/log/anna:  root:anna 750
  user_groups:    anna wheel | [X] not in anna group
```

Rules:
- Each path: owner:group mode (single line)
- user_groups: list groups user is in, or [X] if not in anna
- If path doesn't exist: "missing"

## Section 5: CONFIG

```
CONFIG
  exposure:     silent | summary | dialogue | debug
  teaching:     enabled | disabled
  debug_mode:   (only shown if enabled)
  update_interval: 60s
```

Rules:
- exposure: current exposure level
- teaching: enabled/disabled
- debug_mode: only show if true (never show "off")
- update_interval: seconds

## Section 6: HELPERS

```
HELPERS
  bc            [OK] anna
  ethtool       [OK] user
  jq            [X] missing
```

Rules:
- Sorted alphabetically by name
- Name: left-padded to 14 chars
- Status: [OK] if present, [X] if missing
- Installer: "anna" | "user" | "unknown"
- Only list helpers Anna cares about (from registry)

## Section 7: MODELS

```
MODELS
  translator:   llama3.2:3b
  default:      llama3.2:3b
```

Rules:
- Sorted alphabetically by role
- Role: left-padded to 14 chars
- Model: model name or "unknown"
- Only show if models exist in registry

## Exposure Level Effects

| Field | Silent | Summary | Dialogue | Debug |
|-------|--------|---------|----------|-------|
| VERSION | yes | yes | yes | yes |
| UPDATES | no | yes | yes | yes |
| SERVICE | no | yes | yes | yes |
| PERMISSIONS | no | no | yes | yes |
| CONFIG | no | no | yes | yes |
| HELPERS | no | no | no | yes |
| MODELS | no | no | no | yes |

At Silent level, only VERSION is shown.

## Deterministic Sorting

- HELPERS: alphabetically by name
- MODELS: alphabetically by role
- PERMISSIONS paths: in spec order (not alphabetically)

## Error Handling

- Daemon unreachable: Show only SERVICE section with [X] daemon not running
- Config unreadable: Show "unknown" for config values
- Never emit manual commands or recovery instructions

## Testing Contract

Golden fixtures must exist for:
- status_healthy.fixture: All sections [OK]
- status_daemon_down.fixture: SERVICE [X], others absent
- status_no_group.fixture: PERMISSIONS [X] not in anna group
- status_no_updates.fixture: UPDATES with "unknown"/"never"

## Do Not Regress

- [ ] Sections appear in spec order
- [ ] All indicators use [OK]/[!]/[X]
- [ ] No icons or emojis
- [ ] Unknown values say "unknown"
- [ ] HELPERS/MODELS sorted alphabetically
- [ ] Exposure level filtering works
- [ ] No manual commands in output
