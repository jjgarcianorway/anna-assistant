# Simple Workflow - Just 2 Commands

## To Build and Install

```bash
# 1. Build (handles git, versioning, everything)
./scripts/release.sh

# 2. Install
sudo ./scripts/install.sh
```

That's it. No git commands, no extra scripts.

## What Each Script Does

### `release.sh`
- Reads VERSION file
- Builds binaries
- Auto-commits changes
- Creates git tag
- Pushes to GitHub (triggers CI to build release)

### `install.sh`
- Uses local binaries from `target/release/`
- Stops daemon
- Installs binaries
- Sets up systemd service
- Starts daemon

## That's All You Need

Run `release.sh` then `install.sh`. Everything else is automatic.
