# Anna Assistant

**A Simple, Intelligent Linux System Assistant**

## Project Scope

Anna Assistant is designed to be a lightweight, privacy-focused system assistant for Linux that helps users maintain and optimize their systems through intelligent recommendations and safe automation.

### Core Principles

1. **Simplicity First** - Clean, minimal codebase with clear architecture
2. **Privacy & Security** - All processing happens locally, no external APIs or telemetry
3. **Safety by Design** - Actions require explicit user consent, with full rollback capability
4. **Intelligence** - Context-aware recommendations based on actual system state
5. **Beautiful UX** - Calm, elegant terminal interface that respects users' time

### What Anna Does

- **System Health Monitoring** - Continuously observes system state and identifies issues
- **Intelligent Recommendations** - Provides actionable advice based on official distribution documentation
- **Safe Automation** - Executes approved actions with full audit trail and rollback support
- **Learning & Adaptation** - Understands user patterns and tailors recommendations accordingly

### What Anna Does NOT Do

- ❌ Phone home or send telemetry data
- ❌ Make system changes without explicit permission
- ❌ Use AI/LLM services (everything runs locally)
- ❌ Require complex configuration or setup
- ❌ Clutter your system with unnecessary processes

### Target Platform

**Primary**: Arch Linux
**Future**: Debian, Ubuntu, Fedora

### Architecture (Planned)

```
┌─────────────┐
│   annactl   │  CLI client (user space)
└──────┬──────┘
       │ Unix Socket
       │
┌──────▼──────┐
│    annad    │  System daemon (privileged)
└─────────────┘
       │
       ├─ System Monitors
       ├─ Recommendation Engine
       ├─ Action Executor
       └─ State Manager
```

### Development Status

🚧 **Project Reset** - Starting fresh with clean design

Previous iterations taught us valuable lessons about what works and what doesn't. This version focuses on getting the fundamentals right:

1. ✅ Clear, simple architecture
2. ✅ Comprehensive test coverage from day one
3. ✅ Documentation-driven development
4. ✅ Security and safety as core requirements, not afterthoughts

### Contributing

This project is currently in early design phase. Contributions welcome once the initial architecture is established.

### License

MIT

---

**Built with Rust • Designed for Humans • Privacy First**
