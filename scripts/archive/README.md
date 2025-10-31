# Archived Scripts

This directory contains old scripts that are no longer actively used but kept for historical reference.

## Archived Files

### Version-Specific Installers (Obsolete)
- `install_v10.sh` - v0.10 installer (replaced by install.sh)
- `install_v101.sh` - v0.10.1 installer (replaced by install.sh)
- `install_simple.sh` - Simple installer prototype (replaced by install.sh)
- `uninstall_v10.sh` - v0.10 uninstaller (replaced by uninstall.sh)
- `uninstall_v101.sh` - v0.10.1 uninstaller (replaced by uninstall.sh)

### Utilities (No Longer Used)
- `update_service_file.sh` - Service file updater (functionality integrated into installer)
- `anna_common.sh` - Bash messaging library (not used in current scripts)
- `test_anna_say.sh` - Demo for anna_common.sh messaging

## Why Archived?

These scripts are preserved for:
- Historical reference
- Understanding past implementation approaches
- Recovery if specific functionality needs to be restored
- Documentation of evolution of the project

## Current Scripts

See parent directory (`scripts/`) for actively maintained scripts:
- `install.sh` - Smart installer with binary download support
- `uninstall.sh` - Current uninstaller
- `release.sh` - Automated release script
- Various utility and diagnostic scripts

## Note

**Do not use these archived scripts for new installations.** They are outdated and may not work correctly with current versions of Anna Assistant.

Use the current installer: `./scripts/install.sh`
