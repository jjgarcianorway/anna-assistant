//! Service query classification patterns (v0.0.174).
//!
//! Systemd services, docker, crontab, timers, sockets.

use crate::router::QueryClass;

/// Classify service queries.
/// Returns Some if matched, None otherwise.
pub fn classify_services(q: &str) -> Option<QueryClass> {
    // v0.0.791: Failed services - check first (most specific)
    // Queries like "failed services", "show me failed services", "broken services"
    if q.contains("failed service")
        || q.contains("broken service")
        || q.contains("services failed")
        || q.contains("services broken")
        || q.contains("what service")  && q.contains("fail")
        || q.contains("show") && q.contains("fail") && q.contains("service")
        || q.trim() == "failed"
        || q.trim() == "failures"
    {
        return Some(QueryClass::ServiceStatus);
    }

    // v0.0.307: Running services - check next (more specific than ServiceStatus)
    // Queries like "running services", "what services are running"
    if q.contains("running service")
        || q.contains("active service")
        || q.contains("started service")
        || q.contains("enabled service")
        || (q.contains("service") && q.contains("running"))
        || (q.contains("service") && q.contains("active"))
        || q.contains("list services")
        || q == "running services"
        || q == "services running"
        || q.contains("what services are running")
        || q.contains("services are running")
    {
        return Some(QueryClass::RunningServices);
    }

    // Service status: "is X running", "status of X" - specific service questions
    // v0.0.307: Exclude generic "running" to avoid matching "running services"
    if q.contains("service status")
        || q.contains("systemd")
        || (q.contains("status") && q.contains("service"))
        || (q.contains("is") && (q.contains("active") || q.contains("enabled")))
        || (q.contains("is") && q.contains("running") && !q.contains("services"))
    {
        return Some(QueryClass::ServiceStatus);
    }

    // v0.0.99: Manage service - "restart docker", "start sshd", "stop nginx"
    // v0.0.788: Don't match "enable/disable" when it's about editor config (syntax, highlighting, etc.)
    let is_editor_config = q.contains("syntax")
        || q.contains("highlight")
        || q.contains("line number")
        || q.contains("word wrap")
        || q.contains("auto indent")
        || q.contains("tab size")
        || q.contains("color scheme")
        || q.contains("theme")
        || q.contains("vim")
        || q.contains("nvim")
        || q.contains("nano")
        || q.contains("emacs");

    let service_verbs = [
        "start ", "stop ", "restart ", "reload ",
    ];
    // "enable " and "disable " handled separately to avoid editor config conflicts
    let enable_disable_verbs = ["enable ", "disable "];

    for verb in &service_verbs {
        if q.starts_with(verb) {
            return Some(QueryClass::ManageService);
        }
    }

    // Only match "enable/disable" as service management if NOT editor config
    if !is_editor_config {
        for verb in &enable_disable_verbs {
            if q.starts_with(verb) {
                return Some(QueryClass::ManageService);
            }
        }
    }
    // v0.0.788: Also exclude editor config from polite service requests
    if !is_editor_config
        && (q.contains("can you") || q.contains("please") || q.contains("could you"))
        && (q.contains("start ")
            || q.contains("stop ")
            || q.contains("restart ")
            || q.contains("enable ")
            || q.contains("disable "))
    {
        return Some(QueryClass::ManageService);
    }

    // v0.0.128: Systemd units
    if q.contains("systemd unit")
        || q.contains("list unit")
        || q.contains("all unit")
        || q.contains("enabled unit")
        || (q.contains("show") && q.contains("unit"))
    {
        return Some(QueryClass::SystemdUnits);
    }

    // v0.0.128: Crontabs
    if q.contains("crontab")
        || q.contains("cron job")
        || q.contains("scheduled task")
        || q.contains("scheduled job")
        || q.trim() == "cron"
        || (q.contains("show") && q.contains("cron"))
    {
        return Some(QueryClass::Crontabs);
    }

    // v0.0.129: Docker containers
    if q.contains("docker container")
        || q.contains("docker ps")
        || q.contains("running container")
        || (q.contains("container") && q.contains("running"))
        || (q.contains("list") && q.contains("container"))
    {
        return Some(QueryClass::DockerContainers);
    }

    // v0.0.129: Docker images
    if q.contains("docker image")
        || (q.contains("list") && q.contains("image") && !q.contains("disk"))
        || (q.contains("show") && q.contains("image") && q.contains("docker"))
    {
        return Some(QueryClass::DockerImages);
    }

    // v0.0.129: Systemd timers
    if q.contains("systemd timer")
        || q.contains("list timer")
        || q.contains("scheduled timer")
        || (q.contains("timer") && q.contains("systemd"))
    {
        return Some(QueryClass::SystemdTimers);
    }

    // v0.0.130: Systemd journal
    if q.trim() == "journalctl"
        || q.contains("system log")
        || q.contains("journal log")
        || q.contains("recent log")
        || (q.contains("show") && q.contains("log") && !q.contains("login"))
    {
        return Some(QueryClass::SystemdJournal);
    }

    // v0.0.131: Systemd slices
    if q.contains("systemd slice")
        || q.contains("cgroup slice")
        || q.contains("systemd-cgls")
        || q.trim() == "cgls"
        || (q.contains("cgroup") && q.contains("list"))
    {
        return Some(QueryClass::SystemdSlices);
    }

    // v0.0.132: Systemd targets
    if q.contains("systemd target")
        || q.contains("runlevel")
        || (q.contains("target") && q.contains("active"))
        || (q.contains("list") && q.contains("target"))
    {
        return Some(QueryClass::SystemdTargets);
    }

    // v0.0.133: Systemd sockets
    if q.contains("systemd socket")
        || q.contains("listening socket")
        || q.contains("list-sockets")
        || (q.contains("socket") && q.contains("unit"))
    {
        return Some(QueryClass::SystemdSockets);
    }

    // v0.0.135: Systemd paths
    if q.contains("systemd path")
        || q.contains("path unit")
        || (q.contains("list") && q.contains("path") && q.contains("systemd"))
    {
        return Some(QueryClass::SystemdPaths);
    }

    // v0.0.136: Systemctl masked units
    if q.contains("masked unit")
        || q.contains("masked service")
        || (q.contains("systemctl") && q.contains("mask"))
        || (q.contains("list") && q.contains("masked"))
        || (q.contains("show") && q.contains("masked"))
    {
        return Some(QueryClass::SystemctlMask);
    }

    // v0.0.139: Systemd scopes
    if q.contains("systemd scope")
        || q.contains("scope unit")
        || (q.contains("list") && q.contains("scope"))
        || (q.contains("show") && q.contains("scope"))
    {
        return Some(QueryClass::SystemdScopes);
    }

    // v0.0.141: Systemd mounts
    if q.contains("systemd mount")
        || q.contains("mount unit")
        || (q.contains("list") && q.contains("mount") && q.contains("systemd"))
        || (q.contains("automount") && q.contains("unit"))
    {
        return Some(QueryClass::SystemdMounts);
    }

    None
}
