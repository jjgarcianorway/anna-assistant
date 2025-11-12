# AppArmor profile for Anna Assistant Daemon
# Last modified: 2025-11-12
# Installation: sudo cp apparmor.anna.profile /etc/apparmor.d/usr.bin.annad
#               sudo apparmor_parser -r /etc/apparmor.d/usr.bin.annad

#include <tunables/global>

/usr/bin/annad {
  #include <abstractions/base>
  #include <abstractions/nameservice>

  # Binaries
  /usr/bin/annad mr,

  # State directory - read/write/create
  /var/lib/anna/ rw,
  /var/lib/anna/** rwk,

  # Log directory - read/write/create
  /var/log/anna/ rw,
  /var/log/anna/** rwk,

  # Config directory - read only
  /etc/anna/ r,
  /etc/anna/** r,

  # Temporary files
  /tmp/ rw,
  /tmp/** rw,

  # System information (read-only)
  /proc/meminfo r,
  /proc/cpuinfo r,
  /proc/loadavg r,
  /proc/uptime r,
  /proc/sys/kernel/hostname r,
  /sys/devices/system/cpu/** r,

  # Package database (read-only for advice generation)
  /var/lib/pacman/** r,
  /usr/share/doc/** r,

  # Network access for future distributed consensus (Phase 1.7)
  network inet stream,
  network inet6 stream,
  network unix stream,

  # Unix socket for RPC
  /run/anna/ rw,
  /run/anna/** rw,
  owner /run/anna/annad.sock rw,

  # Deny dangerous operations
  deny /etc/shadow r,
  deny /etc/gshadow r,
  deny /root/** rw,
  deny /home/** rw,
  deny @{PROC}/[0-9]*/mem rw,
  deny /sys/kernel/security/** rw,

  # Capabilities
  capability dac_override,
  capability dac_read_search,
  capability setgid,
  capability setuid,

  # Signal rules
  signal (send) set=(term, kill) peer=unconfined,
  signal (receive) set=(term, kill) peer=unconfined,
}

# ADVISORY MODE ENFORCEMENT:
# This profile DOES NOT prevent advisory mode from being disabled via config,
# but the daemon code itself enforces advisory-only mode at multiple layers:
#   1. No auto-apply code paths in RPC handlers (rpc_server.rs)
#   2. CLI warnings on all adjustment outputs (mirror_commands.rs)
#   3. Security model documented in CHANGELOG.md
#
# Citation: [archwiki:AppArmor]
