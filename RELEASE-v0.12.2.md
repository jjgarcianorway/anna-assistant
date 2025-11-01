# Anna v0.12.2 Release Instructions

## Summary

v0.12.2 adds collectors, radars, and telemetry RPC endpoints with full JSON support.

**Status:** ✓ READY TO RELEASE
- Build: SUCCESS (0 errors, 27 warnings - unused code only)
- Tests: 30/30 passed
- Version: 0.12.2 in Cargo.toml
- Binaries: Built and verified

---

## Changes in v0.12.2

### New Features
- **Focused Collectors** (collectors_v12.rs): sensors, network, disk, top processes
- **Radar Scoring** (radars_v12.rs): Health and Network radars (0-10 scale)
- **RPC Endpoints**: collect, classify, radar_show
- **CLI Commands**:
  - `annactl collect --limit N --json`
  - `annactl classify --json`
  - `annactl radar show --json`
- **Telemetry Schema**: SQLite schema for snapshots, scores, classifications

### Improvements
- install.sh now shows versions dynamically (never hardcoded)
- install.sh has better timeout handling (10s for socket, 5s for response)
- install.sh shows detailed diagnostics on failure

### Documentation
- docs/V0.12.2-IMPLEMENTATION-SUMMARY.md
- docs/CLI-REFERENCE-v0122.md
- CHANGELOG_v0122.md
- TEST-RESULTS-v0122.md

---

## Release Command

**Copy and paste this EXACT command:**

```bash
./scripts/release.sh -t patch -m "v0.12.2: collectors, radars, telemetry RPC endpoints

New Features:
- Focused collectors for sensors, network, disk, top processes
- Health and Network radar scoring systems (0-10 scale)
- Three new RPC endpoints: collect, classify, radar_show
- Three new CLI commands with JSON support
- Complete telemetry database schema

Improvements:
- install.sh shows versions dynamically (annactl --version)
- install.sh has better timeout and error handling
- Full graceful degradation for missing sensors

Testing:
- 30/30 automated verification tests passing
- Smoke tests for all new commands
- JSON schema validation

Documentation:
- Complete implementation summary
- CLI reference with examples
- Troubleshooting guide

See CHANGELOG_v0122.md and docs/V0.12.2-IMPLEMENTATION-SUMMARY.md for details."
```

This will:
1. Create git commit
2. Create git tag v0.12.2
3. Push to origin with tags
4. Trigger GitHub Actions to build and release

---

## After Release

Once released, you can install with:

```bash
./scripts/install.sh
```

The install.sh will:
1. Download v0.12.2 binaries from GitHub
2. Show "Installing version: 0.12.2"
3. Install and start the daemon
4. Show "Daemon version: 0.12.2"

---

## Verify Installation

```bash
# Check version
annactl --version
# Should show: annactl 0.12.2

# Test new commands
annactl collect --limit 1
annactl classify
annactl radar show

# Test JSON output
annactl collect --json | jq '.snapshots[0]'
annactl classify --json | jq '.persona'
annactl radar show --json | jq '.overall'
```

---

## Next: v0.12.3

After v0.12.2 is released and verified, we'll implement v0.12.3 Arch Advisor:
- Arch Linux inventory collectors
- Recommendation engine with safety rules
- `annactl advice` commands
- Policy-gated auto-apply
- Full tests and documentation

But first, let's get v0.12.2 out and tested!
