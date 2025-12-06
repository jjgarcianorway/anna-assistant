# Anna Version Management

## Single Source of Truth

As of v0.0.73, Anna uses a centralized version system with build-time injection.

### Version Constants

All version information flows from `crates/anna-shared/src/version.rs`:

```rust
pub const VERSION: &str = env!("CARGO_PKG_VERSION");    // From Cargo.toml
pub const GIT_SHA: &str = option_env!("ANNA_GIT_SHA");   // Injected at build
pub const BUILD_DATE: &str = option_env!("ANNA_BUILD_DATE"); // Injected at build
pub const PROTOCOL_VERSION: u32 = 2;                     // RPC compatibility
```

### Build-Time Injection

`crates/anna-shared/build.rs` injects:
- `ANNA_GIT_SHA`: Short git commit hash
- `ANNA_BUILD_DATE`: Build date (YYYY-MM-DD)

## Version Flow

```
Cargo.toml (workspace.package.version)
    │
    ├─► anna-shared/src/version.rs (VERSION constant)
    │       │
    │       ├─► annactl --version
    │       ├─► annad --version
    │       ├─► annactl status (version section)
    │       └─► RPC GetDaemonInfo response
    │
    └─► GitHub Release tag (v0.0.XX)
            │
            └─► Auto-update version check
```

## Version Verification

### Local Verification

```bash
# Check client version
annactl --version
# Output: annactl 0.0.73

# Check daemon version
annad --version
# Output: annad 0.0.73

# Check both + comparison
annactl status
# Shows:
# [version]
#   annactl            0.0.73 (abc123)
#   annad              0.0.73 (abc123)
#   protocol           2
```

### Version Mismatch Detection

`annactl status` warns when client and daemon versions differ:

```
[version]
  annactl            0.0.73 (abc123)
  annad              0.0.72 (def456) [MISMATCH]

[health]
  status             WARN: client/daemon version mismatch
```

## Auto-Update Guarantees

### Atomic Pair Update (v0.0.73+)

The auto-updater ensures both binaries are updated together:

1. **Download phase**: Both `annactl` and `annad` downloaded to temp
2. **Verify phase**: Checksums validated, `--version` output checked
3. **Backup phase**: Existing binaries backed up
4. **Install phase**: Both binaries installed atomically
5. **Verify phase**: Installed versions confirmed to match
6. **Rollback**: On any failure, original binaries restored

### Update Safety

- Never downgrades (remote must be newer than installed)
- Never updates to version without downloadable assets
- Verifies SHA256 checksums before installation
- Confirms binary responds to `--version` before replacing

## Release Checklist

When releasing a new version:

1. Update `Cargo.toml` workspace version
2. Update `CHANGELOG.md`
3. Run `cargo test --workspace`
4. Commit and tag: `git tag v0.0.XX`
5. Push tag: `git push origin v0.0.XX`
6. Build release: `cargo build --release --workspace`
7. Upload binaries to GitHub release:
   - `annactl-linux-x86_64`
   - `annad-linux-x86_64`
   - `SHA256SUMS`

## Protocol Version

`PROTOCOL_VERSION` tracks RPC compatibility:
- Increment when RPC message format changes
- Client and daemon should have same protocol version
- Future: Could reject incompatible protocol versions

## Troubleshooting

### "Version mismatch" warning

Run `annactl update` or reinstall:
```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### Version shows "unknown" for git SHA

Binary was built outside git repo or without build.rs running.
Rebuild from git checkout: `cargo build --release`
