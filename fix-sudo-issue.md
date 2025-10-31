# Sudo Authentication Failing - Troubleshooting

## Symptoms:
- `sudo` asks for password but rejects it
- `su` with root password works fine
- Can change password with `passwd lhoqvso` as root
- New password still doesn't work with `sudo`

## Most Likely Cause: Stale Session or PAM Cache

### Solution 1: Full Re-login (Most Likely to Work)
```bash
# Exit your current session completely
exit  # or logout

# Log back in fresh
# Then try:
sudo ls
```

### Solution 2: Check if you need to reload groups
```bash
# Your current groups (in this shell):
groups

# Your actual groups (in /etc/group):
id -nG

# If different, you need to re-login or:
newgrp wheel
sudo ls  # Try again
```

### Solution 3: Verify as Root
```bash
# As root (via su):
su -

# Check user is in wheel group:
groups lhoqvso

# Verify sudoers allows wheel:
grep wheel /etc/sudoers

# Should see something like:
# %wheel ALL=(ALL:ALL) ALL
```

### Solution 4: Test PAM
```bash
# As root, check PAM sudo config:
cat /etc/pam.d/sudo

# Should include:
# auth include system-auth
# account include system-auth
# session include system-auth
```

### Solution 5: Clear authentication cache (if using sssd/systemd)
```bash
# As root:
systemctl restart systemd-logind
# Then log out and back in
```

## Workaround: Run Anna Fix Without Sudo

Since sudo isn't working right now, here's how to fix Anna manually as root:

```bash
# Switch to root
su -

# Navigate to project
cd /home/lhoqvso/anna-assistant

# Install the missing file
install -m 0644 etc/CAPABILITIES.toml /usr/lib/anna/

# Restart daemon
systemctl restart annad

# Check status
systemctl status annad

# Exit root
exit

# Now test (as your user):
annactl status
```

## What I (Claude) Did NOT Do:

I did NOT:
- Modify /etc/sudoers
- Run any usermod/groupmod commands
- Change any system permissions
- Run any sudo commands (I can't - they all fail for me too)

I ONLY:
- Modified files in /home/lhoqvso/anna-assistant/
- Added scripts/fix-capabilities.sh
- Updated scripts/install.sh
- Removed old test files
- Created documentation

All changes are in git - you can verify:
```bash
git diff HEAD~1
git log -1 --stat
```

## Recommended Steps:

1. **Log out and log back in completely** (most likely fix)
2. If that doesn't work, as root run:
   ```bash
   visudo  # Check %wheel line is uncommented
   ```
3. Test sudo with a fresh session

This is NOT related to the Anna installation - it's a system authentication issue.
