# Release Checklist for Anna Assistant

This checklist ensures proper release of Anna Assistant with pre-compiled binaries.

## Pre-Release

- [ ] **Version Bump**
  - [ ] Update version in `Cargo.toml` (workspace.package.version)
  - [ ] Update version in `scripts/install.sh` (VERSION variable)
  - [ ] Update version in `packaging/aur/PKGBUILD` (pkgver)
  - [ ] Update version in `packaging/aur/PKGBUILD-bin` (pkgver)

- [ ] **Testing**
  - [ ] Run full test suite: `cargo test --all`
  - [ ] Build release binaries: `cargo build --release`
  - [ ] Test installer with source build: `./scripts/install.sh --build`
  - [ ] Test daemon: `systemctl status annad`
  - [ ] Run health check: `annactl doctor check`

- [ ] **Documentation**
  - [ ] Update CHANGELOG.md with release notes
  - [ ] Review and update INSTALLATION.md
  - [ ] Update README.md if needed

- [ ] **Commit and Tag**
  ```bash
  git add -A
  git commit -m "chore: bump version to v0.11.0"
  git tag -a v0.11.0 -m "Release v0.11.0 - <brief description>"
  ```

## Release Process

- [ ] **Push Tag to Trigger GitHub Actions**
  ```bash
  git push origin main
  git push origin v0.11.0
  ```

- [ ] **Monitor GitHub Actions**
  - Go to: https://github.com/jjgarcianorway/anna-assistant/actions
  - Verify build completes for both x86_64 and aarch64
  - Check that artifacts are uploaded

- [ ] **Verify GitHub Release**
  - Go to: https://github.com/jjgarcianorway/anna-assistant/releases
  - Verify release was created with tag
  - Verify artifacts are attached:
    - [ ] `anna-linux-x86_64.tar.gz`
    - [ ] `anna-linux-x86_64.tar.gz.sha256`
    - [ ] `anna-linux-aarch64.tar.gz`
    - [ ] `anna-linux-aarch64.tar.gz.sha256`
    - [ ] `SHA256SUMS` (combined checksums)

- [ ] **Test Binary Download**
  ```bash
  # Clean environment
  rm -rf bin/ target/

  # Test installer downloads binaries
  ./scripts/install.sh
  ```

## Post-Release

- [ ] **Update AUR Packages**

  ### Binary Package (anna-assistant-bin)

  ```bash
  # Clone AUR repo (first time only)
  git clone ssh://aur@aur.archlinux.org/anna-assistant-bin.git
  cd anna-assistant-bin

  # Copy latest PKGBUILD
  cp ../anna-assistant/packaging/aur/PKGBUILD-bin PKGBUILD
  cp ../anna-assistant/packaging/aur/anna-assistant.install .

  # Update version and checksums in PKGBUILD
  # Get SHA256 from GitHub release page
  vim PKGBUILD  # Update pkgver and sha256sums_*

  # Generate .SRCINFO
  makepkg --printsrcinfo > .SRCINFO

  # Test locally
  makepkg -si

  # Commit and push
  git add PKGBUILD anna-assistant.install .SRCINFO
  git commit -m "Update to v0.11.0"
  git push
  ```

  ### Source Package (anna-assistant)

  ```bash
  # Clone AUR repo (first time only)
  git clone ssh://aur@aur.archlinux.org/anna-assistant.git
  cd anna-assistant

  # Copy latest PKGBUILD
  cp ../anna-assistant/packaging/aur/PKGBUILD .
  cp ../anna-assistant/packaging/aur/anna-assistant.install .

  # Update version and checksums in PKGBUILD
  vim PKGBUILD  # Update pkgver and sha256sums

  # Generate .SRCINFO
  makepkg --printsrcinfo > .SRCINFO

  # Test locally
  makepkg -si

  # Commit and push
  git add PKGBUILD anna-assistant.install .SRCINFO
  git commit -m "Update to v0.11.0"
  git push
  ```

- [ ] **Test AUR Installation**
  ```bash
  # Test binary package
  yay -S anna-assistant-bin

  # Test source package
  yay -S anna-assistant
  ```

- [ ] **Announcement**
  - [ ] Update README.md with release announcement
  - [ ] Post to relevant forums/communities (if applicable)
  - [ ] Update project website (if exists)

## Release Template

Use this template for GitHub release notes:

```markdown
## Anna Assistant v0.11.0 - <Release Name>

### Highlights

- Feature 1: Description
- Feature 2: Description
- Fix: Description

### Installation

**Quick install (no Rust required):**
\`\`\`bash
curl -LO https://github.com/jjgarcianorway/anna-assistant/releases/download/v0.11.0/anna-linux-x86_64.tar.gz
tar -xzf anna-linux-x86_64.tar.gz -C /tmp
sudo install -m 755 /tmp/annad /usr/local/bin/
sudo install -m 755 /tmp/annactl /usr/local/bin/
\`\`\`

**Or use the smart installer:**
\`\`\`bash
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
./scripts/install.sh
\`\`\`

**Arch Linux (AUR):**
\`\`\`bash
yay -S anna-assistant-bin
\`\`\`

See [INSTALLATION.md](INSTALLATION.md) for all installation methods.

### What's Changed

**Full Changelog**: https://github.com/jjgarcianorway/anna-assistant/compare/v0.10.0...v0.11.0

### Checksums

See attached `SHA256SUMS` file for verification.

### Upgrading

If you have a previous version installed:
\`\`\`bash
sudo systemctl stop annad
./scripts/install.sh
sudo systemctl start annad
annactl doctor check
\`\`\`

### Known Issues

- Issue 1: Description and workaround
- Issue 2: Description and workaround
```

## Rollback Procedure

If release has critical issues:

```bash
# Delete the tag locally and remotely
git tag -d v0.11.0
git push origin :refs/tags/v0.11.0

# Delete GitHub release through web interface

# Fix issues, then re-release with patch version
# e.g., v0.11.1
```

## Version Numbering

Anna uses semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes, major architecture changes
- **MINOR**: New features, non-breaking changes
- **PATCH**: Bug fixes, small improvements

Examples:
- `0.11.0` → `0.11.1` (bug fix)
- `0.11.1` → `0.12.0` (new feature)
- `0.12.0` → `1.0.0` (stable release, breaking changes)

## Automation Opportunities

Future improvements to automate:

- [ ] Automatic version bumping script
- [ ] Automatic CHANGELOG generation from commits
- [ ] Automatic AUR package updates via CI
- [ ] Release announcement automation
- [ ] Smoke tests on released binaries

---

**First Time Releasing?**

1. Ensure you have AUR SSH keys set up
2. Test the entire process in a clean VM or container
3. Have a rollback plan ready
4. Release during low-usage times (weekends)

**Questions?**

Contact the maintainer or open an issue.
