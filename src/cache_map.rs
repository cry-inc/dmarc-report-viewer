use anyhow::{ensure, Result};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::SystemTime;

/// Very simple map for caching data.
/// Cached values are identified by a unique key.
/// The cache only keeps up to `max_size` entries.
/// When inserting new entries, the oldest entry
/// is deleted if `max_size` was already reached.
pub struct CacheMap<K, V> {
    map: HashMap<K, Entry<V>>,
    max_size: usize,
}

struct Entry<T> {
    pub inserted: SystemTime,
    pub value: T,
}

impl<K, V> CacheMap<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(max_size: usize) -> Result<Self> {
        ensure!(max_size >= 1, "max_size needs to be one or bigger");
        Ok(Self {
            map: HashMap::new(),
            max_size,
        })
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|e| &e.value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.map.len() >= self.max_size {
            self.prune();
        }
        let entry = Entry {
            inserted: SystemTime::now(),
            value,
        };
        self.map.insert(key, entry);
    }

    fn prune(&mut self) {
        let oldest = self
            .map
            .iter()
            .min_by(|a, b| a.1.inserted.cmp(&b.1.inserted))
            .map(|m| m.0)
            .cloned();
        if let Some(oldest) = &oldest {
            self.map.remove(oldest);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CacheMap;

    #[test]
    fn basic() {
        let mut cache = CacheMap::new(1).unwrap();

        assert!(cache.get(&1).is_none());
        cache.insert(1, 23);
        assert_eq!(cache.get(&1), Some(&23));

        cache.insert(1, 42);
        assert_eq!(cache.get(&1), Some(&42));

        cache.insert(2, 666);
        assert_eq!(cache.get(&2), Some(&666));
        assert!(cache.get(&1).is_none());
    }

    #[test]
    fn invalid_size() {
        assert!(CacheMap::<i32, i32>::new(0).is_err());
    }

    #[test]
    fn pruning() {
        let mut cache = CacheMap::new(3).unwrap();

        cache.insert(1, 1);
        cache.insert(2, 2);
        cache.insert(3, 3);
        assert_eq!(cache.get(&1), Some(&1));
        assert_eq!(cache.get(&2), Some(&2));
        assert_eq!(cache.get(&3), Some(&3));

        cache.insert(4, 4);
        cache.insert(5, 5);
        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert_eq!(cache.get(&3), Some(&3));
        assert_eq!(cache.get(&4), Some(&4));
        assert_eq!(cache.get(&5), Some(&5));
    }

    #[test]
    fn replacing() {
        let mut cache = CacheMap::new(1).unwrap();

        cache.insert(1, 1);
        assert_eq!(cache.get(&1), Some(&1));
        cache.insert(1, 2);
        assert_eq!(cache.get(&1), Some(&2));
        cache.insert(1, 3);
        assert_eq!(cache.get(&1), Some(&3));
    }
}
