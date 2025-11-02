# Installation Instructions for v0.12.6-pre

## Required Manual Steps (Requires sudo)

Due to the need for elevated privileges, the following commands must be run manually:

```bash
# 1. Install binaries to system
sudo cp target/release/annad /usr/local/bin/
sudo cp target/release/annactl /usr/local/bin/

# 2. Restart the daemon
sudo systemctl restart annad

# 3. Verify daemon is running
sudo systemctl status annad

# 4. Check daemon logs
sudo journalctl -u annad -n 50 --no-pager

# 5. Verify version
annactl --version  # Should show: annactl 0.12.6-pre

# 6. Test RPC connectivity
annactl status

# 7. Run health check
annactl doctor check --verbose

# 8. Test storage command with live daemon
annactl storage btrfs --json
```

## Post-Installation Validation

After installation, run these tests to verify everything is working:

```bash
# Run full test suite
cargo test --workspace

# Run Btrfs smoke test
bash tests/arch_btrfs_smoke.sh

# Test storage TUI
annactl storage btrfs

# Test storage JSON output
annactl storage btrfs --json | jq .
```

## Expected Results

- **Daemon version**: v0.12.6-pre (matches binary version)
- **RPC connectivity**: No timeout errors
- **All tests**: Should pass with daemon running
- **Storage command**: Should return valid JSON with Btrfs profile

## Troubleshooting

If RPC still times out after installation:

```bash
# Check daemon logs for errors
sudo journalctl -u annad -n 100 --no-pager

# Verify socket permissions
ls -la /run/anna/annad.sock

# Check daemon process
ps aux | grep annad

# Manually test daemon startup
sudo /usr/local/bin/annad
```

## Rollback (if needed)

If v0.12.6-pre has issues:

```bash
# Restore previous version (if backed up)
sudo systemctl stop annad
sudo cp /path/to/backup/annad /usr/local/bin/
sudo cp /path/to/backup/annactl /usr/local/bin/
sudo systemctl start annad
```
