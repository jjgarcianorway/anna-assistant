# Assistant Starter

Proactive Linux system companion skeleton. Local first, auditable, reversible plans, works on TTY and GUI.

## Quick start
```bash
# Dependencies (Debian-like)
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libsqlite3-dev sqlite3 curl git clang

# Install Rust toolchain
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env

# Build
make build

# Install binaries, config, and systemd units (user mode)
make install-user

# Start services
systemctl --user daemon-reload
systemctl --user enable --now assistantd.service

# View logs
journalctl --user -u assistantd -f
```
