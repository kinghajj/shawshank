use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// The interface for the key-value map internal to a [`ArenaSet`].
///
/// The Entry API is not supported, because it can't be used as is, anyway:
/// the reference passed as a key to `entry(K)` would be to something outside
/// the arena_set, which we absolutely don't want to store in the map. The Entry
/// API would have to be extended to allow changing the key before insertion.
///
/// [`ArenaSet`]: struct.ArenaSet.html
pub trait Map {
    type Key;
    type Value;

    /// Create an empty map.
    ///
    /// This is required for `ArenaSet` to function properly.
    fn new() -> Self;

    /// Create an empty map with a capacity hint.
    ///
    /// Not all implementations may support this, making it equivalent to
    /// `Map::new`.
    fn with_capacity(usize) -> Self;

    /// Get the number of pairs in the map.
    fn len(&self) -> usize;

    /// Insert a key-value pair. If there was already an entry for the key,
    /// it gets replaced, and the previous returned.
    ///
    /// This is required for `ArenaSet` to function properly.
    fn insert(&mut self, Self::Key, Self::Value) -> Option<Self::Value>;

    /// Get a value by its key.
    ///
    /// This is required for `ArenaSet` to function properly.
    fn get(&self, Self::Key) -> Option<&Self::Value>;

    /// Remove a pair by its key.
    ///
    /// This is required for `ArenaSet` to function properly.
    fn remove(&mut self, Self::Key) -> Option<Self::Value>;

    /// Reduce memory usage as much as possible.
    ///
    /// Not all implementations may support this, making it a no-op.
    fn shrink_to_fit(&mut self);
}

impl<K: Eq + Hash, V> Map for HashMap<K, V> {
    type Key = K;
    type Value = V;

    fn new() -> Self {
        HashMap::new()
    }

    fn with_capacity(capacity: usize) -> Self {
        HashMap::with_capacity(capacity)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.insert(k, v)
    }

    fn get(&self, k: K) -> Option<&V> {
        self.get(&k)
    }

    fn remove(&mut self, k: K) -> Option<V> {
        self.remove(&k)
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit();
    }
}

impl<K: Eq + Ord, V> Map for BTreeMap<K, V> {
    type Key = K;
    type Value = V;

    fn new() -> Self {
        BTreeMap::new()
    }

    fn with_capacity(_: usize) -> Self {
        BTreeMap::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.insert(k, v)
    }

    fn get(&self, k: K) -> Option<&V> {
        self.get(&k)
    }

    fn remove(&mut self, k: K) -> Option<V> {
        self.remove(&k)
    }

    fn shrink_to_fit(&mut self) {}
}
