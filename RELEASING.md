# Releasing Anna

Exact steps to cut a release.

## Prerequisites

- Git access to main branch
- GitHub CLI (`gh`) authenticated

## Steps

### 1. Update version

Edit three files to the same version:

```bash
# Cargo.toml (workspace version)
sed -i 's/^version = "[^"]*"/version = "X.Y.Z"/' Cargo.toml

# VERSION file
echo "X.Y.Z" > VERSION

# Verify consistency
grep '^version = ' Cargo.toml | head -1
cat VERSION
```

### 2. Update CHANGELOG.md

Add entry for the new version:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added/Changed/Fixed
- Description of changes
```

### 3. Commit

```bash
git add Cargo.toml VERSION CHANGELOG.md Cargo.lock
git commit -m "vX.Y.Z: Release description"
```

### 4. Push to main

```bash
git push origin main
```

### 5. Create and push tag

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

### 6. Confirm workflow succeeded

```bash
gh run list --workflow=release.yml --limit=1
```

Wait for status: `completed`.

### 7. Confirm release assets exist

```bash
gh release view vX.Y.Z --json tagName,assets -q '.assets[].name'
```

Expected output:
```
SHA256SUMS
annactl-linux-x86_64
annad-linux-x86_64
```

## Automated Release (Alternative)

If `release_on_version_bump.yml` is enabled, steps 5-7 happen automatically when the version bump is pushed to main.

1. Update version (step 1)
2. Update CHANGELOG.md (step 2)
3. Commit (step 3)
4. Push to main (step 4)
5. Workflow creates tag and release automatically

## Verification

After release, verify auto-update works:

```bash
# On a machine with Anna installed
annactl status
# Should show new version available or already updated
```

## Troubleshooting

### Workflow failed: CHANGELOG missing entry

Add section `## [X.Y.Z]` to CHANGELOG.md and re-push.

### Workflow failed: Version mismatch

Ensure Cargo.toml, VERSION file, and git tag all have the same version.

### Release assets missing

Check workflow logs:
```bash
gh run view --log
```
