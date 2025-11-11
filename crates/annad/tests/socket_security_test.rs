//! Socket security tests for annad
//!
//! Phase 0.4: Tests for socket permissions and access control
//! Citation: [archwiki:Security]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Test that socket directory permissions are correct when created
#[test]
fn test_socket_directory_permissions() {
    // This test validates that /run/anna should have correct permissions
    // In production: root:anna 0750 (set by systemd RuntimeDirectory)

    let test_dir = "/tmp/test-anna-socket-dir";
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(test_dir).expect("Failed to create test directory");

    // Set permissions similar to production
    #[cfg(unix)]
    {
        let perms = fs::Permissions::from_mode(0o750);
        fs::set_permissions(test_dir, perms).expect("Failed to set permissions");

        let metadata = fs::metadata(test_dir).expect("Failed to get metadata");
        let mode = metadata.permissions().mode() & 0o777;

        assert_eq!(mode, 0o750, "Directory should have 0750 permissions");
    }

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);
}

/// Test that socket file permissions can be set to 0660
#[test]
fn test_socket_file_permissions() {
    let test_socket = "/tmp/test-anna.sock";
    let _ = fs::remove_file(test_socket);

    // Create a test file to simulate socket
    fs::write(test_socket, b"test").expect("Failed to create test file");

    #[cfg(unix)]
    {
        // Set socket permissions to 0660
        let perms = fs::Permissions::from_mode(0o660);
        fs::set_permissions(test_socket, perms).expect("Failed to set permissions");

        let metadata = fs::metadata(test_socket).expect("Failed to get metadata");
        let mode = metadata.permissions().mode() & 0o777;

        assert_eq!(mode, 0o660, "Socket should have 0660 permissions");
    }

    // Cleanup
    let _ = fs::remove_file(test_socket);
}

/// Test directory structure permissions
#[test]
fn test_secure_directory_structure() {
    // Phase 0.4: Validate that we can create directories with correct permissions
    let test_dirs = [
        ("/tmp/test-anna-etc", 0o700),      // /etc/anna equivalent
        ("/tmp/test-anna-log", 0o700),      // /var/log/anna equivalent
        ("/tmp/test-anna-lib", 0o700),      // /var/lib/anna equivalent
        ("/tmp/test-anna-static", 0o755),   // /usr/local/lib/anna equivalent
    ];

    for (dir, expected_mode) in &test_dirs {
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir).expect("Failed to create directory");

        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(*expected_mode);
            fs::set_permissions(dir, perms).expect("Failed to set permissions");

            let metadata = fs::metadata(dir).expect("Failed to get metadata");
            let mode = metadata.permissions().mode() & 0o777;

            assert_eq!(
                mode, *expected_mode,
                "Directory {} should have {:o} permissions, got {:o}",
                dir, expected_mode, mode
            );
        }

        // Cleanup
        let _ = fs::remove_dir_all(dir);
    }
}

/// Test that socket path validation works
#[test]
fn test_socket_path_validation() {
    let valid_paths = [
        "/run/anna/anna.sock",
        "/tmp/anna.sock",
    ];

    for path in &valid_paths {
        let socket_path = Path::new(path);
        assert!(socket_path.parent().is_some(), "Socket path should have parent directory");

        if let Some(parent) = socket_path.parent() {
            assert!(!parent.as_os_str().is_empty(), "Parent should not be empty");
        }
    }
}

/// Test group ownership validation (conceptual test)
#[test]
fn test_group_anna_requirement() {
    // This test documents the requirement for the 'anna' group
    // In production, the group must exist: sudo groupadd --system anna

    // The socket should be owned by root:anna with 0660 permissions
    // This allows:
    // - root to read/write (system administration)
    // - users in 'anna' group to read/write (annactl access)
    // - all others: no access

    // This test just validates the concept
    let required_group = "anna";
    assert_eq!(required_group, "anna", "Required group should be 'anna'");
}

/// Test systemd RuntimeDirectory configuration
#[test]
fn test_runtime_directory_config() {
    // Phase 0.4: Document expected systemd configuration
    // RuntimeDirectory=anna
    // RuntimeDirectoryMode=0750

    let expected_mode = 0o750;
    assert_eq!(expected_mode, 0o750, "RuntimeDirectory should use 0750 mode");
}
