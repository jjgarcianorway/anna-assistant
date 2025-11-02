# TUI Snapshot Tests

This directory contains golden output snapshots for TUI formatting tests.

## Creating Snapshots

```bash
# Full capabilities (color + emoji + UTF-8)
annactl hw show > hw_show_full.snap

# No color
NO_COLOR=1 annactl hw show > hw_show_nocolor.snap

# ASCII only (dumb terminal)
TERM=dumb NO_COLOR=1 annactl hw show > hw_show_ascii.snap
```

## Validating Snapshots

Run the actual command and compare output:

```bash
diff -u hw_show_full.snap <(annactl hw show)
```

## Snapshot Files

- `hw_show_full.snap` - Hardware profile with full formatting
- `hw_show_nocolor.snap` - Hardware profile without color
- `hw_show_ascii.snap` - Hardware profile in ASCII-only mode
- `status_live.snap` - Live status output (may vary)
- `status_nocolor.snap` - Status without color

Note: Some snapshots (like `status_live.snap`) will contain dynamic data and should only be used as visual references, not for strict byte-for-byte comparison.
