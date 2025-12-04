# Update Protocol

This document describes how annad checks for and applies updates.

## Overview

annad checks for updates every 600 seconds by querying GitHub releases.
When a new version is available, it downloads, verifies, and installs the update.

## Update Check Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Update Check (every 600s)                    │
├─────────────────────────────────────────────────────────────────┤
│ 1. Fetch latest release from GitHub API                         │
│ 2. Parse version from release tag                               │
│ 3. Compare with current VERSION                                 │
│ 4. If newer version available:                                  │
│    a. Download artifacts to staging                             │
│    b. Download SHA256SUMS                                       │
│    c. Verify checksums                                          │
│    d. Backup current binaries                                   │
│    e. Atomic swap                                               │
│    f. Verify new binaries work                                  │
│    g. If failure, rollback                                      │
│ 5. Log result to audit log                                      │
└─────────────────────────────────────────────────────────────────┘
```

## GitHub API Endpoint

```
GET https://api.github.com/repos/{owner}/{repo}/releases/latest
```

Response includes:
- `tag_name`: Version tag (e.g., "v0.0.1")
- `assets`: Array of downloadable artifacts

## Version Comparison

Uses semantic versioning comparison:
- Parse both versions as (major, minor, patch)
- Compare numerically
- Only update if remote > local

## Artifact Download

Download URLs follow pattern:
```
https://github.com/{owner}/{repo}/releases/download/{tag}/{artifact}
```

Required artifacts:
- `annad-linux-{arch}` where arch is x86_64 or aarch64
- `annactl-linux-{arch}`
- `SHA256SUMS`

## Checksum Verification

SHA256SUMS file format:
```
<hash>  annad-linux-x86_64
<hash>  annad-linux-aarch64
<hash>  annactl-linux-x86_64
<hash>  annactl-linux-aarch64
```

Verification process:
1. Download SHA256SUMS
2. For each downloaded artifact:
   - Compute SHA256 hash
   - Find expected hash in SHA256SUMS
   - Compare hashes
3. If any mismatch, abort update

## Atomic Installation

1. Download to staging: `/var/lib/anna/staging/`
2. Verify all checksums pass
3. Backup current: `/var/lib/anna/backup/`
4. Move staging to install location
5. Test new binary: `annad --version`
6. If test fails, restore from backup

## Rollback

If update verification fails:
1. Log failure reason
2. Remove staged files
3. Restore backed up binaries
4. Continue running previous version
5. Retry on next update check

## Audit Log

All update attempts logged to `/var/lib/anna/update.log`:
```json
{
  "timestamp": "2024-12-04T10:00:00Z",
  "action": "update_check",
  "current_version": "0.0.1",
  "latest_version": "0.0.2",
  "result": "success|failed|skipped",
  "reason": "string if failed/skipped"
}
```

## Status Reporting

`annactl status` shows:
- Current version
- Last update check time
- Next scheduled check
- Last update result (if any)

## Configuration

Currently no user configuration for updates.
Future versions may add:
- Update channel (stable/beta)
- Auto-update enable/disable
- Update check interval override

## TODO (v0.0.1)

The following are not yet implemented in v0.0.1:
- [ ] Actual update check against GitHub API
- [ ] Artifact download and staging
- [ ] Checksum verification during update
- [ ] Atomic swap mechanism
- [ ] Rollback on failure
- [ ] Audit logging

v0.0.1 only records update check timestamps. Full implementation planned for v0.1.0.
