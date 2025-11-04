# Socket Ownership Fix - Installation Instructions

## The Problem
The daemon is running but the socket has wrong group ownership:
- Current: `root:root` with `0770` permissions
- Needed: `root:anna` with `0770` permissions
- Result: Users in `anna` group can't connect

## The Fix Applied
Modified `src/annad/src/rpc_v10.rs` to set socket group to `anna` after creation.
The binary is already built in `target/release/annad`.

## Installation Commands

Run these three commands:

```bash
# 1. Install the new daemon binary
sudo install -m 755 target/release/annad /usr/local/bin/

# 2. Restart the daemon service
sudo systemctl restart annad

# 3. Verify it works
annactl status
```

## Verification

After running the commands, you should see:
1. Socket ownership: `ls -la /run/anna/annad.sock` shows `root:anna`
2. Connection works: `annactl status` shows full status (not "RPC not responding")
3. No RPC errors when running any annactl command

## Quick One-Liner

```bash
sudo install -m 755 target/release/annad /usr/local/bin/ && sudo systemctl restart annad && sleep 2 && annactl status
```
