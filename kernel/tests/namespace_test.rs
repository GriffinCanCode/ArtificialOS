/*!
 * Network Namespace Isolation Tests
 * Comprehensive tests for cross-platform network isolation
 */

use ai_os_kernel::security::namespace::*;
use ai_os_kernel::security::*;

#[test]
fn test_namespace_manager_initialization() {
    let manager = NamespaceManager::new();
    let platform = manager.platform();

    // Should not be Windows since we're ignoring it
    assert_ne!(platform, PlatformType::WindowsWFP);

    // Should be one of the supported platforms
    assert!(matches!(
        platform,
        PlatformType::LinuxNetns | PlatformType::MacOSFilter | PlatformType::Simulation
    ));
}

#[test]
fn test_create_full_isolation_namespace() {
    let manager = NamespaceManager::with_simulation();
    let config = NamespaceConfig::full_isolation(100);

    let result = manager.create(config.clone());
    assert!(result.is_ok(), "Failed to create namespace: {:?}", result);

    assert!(manager.exists(&config.id));
    assert_eq!(manager.count(), 1);

    // Cleanup
    let _ = manager.destroy(&config.id);
}

#[test]
fn test_create_private_network_namespace() {
    let manager = NamespaceManager::with_simulation();
    let config = NamespaceConfig::private_network(101);

    let result = manager.create(config.clone());
    assert!(result.is_ok());

    let info = manager.get_info(&config.id);
    assert!(info.is_some());

    let info = info.unwrap();
    assert_eq!(info.config.mode, IsolationMode::Private);
    assert_eq!(info.config.pid, 101);

    // Cleanup
    let _ = manager.destroy(&config.id);
}

#[test]
fn test_namespace_lifecycle() {
    let manager = NamespaceManager::with_simulation();
    let pid = 200;
    let config = NamespaceConfig::full_isolation(pid);

    // Create
    assert!(manager.create(config.clone()).is_ok());
    assert!(manager.exists(&config.id));

    // Retrieve by PID
    let info = manager.get_by_pid(pid);
    assert!(info.is_some());
    assert_eq!(info.unwrap().config.pid, pid);

    // Destroy
    assert!(manager.destroy(&config.id).is_ok());
    assert!(!manager.exists(&config.id));

    // Should not exist after destroy
    let info = manager.get_by_pid(pid);
    assert!(info.is_none());
}

#[test]
fn test_multiple_namespaces() {
    let manager = NamespaceManager::with_simulation();

    let config1 = NamespaceConfig::full_isolation(301);
    let config2 = NamespaceConfig::private_network(302);
    let config3 = NamespaceConfig::shared_network(303);

    assert!(manager.create(config1.clone()).is_ok());
    assert!(manager.create(config2.clone()).is_ok());
    assert!(manager.create(config3.clone()).is_ok());

    assert_eq!(manager.count(), 3);

    let namespaces = manager.list();
    assert_eq!(namespaces.len(), 3);

    // Verify each namespace
    assert!(manager.exists(&config1.id));
    assert!(manager.exists(&config2.id));
    assert!(manager.exists(&config3.id));

    // Cleanup
    let _ = manager.destroy(&config1.id);
    let _ = manager.destroy(&config2.id);
    let _ = manager.destroy(&config3.id);

    assert_eq!(manager.count(), 0);
}

#[test]
fn test_namespace_stats() {
    let manager = NamespaceManager::with_simulation();
    let config = NamespaceConfig::private_network(400);

    assert!(manager.create(config.clone()).is_ok());

    let stats = manager.get_stats(&config.id);
    assert!(stats.is_some());

    let stats = stats.unwrap();
    assert_eq!(stats.id, config.id);
    assert!(stats.interface_count > 0);

    // Cleanup
    let _ = manager.destroy(&config.id);
}

#[test]
fn test_interface_config() {
    let interface = InterfaceConfig::default();

    assert_eq!(interface.name, "veth0");
    assert_eq!(interface.prefix_len, 24);
    assert_eq!(interface.mtu, 1500);
    assert!(interface.gateway.is_some());
}

#[test]
fn test_isolation_modes() {
    let manager = NamespaceManager::with_simulation();

    let modes = vec![
        IsolationMode::Full,
        IsolationMode::Private,
        IsolationMode::Shared,
        IsolationMode::Bridged,
    ];

    for (idx, mode) in modes.iter().enumerate() {
        let pid = 500 + idx as u32;
        let mut config = NamespaceConfig::private_network(pid);
        config.mode = *mode;

        let result = manager.create(config.clone());
        assert!(
            result.is_ok(),
            "Failed to create namespace with mode {:?}",
            mode
        );

        let info = manager.get_info(&config.id);
        assert!(info.is_some());
        assert_eq!(info.unwrap().config.mode, *mode);

        let _ = manager.destroy(&config.id);
    }
}

#[test]
fn test_sandbox_with_namespace_capability() {
    let mut sandbox = SandboxManager::with_namespaces();
    let pid = 600;

    let mut config = SandboxConfig::privileged(pid);
    config.capabilities.insert(Capability::NetworkNamespace);

    sandbox.create_sandbox(config);

    // Should have network namespace support
    assert!(sandbox.namespace_manager().is_some());

    // Cleanup
    sandbox.remove_sandbox(pid);
}

#[test]
fn test_namespace_id_display() {
    let ns_id = NamespaceId::from_pid(42);
    let display = format!("{}", ns_id);
    assert_eq!(display, "ns-42");
}

#[test]
fn test_namespace_error_types() {
    let err = NamespaceError::NotFound("test-ns".to_string());
    assert!(err.to_string().contains("test-ns"));

    let err = NamespaceError::PlatformNotSupported("Windows".to_string());
    assert!(err.to_string().contains("Windows"));
}

#[test]
fn test_veth_manager() {
    let veth_mgr = VethManager::new();

    // VethManager should be zero-sized
    assert_eq!(std::mem::size_of_val(&veth_mgr), 0);
}

#[test]
fn test_bridge_manager() {
    let bridge_mgr = BridgeManager::new();

    // BridgeManager should be zero-sized
    assert_eq!(std::mem::size_of_val(&bridge_mgr), 0);
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_namespace_support() {
    let manager = NamespaceManager::new();

    // On Linux, should detect namespace support
    let platform = manager.platform();

    // Will be either LinuxNetns (if running with proper permissions)
    // or Simulation (if permissions are lacking)
    assert!(matches!(
        platform,
        PlatformType::LinuxNetns | PlatformType::Simulation
    ));
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_namespace_support() {
    let manager = NamespaceManager::new();
    let platform = manager.platform();

    // On macOS, should use packet filters or simulation
    assert!(matches!(
        platform,
        PlatformType::MacOSFilter | PlatformType::Simulation
    ));
}

#[test]
fn test_namespace_config_builders() {
    // Test full isolation
    let config = NamespaceConfig::full_isolation(1);
    assert_eq!(config.mode, IsolationMode::Full);
    assert!(config.interface.is_none());
    assert_eq!(config.dns_servers.len(), 0);

    // Test private network
    let config = NamespaceConfig::private_network(2);
    assert_eq!(config.mode, IsolationMode::Private);
    assert!(config.interface.is_some());
    assert!(config.dns_servers.len() > 0);

    // Test shared network
    let config = NamespaceConfig::shared_network(3);
    assert_eq!(config.mode, IsolationMode::Shared);
}

#[test]
fn test_concurrent_namespace_operations() {
    use std::thread;

    let manager = NamespaceManager::with_simulation();
    let manager_clone = manager.clone();

    let handle = thread::spawn(move || {
        for i in 1000..1010 {
            let config = NamespaceConfig::full_isolation(i);
            let _ = manager_clone.create(config.clone());
            let _ = manager_clone.destroy(&config.id);
        }
    });

    for i in 2000..2010 {
        let config = NamespaceConfig::private_network(i);
        let _ = manager.create(config.clone());
        let _ = manager.destroy(&config.id);
    }

    handle.join().unwrap();
}

#[test]
fn test_namespace_with_port_forwards() {
    let manager = NamespaceManager::with_simulation();
    let mut config = NamespaceConfig::private_network(700);

    // Add port forwarding rules
    config.port_forwards.push((8080, 80));
    config.port_forwards.push((8443, 443));

    assert!(manager.create(config.clone()).is_ok());

    let info = manager.get_info(&config.id).unwrap();
    assert_eq!(info.config.port_forwards.len(), 2);

    let _ = manager.destroy(&config.id);
}

#[test]
fn test_capability_grants_network_namespace() {
    let cap = Capability::NetworkNamespace;
    assert!(cap.grants(&Capability::NetworkNamespace));
    assert!(!cap.grants(&Capability::SpawnProcess));
}
