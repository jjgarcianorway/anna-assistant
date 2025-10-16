use anyhow::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};


fn now_id() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("harden_ssh_{t}")
}

#[allow(dead_code)]
pub struct PlanPaths {
    pub dir: PathBuf,
    pub apply: PathBuf,
    pub rollback: PathBuf,
    pub readme: PathBuf,
}

/// Create a reversible “harden SSH” plan under `root/plans/<id>/`.
pub fn suggest_harden_ssh(root: &Path) -> Result<PlanPaths> {
    let id = now_id();
    let dir = root.join(&id);
    fs::create_dir_all(&dir)?;

    let apply = dir.join("apply.sh");
    let rollback = dir.join("rollback.sh");
    let readme = dir.join("README.txt");

    let apply_sh = r#"#!/usr/bin/env bash
set -euo pipefail
CONF="/etc/ssh/sshd_config"
BACKUP="/etc/ssh/sshd_config.anna.bak.$(date +%s)"
cp -a "$CONF" "$BACKUP"
# Enforce key-only auth (idempotent edits)
sed -i -E \
  -e 's/^\s*#?\s*PasswordAuthentication\s+.*/PasswordAuthentication no/' \
  -e 's/^\s*#?\s*ChallengeResponseAuthentication\s+.*/ChallengeResponseAuthentication no/' \
  "$CONF"
sshd -t
systemctl reload sshd || systemctl reload ssh || true
echo "$BACKUP" > /var/lib/anna/last_ssh_backup
echo "Applied. Backup at $BACKUP"
"#;

    let rollback_sh = r#"#!/usr/bin/env bash
set -euo pipefail
if [ -f /var/lib/anna/last_ssh_backup ]; then
  BK="$(cat /var/lib/anna/last_ssh_backup)"
  if [ -f "$BK" ]; then
    cp -a "$BK" /etc/ssh/sshd_config
    sshd -t
    systemctl reload sshd || systemctl reload ssh || true
    echo "Rolled back to $BK"
  else
    echo "Recorded backup not found: $BK" >&2
    exit 1
  fi
else
  echo "No recorded backup to rollback" >&2
  exit 1
fi
"#;

    let readme_txt = r#"Plan: Harden SSH (disable password auth; rely on keys)
Why: Repeated failed SSH logins detected in the last 10 minutes.
Safety:
  - Backs up /etc/ssh/sshd_config with a timestamp
  - Validates with `sshd -t` before reload
Files:
  - apply.sh    : applies the change and reloads sshd
  - rollback.sh : restores last backup and reloads sshd
Apply:    sudo bash apply.sh
Rollback: sudo bash rollback.sh
"#;

    fs::write(&apply, apply_sh)?;
    fs::write(&rollback, rollback_sh)?;
    fs::write(&readme, readme_txt)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&apply, fs::Permissions::from_mode(0o755))?;
        fs::set_permissions(&rollback, fs::Permissions::from_mode(0o755))?;
    }
    Ok(PlanPaths { dir, apply, rollback, readme })
}
