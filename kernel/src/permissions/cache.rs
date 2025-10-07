/*!
 * Permission Cache
 * Simple LRU cache for permission check results
 */

use super::types::{PermissionRequest, PermissionResponse, Resource};
use crate::core::types::Pid;
use dashmap::DashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

/// Cache key for permission lookups
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    pid: Pid,
    resource_hash: u64,
    action: super::types::Action,
}

impl CacheKey {
    fn new(pid: Pid, resource: &Resource, action: super::types::Action) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        match resource {
            Resource::File(p) => p.hash(&mut hasher),
            Resource::Directory(p) => p.hash(&mut hasher),
            Resource::Network { host, port } => {
                host.hash(&mut hasher);
                port.hash(&mut hasher);
            }
            Resource::IpcChannel(id) => id.hash(&mut hasher),
            Resource::Process(pid) => pid.hash(&mut hasher),
            Resource::System(s) => s.hash(&mut hasher),
        }

        Self {
            pid,
            resource_hash: hasher.finish(),
            action,
        }
    }
}

/// Cached permission decision
struct CachedDecision {
    response: PermissionResponse,
    expires_at: SystemTime,
}

/// Permission cache with LRU eviction
pub struct PermissionCache {
    cache: DashMap<CacheKey, CachedDecision>,
    max_size: usize,
    ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl PermissionCache {
    /// Create new cache
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: DashMap::with_capacity(max_size),
            max_size,
            ttl,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Get cached decision
    pub fn get(&self, request: &PermissionRequest) -> Option<PermissionResponse> {
        let key = CacheKey::new(request.pid, &request.resource, request.action);

        if let Some(entry) = self.cache.get(&key) {
            let now = SystemTime::now();
            if entry.expires_at > now {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.response.clone().with_cached(true));
            } else {
                // Expired, remove it
                drop(entry);
                self.cache.remove(&key);
            }
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Store decision in cache
    pub fn put(&self, request: PermissionRequest, response: PermissionResponse) {
        // Simple size limit - remove random entry if full
        if self.cache.len() >= self.max_size {
            if let Some(entry) = self.cache.iter().next() {
                let key = entry.key().clone();
                drop(entry);
                self.cache.remove(&key);
            }
        }

        let key = CacheKey::new(request.pid, &request.resource, request.action);
        let expires_at = SystemTime::now() + self.ttl;

        self.cache.insert(
            key,
            CachedDecision {
                response,
                expires_at,
            },
        );
    }

    /// Clear all cached decisions for a PID
    pub fn invalidate_pid(&self, pid: Pid) {
        self.cache.retain(|k, _| k.pid != pid);
    }

    /// Clear entire cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            hits,
            misses,
            hit_rate,
        }
    }
}

impl Default for PermissionCache {
    fn default() -> Self {
        // 10K entries, 5 second TTL
        Self::new(10_000, Duration::from_secs(5))
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cache_hit() {
        let cache = PermissionCache::new(100, Duration::from_secs(10));
        let req = PermissionRequest::file_read(100, PathBuf::from("/test"));
        let resp = PermissionResponse::allow(req.clone(), "test");

        cache.put(req.clone(), resp.clone());

        let cached = cache.get(&req);
        assert!(cached.is_some());
        assert!(cached.unwrap().cached);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss() {
        let cache = PermissionCache::new(100, Duration::from_secs(10));
        let req = PermissionRequest::file_read(100, PathBuf::from("/test"));

        let cached = cache.get(&req);
        assert!(cached.is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_expiry() {
        let cache = PermissionCache::new(100, Duration::from_millis(10));
        let req = PermissionRequest::file_read(100, PathBuf::from("/test"));
        let resp = PermissionResponse::allow(req.clone(), "test");

        cache.put(req.clone(), resp);

        // Sleep to let it expire
        std::thread::sleep(Duration::from_millis(20));

        let cached = cache.get(&req);
        assert!(cached.is_none());
    }

    #[test]
    fn test_invalidate_pid() {
        let cache = PermissionCache::new(100, Duration::from_secs(10));
        let req1 = PermissionRequest::file_read(100, PathBuf::from("/test1"));
        let req2 = PermissionRequest::file_read(200, PathBuf::from("/test2"));

        cache.put(req1.clone(), PermissionResponse::allow(req1.clone(), "test"));
        cache.put(req2.clone(), PermissionResponse::allow(req2.clone(), "test"));

        cache.invalidate_pid(100);

        assert!(cache.get(&req1).is_none());
        assert!(cache.get(&req2).is_some());
    }
}
