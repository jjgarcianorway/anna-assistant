# Anna Assistant - Testing Guide

## Testing IPC Communication

The Unix socket IPC system allows `annactl` to communicate with the `annad` daemon.

### Prerequisites

1. Build the latest binaries:
   ```bash
   cargo build --release
   ```

2. Install the binaries (requires sudo):
   ```bash
   sudo cp target/release/annad /usr/local/bin/annad
   sudo cp target/release/annactl /usr/local/bin/annactl
   ```

### Test Scenarios

#### 1. Test without daemon running

```bash
annactl status
```

**Expected**: Error message saying "Daemon not running" with instructions to start it.

#### 2. Start the daemon manually

In terminal 1:
```bash
sudo annad
```

You should see:
- "Anna Daemon v1.0.0-alpha.3 starting"
- "System facts collected: N packages installed"
- "Generated M recommendations"
- "Anna Daemon ready"
- "RPC server listening on /run/anna/anna.sock"

#### 3. Test status command

In terminal 2:
```bash
annactl status
```

**Expected**:
- Real hostname from your system
- Real kernel version
- Daemon version (v1.0.0-alpha.3)
- Daemon uptime in seconds
- Number of pending recommendations

#### 4. Test advise command

```bash
annactl advise
```

**Expected**:
- Real recommendations based on your system
- Grouped by risk level (Critical, Maintenance, Suggestions)
- Actual package counts, microcode status, etc.

#### 5. Test config command

```bash
annactl config
```

**Expected**:
- Current autonomy tier (0 = Advise Only by default)
- Auto-update check status
- Wiki cache path

#### 6. Test report command

```bash
annactl report
```

**Expected**: Health report with system status

#### 7. Stop daemon with Ctrl+C

In terminal 1, press Ctrl+C

**Expected**: "Shutting down gracefully"

#### 8. Verify socket is cleaned up

```bash
ls /run/anna/anna.sock
```

**Expected**: File should not exist after daemon stops

### Verification Checklist

- [ ] Client shows error when daemon is not running
- [ ] Daemon creates socket at `/run/anna/anna.sock`
- [ ] `annactl status` shows real system information
- [ ] `annactl advise` shows actual recommendations
- [ ] Daemon logs show RPC server listening
- [ ] Socket is cleaned up when daemon stops
- [ ] Multiple clients can connect simultaneously

### Known Issues

- Socket requires `/run/anna/` directory (daemon creates it automatically)
- Daemon must run as root to create socket in `/run/`
- Version mismatch warnings may appear if binaries are out of sync

### Next Steps

Once IPC is verified working:
1. Set up systemd service for automatic daemon startup
2. Implement action executor for applying recommendations
3. Add Arch Wiki caching system
4. Expand recommendation rules
