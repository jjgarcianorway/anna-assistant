# Contributing to Anna

Thank you for your interest in contributing to Anna! This document provides guidelines and standards for development.

---

## Design Philosophy

Anna embodies three core principles that guide all development:

1. **Beauty** — Calm, elegant, consistent visual output
2. **Intelligence** — Deep system awareness and contextual recommendations
3. **Safety** — Autonomous action with complete rollback and audit trails

Every contribution should reflect these principles.

---

## Development Setup

### Prerequisites

- **Rust** 1.70+ (stable)
- **Arch Linux** (primary target, other distros for testing)
- **systemd** (for daemon integration)
- **polkit** (for privilege management)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/anna-assistant.git
cd anna-assistant

# Build debug binary
cargo build

# Build release binary
cargo build --release

# Run tests
cargo test

# Check for errors without building
cargo check
```

### Project Structure

```
anna-assistant/
├── src/
│   ├── annactl/          # CLI binary
│   │   ├── main.rs       # Command routing
│   │   ├── *_cmd.rs      # Command implementations
│   │   └── ...
│   ├── annad/            # Daemon binary
│   │   ├── main.rs       # RPC server
│   │   ├── advisor_v13.rs # Advisor engine
│   │   └── ...
│   └── anna_common/      # Shared library
│       ├── beautiful.rs  # Beautiful output library
│       └── ...
├── scripts/
│   ├── install.sh        # System installer
│   └── ...
├── docs/
│   ├── ARCHITECTURE.md   # Architecture documentation
│   └── ...
└── tests/
    └── ...
```

---

## Coding Standards

### Rust Style

Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Maximum line length: 100 characters
- Use 4 spaces for indentation (not tabs)

### Beautiful Output

**ALWAYS** use the `anna_common::beautiful` library for user-facing output:

```rust
use anna_common::{header, section, status, Level, TermCaps};

let caps = TermCaps::detect();
println!("{}", header(&caps, "My Command"));
println!("{}", section(&caps, "My Section"));
println!("{}", status(&caps, Level::Ok, "Success message"));
```

**NEVER** use raw ANSI codes like `\x1b[32m` in user-facing commands.

### Error Handling

Use `anyhow` for error handling:

```rust
use anyhow::{Context, Result};

fn my_function() -> Result<()> {
    let value = read_file()
        .context("Failed to read configuration file")?;

    Ok(())
}
```

### Async Code

Use `tokio` for async operations:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Async code here
    Ok(())
}
```

---

## Command Development

### Adding a New Command

1. **Create command module** (`src/annactl/src/my_cmd.rs`):

```rust
//! My command for Anna vX.Y
//!
//! Brief description of what this command does

use anyhow::Result;
use anna_common::{header, section, status, Level, TermCaps};

pub fn run_my_command() -> Result<()> {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "My Command"));
    println!();

    // Command implementation

    println!("{}", status(&caps, Level::Ok, "Command completed"));
    Ok(())
}
```

2. **Add module to `main.rs`**:

```rust
mod my_cmd;
```

3. **Add command to enum**:

```rust
#[derive(Subcommand)]
enum Commands {
    // ...

    /// Brief description for --help
    MyCommand {
        /// Flag description
        #[arg(long)]
        my_flag: bool,
    },
}
```

4. **Add command handler**:

```rust
match cli.command {
    // ...
    Commands::MyCommand { my_flag } => {
        my_cmd::run_my_command()?;
        Ok(())
    }
}
```

### Experimental Commands

Commands under development should be gated:

```rust
/// [EXPERIMENTAL] My experimental command
#[command(hide = true)]
MyExperimentalCommand {
    // ...
}

// In handler:
Commands::MyExperimentalCommand { ... } => {
    require_experimental!("my-experimental-command");
    // Implementation
}
```

Enable with: `ANNA_EXPERIMENTAL=1 annactl my-experimental-command`

---

## Advisor Rules

### Adding New Advisor Rules

Advisor rules go in `src/annad/src/advisor_v13.rs`:

```rust
/// Rule N: Brief description
fn check_my_rule(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
    let mut result = Vec::new();

    // Condition check
    if should_recommend() {
        result.push(Advice {
            id: "my-rule-id".to_string(),
            level: Level::Warn,
            category: "system".to_string(),
            title: "Brief title".to_string(),
            reason: "Why this is recommended".to_string(),
            action: "What the user should do".to_string(),
            explain: Some("Detailed explanation with context".to_string()),
            fix_cmd: Some("sudo pacman -S package".to_string()),
            fix_risk: Some("Low - only installs package, safe".to_string()),
            refs: vec!["https://wiki.archlinux.org/title/Topic".to_string()],
        });
    }

    result
}
```

**Requirements**:
- Every rule MUST have an Arch Wiki citation in `refs`
- Every rule MUST specify `fix_risk` (Low, Medium, High)
- `fix_cmd` should be a valid shell command
- `explain` should provide context and rationale

### Risk Levels

- **Low** — Safe for auto-apply (installs packages, config tweaks)
- **Medium** — Requires confirmation (removes packages, changes settings)
- **High** — Manual only (bootloader, kernel, critical system files)

---

## Safety Guidelines

### Autonomous Actions

When implementing autonomous actions (`apply` command):

1. **Filter by risk** — Only low-risk actions may auto-apply
2. **Create rollback tokens** — Every action MUST generate a rollback token
3. **Log to audit trail** — Record actor, action, result, details
4. **Capture state** — Store before/after snapshots when possible
5. **Use `sh -c`** — Execute commands safely without shell injection

### Rollback Support

Every autonomous action should support rollback:

```rust
// In apply_cmd.rs
let token = RollbackToken {
    advice_id: advice.id.clone(),
    executed_at: now,
    command: cmd.clone(),
    success,
    output: output_str,
    state_snapshot: Some(capture_state()?),
};

save_rollback_token(&token)?;
```

---

## Testing

### Manual Testing

```bash
# Test core commands
./target/release/annactl --help
./target/release/annactl version
./target/release/annactl status
./target/release/annactl doctor check
./target/release/annactl advisor
./target/release/annactl report

# Test apply/rollback
./target/release/annactl apply --dry-run
./target/release/annactl rollback --list

# Test experimental guard
./target/release/annactl radar                    # Should fail
ANNA_EXPERIMENTAL=1 ./target/release/annactl radar # Should work
```

### Automated Tests

Run the test suite:

```bash
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run full validation suite
./tests/validate_v0126.sh

# Run smoke tests
./tests/smoke_v101.sh
```

---

## Git Workflow

### Branching

- `main` — Stable releases only
- `develop` — Integration branch
- `feature/<name>` — New features
- `fix/<name>` — Bug fixes

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation
- `style:` — Formatting
- `refactor:` — Code restructuring
- `test:` — Tests
- `chore:` — Maintenance

**Example**:
```
feat(apply): add rollback command with state snapshots

Implement annactl rollback with three modes:
- --last: undo most recent action
- --id: undo specific action
- --list: show rollback history

Each rollback operation:
- Reads rollback tokens from disk
- Determines rollback strategy
- Executes rollback command
- Logs to audit trail
- Removes token on success

Closes #42
```

### Pull Requests

1. **Fork** the repository
2. **Create** feature branch
3. **Commit** changes with conventional commits
4. **Test** thoroughly (manual + automated)
5. **Push** to your fork
6. **Open** pull request with:
   - Clear description
   - Test results
   - Screenshots (if UI changes)
   - Breaking changes (if any)

---

## Release Process

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH[-PRERELEASE]`
- Example: `1.2.0-beta`

**Increment**:
- `MAJOR` — Breaking changes
- `MINOR` — New features (backwards compatible)
- `PATCH` — Bug fixes
- `PRERELEASE` — `-alpha`, `-beta`, `-rc.1`

### Creating a Release

1. **Update version** in `Cargo.toml`:
   ```toml
   [package]
   version = "1.2.0"
   ```

2. **Update `CHANGELOG.md`**:
   ```markdown
   ## [1.2.0] - 2024-01-15

   ### Added
   - Rollback command with three modes
   - State snapshot capture

   ### Fixed
   - Bug in advisor rule detection
   ```

3. **Build and test**:
   ```bash
   cargo build --release
   cargo test
   ./tests/smoke_v101.sh
   ```

4. **Create tag**:
   ```bash
   git tag -a v1.2.0 -m "Release v1.2.0 - Rollback & Foresight"
   git push origin v1.2.0
   ```

5. **Build packages** (if applicable):
   ```bash
   ./scripts/build-packages.sh
   ```

---

## Documentation

### Code Comments

Use rustdoc format:

```rust
/// Brief one-line description.
///
/// More detailed explanation with examples:
///
/// # Examples
///
/// ```
/// let result = my_function()?;
/// assert!(result.is_some());
/// ```
///
/// # Errors
///
/// Returns error if file cannot be read.
pub fn my_function() -> Result<Option<String>> {
    // Implementation
}
```

### Module Documentation

Every module (`mod.rs` or `file.rs`) should have a header:

```rust
//! Brief module description
//!
//! Longer explanation of what this module does,
//! its purpose, and how it fits into the system.
```

### Architecture Documentation

Major architectural changes should be documented in `docs/ARCHITECTURE.md`.

---

## Performance

### Benchmarking

Use `criterion` for benchmarks:

```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn benchmark_function(c: &mut Criterion) {
        c.bench_function("my function", |b| {
            b.iter(|| my_function(black_box(input)))
        });
    }

    criterion_group!(benches, benchmark_function);
    criterion_main!(benches);
}
```

### Optimization Guidelines

- Use `cargo build --release` for production builds
- Profile with `perf` or `cargo flamegraph`
- Minimize allocations in hot paths
- Use `Arc` for shared data, not `Mutex` unless needed
- Prefer `&str` over `String` for function parameters

---

## Security

### Vulnerability Reporting

**DO NOT** open public issues for security vulnerabilities.

Instead, email: security@anna-assistant.local (replace with actual email)

### Security Guidelines

1. **No secrets in code** — Use environment variables or config files
2. **Validate all input** — Never trust user input
3. **Use parameterized commands** — No string interpolation in shell commands
4. **Principle of least privilege** — Request minimum permissions needed
5. **Audit all `unsafe` blocks** — Must have safety comment explaining why

---

## Community

### Code of Conduct

Be respectful, inclusive, and constructive. See `CODE_OF_CONDUCT.md` for details.

### Getting Help

- **GitHub Issues** — Bug reports and feature requests
- **GitHub Discussions** — Questions and general discussion
- **Wiki** — Tutorials and guides

---

## License

By contributing to Anna, you agree that your contributions will be licensed under the same license as the project (see `LICENSE` file).

---

**Thank you for contributing to Anna!** 🌸

Every contribution helps make Anna more beautiful, intelligent, and safe.
