# Contributing to Anna Assistant

Thank you for your interest in contributing to Anna! We're excited to have you here. 🎉

Anna is designed to be accessible, helpful, and beautiful. Whether you're fixing bugs, adding features, or improving documentation, your contributions make Anna better for everyone.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Adding Detection Rules](#adding-detection-rules)
- [Writing Good Messages](#writing-good-messages)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)

## 🤝 Code of Conduct

Be kind, be respectful, be helpful. We're all here to make Anna better.

- Use welcoming and inclusive language
- Respect different viewpoints and experiences
- Accept constructive criticism gracefully
- Focus on what's best for Anna and the community

## 💡 How Can I Contribute?

### Reporting Bugs

Found a bug? Let us know!

1. Check if it's already reported in [Issues](https://github.com/jjgarcianorway/anna-assistant/issues)
2. If not, create a new issue with:
   - Clear title describing the problem
   - Steps to reproduce
   - Expected vs actual behavior
   - Anna version (`annactl --version`)
   - System info (Arch Linux version, kernel, DE, etc.)

### Suggesting Features

Have an idea for Anna?

1. Check existing issues to avoid duplicates
2. Create a new issue explaining:
   - What problem does it solve?
   - How would it work?
   - Why is it useful for Anna users?
   - Any Arch Wiki references related to it

### Improving Documentation

Documentation is always appreciated!

- Fix typos and grammar
- Improve explanations
- Add examples
- Update outdated information
- Translate to other languages (future)

### Adding Detection Rules

This is the most common contribution! See [Adding Detection Rules](#adding-detection-rules) below.

## 🛠️ Development Setup

### Prerequisites

- Arch Linux (or Arch-based distro)
- Rust toolchain (1.70+)
- Basic knowledge of Rust
- Git

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant

# Build all crates
cargo build

# Run tests
cargo test

# Build release version
cargo build --release
```

### Testing Locally

```bash
# Stop the system daemon if running
sudo systemctl stop annad

# Run daemon in foreground for testing
sudo ./target/debug/annad

# In another terminal, test the CLI
./target/debug/annactl status
./target/debug/annactl advise
```

## 📁 Project Structure

```
anna-assistant/
├── crates/
│   ├── annad/              # Daemon (privileged, runs as root)
│   │   ├── src/
│   │   │   ├── main.rs     # Entry point
│   │   │   ├── telemetry.rs         # System fact collection
│   │   │   ├── recommender.rs       # Detection rules
│   │   │   ├── intelligent_recommender.rs  # Behavior-based rules
│   │   │   ├── rpc_server.rs        # IPC server
│   │   │   ├── executor.rs          # Action execution
│   │   │   ├── audit.rs             # Audit logging
│   │   │   ├── watcher.rs           # Filesystem monitoring
│   │   │   └── notifier.rs          # Notifications
│   │   └── Cargo.toml
│   ├── annactl/            # CLI client (user-facing)
│   │   ├── src/
│   │   │   ├── main.rs     # CLI argument parsing
│   │   │   ├── commands.rs # Command implementations
│   │   │   └── rpc_client.rs        # IPC client
│   │   └── Cargo.toml
│   └── anna_common/        # Shared types and utilities
│       ├── src/
│       │   ├── lib.rs      # Module exports
│       │   ├── types.rs    # Core data structures
│       │   ├── ipc.rs      # IPC protocol
│       │   └── beautiful.rs # Terminal formatting
│       └── Cargo.toml
├── scripts/
│   ├── install.sh          # Installation script
│   └── release.sh          # Release automation
├── annad.service           # Systemd service file
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md         # This file
└── Cargo.toml              # Workspace root
```

## 🎯 Adding Detection Rules

Detection rules are the heart of Anna! Here's how to add one.

### 1. Decide Where It Goes

- **`recommender.rs`** - System-wide checks (microcode, firewall, packages)
- **`intelligent_recommender.rs`** - Behavior-based (dev tools, CLI tools, media players)

### 2. Write the Detection Function

Example: Detecting missing Bluetooth support

```rust
// In crates/annad/src/intelligent_recommender.rs

fn recommend_bluetooth_support() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if Bluetooth hardware exists
    let has_bluetooth_hw = Command::new("lsusb")
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.contains("Bluetooth")
        })
        .unwrap_or(false);

    // Only suggest if hardware exists but software is missing
    if has_bluetooth_hw && !package_installed("bluez") {
        result.push(Advice {
            id: "bluetooth-stack".to_string(),
            title: "Install Bluetooth support".to_string(),
            reason: "You have Bluetooth hardware but the BlueZ stack isn't installed. BlueZ provides the Bluetooth protocol stack and bluez-utils gives you bluetoothctl for pairing devices.".to_string(),
            action: "Install Bluetooth stack and utilities".to_string(),
            command: Some("pacman -S --noconfirm bluez bluez-utils && systemctl enable bluetooth".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Bluetooth".to_string()],
        });
    }

    result
}
```

### 3. Call Your Function

Add it to the main recommendation generator:

```rust
// In generate_intelligent_advice()
advice.extend(recommend_bluetooth_support());
```

### 4. Test It

```bash
# Build and run
cargo build
sudo ./target/debug/annad &

# Check if your advice appears
./target/debug/annactl advise
```

### Detection Rule Guidelines

✅ **DO:**
- Check for actual hardware/usage before suggesting
- Explain WHY something is useful
- Provide Arch Wiki links
- Use appropriate risk levels
- Make commands idempotent when possible

❌ **DON'T:**
- Suggest things users don't need
- Use technical jargon without explanation
- Assume everyone needs your favorite tool
- Create destructive recommendations

## 💬 Writing Good Messages

Anna speaks **plain English**, not Linux-ese!

### ❌ Bad Example
```
"AMD CPU detected without microcode updates"
```

### ✅ Good Example
```
"Your AMD processor needs microcode updates to protect against security vulnerabilities like Spectre and Meltdown. Think of it like a security patch for your CPU itself."
```

### Message Writing Tips

1. **Explain the "why"** - Don't just say what's missing, explain why it matters
2. **Use analogies** - Help beginners understand: "Think of it like..."
3. **Be conversational** - "Your system" not "The system"
4. **Avoid jargon** - Or explain it when you must use it
5. **Be encouraging** - "This will make your system faster!" not "You're missing this"

## 🧪 Testing

### Run Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p anna_common
cargo test -p annad
cargo test -p annactl

# Specific test
cargo test test_header_formatting
```

### Manual Testing Checklist

Before submitting a PR, test:

- [ ] `annactl status` - Shows daemon status
- [ ] `annactl advise` - Lists recommendations
- [ ] `annactl report` - Generates plain English report
- [ ] `annactl apply --nums 1 --dry-run` - Shows what would happen
- [ ] `annactl apply --nums 1` - Actually applies (test carefully!)
- [ ] Daemon restarts cleanly: `sudo systemctl restart annad`
- [ ] Boxes render correctly (no misalignment)
- [ ] New advice appears when expected
- [ ] Advice disappears when conditions are met

## 🔄 Pull Request Process

1. **Fork the repository**

2. **Create a feature branch**
   ```bash
   git checkout -b feature/bluetooth-detection
   ```

3. **Make your changes**
   - Write clear, focused commits
   - Follow Rust style guidelines
   - Add tests if applicable
   - Update documentation

4. **Test thoroughly**
   - Run `cargo test`
   - Test manually with daemon
   - Check for warnings: `cargo clippy`

5. **Update CHANGELOG.md**
   - Add your changes under `[Unreleased]`
   - Use the existing format

6. **Push and create PR**
   ```bash
   git push origin feature/bluetooth-detection
   ```
   - Clear title describing the change
   - Explain WHAT you changed and WHY
   - Reference any related issues

7. **Respond to feedback**
   - Be open to suggestions
   - Make requested changes
   - Ask questions if unclear

## 🎨 Style Guidelines

### Rust Code

- Follow standard Rust conventions (rustfmt)
- Use meaningful variable names
- Add comments for complex logic
- Keep functions focused and small
- Use `?` for error propagation

```rust
// Good
fn check_bluetooth_hardware() -> bool {
    Command::new("lsusb")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Bluetooth"))
        .unwrap_or(false)
}

// Not ideal - too many responsibilities
fn check_and_recommend_bluetooth_with_firmware_and_gui() -> Vec<Advice> {
    // Too much in one function!
}
```

### Commit Messages

```
✅ Good:
"Add Bluetooth detection with hardware checking"
"Fix box rendering alignment issues"
"Update README with batch apply examples"

❌ Bad:
"fix stuff"
"WIP"
"changes"
```

### Documentation

- Keep README.md current with new features
- Update CHANGELOG.md for every change
- Add inline comments for complex code
- Update Arch Wiki references

## 🏆 Recognition

Contributors will be:
- Listed in release notes
- Mentioned in the README (for significant contributions)
- Thanked in commit messages

## ❓ Questions?

- Open an issue for questions
- Tag it with `question` label
- We're friendly and helpful!

## 📚 Useful Resources

- [Arch Wiki](https://wiki.archlinux.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/)
- [Anna's Roadmap](ROADMAP.md)

---

**Thank you for contributing to Anna!** 🌟

Every contribution, no matter how small, makes Anna better for the Arch Linux community.
