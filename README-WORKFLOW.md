# Simple Release Workflow

## Quick Start

### 1. SSH Setup (One Time)

Add this to your `~/.bashrc`:

```bash
# Auto-start SSH agent and add key
if [ -z "$SSH_AUTH_SOCK" ]; then
    eval "$(ssh-agent -s)" > /dev/null
    ssh-add ~/.ssh/id_ed25519 2>/dev/null
fi
```

Then reload: `source ~/.bashrc`

Now you only enter your SSH password once per session.

### 2. Make a Release

That's it! Just:

```bash
./release
```

This will:
- Read version from `Cargo.toml`
- Read commit message from `.release-message`
- Clean up any conflicting tags
- Create the release
- Push to GitHub

### 3. Install

```bash
./scripts/install.sh
```

## How It Works

**Cargo.toml** = single source of truth for version

**`.release-message`** = commit message template

When you want to release:
1. Edit `Cargo.toml` version (e.g., `0.12.3` → `0.12.4`)
2. Update `.release-message` with what's new
3. Run `./release`

That's it. No parameters. No confusion.

## Example

```bash
# Edit version
vim Cargo.toml  # Change version = "0.12.4"

# Edit release message
vim .release-message

# Release
./release

# Install
./scripts/install.sh
```

Done.
