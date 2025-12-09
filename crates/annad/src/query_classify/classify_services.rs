//! Service query classification patterns (v0.0.174).
//!
//! Systemd services, docker, crontab, timers, sockets.

use crate::router::QueryClass;

/// Classify service queries.
/// Returns Some if matched, None otherwise.
pub fn classify_services(q: &str) -> Option<QueryClass> {
    // Service status: "is X running", "status of X"
    if q.contains("running")
        || q.contains("service status")
        || q.contains("systemd")
        || (q.contains("status") && q.contains("service"))
        || (q.contains("is") && (q.contains("active") || q.contains("enabled")))
    {
        return Some(QueryClass::ServiceStatus);
    }

    // v0.0.99: Manage service - "restart docker", "start sshd", "stop nginx"
    let service_verbs = [
        "start ", "stop ", "restart ", "enable ", "disable ", "reload ",
    ];
    for verb in &service_verbs {
        if q.starts_with(verb) {
            return Some(QueryClass::ManageService);
        }
    }
    if (q.contains("can you") || q.contains("please") || q.contains("could you"))
        && (q.contains("start ")
            || q.contains("stop ")
            || q.contains("restart ")
            || q.contains("enable ")
            || q.contains("disable "))
    {
        return Some(QueryClass::ManageService);
    }

    // v0.0.125: Running services
    if q.contains("running service")
        || q.contains("active service")
        || q.contains("started service")
        || q.contains("enabled service")
        || (q.contains("service") && q.contains("running"))
        || (q.contains("service") && q.contains("active"))
        || q.contains("list services")
    {
        return Some(QueryClass::RunningServices);
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
