//! HashMap<K,V> — Robin Hood open addressing with SipHash-1-3.

use super::hash::SipBuildHasher;
use super::vec::Vec;
use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash, Hasher};
use core::mem;

const MIN_CAPACITY: usize = 8;
const LOAD_FACTOR_NUM: usize = 7;
const LOAD_FACTOR_DEN: usize = 8;

/// An entry in the hash table.
enum Bucket<K, V> {
    Empty,
    Occupied { key: K, value: V, hash: u64 },
    // Tombstone for deletions (simplifies Robin Hood)
    Tombstone,
}

impl<K, V> Bucket<K, V> {
    fn is_empty_or_tombstone(&self) -> bool {
        matches!(self, Bucket::Empty | Bucket::Tombstone)
    }
}

/// A hash map using Robin Hood open addressing.
pub struct HashMap<K, V> {
    buckets: Vec<Bucket<K, V>>,
    len: usize,
    hasher_builder: SipBuildHasher,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    /// Creates an empty HashMap.
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            len: 0,
            hasher_builder: SipBuildHasher::new(),
        }
    }

    /// Creates a HashMap with pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.max(MIN_CAPACITY).next_power_of_two();
        let mut buckets = Vec::with_capacity(cap);
        for _ in 0..cap {
            buckets.push(Bucket::Empty);
        }
        Self {
            buckets,
            len: 0,
            hasher_builder: SipBuildHasher::new(),
        }
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn make_hash<Q: Hash + ?Sized>(&self, key: &Q) -> u64 {
        let mut hasher = self.hasher_builder.build_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn bucket_index(&self, hash: u64) -> usize {
        (hash as usize) & (self.buckets.len() - 1)
    }

    fn probe_distance(&self, hash: u64, current: usize) -> usize {
        let ideal = self.bucket_index(hash);
        if current >= ideal {
            current - ideal
        } else {
            self.buckets.len() - ideal + current
        }
    }

    fn should_grow(&self) -> bool {
        if self.buckets.is_empty() {
            return true;
        }
        self.len * LOAD_FACTOR_DEN >= self.buckets.len() * LOAD_FACTOR_NUM
    }

    fn grow(&mut self) {
        let new_cap = if self.buckets.is_empty() {
            MIN_CAPACITY
        } else {
            self.buckets.len() * 2
        };

        let mut new_buckets = Vec::with_capacity(new_cap);
        for _ in 0..new_cap {
            new_buckets.push(Bucket::Empty);
        }

        let old_buckets = mem::replace(&mut self.buckets, new_buckets);
        let old_len = self.len;
        self.len = 0;

        for bucket in old_buckets.into_iter() {
            if let Bucket::Occupied { key, value, .. } = bucket {
                self.insert_no_grow(key, value);
            }
        }

        debug_assert_eq!(self.len, old_len);
    }

    /// Insert without growing. Returns (old_value, index_of_inserted_entry).
    fn insert_no_grow(&mut self, key: K, value: V) -> (Option<V>, usize) {
        let hash = self.make_hash(&key);
        let mask = self.buckets.len() - 1;
        let mut idx = (hash as usize) & mask;
        let mut dist = 0;
        let mut ins = Bucket::Occupied { key, value, hash };
        // Track where the original entry lands (may move due to Robin Hood).
        let mut inserted_idx = usize::MAX;

        loop {
            if self.buckets[idx].is_empty_or_tombstone() {
                if inserted_idx == usize::MAX {
                    inserted_idx = idx;
                }
                self.buckets[idx] = ins;
                self.len += 1;
                return (None, inserted_idx);
            }

            // Check if existing key matches
            let (existing_hash, keys_equal) = match (&self.buckets[idx], &ins) {
                (
                    Bucket::Occupied {
                        hash: eh, key: ek, ..
                    },
                    Bucket::Occupied {
                        hash: ih, key: ik, ..
                    },
                ) => (*eh, *eh == *ih && *ek == *ik),
                _ => unreachable!(),
            };

            if keys_equal {
                // Replace: swap in the new bucket, extract old value
                let old = mem::replace(&mut self.buckets[idx], ins);
                if let Bucket::Occupied { value: old_val, .. } = old {
                    return (Some(old_val), idx);
                }
                unreachable!();
            }

            let existing_dist = self.probe_distance(existing_hash, idx);
            if dist > existing_dist {
                // Robin Hood: steal this spot — our entry lands here
                if inserted_idx == usize::MAX {
                    inserted_idx = idx;
                }
                ins = mem::replace(&mut self.buckets[idx], ins);
                dist = existing_dist;
            }

            idx = (idx + 1) & mask;
            dist += 1;
        }
    }

    /// Insert a key-value pair. Returns the old value if the key existed.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.should_grow() {
            self.grow();
        }
        self.insert_no_grow(key, value).0
    }

    /// Get a reference to the value for a key.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.buckets.is_empty() {
            return None;
        }
        let hash = self.make_hash(key);
        let mut idx = self.bucket_index(hash);
        let mut dist = 0;

        loop {
            match &self.buckets[idx] {
                Bucket::Empty => return None,
                Bucket::Tombstone => {}
                Bucket::Occupied {
                    key: k,
                    value: v,
                    hash: h,
                } => {
                    if *h == hash && k.borrow() == key {
                        return Some(v);
                    }
                    let d = self.probe_distance(*h, idx);
                    if dist > d {
                        return None;
                    }
                }
            }
            idx = (idx + 1) & (self.buckets.len() - 1);
            dist += 1;
        }
    }

    /// Get a mutable reference to the value for a key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let found_idx = self.find_index(key);
        match found_idx {
            Some(idx) => {
                if let Bucket::Occupied { value, .. } = &mut self.buckets[idx] {
                    Some(value)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Internal: find the bucket index for a key.
    fn find_index<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.buckets.is_empty() {
            return None;
        }
        let hash = self.make_hash(key);
        let mut idx = self.bucket_index(hash);
        let mut dist = 0;

        loop {
            match &self.buckets[idx] {
                Bucket::Empty => return None,
                Bucket::Tombstone => {}
                Bucket::Occupied {
                    key: k, hash: h, ..
                } => {
                    if *h == hash && k.borrow() == key {
                        return Some(idx);
                    }
                    let d = self.probe_distance(*h, idx);
                    if dist > d {
                        return None;
                    }
                }
            }
            idx = (idx + 1) & (self.buckets.len() - 1);
            dist += 1;
        }
    }

    /// Check if a key exists.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Remove a key, returning its value.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let found_idx = self.find_index(key);
        match found_idx {
            Some(idx) => {
                let old = mem::replace(&mut self.buckets[idx], Bucket::Tombstone);
                self.len -= 1;
                if let Bucket::Occupied { value, .. } = old {
                    Some(value)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Entry API.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        if self.should_grow() {
            self.grow();
        }
        let hash = self.make_hash(&key);
        let mut idx = self.bucket_index(hash);
        let mut dist = 0;

        loop {
            match &self.buckets[idx] {
                Bucket::Empty | Bucket::Tombstone => {
                    return Entry::Vacant(VacantEntry {
                        map: self,
                        key,
                        hash,
                    });
                }
                Bucket::Occupied {
                    key: k, hash: h, ..
                } => {
                    if *h == hash && *k == key {
                        return Entry::Occupied(OccupiedEntry { map: self, idx });
                    }
                    let d = self.probe_distance(*h, idx);
                    if dist > d {
                        return Entry::Vacant(VacantEntry {
                            map: self,
                            key,
                            hash,
                        });
                    }
                }
            }
            idx = (idx + 1) & (self.buckets.len() - 1);
            dist += 1;
        }
    }

    /// Returns an iterator over keys.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { iter: self.iter() }
    }

    /// Returns an iterator over values.
    pub fn values(&self) -> Values<'_, K, V> {
        Values { iter: self.iter() }
    }

    /// Returns an iterator over key-value pairs.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: &self.buckets,
            idx: 0,
        }
    }

    /// Returns a mutable iterator over key-value pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            buckets: &mut self.buckets,
            idx: 0,
        }
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        for bucket in self.buckets.iter_mut() {
            *bucket = Bucket::Empty;
        }
        self.len = 0;
    }
}

// ── Entry API ───────────────────────────────────────────────────────────────

pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

pub struct OccupiedEntry<'a, K, V> {
    map: &'a mut HashMap<K, V>,
    idx: usize,
}

pub struct VacantEntry<'a, K, V> {
    map: &'a mut HashMap<K, V>,
    key: K,
    hash: u64,
}

impl<'a, K: Hash + Eq, V> Entry<'a, K, V> {
    /// Get the value or insert a default.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Get the value or insert with a closure.
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(f()),
        }
    }

    /// Get the value or insert the default value.
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        }
    }

    /// Modify an existing value in place.
    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K: Hash + Eq, V> OccupiedEntry<'a, K, V> {
    pub fn get(&self) -> &V {
        if let Bucket::Occupied { value, .. } = &self.map.buckets[self.idx] {
            value
        } else {
            unreachable!()
        }
    }

    pub fn get_mut(&mut self) -> &mut V {
        if let Bucket::Occupied { value, .. } = &mut self.map.buckets[self.idx] {
            value
        } else {
            unreachable!()
        }
    }

    pub fn into_mut(self) -> &'a mut V {
        if let Bucket::Occupied { value, .. } = &mut self.map.buckets[self.idx] {
            value
        } else {
            unreachable!()
        }
    }

    pub fn insert(&mut self, new_value: V) -> V {
        if let Bucket::Occupied { value, .. } = &mut self.map.buckets[self.idx] {
            mem::replace(value, new_value)
        } else {
            unreachable!()
        }
    }

    pub fn remove(self) -> V {
        let old = mem::replace(&mut self.map.buckets[self.idx], Bucket::Tombstone);
        self.map.len -= 1;
        if let Bucket::Occupied { value, .. } = old {
            value
        } else {
            unreachable!()
        }
    }
}

impl<'a, K: Hash + Eq, V> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V) -> &'a mut V {
        let (_, idx) = self.map.insert_no_grow(self.key, value);
        if let Bucket::Occupied { value, .. } = &mut self.map.buckets[idx] {
            value
        } else {
            unreachable!()
        }
    }

    pub fn key(&self) -> &K {
        &self.key
    }
}

// ── Iterators ───────────────────────────────────────────────────────────────

pub struct Iter<'a, K, V> {
    buckets: &'a Vec<Bucket<K, V>>,
    idx: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.buckets.len() {
            let i = self.idx;
            self.idx += 1;
            if let Bucket::Occupied { key, value, .. } = &self.buckets[i] {
                return Some((key, value));
            }
        }
        None
    }
}

pub struct IterMut<'a, K, V> {
    buckets: &'a mut Vec<Bucket<K, V>>,
    idx: usize,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.buckets.len() {
            let i = self.idx;
            self.idx += 1;
            if let Bucket::Occupied { key, value, .. } = &mut self.buckets[i] {
                // Safety: we never yield the same index twice, so aliasing is fine
                let key_ref: &'a K = unsafe { &*(key as *const K) };
                let val_ref: &'a mut V = unsafe { &mut *(value as *mut V) };
                return Some((key_ref, val_ref));
            }
        }
        None
    }
}

pub struct Keys<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

pub struct Values<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}

/// Owning iterator.
pub struct IntoIter<K, V> {
    inner: super::vec::IntoIter<Bucket<K, V>>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                None => return None,
                Some(Bucket::Occupied { key, value, .. }) => return Some((key, value)),
                Some(_) => continue,
            }
        }
    }
}

impl<K: Hash + Eq, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            inner: self.buckets.into_iter(),
        }
    }
}

impl<'a, K: Hash + Eq, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

// ── Trait impls ─────────────────────────────────────────────────────────────

impl<K: Hash + Eq, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq, V> FromIterator<(K, V)> for HashMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = HashMap::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

impl<K: Hash + Eq + core::fmt::Debug, V: core::fmt::Debug> core::fmt::Debug for HashMap<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Clone for HashMap<K, V> {
    fn clone(&self) -> Self {
        let mut new = HashMap::with_capacity(self.len());
        for (k, v) in self.iter() {
            new.insert(k.clone(), v.clone());
        }
        new
    }
}

impl<K: Hash + Eq, V> core::ops::Index<&K> for HashMap<K, V> {
    type Output = V;
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K: Hash + Eq, V> Extend<(K, V)> for HashMap<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut m = HashMap::new();
        m.insert("key1", 1);
        m.insert("key2", 2);
        assert_eq!(m.get(&"key1"), Some(&1));
        assert_eq!(m.get(&"key2"), Some(&2));
        assert_eq!(m.get(&"key3"), None);
    }

    #[test]
    fn test_overwrite() {
        let mut m = HashMap::new();
        m.insert("a", 1);
        let old = m.insert("a", 2);
        assert_eq!(old, Some(1));
        assert_eq!(m.get(&"a"), Some(&2));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut m = HashMap::new();
        m.insert(1, "a");
        m.insert(2, "b");
        assert_eq!(m.remove(&1), Some("a"));
        assert_eq!(m.get(&1), None);
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_entry_or_default() {
        let mut m: HashMap<&str, i32> = HashMap::new();
        *m.entry("counter").or_default() += 1;
        *m.entry("counter").or_default() += 1;
        assert_eq!(m.get(&"counter"), Some(&2));
    }

    #[test]
    fn test_entry_or_insert_with() {
        let mut m = HashMap::new();
        m.entry("key").or_insert_with(|| 42);
        assert_eq!(m.get(&"key"), Some(&42));
    }

    #[test]
    fn test_grow() {
        let mut m = HashMap::new();
        for i in 0..100 {
            m.insert(i, i * 2);
        }
        assert_eq!(m.len(), 100);
        for i in 0..100 {
            assert_eq!(m.get(&i), Some(&(i * 2)));
        }
    }

    #[test]
    fn test_from_iter() {
        let m: HashMap<i32, i32> = [(1, 10), (2, 20), (3, 30)].iter().copied().collect();
        assert_eq!(m.len(), 3);
        assert_eq!(m[&2], 20);
    }
}
