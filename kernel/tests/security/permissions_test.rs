/*!
 * Permissions Module Integration Tests
 */

use ai_os_kernel::permissions::{
    Action, PermissionChecker, PermissionManager, PermissionRequest, Resource,
};
use ai_os_kernel::security::{
    Capability, NetworkRule, SandboxConfig, SandboxManager, SandboxProvider,
};
use std::path::PathBuf;

#[test]
fn test_file_permissions_integration() {
    let sandbox = SandboxManager::new();

    // Create sandbox with file access
    let mut config = SandboxConfig::minimal(100);
    config.grant_capability(Capability::ReadFile(None));
    config.grant_capability(Capability::WriteFile(None));
    config.allow_path(PathBuf::from("/tmp"));
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);

    // Test allowed read
    let read_req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));
    let read_resp = manager.check(&read_req);
    assert!(read_resp.is_allowed(), "Should allow /tmp read");

    // Test allowed write
    let write_req = PermissionRequest::file_write(100, PathBuf::from("/tmp/test.txt"));
    let write_resp = manager.check(&write_req);
    assert!(write_resp.is_allowed(), "Should allow /tmp write");

    // Test denied read
    let denied_req = PermissionRequest::file_read(100, PathBuf::from("/etc/passwd"));
    let denied_resp = manager.check(&denied_req);
    assert!(!denied_resp.is_allowed(), "Should deny /etc read");
}

#[test]
fn test_network_permissions_integration() {
    let sandbox = SandboxManager::new();

    // Create sandbox with network access
    let mut config = SandboxConfig::minimal(200);
    config.network_rules.push(NetworkRule::AllowHost {
        host: "api.example.com".to_string().into(),
        port: Some(443),
    });
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);

    // Test allowed connection
    let allowed_req = PermissionRequest::net_connect(200, "api.example.com".to_string(), Some(443));
    let allowed_resp = manager.check(&allowed_req);
    assert!(
        allowed_resp.is_allowed(),
        "Should allow api.example.com:443"
    );

    // Test denied connection
    let denied_req = PermissionRequest::net_connect(200, "evil.com".to_string(), Some(80));
    let denied_resp = manager.check(&denied_req);
    assert!(!denied_resp.is_allowed(), "Should deny evil.com:80");
}

#[test]
fn test_process_permissions_integration() {
    let sandbox = SandboxManager::new();

    // Sandbox with process control
    let mut config = SandboxConfig::minimal(300);
    config.grant_capability(Capability::SpawnProcess);
    config.grant_capability(Capability::KillProcess);
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);

    // Test spawn permission
    let spawn_req = PermissionRequest::new(300, Resource::Process { pid: 400 }, Action::Create);
    let spawn_resp = manager.check(&spawn_req);
    assert!(spawn_resp.is_allowed(), "Should allow process spawn");

    // Test kill permission
    let kill_req = PermissionRequest::proc_kill(300, 400);
    let kill_resp = manager.check(&kill_req);
    assert!(kill_resp.is_allowed(), "Should allow process kill");
}

#[test]
fn test_caching_integration() {
    let sandbox = SandboxManager::new();

    let mut config = SandboxConfig::minimal(400);
    config.grant_capability(Capability::ReadFile(None));
    config.allow_path(PathBuf::from("/tmp"));
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);
    let req = PermissionRequest::file_read(400, PathBuf::from("/tmp/test.txt"));

    // First check - should be uncached
    let resp1 = manager.check(&req);
    assert!(resp1.is_allowed());
    assert!(!resp1.cached);

    // Second check - should be cached
    let resp2 = manager.check(&req);
    assert!(resp2.is_allowed());
    assert!(resp2.cached);

    // Verify cache statistics
    let stats = manager.cache_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert!(stats.hit_rate > 0.0);
}

#[test]
fn test_audit_integration() {
    let sandbox = SandboxManager::new();

    // Minimal sandbox (denies everything)
    let config = SandboxConfig::minimal(500);
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);

    // Make multiple requests
    let requests = vec![
        PermissionRequest::file_read(500, PathBuf::from("/etc/passwd")),
        PermissionRequest::file_write(500, PathBuf::from("/etc/shadow")),
        PermissionRequest::net_connect(500, "example.com".to_string(), Some(80)),
    ];

    for req in requests {
        manager.check_and_audit(&req);
    }

    // Check audit trail
    let stats = manager.audit_stats();
    assert_eq!(stats.total_events, 3);
    assert_eq!(stats.total_denials, 3);

    let recent = manager.audit().recent(10);
    assert_eq!(recent.len(), 3);

    let for_pid = manager.audit().for_pid(500, 10);
    assert_eq!(for_pid.len(), 3);

    let denial_count = manager.audit().denial_count(500);
    assert_eq!(denial_count, 3);
}

#[test]
fn test_batch_check_integration() {
    let sandbox = SandboxManager::new();

    let mut config = SandboxConfig::minimal(600);
    config.grant_capability(Capability::ReadFile(None));
    config.allow_path(PathBuf::from("/tmp"));
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);

    let requests = vec![
        PermissionRequest::file_read(600, PathBuf::from("/tmp/file1.txt")),
        PermissionRequest::file_read(600, PathBuf::from("/tmp/file2.txt")),
        PermissionRequest::file_read(600, PathBuf::from("/etc/passwd")),
        PermissionRequest::file_read(600, PathBuf::from("/tmp/file3.txt")),
    ];

    let responses = manager.check_batch(&requests);
    assert_eq!(responses.len(), 4);

    // First 2 and last should be allowed
    assert!(responses[0].is_allowed());
    assert!(responses[1].is_allowed());
    assert!(!responses[2].is_allowed()); // /etc denied
    assert!(responses[3].is_allowed());
}

#[test]
fn test_cache_invalidation_integration() {
    let sandbox = SandboxManager::new();

    let mut config = SandboxConfig::minimal(700);
    config.grant_capability(Capability::ReadFile(None));
    config.allow_path(PathBuf::from("/tmp"));
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);
    let req = PermissionRequest::file_read(700, PathBuf::from("/tmp/test.txt"));

    // Cache the result
    let resp1 = manager.check(&req);
    assert!(resp1.is_allowed());

    let resp2 = manager.check(&req);
    assert!(resp2.cached);

    // Invalidate cache
    manager.invalidate_cache(700);

    // Next check should be uncached
    let resp3 = manager.check(&req);
    assert!(resp3.is_allowed());
    assert!(!resp3.cached);
}

#[test]
fn test_no_sandbox_integration() {
    let sandbox = SandboxManager::new();
    let manager = PermissionManager::new(sandbox);

    // Request for PID without sandbox
    let req = PermissionRequest::file_read(999, PathBuf::from("/tmp/test.txt"));
    let resp = manager.check(&req);

    assert!(!resp.is_allowed());
    assert!(resp.reason().contains("No sandbox"));
}

#[test]
fn test_granular_file_permissions() {
    let sandbox = SandboxManager::new();

    // Grant granular file access
    let mut config = SandboxConfig::minimal(800);
    config.grant_capability(Capability::ReadFile(Some(PathBuf::from("/tmp"))));
    config.allow_path(PathBuf::from("/tmp"));
    sandbox.create_sandbox(config);

    let manager = PermissionManager::new(sandbox);

    // Should allow /tmp
    let tmp_req = PermissionRequest::file_read(800, PathBuf::from("/tmp/test.txt"));
    assert!(manager.check(&tmp_req).is_allowed());

    // Should deny other paths
    let etc_req = PermissionRequest::file_read(800, PathBuf::from("/etc/passwd"));
    assert!(!manager.check(&etc_req).is_allowed());
}
