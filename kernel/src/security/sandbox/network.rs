/*!
 * Fine-Grained Network Access Control
 */

use crate::security::types::NetworkRule;
use std::net::IpAddr;

/// Check if network access to a host/port is allowed
pub fn check_network_access(rules: &[NetworkRule], host: &str, port: Option<u16>) -> bool {
    if rules.is_empty() {
        return false;
    }

    // First pass: check for explicit blocks (highest priority)
    for rule in rules {
        if let NetworkRule::BlockHost {
            host: blocked_host,
            port: blocked_port,
        } = rule
        {
            if host_matches(host, blocked_host) && port_matches(port, *blocked_port) {
                return false;
            }
        }
    }

    // Second pass: check for allows
    for rule in rules {
        match rule {
            NetworkRule::AllowAll => return true,
            NetworkRule::AllowHost {
                host: allowed_host,
                port: allowed_port,
            } => {
                if host_matches(host, allowed_host) && port_matches(port, *allowed_port) {
                    return true;
                }
            }
            NetworkRule::AllowCIDR(cidr) => {
                if matches_cidr(host, cidr) {
                    return true;
                }
            }
            NetworkRule::BlockHost { .. } => {
                // Already handled in first pass
            }
        }
    }

    false
}

fn host_matches(host: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == host {
        return true;
    }

    // Wildcard domain matching (e.g., "*.example.com")
    // Should match "api.example.com" or "www.example.com" but not "example.com"
    if pattern.starts_with("*.") {
        let domain = &pattern[1..]; // Keep the leading dot: ".example.com"
                                    // Host must end with ".example.com" and have at least one char before it
        host.ends_with(domain) && host.len() > domain.len()
    } else {
        host == pattern
    }
}

fn port_matches(port: Option<u16>, pattern: Option<u16>) -> bool {
    match (port, pattern) {
        (_, None) => true,        // Pattern allows any port
        (None, Some(_)) => false, // No port specified but pattern requires one
        (Some(p), Some(pat)) => p == pat,
    }
}

fn matches_cidr(host: &str, cidr: &str) -> bool {
    // Parse CIDR notation
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let Ok(network_addr) = parts[0].parse::<IpAddr>() else {
        return false;
    };
    let Ok(prefix_len) = parts[1].parse::<u8>() else {
        return false;
    };
    let Ok(host_addr) = host.parse::<IpAddr>() else {
        return false;
    };

    // Simple CIDR matching for IPv4
    match (network_addr, host_addr) {
        (IpAddr::V4(net), IpAddr::V4(host)) => {
            let net_bits = u32::from(net);
            let host_bits = u32::from(host);
            let mask = !((1u32 << (32 - prefix_len)) - 1);
            (net_bits & mask) == (host_bits & mask)
        }
        (IpAddr::V6(net), IpAddr::V6(host)) => {
            let net_bits = u128::from(net);
            let host_bits = u128::from(host);
            let mask = !((1u128 << (128 - prefix_len)) - 1);
            (net_bits & mask) == (host_bits & mask)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all() {
        let rules = vec![NetworkRule::AllowAll];
        assert!(check_network_access(&rules, "example.com", Some(80)));
    }

    #[test]
    fn test_specific_host() {
        let rules = vec![NetworkRule::AllowHost {
            host: "example.com".to_string(),
            port: Some(443),
        }];
        assert!(check_network_access(&rules, "example.com", Some(443)));
        assert!(!check_network_access(&rules, "example.com", Some(80)));
    }

    #[test]
    fn test_wildcard_domain() {
        let rules = vec![NetworkRule::AllowHost {
            host: "*.example.com".to_string(),
            port: None,
        }];
        assert!(check_network_access(&rules, "api.example.com", Some(443)));
        assert!(!check_network_access(&rules, "other.com", Some(443)));
    }

    #[test]
    fn test_cidr_matching() {
        let rules = vec![NetworkRule::AllowCIDR("192.168.1.0/24".to_string())];
        assert!(check_network_access(&rules, "192.168.1.100", None));
        assert!(!check_network_access(&rules, "192.168.2.100", None));
    }
}
