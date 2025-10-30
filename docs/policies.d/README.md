# Anna Policy Files

This directory contains example policy files for Anna's Policy Engine (Sprint 3).

## Policy Syntax

Policies are written in YAML format with the following structure:

```yaml
- when: <condition>
  then: <action>
  enabled: <boolean>
```

### Condition Syntax

Conditions follow the pattern: `field operator value`

**Supported Operators:**
- `>` - Greater than
- `<` - Less than
- `>=` - Greater or equal
- `<=` - Less or equal
- `==` - Equal
- `!=` - Not equal

**Value Types:**
- Numbers: `100`, `3.14`
- Percentages: `5%`, `0.5%`
- Strings: `"value"`
- Booleans: `true`, `false`

### Actions

**Built-in Actions:**
- `disable_autonomy` - Disable autonomous operations
- `enable_autonomy` - Enable autonomous operations
- `run_doctor` - Execute system diagnostics
- `restart_service` - Restart the Anna daemon
- `send_alert` - Send alert notification
- `custom: <command>` - Execute custom command

## Installation

1. Copy policy files to `/etc/anna/policies.d/`
2. Reload policies: `annactl policy reload`
3. View loaded policies: `annactl policy list`

## Examples

See the example files in this directory:
- `example-telemetry.yaml` - Telemetry-based reactions
- `example-system.yaml` - System health monitoring

## Testing Policies

Evaluate policies against test context:

```bash
annactl policy eval --context '{"telemetry.error_rate": 0.06}'
```

## Policy Best Practices

1. **Start Conservative** - Begin with policies set to `enabled: false` and monitor
2. **Test Thoroughly** - Use `annactl policy eval` to test before enabling
3. **Document Intent** - Add comments explaining why each policy exists
4. **Monitor Results** - Check event logs after policy reactions
5. **Iterate** - Adjust thresholds based on learning cache data
