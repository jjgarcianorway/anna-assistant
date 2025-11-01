# Simple Workflow - How It Should Work

## Current Problem

The running daemon is **v0.13.6 (old)** from October 31.
The client (annactl) is trying to be **v0.12.2 (new)** but can't talk to the old daemon.

## Quick Fix Right Now

Run this one command:

```bash
./FIX-NOW.sh
```

This will:
1. Stop the old daemon
2. Update `bin/` with new v0.12.2 binaries
3. Run install.sh to install v0.12.2
4. Start the new daemon

Then test:
```bash
annactl status
annactl collect --limit 1
annactl classify
annactl radar show
```

---

## The PROPER Workflow (After This Fix)

### What I Do (Claude):

1. **Implement features completely**
   - Write all code
   - Build binaries
   - Test locally
   - Update `bin/` directory
   - Write tests and docs

2. **Give you ONE release command**
   ```bash
   ./scripts/release.sh -t patch -m "clear message"
   ```

### What You Do:

1. **Release** (creates GitHub release):
   ```bash
   ./scripts/release.sh -t patch -m "the message I gave you"
   ```

2. **Install** (downloads from GitHub):
   ```bash
   ./scripts/install.sh
   ```

3. **Test**:
   ```bash
   annactl status
   annactl <new-commands>
   ```

4. **Done!** ✓

---

## Why It Failed This Time

1. I built v0.12.2 in `target/release/` but didn't update `bin/`
2. `install.sh` uses `bin/` not `target/release/`
3. Old binaries (v0.13.6) got installed
4. Version mismatch → daemon doesn't respond

## The Fix

Run `./FIX-NOW.sh` and we're back on track!

---

## Next Time Will Be Clean

I promise:
- I'll update `bin/` before giving you the release command
- You run `release.sh` → `install.sh` → test
- It just works ✓
