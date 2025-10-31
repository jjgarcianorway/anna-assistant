# Anna Assistant - AUR Packages

This directory contains PKGBUILD files for submitting Anna Assistant to the Arch User Repository (AUR).

## Packages

### 1. `anna-assistant` (Source Build)
- **File:** `PKGBUILD`
- **Description:** Builds Anna from source using Cargo
- **Requirements:** `rust`, `cargo`
- **Install time:** 2-5 minutes (compilation)
- **Best for:** Users who want to build from source

### 2. `anna-assistant-bin` (Binary Release)
- **File:** `PKGBUILD-bin`
- **Description:** Installs pre-compiled binaries from GitHub releases
- **Requirements:** None (no Rust needed)
- **Install time:** ~30 seconds
- **Best for:** Quick installation without compilation

## How to Submit to AUR

### First Time Setup

1. **Create AUR account:** https://aur.archlinux.org/register
2. **Set up SSH keys:**
   ```bash
   ssh-keygen -t ed25519 -C "your_email@example.com"
   # Add public key to AUR account settings
   ```

### Submit Binary Package

```bash
# Clone AUR repository (first time)
git clone ssh://aur@aur.archlinux.org/anna-assistant-bin.git
cd anna-assistant-bin

# Copy files
cp ../PKGBUILD-bin PKGBUILD
cp ../anna-assistant.install .

# Update checksums in PKGBUILD
# Get SHA256 from: https://github.com/jjgarcianorway/anna-assistant/releases
# Edit PKGBUILD and replace 'SKIP' with actual checksums

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD anna-assistant.install .SRCINFO
git commit -m "Initial import: anna-assistant-bin 0.11.0-1"
git push
```

### Submit Source Package

```bash
# Clone AUR repository (first time)
git clone ssh://aur@aur.archlinux.org/anna-assistant.git
cd anna-assistant

# Copy files
cp ../PKGBUILD .
cp ../anna-assistant.install .

# Update checksums
# Get SHA256 from: https://github.com/jjgarcianorway/anna-assistant/releases
# Edit PKGBUILD and replace 'SKIP' with actual checksums

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD anna-assistant.install .SRCINFO
git commit -m "Initial import: anna-assistant 0.11.0-1"
git push
```

## Testing Locally

Before submitting, test the packages locally:

```bash
# Test binary package
cd /tmp
cp /path/to/anna-assistant/packaging/aur/PKGBUILD-bin PKGBUILD
cp /path/to/anna-assistant/packaging/aur/anna-assistant.install .
makepkg -si

# Test source package
cd /tmp
cp /path/to/anna-assistant/packaging/aur/PKGBUILD .
cp /path/to/anna-assistant/packaging/aur/anna-assistant.install .
makepkg -si
```

## Updating Packages

When releasing a new version:

1. Update `pkgver` in both PKGBUILD files
2. Reset `pkgrel` to 1
3. Update checksums from new GitHub release
4. Regenerate .SRCINFO: `makepkg --printsrcinfo > .SRCINFO`
5. Commit and push changes

## Post-Installation

After installing via AUR, users should:

```bash
# Add user to anna group
sudo usermod -aG anna $USER
# Log out and back in

# Enable and start daemon
sudo systemctl enable --now annad

# Verify installation
annactl doctor check
annactl status
```

## Notes

- The `.install` file handles post-installation tasks (user/group creation, permissions)
- Both packages create the same system user/group (`anna`)
- Configuration is preserved during upgrades (see `backup=()` in PKGBUILD)
- systemd services are installed but not enabled by default

## See Also

- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [PKGBUILD Guidelines](https://wiki.archlinux.org/title/PKGBUILD)
- [.SRCINFO Format](https://wiki.archlinux.org/title/.SRCINFO)
