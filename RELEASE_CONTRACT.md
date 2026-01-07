# Release Contract

This document defines the invariants that must always be true for Anna releases.
CI enforces these rules and will block any PR that violates them.

## Version Source of Truth

1. **Single source**: The `VERSION` file in the repository root is the only source of truth
2. **Format**: Semantic versioning (MAJOR.MINOR.PATCH), e.g., `0.0.1`
3. **Derived versions**: All other version references must match VERSION:
   - `Cargo.toml` workspace version
   - `scripts/install.sh` VERSION variable
   - Git tags (`v0.0.1`)
   - GitHub release names

## Release Invariants

Every release MUST have:

1. **VERSION file** - Updated with new version number
2. **CHANGELOG.md** - Entry for the new version with date and changes
3. **Git tag** - Annotated tag `vX.Y.Z` pointing to release commit
4. **GitHub release** - Named `vX.Y.Z` with release notes from CHANGELOG
5. **Artifacts** - Built binaries for supported platforms:
   - `annad-linux-x86_64`
   - `annad-linux-aarch64`
   - `annactl-linux-x86_64`
   - `annactl-linux-aarch64`
6. **SHA256SUMS** - Checksum file for all artifacts

## Release Process

Use `scripts/release.sh` which:

1. Refuses to run on dirty working tree
2. Reads version from VERSION
3. Validates all required files exist
4. Builds release binaries
5. Generates SHA256SUMS
6. Creates release commit "release vX.Y.Z"
7. Creates annotated tag
8. Pushes commit and tag
9. Creates/updates GitHub release
10. Uploads artifacts

## CI Gates

### Required Files (ci.yml)
- `VERSION` - Must exist and be valid semver
- `SPEC.md` - Must exist
- `CHANGELOG.md` - Must exist
- `scripts/install.sh` - Must exist and be executable
- `scripts/uninstall.sh` - Must exist and be executable
- `scripts/release.sh` - Must exist and be executable

### Enforcement Rules

1. **400-line limit**: No source file may exceed 400 lines
2. **CLI surface lock**: Only these commands are allowed:
   - `annactl <request>`
   - `annactl` (REPL)
   - `annactl status`
   - `annactl uninstall`
   - `annactl reset`
   - `annactl -V/--version`
3. **Version consistency**: VERSION must match all derived locations
4. **CHANGELOG discipline**: VERSION changes require CHANGELOG update

### Release Discipline (release-discipline.yml)

PRs that change VERSION must also:
- Update CHANGELOG.md with new version entry
- Update any version-dependent documentation

PRs that change update logic must:
- Update docs/UPDATE_PROTOCOL.md

## Auto-Update Protocol

See [docs/UPDATE_PROTOCOL.md](docs/UPDATE_PROTOCOL.md) for how annad checks and
applies updates using GitHub releases.

## Artifact Verification

1. Installer downloads binaries from GitHub release
2. Downloads SHA256SUMS from same release
3. Verifies checksums before installation
4. Aborts with error if verification fails

## Rollback Mechanism

TODO: Implement in future version

When auto-update downloads new binaries:
1. Stage to temporary location
2. Verify checksums
3. Backup current binaries
4. Atomic swap
5. Verify new binary runs
6. If failure, restore backup
