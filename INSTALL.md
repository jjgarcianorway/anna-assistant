# Anna Assistant Installation

## Quick Install

```bash
# 1. Build
cargo build --release

# 2. Install (will ask for password)
sudo install -m 755 target/release/annad /usr/local/bin/
sudo install -m 755 target/release/annactl /usr/local/bin/

# 3. Verify
annactl --help
```

That's it. The binaries are now in your PATH.

## What the fancy installer was trying to do

The `scripts/install.sh` attempts to:
- Create system directories
- Install systemd service
- Set up policies
- Configure permissions

But it's currently broken. Use the manual steps above for now.

## Uninstall

```bash
sudo rm /usr/local/bin/annad /usr/local/bin/annactl
```

## Sorry

The installer was overcomplicated and broken. These manual steps work.
