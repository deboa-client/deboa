use crate::cache::DeboaCache;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default, Clone)]
struct MemoryCache {
    inner: Arc<Mutex<HashMap<String, String>>>,
}

impl DeboaCache for MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.inner
            .lock()
            .unwrap()
            .get(key)
            .cloned()
    }

    fn set(&self, key: &str, value: &str) {
        self.inner
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }

    fn delete(&self, key: &str) {
        self.inner
            .lock()
            .unwrap()
            .remove(key);
    }
}

#[test]
fn test_cache_put() {
    let cache = MemoryCache::default();

    cache.set("session", "abc123");

    assert_eq!(cache.get("session"), Some("abc123".to_string()));
}

#[test]
fn test_cache_get() {
    let cache = MemoryCache::default();
    cache.set("token", "secret");

    assert_eq!(cache.get("token"), Some("secret".to_string()));
    assert_eq!(cache.get("missing"), None);
}

#[test]
fn test_cache_remove() {
    let cache = MemoryCache::default();
    cache.set("user", "alice");

    cache.delete("user");

    assert_eq!(cache.get("user"), None);
}
