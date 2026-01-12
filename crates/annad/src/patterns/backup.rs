//! Backup patterns for rsync, borg, restic, tar.
//! v0.0.968: Initial implementation.
//! v0.0.989: Expanded patterns for better coverage.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a backup-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type BackupPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match backup-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_rsync(q)
        .or_else(|| match_borg(q))
        .or_else(|| match_restic(q))
        .or_else(|| match_tar(q))
        .or_else(|| match_general_backup(q))
}

/// Rsync patterns
fn match_rsync(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BackupPattern] = &[
        // Rsync installed
        (&["rsync", "version"], "show rsync version", "backup",
         &["rsync --version | head -1"]),
        (&["rsync", "installed"], "check if rsync is installed", "backup",
         &["which rsync && rsync --version | head -1"]),
        // Rsync syntax
        (&["rsync", "syntax"], "show rsync syntax", "backup",
         &["echo 'rsync [options] source destination'; echo 'Common: -av (archive+verbose), -z (compress), --delete (mirror), --dry-run (test)'"]),
        (&["rsync", "options"], "show rsync options", "backup",
         &["rsync --help | head -40"]),
        // Rsync progress
        (&["rsync", "progress"], "show rsync in progress", "backup",
         &["ps aux | grep rsync | grep -v grep"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Borg backup patterns
fn match_borg(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BackupPattern] = &[
        // Borg version
        (&["borg", "version"], "show borg version", "backup",
         &["borg --version 2>/dev/null || echo 'borg not installed'"]),
        (&["borg", "installed"], "check if borg is installed", "backup",
         &["which borg && borg --version"]),
        // Borg backups (general)
        (&["borg", "backup"], "show borg backup info", "backup",
         &["borg --version", "echo 'List: borg list REPO'", "echo 'Create: borg create REPO::NAME /path'"]),
        // Borg repos
        (&["borg", "repos"], "list borg repositories", "backup",
         &["echo 'Borg repos are typically at ~/.borg or /path/to/backup'", "ls -la ~/.borg 2>/dev/null"]),
        (&["borg", "list"], "list borg archives", "backup",
         &["echo 'Use: borg list /path/to/repo'"]),
        // Borg info
        (&["borg", "info"], "show borg repository info", "backup",
         &["echo 'Use: borg info /path/to/repo'"]),
        (&["borg", "archives"], "list borg archives", "backup",
         &["echo 'Use: borg list /path/to/repo'"]),
        // Borg mount
        (&["borg", "mount"], "how to mount borg archive", "backup",
         &["echo 'Use: borg mount /path/to/repo::archive /mount/point'"]),
        // Borg check
        (&["borg", "check"], "check borg repository", "backup",
         &["echo 'Use: borg check /path/to/repo'"]),
        // Borg compact
        (&["borg", "compact"], "compact borg repository", "backup",
         &["echo 'Use: borg compact /path/to/repo'"]),
        // Borg restore
        (&["borg", "restore"], "how to restore borg backup", "backup",
         &["echo 'Use: borg extract /path/to/repo::archive'", "echo 'Or mount: borg mount /path/to/repo::archive /mnt'"]),
        // Borg prune
        (&["borg", "prune"], "manage borg retention", "backup",
         &["echo 'Use: borg prune --keep-daily 7 --keep-weekly 4 /path/to/repo'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Restic patterns
fn match_restic(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BackupPattern] = &[
        // Restic version
        (&["restic", "version"], "show restic version", "backup",
         &["restic version 2>/dev/null || echo 'restic not installed'"]),
        (&["restic", "installed"], "check if restic is installed", "backup",
         &["which restic && restic version"]),
        // Restic snapshots
        (&["restic", "snapshots"], "list restic snapshots", "backup",
         &["echo 'Use: restic -r /path/to/repo snapshots'"]),
        (&["restic", "list"], "list restic snapshots", "backup",
         &["echo 'Use: restic -r /path/to/repo snapshots'"]),
        // Restic info
        (&["restic", "stats"], "show restic stats", "backup",
         &["echo 'Use: restic -r /path/to/repo stats'"]),
        // Restic check
        (&["restic", "check"], "check restic repository", "backup",
         &["echo 'Use: restic -r /path/to/repo check'"]),
        // Restic mount
        (&["restic", "mount"], "how to mount restic snapshot", "backup",
         &["echo 'Use: restic -r /path/to/repo mount /mount/point'"]),
        // Restic forget
        (&["restic", "forget"], "manage restic retention", "backup",
         &["echo 'Use: restic -r /path/to/repo forget --keep-last N'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Tar patterns
fn match_tar(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BackupPattern] = &[
        // Tar archives (general)
        (&["tar", "archive"], "tar archive info", "backup",
         &["echo 'Create: tar -cvf archive.tar files'", "echo 'Extract: tar -xvf archive.tar'", "echo 'List: tar -tvf archive.tar'"]),
        // Tar syntax
        (&["tar", "syntax"], "show tar syntax", "backup",
         &["echo 'Create: tar -cvf archive.tar files'; echo 'Extract: tar -xvf archive.tar'; echo 'Compressed: tar -czvf archive.tar.gz files'"]),
        (&["tar", "options"], "show tar options", "backup",
         &["tar --help | head -30"]),
        // Tar list
        (&["tar", "list"], "list tar archive contents", "backup",
         &["echo 'Use: tar -tvf archive.tar'"]),
        (&["tar", "contents"], "show tar archive contents", "backup",
         &["echo 'Use: tar -tvf archive.tar'"]),
        // Tar extract
        (&["tar", "extract"], "how to extract tar archive", "backup",
         &["echo 'tar -xvf archive.tar (regular)'; echo 'tar -xzvf archive.tar.gz (gzip)'; echo 'tar -xjvf archive.tar.bz2 (bzip2)'; echo 'tar -xJvf archive.tar.xz (xz)'"]),
        // Tar create
        (&["tar", "create"], "how to create tar archive", "backup",
         &["echo 'tar -cvf archive.tar files/'", "echo 'tar -czvf archive.tar.gz files/ (gzip)'", "echo 'tar -cJvf archive.tar.xz files/ (xz)'"]),
        // Tar compress
        (&["tar", "compress"], "tar compression options", "backup",
         &["echo '-z = gzip (.tar.gz)'; echo '-j = bzip2 (.tar.bz2)'; echo '-J = xz (.tar.xz)'; echo '--zstd = zstd (.tar.zst)'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// General backup patterns
fn match_general_backup(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BackupPattern] = &[
        // Backup tools installed
        (&["backup", "tools"], "list installed backup tools", "backup",
         &["which rsync borg restic duplicity timeshift 2>/dev/null | xargs -I{} basename {}"]),
        (&["backup", "software"], "show backup software", "backup",
         &["pacman -Qs 'backup\\|rsync\\|borg\\|restic\\|timeshift' 2>/dev/null | head -20"]),
        // Backup schedule
        (&["backup", "schedule"], "show backup schedules", "backup",
         &["systemctl list-timers | grep -iE 'backup|borg|restic|rsync'", "crontab -l 2>/dev/null | grep -iE 'backup|borg|restic|rsync'"]),
        (&["scheduled", "backup"], "show scheduled backups", "backup",
         &["systemctl list-timers | grep -i backup", "crontab -l 2>/dev/null | grep -i backup"]),
        // Incremental backup
        (&["incremental", "backup"], "incremental backup info", "backup",
         &["echo 'rsync: rsync -av --link-dest=PREV src/ dst/'", "echo 'borg: borg create (always incremental)'", "echo 'restic: restic backup (always incremental)'"]),
        // Restore backup
        (&["restore", "backup"], "how to restore backup", "backup",
         &["echo 'rsync: rsync -av backup/ destination/'", "echo 'borg: borg extract REPO::ARCHIVE'", "echo 'restic: restic restore SNAPSHOT --target /'"]),
        (&["backup", "restore"], "how to restore from backup", "backup",
         &["echo 'rsync: rsync -av backup/ destination/'", "echo 'borg: borg extract REPO::ARCHIVE'", "echo 'restic: restic restore SNAPSHOT --target /'"]),
        // Backup verification
        (&["backup", "verification"], "verify backup integrity", "backup",
         &["echo 'borg: borg check REPO'", "echo 'restic: restic check'", "echo 'rsync: rsync -avnc src/ dst/ (dry-run checksum)'"]),
        (&["verify", "backup"], "verify backup", "backup",
         &["echo 'borg: borg check REPO'", "echo 'restic: restic check'", "echo 'Compare: diff -r original/ backup/'"]),
        // Timeshift
        (&["timeshift", "status"], "show timeshift status", "backup",
         &["timeshift --list 2>/dev/null || echo 'timeshift not installed'"]),
        (&["timeshift", "snapshots"], "list timeshift snapshots", "backup",
         &["timeshift --list 2>/dev/null"]),
        // Snapper (btrfs)
        (&["snapper", "list"], "list snapper snapshots", "backup",
         &["snapper list 2>/dev/null || echo 'snapper not installed'"]),
        (&["snapper", "snapshots"], "show snapper snapshots", "backup",
         &["snapper list 2>/dev/null"]),
        // Backup running
        (&["backup", "running"], "check if backup is running", "backup",
         &["ps aux | grep -E 'rsync|borg|restic|duplicity|timeshift' | grep -v grep"]),
        (&["backup", "progress"], "show backup progress", "backup",
         &["ps aux | grep -E 'rsync|borg|restic|duplicity' | grep -v grep"]),
        // Backup size
        (&["backup", "size"], "check backup size", "backup",
         &["du -sh /path/to/backup 2>/dev/null", "echo 'borg: borg info REPO'", "echo 'restic: restic stats'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsync() {
        assert!(match_patterns("rsync version").is_some());
        assert!(match_patterns("rsync syntax").is_some());
        assert!(match_patterns("rsync options").is_some());
    }

    #[test]
    fn test_borg() {
        assert!(match_patterns("borg version").is_some());
        assert!(match_patterns("borg list").is_some());
        assert!(match_patterns("borg info").is_some());
    }

    #[test]
    fn test_restic() {
        assert!(match_patterns("restic version").is_some());
        assert!(match_patterns("restic snapshots").is_some());
        assert!(match_patterns("restic check").is_some());
    }

    #[test]
    fn test_tar() {
        assert!(match_patterns("tar syntax").is_some());
        assert!(match_patterns("tar list").is_some());
        assert!(match_patterns("tar extract").is_some());
    }

    #[test]
    fn test_general_backup() {
        assert!(match_patterns("backup tools").is_some());
        assert!(match_patterns("timeshift status").is_some());
        assert!(match_patterns("backup running").is_some());
    }
}
