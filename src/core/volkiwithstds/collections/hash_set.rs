//! HashSet<T> — wraps HashMap<T, ()>.

use super::hash_map::HashMap;
use core::hash::Hash;

/// A hash set backed by HashMap<T, ()>.
pub struct HashSet<T: Hash + Eq> {
    map: HashMap<T, ()>,
}

impl<T: Hash + Eq> HashSet<T> {
    /// Creates an empty HashSet.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a HashSet with pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            map: HashMap::with_capacity(cap),
        }
    }

    /// Insert a value. Returns true if the value was not already present.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Check if the set contains a value.
    pub fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    /// Remove a value. Returns true if it was present.
    pub fn remove(&mut self, value: &T) -> bool {
        self.map.remove(value).is_some()
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.map.keys(),
        }
    }

    /// Returns elements in self but not in other.
    pub fn difference<'a>(&'a self, other: &'a HashSet<T>) -> Difference<'a, T> {
        Difference {
            iter: self.iter(),
            other,
        }
    }

    /// Returns elements in both self and other.
    pub fn intersection<'a>(&'a self, other: &'a HashSet<T>) -> Intersection<'a, T> {
        Intersection {
            iter: self.iter(),
            other,
        }
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

// ── Iterators ───────────────────────────────────────────────────────────────

pub struct Iter<'a, T> {
    inner: super::hash_map::Keys<'a, T, ()>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct Difference<'a, T: Hash + Eq> {
    iter: Iter<'a, T>,
    other: &'a HashSet<T>,
}

impl<'a, T: Hash + Eq> Iterator for Difference<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.iter.next()?;
            if !self.other.contains(item) {
                return Some(item);
            }
        }
    }
}

pub struct Intersection<'a, T: Hash + Eq> {
    iter: Iter<'a, T>,
    other: &'a HashSet<T>,
}

impl<'a, T: Hash + Eq> Iterator for Intersection<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.iter.next()?;
            if self.other.contains(item) {
                return Some(item);
            }
        }
    }
}

// ── IntoIterator ────────────────────────────────────────────────────────────

pub struct IntoIter<T> {
    inner: super::hash_map::IntoIter<T, ()>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| k)
    }
}

impl<T: Hash + Eq> IntoIterator for HashSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: self.map.into_iter(),
        }
    }
}

impl<'a, T: Hash + Eq> IntoIterator for &'a HashSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

// ── Trait impls ─────────────────────────────────────────────────────────────

impl<T: Hash + Eq> Default for HashSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq> FromIterator<T> for HashSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = HashSet::new();
        for item in iter {
            set.insert(item);
        }
        set
    }
}

impl<T: Hash + Eq> Extend<T> for HashSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<T: Hash + Eq + core::fmt::Debug> core::fmt::Debug for HashSet<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T: Hash + Eq + Clone> Clone for HashSet<T> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_contains() {
        let mut s = HashSet::new();
        assert!(s.insert(1));
        assert!(s.insert(2));
        assert!(!s.insert(1));
        assert!(s.contains(&1));
        assert!(s.contains(&2));
        assert!(!s.contains(&3));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn test_difference() {
        let a: HashSet<i32> = [1, 2, 3].iter().copied().collect();
        let b: HashSet<i32> = [2, 3, 4].iter().copied().collect();
        let diff: HashSet<i32> = a.difference(&b).copied().collect();
        assert!(diff.contains(&1));
        assert!(!diff.contains(&2));
        assert_eq!(diff.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut s = HashSet::new();
        s.insert("a");
        s.insert("b");
        assert!(s.remove(&"a"));
        assert!(!s.contains(&"a"));
        assert_eq!(s.len(), 1);
    }
}
