# Network Namespace Isolation

## Overview

Cross-platform network isolation system providing true network namespace isolation on Linux, alternative isolation on macOS, and simulation mode fallback.

## Features

### Platform Support

- **Linux**: True network namespaces with full isolation
  - Leverages Linux kernel network namespaces (`/proc/self/ns/net`)
  - Virtual ethernet (veth) pairs for connectivity
  - Bridge networking for inter-namespace communication
  - NAT support for private networks
  
- **macOS**: Packet filter-based isolation
  - Uses `pfctl` for network filtering
  - Process-based network rules
  - Application firewall integration

- **Simulation**: Fallback capability-based restrictions
  - API-compatible with full implementations
  - Used when OS-level support unavailable
  - Suitable for development and testing

### Isolation Modes

1. **Full Isolation**: Complete network lockdown
   - No external network access
   - Only loopback interface available
   - Maximum security for untrusted processes

2. **Private Network**: Isolated network with NAT
   - Private IP address range (10.0.0.0/24)
   - NAT gateway for outbound connectivity
   - Configurable DNS servers
   - Optional port forwarding

3. **Shared Network**: Host network access
   - Uses host's network stack
   - No isolation, full network access
   - Suitable for trusted processes

4. **Bridged Network**: Custom bridge configuration
   - Connect multiple namespaces
   - Inter-namespace communication
   - Flexible network topology

## Architecture

### Module Structure

```
namespace/
├── mod.rs           - Public exports
├── types.rs         - Core types and configurations
├── traits.rs        - Platform-agnostic interfaces
├── manager.rs       - Unified manager with auto-detection
├── linux.rs         - Linux namespace implementation
├── macos.rs         - macOS packet filter implementation
├── simulation.rs    - Fallback simulation mode
├── veth.rs          - Virtual ethernet pair management
├── bridge.rs        - Network bridge management
└── README.md        - This file
```

### Key Types

- **`NamespaceId`**: Unique identifier for namespaces
- **`IsolationMode`**: Network isolation level (Full/Private/Shared/Bridged)
- **`NamespaceConfig`**: Complete configuration for a namespace
- **`InterfaceConfig`**: Virtual network interface settings
- **`NamespaceStats`**: Runtime statistics (bytes, packets, interfaces)

## Usage

### Basic Usage

```rust
use ai_os_kernel::security::namespace::*;

// Create namespace manager (auto-detects platform)
let manager = NamespaceManager::new();

// Create fully isolated namespace
let config = NamespaceConfig::full_isolation(pid);
manager.create(config)?;

// Create private network with NAT
let config = NamespaceConfig::private_network(pid);
manager.create(config)?;
```

### Sandbox Integration

```rust
use ai_os_kernel::security::*;

// Create sandbox with namespace support
let sandbox = SandboxManager::with_namespaces();

// Grant network namespace capability
let mut config = SandboxConfig::privileged(pid);
config.capabilities.insert(Capability::NetworkNamespace);

sandbox.create_sandbox(config);
```

### Custom Configuration

```rust
use ai_os_kernel::security::namespace::*;
use std::net::{IpAddr, Ipv4Addr};

let mut config = NamespaceConfig::private_network(pid);

// Custom interface
config.interface = Some(InterfaceConfig {
    name: "eth0".to_string(),
    ip_addr: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
    prefix_len: 24,
    gateway: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
    mtu: 1500,
});

// Port forwarding
config.port_forwards.push((8080, 80));  // Host 8080 -> NS 80
config.port_forwards.push((8443, 443)); // Host 8443 -> NS 443

manager.create(config)?;
```

### Namespace Lifecycle

```rust
// Create
let config = NamespaceConfig::full_isolation(100);
manager.create(config.clone())?;

// Query
assert!(manager.exists(&config.id));
let info = manager.get_info(&config.id)?;
let stats = manager.get_stats(&config.id)?;

// List all
let namespaces = manager.list();

// Retrieve by PID
let info = manager.get_by_pid(100)?;

// Destroy
manager.destroy(&config.id)?;
```

## Implementation Details

### Linux Implementation

On Linux, the system uses:

1. **Network Namespaces**: Kernel-level isolation via `/var/run/netns/`
2. **veth Pairs**: Virtual ethernet for host-namespace connectivity
3. **Bridges**: Linux bridges for multi-namespace networks
4. **iptables/nftables**: NAT and firewall rules
5. **rtnetlink**: Netlink protocol for programmatic configuration

### macOS Implementation

On macOS, the system provides:

1. **Packet Filters**: `pfctl` rules for network filtering
2. **Process Tracking**: Per-process network restrictions
3. **Application Firewall**: Integration with macOS security
4. **Simulation Layer**: API compatibility with Linux

### Simulation Mode

Simulation mode:

1. **In-Memory State**: Tracks namespace configurations
2. **No OS Changes**: Pure Rust data structures
3. **API Compatibility**: Same interface as true implementations
4. **Testing**: Full test suite without root privileges

## Performance

- **Creation**: Sub-millisecond namespace creation (simulation), <10ms (Linux)
- **Overhead**: Minimal runtime overhead, native network stack performance
- **Scalability**: Hundreds of concurrent namespaces supported
- **Cleanup**: Automatic cleanup on process termination

## Security Considerations

### Capabilities Required

- Linux: `CAP_NET_ADMIN` or root for namespace creation
- macOS: Root or admin privileges for pfctl
- Simulation: No special privileges required

### Isolation Guarantees

- **Full Mode**: Complete network isolation, no data leakage
- **Private Mode**: Isolated from other namespaces, NAT gateway controlled
- **Shared Mode**: No isolation, uses host network
- **Bridged Mode**: Isolation from host, connectivity between namespaces

### Best Practices

1. Use **Full Isolation** for untrusted code
2. Use **Private Network** for isolated services needing internet
3. Grant `NetworkNamespace` capability sparingly
4. Monitor namespace statistics for anomalies
5. Clean up namespaces on process termination
6. Use port forwarding instead of exposing all ports

## Testing

Run namespace tests:

```bash
cd kernel
cargo test namespace --lib -- --nocapture
```

Run integration tests:

```bash
cargo test --test namespace_test
```

Platform-specific tests:

```bash
# Linux only
cargo test --test namespace_test --features linux-tests

# macOS only
cargo test --test namespace_test --features macos-tests
```

## Future Enhancements

- [ ] IPv6 support with dual-stack configuration
- [ ] Firewall rules (iptables/nftables) integration
- [ ] Network QoS and bandwidth limiting
- [ ] Container networking compatibility
- [ ] Windows support via Hyper-V network isolation
- [ ] DNS server in-namespace (dnsmasq integration)
- [ ] VPN support for namespace outbound traffic
- [ ] Network monitoring and traffic analysis
- [ ] Automatic IP allocation (DHCP)
- [ ] Multi-host namespace orchestration

## Dependencies

- `nix`: Linux system calls and namespace operations
- `rtnetlink`: Netlink protocol for network configuration
- `ipnetwork`: IP address and CIDR manipulation
- `netlink-packet-route`: Low-level netlink packets
- `dashmap`: Concurrent hash map for namespace tracking

## References

- [Linux Network Namespaces](https://man7.org/linux/man-pages/man7/network_namespaces.7.html)
- [veth(4) man page](https://man7.org/linux/man-pages/man4/veth.4.html)
- [bridge(8) man page](https://man7.org/linux/man-pages/man8/bridge.8.html)
- [macOS Packet Filter (pf)](https://www.openbsd.org/faq/pf/)
- [rtnetlink crate](https://docs.rs/rtnetlink/)
