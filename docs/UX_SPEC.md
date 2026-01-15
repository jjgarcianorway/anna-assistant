# Anna UX Contract

Canonical formatting rules. Every output MUST match this spec.

## Prefix Rules

### Dialogue Prefix
All Anna speech uses `Anna:` prefix with color based on context:
- GREEN: Final answers, successful completions
- YELLOW: Warnings, confirmations, timeouts, missing info
- CYAN: Investigation start, understanding checks
- RED: Errors
- MAGENTA: Experiments

### Status Indicators
- `[OK]` (GREEN): Success
- `[!]` (YELLOW): Warning
- `[X]` (RED): Error

### Internal Comms Format
```
[0.0s] Speaker -> Recipient: message
```
Where:
- Timestamp: seconds with one decimal
- Speaker: specialist name
- Recipient: optional, omit arrow if missing
- Message: dialogue content

## Spinner Behavior

- Spinners may be silent (empty message) or display brief status
- Braille animation: `["]`, rotating
- Cleared on next output

## Message Templates

### Timeout
```
Anna: Request took longer than Xs. Try again shortly.
```

### Stream Interruption
```
Unable to complete request. Connection interrupted before completion.
```
Or if no content received:
```
Unable to complete request. No response received.
```

### Confirmation Request
```
Anna: [plan content]

```
No additional "Please confirm:" wrapper.

### Missing Information
```
Anna: [content]
```
Color: YELLOW. No "Missing information:" prefix.

### System Alert
```
Anna: [content]
```
Color: YELLOW. No "SYSTEM ALERT" header.

### LLM Error
```
Anna: [error message]
```
Or if no message available:
```
Anna: Unable to process request.
```

### Investigation
Start:
```
Anna: Investigating: [topic]
```
Complete:
```
Anna: [conclusion]
```

### Experiment
Start:
```
Anna: Trying: [description]
```
Result:
```
  Result: [outcome]
```

### Cancellation
```
No problem, I won't make any changes.
```

### Plan Confirmation Prompt
```
Proceed? (yes/no)
```

## Exposure Levels

### Silent
- No streaming output
- Final answer only

### Summary
- Brief progress indicators
- Final answer with evidence

### Dialogue
- All dialogue steps visible
- Team communications
- Ticket timeline

### Debug
- All dialogue + internal steps
- LLM prompts/responses
- Command execution details
- Raw probe output

## Color Semantics

| Color | ANSI | Meaning |
|-------|------|---------|
| GREEN | 32 | Success, final answer |
| YELLOW | 33 | Warning, confirmation |
| RED | 31 | Error |
| CYAN | 36 | Info, investigation |
| MAGENTA | 35 | Experiments, specialists |
| DIM | 2 | Debug info, timestamps |
| WHITE | 37;1 | Emphasis |

## Determinism Requirements

Output must be reproducible:
- No timestamps in user-facing output (only in dialogue timeline)
- No random IDs in user-facing output
- No terminal width dependence
- Fixed color codes (not truecolor negotiation)

## Do Not Regress

- [ ] `Anna:` prefix for all dialogue, never `ANSWER:` or `[FAILED]`
- [ ] No "Please confirm:" wrapper on confirmation requests
- [ ] No "Missing information:" wrapper on missing info
- [ ] No "SYSTEM ALERT" header on alerts
- [ ] No verbose timeout explanations
- [ ] Consistent `[OK]/[!]/[X]` indicators
- [ ] Silent spinners allowed (empty message)
- [ ] Exposure levels respected
