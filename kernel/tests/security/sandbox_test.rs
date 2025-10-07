/*!
 * Sandbox System Integration Tests
 * Verifies granular capabilities, TOCTOU-safe paths, and network control
 */

use ai_os_kernel::security::{
    Capability, CapabilityManager, NetworkRule, SandboxConfig, SandboxManager, SandboxProvider,
};
use std::path::PathBuf;

#[test]
fn test_granular_file_capabilities() {
    let manager = SandboxManager::new();
    let pid = 100;

    // Create sandbox with granular read access to /tmp only
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::ReadFile(Some(PathBuf::from("/tmp"))));
    config.allow_path(PathBuf::from("/tmp"));

    manager.create_sandbox(config);

    // Should allow reading from /tmp
    assert!(
        manager.check_permission(
            pid,
            &Capability::ReadFile(Some(PathBuf::from("/tmp/test.txt")))
        ),
        "Should allow reading /tmp/test.txt"
    );

    // Should deny reading from /etc (not granted)
    assert!(
        !manager.check_permission(
            pid,
            &Capability::ReadFile(Some(PathBuf::from("/etc/passwd")))
        ),
        "Should deny reading /etc/passwd"
    );

    // Wildcard should grant access to both
    let mut config2 = SandboxConfig::minimal(200);
    config2.grant_capability(Capability::ReadFile(None));
    config2.allow_path(PathBuf::from("/"));
    manager.create_sandbox(config2);

    assert!(
        manager.check_permission(
            200,
            &Capability::ReadFile(Some(PathBuf::from("/tmp/test.txt")))
        ),
        "Wildcard should allow /tmp"
    );
    assert!(
        manager.check_permission(
            200,
            &Capability::ReadFile(Some(PathBuf::from("/etc/passwd")))
        ),
        "Wildcard should allow /etc"
    );
}

#[test]
fn test_capability_grants_hierarchy() {
    let manager = SandboxManager::new();
    let pid = 101;

    // Grant read access to /home/user
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::ReadFile(Some(PathBuf::from("/home/user"))));
    config.allow_path(PathBuf::from("/home/user"));

    manager.create_sandbox(config);

    // Should allow subdirectories
    assert!(
        manager.check_permission(
            pid,
            &Capability::ReadFile(Some(PathBuf::from("/home/user/documents")))
        ),
        "Should allow reading subdirectories"
    );

    // Should deny parent directories
    assert!(
        !manager.check_permission(pid, &Capability::ReadFile(Some(PathBuf::from("/home")))),
        "Should deny reading parent directory"
    );
}

#[test]
fn test_multiple_granular_capabilities() {
    let manager = SandboxManager::new();
    let pid = 102;

    // Grant different operations on different paths
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::ReadFile(Some(PathBuf::from("/tmp"))));
    config.grant_capability(Capability::WriteFile(Some(PathBuf::from("/var/log"))));
    config.allow_path(PathBuf::from("/tmp"));
    config.allow_path(PathBuf::from("/var/log"));

    manager.create_sandbox(config);

    // Can read from /tmp
    assert!(
        manager.check_permission(pid, &Capability::ReadFile(Some(PathBuf::from("/tmp/data")))),
        "Should allow reading /tmp"
    );

    // Cannot write to /tmp (only read granted)
    assert!(
        !manager.check_permission(
            pid,
            &Capability::WriteFile(Some(PathBuf::from("/tmp/data")))
        ),
        "Should deny writing to /tmp"
    );

    // Can write to /var/log
    assert!(
        manager.check_permission(
            pid,
            &Capability::WriteFile(Some(PathBuf::from("/var/log/app.log")))
        ),
        "Should allow writing to /var/log"
    );

    // Cannot read from /var/log (only write granted)
    assert!(
        !manager.check_permission(
            pid,
            &Capability::ReadFile(Some(PathBuf::from("/var/log/app.log")))
        ),
        "Should deny reading /var/log"
    );
}

#[test]
fn test_path_access_with_blocked_paths() {
    let manager = SandboxManager::new();
    let pid = 103;

    // Use actual temp directory for reliable canonicalization
    let temp_dir = std::env::temp_dir();
    let sensitive_path = temp_dir.join("sensitive");

    // Allow temp but block temp/sensitive
    let mut config = SandboxConfig::minimal(pid);
    config.allow_path(temp_dir.clone());
    config.block_path(sensitive_path.clone());

    manager.create_sandbox(config);

    // Should allow access to temp/data
    assert!(
        manager.check_path_access(pid, &temp_dir.join("data")),
        "Should allow temp/data"
    );

    // Should block access to temp/sensitive
    assert!(
        !manager.check_path_access(pid, &sensitive_path.join("secret.txt")),
        "Should block temp/sensitive"
    );
}

#[test]
fn test_network_rule_allow_all() {
    let manager = SandboxManager::new();
    let pid = 104;

    let mut config = SandboxConfig::minimal(pid);
    config.network_rules.push(NetworkRule::AllowAll);
    config.grant_capability(Capability::NetworkAccess(NetworkRule::AllowAll));

    manager.create_sandbox(config);

    // Should allow any host/port
    assert!(
        manager.check_network_access(pid, "example.com", Some(80)),
        "AllowAll should permit any host"
    );
    assert!(
        manager.check_network_access(pid, "192.168.1.1", Some(443)),
        "AllowAll should permit any IP"
    );
}

#[test]
fn test_network_rule_specific_host() {
    let manager = SandboxManager::new();
    let pid = 105;

    let mut config = SandboxConfig::minimal(pid);
    config.network_rules.push(NetworkRule::AllowHost {
        host: "api.example.com".to_string(),
        port: Some(443),
    });

    manager.create_sandbox(config);

    // Should allow exact match
    assert!(
        manager.check_network_access(pid, "api.example.com", Some(443)),
        "Should allow api.example.com:443"
    );

    // Should deny different port
    assert!(
        !manager.check_network_access(pid, "api.example.com", Some(80)),
        "Should deny different port"
    );

    // Should deny different host
    assert!(
        !manager.check_network_access(pid, "evil.com", Some(443)),
        "Should deny different host"
    );
}

#[test]
fn test_network_rule_wildcard_domain() {
    let manager = SandboxManager::new();
    let pid = 106;

    let mut config = SandboxConfig::minimal(pid);
    config.network_rules.push(NetworkRule::AllowHost {
        host: "*.example.com".to_string(),
        port: None, // Any port
    });

    manager.create_sandbox(config);

    // Should allow subdomains
    assert!(
        manager.check_network_access(pid, "api.example.com", Some(443)),
        "Should allow api.example.com"
    );
    assert!(
        manager.check_network_access(pid, "www.example.com", Some(80)),
        "Should allow www.example.com"
    );

    // Should deny non-subdomains
    assert!(
        !manager.check_network_access(pid, "example.com", Some(443)),
        "Should deny exact domain (not a subdomain)"
    );
    assert!(
        !manager.check_network_access(pid, "other.com", Some(443)),
        "Should deny different domain"
    );
}

#[test]
fn test_network_rule_cidr_ipv4() {
    let manager = SandboxManager::new();
    let pid = 107;

    let mut config = SandboxConfig::minimal(pid);
    config
        .network_rules
        .push(NetworkRule::AllowCIDR("192.168.1.0/24".to_string()));

    manager.create_sandbox(config);

    // Should allow IPs in range
    assert!(
        manager.check_network_access(pid, "192.168.1.1", None),
        "Should allow 192.168.1.1"
    );
    assert!(
        manager.check_network_access(pid, "192.168.1.100", None),
        "Should allow 192.168.1.100"
    );
    assert!(
        manager.check_network_access(pid, "192.168.1.255", None),
        "Should allow 192.168.1.255"
    );

    // Should deny IPs outside range
    assert!(
        !manager.check_network_access(pid, "192.168.2.1", None),
        "Should deny 192.168.2.1"
    );
    assert!(
        !manager.check_network_access(pid, "10.0.0.1", None),
        "Should deny 10.0.0.1"
    );
}

#[test]
fn test_network_rule_block_priority() {
    let manager = SandboxManager::new();
    let pid = 108;

    let mut config = SandboxConfig::minimal(pid);
    // Allow all, but block specific host
    config.network_rules.push(NetworkRule::AllowAll);
    config.network_rules.push(NetworkRule::BlockHost {
        host: "malicious.com".to_string(),
        port: None,
    });

    manager.create_sandbox(config);

    // Should allow most hosts
    assert!(
        manager.check_network_access(pid, "safe.com", Some(443)),
        "Should allow safe.com"
    );

    // Should block explicitly blocked host
    assert!(
        !manager.check_network_access(pid, "malicious.com", Some(443)),
        "Should block malicious.com despite AllowAll"
    );
}

#[test]
fn test_sandbox_lifecycle() {
    let manager = SandboxManager::new();
    let pid = 109;

    // Create sandbox
    let config = SandboxConfig::standard(pid);
    manager.create_sandbox(config.clone());

    assert!(manager.has_sandbox(pid), "Sandbox should exist");

    // Get sandbox
    let retrieved = manager.get_sandbox(pid);
    assert!(retrieved.is_some(), "Should retrieve sandbox");
    assert_eq!(retrieved.unwrap().pid, pid, "PID should match");

    // Remove sandbox
    assert!(manager.remove_sandbox(pid), "Should remove sandbox");
    assert!(
        !manager.has_sandbox(pid),
        "Sandbox should not exist after removal"
    );
}

#[test]
fn test_sandbox_configs() {
    // Test minimal config
    let minimal = SandboxConfig::minimal(1);
    assert!(
        minimal.capabilities.is_empty(),
        "Minimal should have no capabilities"
    );
    assert!(
        minimal.network_rules.is_empty(),
        "Minimal should have no network access"
    );

    // Test standard config
    let standard = SandboxConfig::standard(2);
    assert!(
        standard.has_capability(&Capability::ReadFile(None)),
        "Standard should have read capability"
    );
    assert!(
        standard.has_capability(&Capability::WriteFile(None)),
        "Standard should have write capability"
    );

    // Test privileged config
    let privileged = SandboxConfig::privileged(3);
    assert!(
        privileged.has_capability(&Capability::SpawnProcess),
        "Privileged should have spawn capability"
    );
    assert!(
        privileged.has_capability(&Capability::NetworkAccess(NetworkRule::AllowAll)),
        "Privileged should have network access"
    );
    assert!(
        !privileged.network_rules.is_empty(),
        "Privileged should have network rules"
    );
}

#[test]
fn test_capability_revocation() {
    let manager = SandboxManager::new();
    let pid = 110;

    // Create with capabilities
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::ReadFile(None));
    config.grant_capability(Capability::WriteFile(None));
    manager.create_sandbox(config);

    // Verify capabilities exist
    assert!(
        manager.check_permission(pid, &Capability::ReadFile(None)),
        "Should have read capability"
    );

    // Revoke read capability
    manager
        .revoke_capability(pid, &Capability::ReadFile(None))
        .expect("Should revoke capability");

    // Verify revocation
    assert!(
        !manager.check_permission(pid, &Capability::ReadFile(None)),
        "Should not have read capability after revocation"
    );
    assert!(
        manager.check_permission(pid, &Capability::WriteFile(None)),
        "Should still have write capability"
    );
}

#[test]
fn test_no_sandbox_denies_all() {
    let manager = SandboxManager::new();
    let pid = 999;

    // Without creating a sandbox, all operations should be denied
    assert!(
        !manager.check_permission(pid, &Capability::ReadFile(None)),
        "Non-existent sandbox should deny capabilities"
    );
    assert!(
        !manager.check_path_access(pid, &PathBuf::from("/tmp")),
        "Non-existent sandbox should deny paths"
    );
    assert!(
        !manager.check_network_access(pid, "example.com", None),
        "Non-existent sandbox should deny network"
    );
}

#[test]
fn test_empty_allowed_paths_denies_all() {
    let manager = SandboxManager::new();
    let pid = 111;

    // Create sandbox with no allowed paths
    let config = SandboxConfig::minimal(pid);
    manager.create_sandbox(config);

    // Should deny all path access
    assert!(
        !manager.check_path_access(pid, &PathBuf::from("/tmp")),
        "Empty allowed_paths should deny all"
    );
    assert!(
        !manager.check_path_access(pid, &PathBuf::from("/")),
        "Empty allowed_paths should deny even root"
    );
}
