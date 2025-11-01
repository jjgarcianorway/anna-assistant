# Clean Install Instructions for v0.12.2

## The Problem

You were added to the `anna` group, but you need to **log out and log back in** (or start a new shell session) for the group membership to take effect.

## Quick Fix

Run this in a **new terminal**:

```bash
# Start a new shell with the anna group active
newgrp anna

# Now test
annactl status
```

Or simply **log out and log back in**, then:

```bash
annactl status  # Should work now
```

---

## Clean Release Process

Here's the proper workflow going forward:

### Step 1: I Build & Test

I'll:
1. Implement features completely
2. Build and test locally
3. Update bin/ directory with working binaries
4. Give you ONE release.sh command

### Step 2: You Release

```bash
./scripts/release.sh -t patch -m "version message"
```

This creates the GitHub release with binaries.

### Step 3: You Install

```bash
./scripts/install.sh
```

This downloads from GitHub and installs.

### Step 4: You Test

```bash
annactl status
annactl <new commands>
```

Done!

---

## Current Status

**The daemon is running fine.** The issue is just that your current terminal session doesn't have the `anna` group active yet.

**Solution:** Either:
- Open a new terminal, OR
- Run: `newgrp anna`, OR
- Log out and back in

Then `annactl status` will work.

---

## Next Steps for v0.12.2

Once you can run `annactl status` successfully in a new session:

1. I'll give you the proper release.sh command for v0.12.2
2. You run it
3. GitHub builds and releases
4. You can install from GitHub
5. Everything works

Sound good?
