# Anna Assistant - Packaging Guide

**Phase 2.0.0-alpha.1** - Maintainer documentation for package managers

Citation: [aur:pkgbuild-guidelines][homebrew:formula-cookbook][github:releases-api]

## Overview

This guide covers maintaining Anna Assistant packages for various distribution channels:
- **AUR** (Arch User Repository) - Arch Linux
- **Homebrew** - macOS and Linux
- **Manual** - Direct binary installation

## Release Process

### 1. Version Bump

Update version in `Cargo.toml`:

```toml
[workspace.package]
version = "1.17.0"
```

Update `CHANGELOG.md` with release notes.

### 2. Build Release Binaries

```bash
# Build for Linux (x86_64)
cargo build --release

# Binaries in: target/release/annad, target/release/annactl

# For cross-compilation (macOS):
# cargo build --release --target x86_64-apple-darwin
# cargo build --release --target aarch64-apple-darwin
```

### 3. Generate Checksums

```bash
cd target/release
sha256sum annad annactl > SHA256SUMS
cat SHA256SUMS
```

### 4. Create GitHub Release

```bash
# Tag the release
git tag -a v1.17.0 -m "Release v1.17.0 - Description"
git push origin v1.17.0

# Create release via gh CLI
gh release create v1.17.0 \
  --title "v1.17.0" \
  --notes "Release notes here" \
  target/release/annad \
  target/release/annactl \
  target/release/SHA256SUMS
```

### 5. Update Package Managers

#### AUR Update

```bash
cd packaging/aur/anna-assistant-bin

# Update PKGBUILD version
sed -i 's/pkgver=.*/pkgver=1.17.0/' PKGBUILD

# Update checksums
updpkgsums

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Test build
makepkg -si

# Commit to AUR repo
git add PKGBUILD .SRCINFO
git commit -m "Update to v1.17.0"
git push
```

#### Homebrew Update

```bash
cd packaging/homebrew

# Update formula version
sed -i 's/version ".*"/version "1.17.0"/' anna-assistant.rb

# Update SHA256 checksums
# Download release artifacts and compute:
sha256sum annad-1.17.0-x86_64-apple-darwin.tar.gz
sha256sum annad-1.17.0-aarch64-apple-darwin.tar.gz
sha256sum annad-1.17.0-x86_64-unknown-linux-gnu.tar.gz

# Update checksums in formula
# sha256 "NEW_CHECKSUM_HERE"

# Test installation (requires tap)
brew install --build-from-source anna-assistant.rb
brew test anna-assistant

# Submit to homebrew-core (if applicable)
# Or publish to custom tap
```

## AUR Package (`anna-assistant-bin`)

### Structure

```
packaging/aur/anna-assistant-bin/
├── PKGBUILD           # Main build script
├── .SRCINFO           # Metadata (auto-generated)
├── annad.service      # systemd service file
└── config.toml        # Default configuration
```

### PKGBUILD Anatomy

```bash
# Package metadata
pkgname=anna-assistant-bin
pkgver=1.16.3_alpha.1  # Note: Replace '.' with '_' in alpha versions
pkgrel=1
pkgdesc="..."
arch=('x86_64')
url="https://github.com/jjgarcianorway/anna-assistant"
license=('custom')

# Dependencies
depends=('systemd')
optdepends=('prometheus: metrics' 'grafana: dashboards')

# Conflict handling
provides=('anna-assistant')
conflicts=('anna-assistant')

# Config files to preserve on upgrade
backup=('etc/anna/config.toml' 'etc/anna/pinning.toml')

# Download sources
source=(
    "annad-${pkgver}-x86_64::https://github.com/..."
    "annactl-${pkgver}-x86_64::https://github.com/..."
    "SHA256SUMS::https://github.com/..."
    "annad.service"
    "config.toml"
)

# Checksums (generate with: makepkg -g)
sha256sums=('...' '...' ...)
```

### Hooks

```bash
post_install() {
    # Create anna group
    getent group anna >/dev/null || groupadd -r anna

    # Set permissions
    chown -R root:anna /var/lib/anna
    chmod -R 750 /var/lib/anna

    echo "Run: sudo usermod -aG anna \$USER && newgrp anna"
}

post_upgrade() {
    post_install
}

pre_remove() {
    systemctl stop annad.service
    systemctl disable annad.service
}
```

### Testing

```bash
# Validate PKGBUILD
namcap PKGBUILD

# Build package
makepkg -si

# Check installed files
pacman -Ql anna-assistant-bin

# Test functionality
annactl status
sudo systemctl status annad

# Uninstall
sudo pacman -R anna-assistant-bin
```

### Publishing to AUR

```bash
# Clone AUR repo (first time)
git clone ssh://aur@aur.archlinux.org/anna-assistant-bin.git

# Update files
cp packaging/aur/anna-assistant-bin/* anna-assistant-bin/

# Commit and push
cd anna-assistant-bin
git add PKGBUILD .SRCINFO annad.service config.toml
git commit -m "Update to v1.17.0"
git push
```

## Homebrew Formula (`anna-assistant`)

### Structure

```
packaging/homebrew/
└── anna-assistant.rb    # Formula definition
```

### Formula Anatomy

```ruby
class AnnaAssistant < Formula
  desc "..."
  homepage "https://github.com/jjgarcianorway/anna-assistant"
  version "1.16.3-alpha.1"
  license "Custom"

  # Platform-specific URLs
  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/.../annad-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    elsif Hardware::CPU.arm?
      url "https://github.com/.../annad-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    url "https://github.com/.../annad-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "..."
  end

  depends_on "openssl@3"

  def install
    bin.install "annad"
    bin.install "annactl"
    (etc/"anna").install "config.toml"
    # ...
  end

  service do
    run [opt_bin/"annad"]
    keep_alive true
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/annactl --version")
  end
end
```

### Testing

```bash
# Audit formula
brew audit --strict anna-assistant.rb

# Test installation
brew install --build-from-source anna-assistant.rb

# Test service
brew services start anna-assistant
annactl status
brew services stop anna-assistant

# Test upgrade
brew upgrade anna-assistant

# Uninstall
brew uninstall anna-assistant
brew cleanup
```

### Publishing

**Option 1: Custom Tap** (Recommended for alpha releases)

```bash
# Create tap repo
# https://github.com/jjgarcianorway/homebrew-anna

# Add formula
cp packaging/homebrew/anna-assistant.rb Formula/

# Users install with:
brew tap jjgarcianorway/anna
brew install anna-assistant
```

**Option 2: Homebrew Core** (For stable releases)

```bash
# Fork homebrew-core
git clone https://github.com/Homebrew/homebrew-core

# Add formula
cp anna-assistant.rb homebrew-core/Formula/

# Test
brew install --build-from-source homebrew-core/Formula/anna-assistant.rb

# Submit PR to Homebrew/homebrew-core
```

## Manual Installation

### Linux (Binary)

```bash
# Download latest release
VERSION="1.16.3-alpha.1"
wget "https://github.com/jjgarcianorway/anna-assistant/releases/download/v${VERSION}/annad-${VERSION}-x86_64-unknown-linux-gnu"
wget "https://github.com/jjgarcianorway/anna-assistant/releases/download/v${VERSION}/annactl-${VERSION}-x86_64-unknown-linux-gnu"
wget "https://github.com/jjgarcianorway/anna-assistant/releases/download/v${VERSION}/SHA256SUMS"

# Verify checksums
sha256sum -c SHA256SUMS

# Install binaries
sudo install -m 755 annad-${VERSION}-x86_64-unknown-linux-gnu /usr/local/bin/annad
sudo install -m 755 annactl-${VERSION}-x86_64-unknown-linux-gnu /usr/local/bin/annactl

# Create group and directories
sudo groupadd -r anna
sudo mkdir -p /var/lib/anna/{keys,chronos,collective,reports}
sudo chown -R root:anna /var/lib/anna
sudo chmod -R 750 /var/lib/anna

# Add user to group
sudo usermod -aG anna $USER
newgrp anna

# Create config
sudo mkdir -p /etc/anna
sudo tee /etc/anna/config.toml <<EOF
[daemon]
socket_path = "/run/anna/anna.sock"
metrics_port = 9090
log_level = "info"
EOF

# Install systemd service
sudo tee /etc/systemd/system/annad.service <<EOF
[Unit]
Description=Anna Assistant
After=network.target

[Service]
Type=simple
User=root
Group=anna
ExecStart=/usr/local/bin/annad
Restart=on-failure
RuntimeDirectory=anna
RuntimeDirectoryMode=0750
UMask=0007

[Install]
WantedBy=multi-user.target
EOF

# Start service
sudo systemctl daemon-reload
sudo systemctl enable --now annad
annactl status
```

### macOS (Binary)

```bash
# Similar to Linux but use darwin binaries
# Place in /usr/local/bin
# Use launchd instead of systemd
```

## Troubleshooting

### AUR Issues

**Problem**: `makepkg` fails with checksum mismatch

**Solution**:
```bash
# Regenerate checksums
updpkgsums
# Or manually: makepkg -g >> PKGBUILD
```

**Problem**: namcap warnings about missing dependencies

**Solution**: Update `depends=()` and `makedepends=()` in PKGBUILD

### Homebrew Issues

**Problem**: Formula audit fails

**Solution**:
```bash
# Check for common issues
brew audit --strict --online anna-assistant.rb

# Fix style issues
brew style --fix anna-assistant.rb
```

**Problem**: Service doesn't start

**Solution**:
```bash
# Check logs
brew services log anna-assistant

# Verify permissions
ls -la $(brew --prefix)/var/lib/anna
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Package Release

on:
  release:
    types: [published]

jobs:
  update-aur:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout AUR repo
        uses: actions/checkout@v4
        with:
          repository: jjgarcianorway/aur-anna-assistant-bin
          ssh-key: ${{ secrets.AUR_SSH_KEY }}

      - name: Update PKGBUILD version
        run: |
          VERSION="${{ github.event.release.tag_name }}"
          sed -i "s/pkgver=.*/pkgver=${VERSION#v}/" PKGBUILD
          updpkgsums
          makepkg --printsrcinfo > .SRCINFO

      - name: Commit and push
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add PKGBUILD .SRCINFO
          git commit -m "Update to ${{ github.event.release.tag_name }}"
          git push
```

## Version Scheme

- **Stable**: `1.0.0`, `1.1.0`, `1.2.0`
- **Alpha**: `1.16.3-alpha.1` (AUR: `1.16.3_alpha.1`)
- **Beta**: `1.17.0-beta.1` (AUR: `1.17.0_beta.1`)
- **RC**: `2.0.0-rc.1` (AUR: `2.0.0_rc.1`)

Note: AUR uses `_` instead of `.` in prerelease versions due to version comparison rules.

## Checklist

Before releasing a new version:

- [ ] Update `Cargo.toml` version
- [ ] Update `CHANGELOG.md`
- [ ] Run tests: `cargo test --workspace`
- [ ] Build release binaries
- [ ] Generate checksums
- [ ] Create GitHub release
- [ ] Update AUR PKGBUILD
- [ ] Test AUR package: `makepkg -si`
- [ ] Push to AUR repo
- [ ] Update Homebrew formula
- [ ] Test Homebrew formula
- [ ] Update documentation
- [ ] Announce release

## References

- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [Arch Packaging Standards](https://wiki.archlinux.org/title/Arch_package_guidelines)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [GitHub Releases API](https://docs.github.com/en/rest/releases)
- [Semantic Versioning](https://semver.org/)

---

**v1.0 - Phase 2.0.0-alpha.1**

For questions: https://github.com/jjgarcianorway/anna-assistant/issues
