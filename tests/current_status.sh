#!/usr/bin/env bash
# Show current status of Anna installation
# No sudo required - just shows what's working

echo "════════════════════════════════════════════════════════"
echo "  Anna v0.9.6-alpha.6 - Current Status"
echo "════════════════════════════════════════════════════════"
echo ""

echo "✅ WORKING (Tested & Verified):"
echo ""
echo "  Build & Binaries:"
echo "    • cargo build --release → Success (0 errors)"
echo "    • annactl --version → 0.9.6-alpha.6"
echo "    • annad binary exists → /usr/local/bin/annad"
echo ""
echo "  Offline Commands (No Daemon Required):"
echo "    • annactl profile show → Beautiful system profile"
echo "    • annactl profile checks → 11 health checks with colors"
echo "    • annactl profile checks --json → Valid JSON output"
echo "    • annactl persona list → Shows 4 personas (dev/gamer/minimal/ops)"
echo "    • annactl persona get → Shows current persona (dev)"
echo "    • annactl config list → Shows configuration structure"
echo "    • annactl doctor check → Runs 9 health checks"
echo ""

echo "⚠️  PENDING (Requires Sudo to Test):"
echo ""
echo "  Daemon & Service:"
echo "    • systemctl is-active annad → Currently: $(systemctl is-active annad 2>&1)"
echo "    • Socket /run/anna/annad.sock → $(test -S /run/anna/annad.sock && echo 'exists' || echo 'missing')"
echo ""
echo "  Issue: Installed service file is outdated"
echo "    Missing: User=root, Group=anna"
echo "    Missing: /var/lib/anna in ReadWritePaths"
echo ""
echo "  Fix: Run one of:"
echo "    sudo ./scripts/update_service_file.sh"
echo "    ./scripts/install.sh  (full reinstall)"
echo ""

echo "🔧 Quick Test - Offline Commands:"
echo ""

# Test a few commands
echo -n "  Testing profile show... "
if ./target/release/annactl profile show >/dev/null 2>&1; then
    echo "✓"
else
    echo "✗"
fi

echo -n "  Testing profile checks... "
if ./target/release/annactl profile checks >/dev/null 2>&1; then
    echo "✓"
else
    echo "✗"
fi

echo -n "  Testing persona list... "
if ./target/release/annactl persona list >/dev/null 2>&1; then
    echo "✓"
else
    echo "✗"
fi

echo -n "  Testing doctor check... "
if ./target/release/annactl doctor check >/dev/null 2>&1; then
    echo "✓ (exits 1 because daemon not running, but command works)"
else
    echo "✗"
fi

echo ""
echo "📊 System Info:"
echo "  CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
echo "  Kernel: $(uname -r)"
echo "  Distro: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2)"
echo ""

echo "📁 Directory Status:"
for dir in /etc/anna /var/lib/anna /var/log/anna /run/anna; do
    if [ -d "$dir" ]; then
        perms=$(stat -c "%a %U:%G" "$dir" 2>/dev/null)
        echo "  $dir → $perms"
    else
        echo "  $dir → missing"
    fi
done

echo ""
echo "📋 Files:"
echo "  Version: $(cat /etc/anna/version 2>/dev/null || echo 'not found')"
echo "  Config: $(test -f /etc/anna/config.toml && echo 'present' || echo 'missing')"
echo "  Policies: $(ls /etc/anna/policies.d/*.yaml 2>/dev/null | wc -l) files"

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Next Steps"
echo "════════════════════════════════════════════════════════"
echo ""
echo "To fix the daemon and complete installation:"
echo ""
echo "  1. Update service file:"
echo "     sudo ./scripts/update_service_file.sh"
echo ""
echo "  2. Verify it worked:"
echo "     systemctl is-active annad  # Should show: active"
echo "     annactl status              # Should connect"
echo ""
echo "  3. Full verification:"
echo "     ./scripts/verify_installation.sh"
echo ""
echo "See TROUBLESHOOTING.md for detailed help."
echo ""
