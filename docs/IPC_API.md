# Anna Assistant IPC API Documentation

This document describes the Inter-Process Communication (IPC) protocol used between the Anna daemon (`annad`) and client (`annactl`).

## Overview

Anna uses a **Unix socket** for IPC with a **JSON-based RPC protocol**.

- **Transport**: Unix domain socket  
- **Protocol**: JSON-RPC-like (custom)  
- **Socket Path**: `/run/anna/anna.sock`  
- **Permissions**: `0666` (readable/writable by all users)  
- **Format**: Line-delimited JSON (`\n` terminated)

## Request Format

```json
{
  "id": 1,
  "method": {
    "type": "MethodName",
    "params": {...}
  }
}
```

## Response Format

```json
{
  "id": 1,
  "result": {
    "Ok": {
      "type": "ResponseType",
      "data": {...}
    }
  }
}
```

## Available Methods

See full documentation in crates/anna_common/src/ipc.rs

1. **Ping** - Health check
2. **Status** - Get daemon status  
3. **GetFacts** - Retrieve system facts
4. **GetAdvice** - Get all recommendations
5. **GetAdviceWithContext** - Get personalized recommendations
6. **ApplyAction** - Execute a recommendation
7. **Refresh** - Trigger manual scan (internal)
8. **GetConfig** - Retrieve configuration
9. **SetConfig** - Update configuration

For complete API details, see the source code.
