# Anna Assistant v1.0 "Hildegard"

**Autonomous, Arch Linux-Native System Administrator**

```
╭─────────────────────────────────────────────╮
│      Intelligent • Safe • Beautiful         │
╰─────────────────────────────────────────────╯
```

Anna is a local-first, privacy-focused system assistant that understands your Arch Linux system, provides intelligent recommendations grounded in the Arch Wiki, and can safely execute approved actions with full audit trails and rollback capability.

## ✨ Features

- **Deep System Intelligence** - Hardware, software, and behavioral telemetry
- **Arch Wiki Integration** - All recommendations cite official documentation
- **Risk-Based Autonomy** - 4 tiers from advise-only to fully autonomous
- **Safe Execution** - Full audit logs and rollback tokens for every action
- **Beautiful CLI** - Elegant, pastel-colored terminal interface
- **Offline-First** - Works entirely offline with cached wiki data

## 🚀 Quick Start

### Installation (Alpha)

For now, build from source:

```bash
# Clone repository
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant

# Build release binaries
cargo build --release

# Install (optional)
sudo cp target/release/{annad,annactl} /usr/local/bin/
```

### Usage

```bash
# Check system status
annactl status

# Get recommendations
annactl advise

# Generate health report
annactl report

# Run diagnostics
annactl doctor

# View configuration
annactl config
```

## 🎯 Core Goals

1. **System Intelligence** - Understand hardware, software, and user behavior
2. **Wiki-Native Knowledge** - Use Arch Wiki as source of truth
3. **Autonomy × Risk** - Safe, policy-driven automation
4. **Offline-First** - No internet required for core functionality
5. **Beautiful UX** - Calm, elegant interface

## 🏗️ Architecture

```
crates/
├── annad/          # Daemon (privileged, root)
│   ├── telemetry   # System fact collection
│   ├── recommender # Rule-based advisor
│   └── executor    # Safe action execution
├── annactl/        # CLI client (user space)
│   └── commands    # User interface
└── anna_common/    # Shared types
    ├── types       # Data models
    └── beautiful   # Output formatting
```

## 🔒 Safety Model

- **Risk Levels**: Low, Medium, High
- **Autonomy Tiers**:
  - Tier 0: Advise only (default)
  - Tier 1: Auto-execute Low risk
  - Tier 2: Auto-execute Low + Medium
  - Tier 3: Fully autonomous
- **Audit Logging**: Every action logged to JSONL
- **Rollback Tokens**: Reversible operations

## 📊 Current Status

**Version**: 1.0.0-alpha.1
**Status**: Early Development

✅ Core data models
✅ System telemetry collection
✅ Recommendation engine (5 rules)
✅ Beautiful CLI interface
🚧 Unix socket IPC (planned)
🚧 Action executor (planned)
🚧 Arch Wiki caching (planned)

## 🤝 Contributing

This project is in early development. We welcome contributions once the core architecture stabilizes.

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines (coming soon).

## 📜 License

MIT - See [LICENSE](LICENSE)

## 🌐 Vision

Anna evolves from a diagnostic tool into an autonomous system administrator — a resident expert that:
- Understands your system better than you do
- Learns your habits and environment
- Provides intelligent, contextual guidance
- Keeps your machine secure, optimized, and reliable
- Quietly, intelligently, beautifully

---

**Built with Rust • Powered by Arch Wiki • Privacy First**
