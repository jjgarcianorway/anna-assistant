Name:           anna-daemon
Version:        1.6.0
Release:        0.1.rc1%{?dist}
Summary:        Anna Assistant - Ethical AI system maintenance daemon

License:        MIT
URL:            https://github.com/yourusername/anna-assistant
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

Requires:       systemd
Requires:       logrotate

%description
Anna is an autonomous system assistant that provides ethical, wiki-grounded
advice for Arch Linux maintenance tasks. Features include temporal forecasting
with bias detection, mirror audit and self-reflection, and advisory-only
parameter adjustments with conscience sovereignty enforcement.

%prep
%setup -q

%build
cargo build --release --bins

%install
rm -rf $RPM_BUILD_ROOT

# Install binaries
install -D -m 0755 target/release/annad %{buildroot}%{_bindir}/annad
install -D -m 0755 target/release/annactl %{buildroot}%{_bindir}/annactl

# Install systemd service
install -D -m 0644 systemd/anna-daemon.service %{buildroot}%{_unitdir}/anna-daemon.service

# Install logrotate config
install -D -m 0644 logrotate/anna %{buildroot}%{_sysconfdir}/logrotate.d/anna

# Install documentation
install -D -m 0644 docs/PRODUCTION_DEPLOYMENT.md %{buildroot}%{_docdir}/%{name}/PRODUCTION_DEPLOYMENT.md
install -D -m 0644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -D -m 0644 CHANGELOG.md %{buildroot}%{_docdir}/%{name}/CHANGELOG.md

# Install SELinux policy (if available)
%if 0%{?fedora} || 0%{?rhel} >= 7
install -D -m 0644 security/selinux.anna.te %{buildroot}%{_datadir}/selinux/packages/%{name}/anna.te
%endif

%pre
# Create system user and group
getent group anna >/dev/null || groupadd -r anna
getent passwd anna >/dev/null || \
    useradd -r -g anna -d /var/lib/anna -s /sbin/nologin \
    -c "Anna Assistant Daemon" anna
exit 0

%post
# Create directories
mkdir -p /var/lib/anna /var/log/anna
chmod 750 /var/lib/anna /var/log/anna
chown anna:anna /var/lib/anna /var/log/anna

# Create README files
cat > /var/lib/anna/README <<'EOF'
Anna Assistant State Directory
===============================
This directory contains persistent state for the Anna daemon.
See %{_docdir}/%{name}/PRODUCTION_DEPLOYMENT.md for details.
EOF

cat > /var/log/anna/README <<'EOF'
Anna Assistant Log Directory
============================
This directory contains append-only logs for the Anna daemon.
See %{_docdir}/%{name}/PRODUCTION_DEPLOYMENT.md for monitoring commands.
EOF

chown anna:anna /var/lib/anna/README /var/log/anna/README
chmod 640 /var/lib/anna/README /var/log/anna/README

%systemd_post anna-daemon.service

echo ""
echo "Anna Assistant installed successfully."
echo ""
echo "The service is NOT enabled by default."
echo "Review the configuration and then:"
echo "  systemctl enable anna-daemon"
echo "  systemctl start anna-daemon"
echo ""
echo "See %{_docdir}/%{name}/PRODUCTION_DEPLOYMENT.md for details."
echo ""

%preun
%systemd_preun anna-daemon.service

%postun
%systemd_postun_with_restart anna-daemon.service

%files
%license LICENSE
%doc %{_docdir}/%{name}/PRODUCTION_DEPLOYMENT.md
%doc %{_docdir}/%{name}/README.md
%doc %{_docdir}/%{name}/CHANGELOG.md

%{_bindir}/annad
%{_bindir}/annactl
%{_unitdir}/anna-daemon.service
%config(noreplace) %{_sysconfdir}/logrotate.d/anna

%if 0%{?fedora} || 0%{?rhel} >= 7
%{_datadir}/selinux/packages/%{name}/anna.te
%endif

%changelog
* Tue Nov 12 2025 Anna Team <maintainer@example.com> - 1.6.0-0.1.rc1
- Phase 1.6: Mirror Audit and Temporal Self-Reflection
- Temporal Integrity Score (TIS) calculation
- Bias detection: 6 types with statistical thresholds
- Advisory-only adjustments (never auto-applied)
- Append-only audit trail with durability guarantees
- Conscience sovereignty enforcement
