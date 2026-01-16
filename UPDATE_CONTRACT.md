# Update Contract

This document defines the exact behavior of Anna's auto-update mechanism.

## Where Anna Checks for Updates

Anna checks for updates at the GitHub Releases API endpoint:

```
GET https://api.github.com/repos/jjgarcianorway/anna-assistant/releases/latest
```

The response includes:
- `tag_name`: Version tag (e.g., "v0.3.61")
- `assets`: Array of downloadable artifacts

Check interval: 600 seconds (10 minutes).

## How Anna Selects Artifacts

Artifact selection is deterministic based on platform and architecture:

| Platform | Architecture | Artifact Name |
|----------|--------------|---------------|
| Linux | x86_64 | `annad-linux-x86_64`, `annactl-linux-x86_64` |
| Linux | aarch64 | `annad-linux-aarch64`, `annactl-linux-aarch64` |

Download URL pattern:
```
https://github.com/jjgarcianorway/anna-assistant/releases/download/v{VERSION}/{ARTIFACT}
```

## How Anna Verifies Integrity

### SHA256 Checksum Verification

Every release includes a `SHA256SUMS` file containing checksums for all artifacts.

Format:
```
{hash}  annad-linux-x86_64
{hash}  annactl-linux-x86_64
{hash}  annad-linux-aarch64
{hash}  annactl-linux-aarch64
```

Verification process:
1. Download artifact to staging directory
2. Download SHA256SUMS
3. Compute SHA256 hash of downloaded artifact
4. Compare computed hash against expected hash in SHA256SUMS
5. If mismatch, abort update and delete staged files

### Checksum Computation

SHA256 checksums are computed using:
```bash
sha256sum annad-linux-x86_64 annactl-linux-x86_64 > SHA256SUMS
```

Verification uses the same algorithm:
```bash
sha256sum -c SHA256SUMS
```

## What Anna Will Never Do

Anna's update mechanism has explicit constraints:

1. **No script execution** - Will never execute scripts downloaded from the network
2. **No arbitrary commands** - Will never run commands specified in release metadata
3. **No code evaluation** - Will never eval or interpret downloaded content as code
4. **No post-install hooks** - Will never run post-installation scripts from releases
5. **No network-sourced configuration** - Will never apply configuration from remote sources

The update mechanism performs exactly these operations:
- Download binary artifacts
- Download SHA256SUMS
- Verify checksums
- Replace local binaries atomically
- Restart daemon if needed

Nothing else.

## Update Flow

```
1. Fetch latest release metadata (GitHub API)
2. Compare version (semantic versioning)
3. If remote > local:
   a. Download artifacts to /var/lib/anna/staging/
   b. Download SHA256SUMS
   c. Verify all checksums
   d. Backup current binaries to /var/lib/anna/backup/
   e. Atomic swap: staging -> install location
   f. Verify new binaries work (--version check)
   g. If failure: restore from backup
4. Log result to /var/lib/anna/update.log
```

## Rollback Guarantee

If any verification fails:
1. Staged files are deleted
2. Backup is restored
3. Previous version continues running
4. Failure is logged
5. Update retried on next check interval

## Audit Trail

All update attempts are logged:
```json
{
  "timestamp": "2026-01-15T10:00:00Z",
  "action": "update_check",
  "current_version": "0.3.60",
  "latest_version": "0.3.61",
  "result": "success",
  "checksum_verified": true
}
```

Log location: `/var/lib/anna/update.log`

This update mechanism downloads artifacts only. It performs no remote execution.
