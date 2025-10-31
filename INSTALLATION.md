# Anna Assistant - Installation Guide

Multiple installation methods are available to suit your needs and system setup.

## Quick Start

**Recommended:** Use the smart installer that automatically downloads pre-compiled binaries:

```bash
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
./scripts/install.sh
```

The installer will:
1. Check for existing binaries in `./bin/`
2. Download pre-compiled binaries from GitHub releases (if needed)
3. Fall back to building from source (if downloads fail)

**No Rust installation required** for binary downloads!

---

## Installation Methods

### Method 1: Smart Installer (Recommended)

The smart installer tries multiple approaches automatically:

```bash
./scripts/install.sh
```

**Installation flow:**
```
1. Check for binaries in ./bin/        ← Manual/offline install
    ↓ (not found)
2. Download from GitHub releases        ← Fastest (no build needed)
    ↓ (download fails)
3. Build from source                    ← Requires Rust/Cargo
```

**Supported architectures:**
- x86_64 (Intel/AMD 64-bit)
- aarch64 (ARM 64-bit)

**Requirements for binary downloads:**
- `curl` or `wget` (for downloading)
- Internet connection

**Requirements for source builds:**
- Rust/Cargo (`sudo pacman -S rust` or via rustup)
- 2-5 minutes build time

---

### Method 2: AUR Package (Arch Linux Only)

#### Option A: Binary Package (Fast)

```bash
yay -S anna-assistant-bin
# or
paru -S anna-assistant-bin
```

**Advantages:**
- Fastest installation (~30 seconds)
- No Rust/Cargo required
- Automatic updates via AUR helper
- Native Arch package management

#### Option B: Source Package (Latest)

```bash
yay -S anna-assistant
# or
paru -S anna-assistant
```

**Advantages:**
- Builds from source (latest code)
- Native Arch package management
- Automatic updates via AUR helper

**Post-installation (both options):**

```bash
# Add your user to anna group
sudo usermod -aG anna $USER
# Log out and back in

# Enable and start daemon
sudo systemctl enable --now annad

# Verify installation
annactl doctor check
annactl status
```

---

### Method 3: Manual Binary Installation

For offline installation or air-gapped systems:

1. **Download pre-compiled binaries** from GitHub releases:
   - https://github.com/jjgarcianorway/anna-assistant/releases

2. **Choose your architecture:**
   - `anna-linux-x86_64.tar.gz` (Intel/AMD 64-bit)
   - `anna-linux-aarch64.tar.gz` (ARM 64-bit)

3. **Extract binaries:**
   ```bash
   mkdir -p anna-assistant/bin
   cd anna-assistant
   tar -xzf /path/to/anna-linux-*.tar.gz -C bin/
   ```

4. **Run installer:**
   ```bash
   ./scripts/install.sh
   ```

The installer will detect and use the binaries in `./bin/`.

---

### Method 4: Build from Source

For development or unsupported architectures:

1. **Install Rust:**
   ```bash
   # System-wide (Arch Linux)
   sudo pacman -S rust

   # Or user-level (all distros)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. **Clone and build:**
   ```bash
   git clone https://github.com/jjgarcianorway/anna-assistant.git
   cd anna-assistant
   ./scripts/install.sh --build
   ```

**The `--build` flag forces source compilation** (skips binary downloads).

---

## Installation Options

### Installer Flags

```bash
./scripts/install.sh [OPTIONS]
```

**Options:**
- `--build`, `--source` - Force building from source (skip downloads)
- `--help`, `-h` - Show help message

### Examples

```bash
# Let installer choose best method
./scripts/install.sh

# Force source build (even if binaries available)
./scripts/install.sh --build

# Show help
./scripts/install.sh --help
```

---

## Verification

After installation, verify that Anna is working correctly:

```bash
# Check daemon status
systemctl status annad

# Run health check
annactl doctor check

# View system status
annactl status

# Check version
annactl --version
```

---

## What Gets Installed

### Binaries
- `/usr/local/bin/annad` - System daemon (root-privileged)
- `/usr/local/bin/annactl` - User CLI client

### System User/Group
- User: `anna` (system user, no login)
- Group: `anna`

### Directories
- `/etc/anna/` - Configuration and policies
- `/var/lib/anna/` - Database and persistent state
- `/var/log/anna/` - Log files
- `/usr/lib/anna/` - Capability registry

### Systemd Services
- `annad.service` - Main daemon
- `anna-fans.service` - Thermal management (ASUS laptops)

### Permissions
Your user is added to the `anna` group for socket access.
**You must log out and back in** for group membership to take effect.

---

## Troubleshooting

### Issue: "annad: No such file or directory"

**Cause:** Binary download or build failed, but installer continued.

**Solution:**
```bash
# Check if Rust is installed
cargo --version

# If not installed:
sudo pacman -S rust

# Then reinstall:
./scripts/install.sh --build
```

### Issue: "curl: command not found"

**Cause:** Neither curl nor wget is installed.

**Solution:**
```bash
sudo pacman -S curl
./scripts/install.sh
```

### Issue: "Permission denied" when running annactl

**Cause:** Your user isn't in the `anna` group yet.

**Solution:**
```bash
# Verify group membership
groups | grep anna

# If not present:
sudo usermod -aG anna $USER
# Log out and back in
```

### Issue: Download fails with 404

**Cause:** Pre-compiled binaries not yet released for this version.

**Solution:**
```bash
# Build from source instead
./scripts/install.sh --build
```

### Issue: Architecture not supported

**Cause:** You're on an unsupported architecture (not x86_64 or aarch64).

**Solution:**
```bash
# Build from source (works on any architecture Rust supports)
sudo pacman -S rust
./scripts/install.sh --build
```

---

## Uninstallation

To remove Anna:

```bash
# Stop and disable services
sudo systemctl stop annad
sudo systemctl disable annad

# Remove binaries
sudo rm /usr/local/bin/annad /usr/local/bin/annactl

# Remove systemd services
sudo rm /etc/systemd/system/annad.service
sudo systemctl daemon-reload

# Optional: Remove data and configuration
sudo rm -rf /etc/anna /var/lib/anna /var/log/anna

# Optional: Remove user and group
sudo userdel anna
sudo groupdel anna
```

Or use the uninstall script (if available):

```bash
./scripts/uninstall.sh
```

---

## Next Steps

After successful installation:

1. **Run system diagnostics:**
   ```bash
   annactl doctor check
   ```

2. **Explore configuration:**
   ```bash
   annactl config list
   ```

3. **View available commands:**
   ```bash
   annactl --help
   ```

4. **Read the documentation:**
   - [README.md](README.md) - Overview and features
   - [docs/GETTING-STARTED.md](docs/GETTING-STARTED.md) - Quickstart guide
   - [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) - Common issues

---

## Contributing

If you encounter installation issues:

1. Check existing issues: https://github.com/jjgarcianorway/anna-assistant/issues
2. Report bugs with:
   - Your architecture (`uname -m`)
   - Installation method attempted
   - Full installer output
   - System info (`uname -a`)

---

## Distribution-Specific Notes

### Arch Linux
- **Recommended:** Use AUR packages (`anna-assistant-bin`)
- Pacman integration for updates
- Native systemd support

### Other Distributions
- Use the smart installer (works on any systemd-based distro)
- Pre-compiled binaries are statically linked (minimal dependencies)
- May need to adapt systemd service paths

---

## Security Notes

- Downloaded binaries are verified with SHA256 checksums
- All binaries are checked for ELF format before installation
- Installer creates a system user with no login shell
- Daemon runs with minimal privileges (only what systemd grants)
- All privileged operations require explicit user approval

---

**Installation taking too long?**

Binary downloads typically complete in 10-30 seconds. Source builds take 2-5 minutes on first run (subsequent builds are cached).

If downloads are slow, you can:
1. Download binaries manually from GitHub releases
2. Place them in `./bin/` directory
3. Run installer (will use local binaries)

This enables offline/air-gapped installation.
