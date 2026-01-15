# Anna Features

Shipped features only. Every item is verified working in v0.3.50.

## Query System
- Natural language questions about system state
- Evidence-backed answers (ClaimGate)
- Iterative investigation with multiple probe rounds
- Real-time streaming responses

## Action Execution (v0.3.50)
- Template plans for common operations
- User confirmation before execution
- pkexec privilege escalation
- Per-step and final verification
- Automatic rollback on failure
- Idempotency (skip if already configured)

### Template Plans
| Operation | Trigger Keywords |
|-----------|------------------|
| GDM Resolution | "gdm" + "resolution" |
| Disable Sleep | "disable/prevent/stop" + "sleep/suspend" |
| Lid Close | "lid" + "close/closing" |

## Specialist System
- 8 domains: Desktop, Network, Storage, Security, Services, Performance, Hardware, Packages
- Junior/Senior levels per domain (16 specialists)
- Deterministic routing based on keywords
- Escalation to seniors for complex issues

## Exposure Model
- Four levels: Silent, Summary, Dialogue, Debug
- Forbidden pattern sanitization
- Consent-based dialogue visibility

## Self-Healing
- Auto-recovery for daemon, Ollama, permissions, models, wiki
- No manual commands in error messages
- pkexec for automated privilege escalation

## Safe Operations
- Backups before modifications
- Undo capability
- Confirmation prompts (unless auto-confirm enabled)

## Local Privacy
- 100% local execution
- No cloud APIs
- No telemetry
- Ollama for LLM inference

## UX Regression Lock (v0.3.51)
- Canonical UX spec (docs/UX_SPEC.md)
- Snapshot tests for rendering
- Golden transcript fixtures
- Pattern-based contract validation
- CI-integrated regression gate

## System Integration
- Systemd service
- Auto-update from GitHub releases
- System paths only (/etc/anna, /var/lib/anna, /run/anna)
- Group-based permissions (anna group)
