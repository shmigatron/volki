//! Vec<T> — growable array.

use super::raw_vec::RawVec;
use core::fmt;
use core::mem;
use core::ops::{
    Bound, Deref, DerefMut, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull,
    RangeInclusive, RangeTo, RangeToInclusive,
};
use core::ptr;
use core::slice;

/// A growable, heap-allocated array.
pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> Vec<T> {
    /// Creates an empty Vec.
    pub const fn new() -> Self {
        Self {
            buf: RawVec::new(),
            len: 0,
        }
    }

    /// Creates a Vec with pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: RawVec::with_capacity(cap),
            len: 0,
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the allocated capacity.
    pub fn capacity(&self) -> usize {
        self.buf.cap()
    }

    /// Push an element onto the end.
    pub fn push(&mut self, value: T) {
        if self.len == self.buf.cap() {
            self.buf.grow(self.len + 1);
        }
        unsafe {
            ptr::write(self.buf.ptr().add(self.len), value);
        }
        self.len += 1;
    }

    /// Pop the last element.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.buf.ptr().add(self.len))) }
        }
    }

    /// Get a reference to element at index.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.buf.ptr().add(index)) }
        } else {
            None
        }
    }

    /// Get a mutable reference to element at index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.buf.ptr().add(index)) }
        } else {
            None
        }
    }

    /// Returns a slice of the contents.
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Returns a mutable slice of the contents.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    /// Returns a raw pointer to the buffer.
    pub fn as_ptr(&self) -> *const T {
        self.buf.ptr()
    }

    /// Returns a mutable raw pointer to the buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.ptr()
    }

    /// Insert an element at position `index`, shifting elements after it.
    pub fn insert(&mut self, index: usize, element: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.len == self.buf.cap() {
            self.buf.grow(self.len + 1);
        }
        unsafe {
            let p = self.buf.ptr().add(index);
            if index < self.len {
                ptr::copy(p, p.add(1), self.len - index);
            }
            ptr::write(p, element);
        }
        self.len += 1;
    }

    /// Remove and return element at `index`, shifting elements after it.
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        self.len -= 1;
        unsafe {
            let p = self.buf.ptr().add(index);
            let val = ptr::read(p);
            ptr::copy(p.add(1), p, self.len - index);
            val
        }
    }

    /// Retain only elements for which the predicate returns true.
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, mut f: F) {
        let mut i = 0;
        while i < self.len {
            if !f(unsafe { &*self.buf.ptr().add(i) }) {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Truncate to `len` elements, dropping the rest.
    pub fn truncate(&mut self, len: usize) {
        while self.len > len {
            self.pop();
        }
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Extend from a slice of copyable elements.
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        for item in other {
            self.push(item.clone());
        }
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Self) {
        if other.len == 0 {
            return;
        }

        let new_len = self
            .len
            .checked_add(other.len)
            .expect("length overflow in Vec::append");

        if new_len > self.buf.cap() {
            self.buf.grow(new_len);
        }

        unsafe {
            ptr::copy_nonoverlapping(other.buf.ptr(), self.buf.ptr().add(self.len), other.len);
        }

        self.len = new_len;
        other.len = 0;
    }

    /// Returns an iterator over references.
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns an iterator over mutable references.
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Sort (stable) — insertion sort for small, merge sort for large.
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.sort_by(|a, b| a.cmp(b));
    }

    /// Sort with a custom comparison function.
    pub fn sort_by<F: FnMut(&T, &T) -> core::cmp::Ordering>(&mut self, mut compare: F) {
        let len = self.len;
        if len <= 1 {
            return;
        }
        if len <= 32 {
            // Insertion sort for small arrays
            for i in 1..len {
                let mut j = i;
                while j > 0
                    && compare(unsafe { &*self.buf.ptr().add(j - 1) }, unsafe {
                        &*self.buf.ptr().add(j)
                    }) == core::cmp::Ordering::Greater
                {
                    unsafe {
                        let a = self.buf.ptr().add(j - 1);
                        let b = self.buf.ptr().add(j);
                        ptr::swap(a, b);
                    }
                    j -= 1;
                }
            }
        } else {
            // Simple in-place merge sort
            merge_sort(self.as_mut_slice(), &mut compare);
        }
    }

    /// Sort by a key extraction function.
    pub fn sort_by_key<K: Ord, F: FnMut(&T) -> K>(&mut self, mut f: F) {
        self.sort_by(|a, b| f(a).cmp(&f(b)));
    }

    /// Dedup consecutive equal elements.
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        if self.len <= 1 {
            return;
        }
        let mut write = 1;
        for read in 1..self.len {
            let eq = unsafe {
                let a = &*self.buf.ptr().add(write - 1);
                let b = &*self.buf.ptr().add(read);
                a == b
            };
            if !eq {
                if write != read {
                    unsafe {
                        let src = self.buf.ptr().add(read);
                        let dst = self.buf.ptr().add(write);
                        ptr::copy_nonoverlapping(src, dst, 1);
                    }
                }
                write += 1;
            } else {
                unsafe {
                    ptr::drop_in_place(self.buf.ptr().add(read));
                }
            }
        }
        self.len = write;
    }

    /// Dedup consecutive elements using a custom equality predicate.
    pub fn dedup_by<F>(&mut self, mut same_bucket: F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        if self.len <= 1 {
            return;
        }

        let mut write = 1;
        for read in 1..self.len {
            let is_dup = unsafe {
                let a = &*self.buf.ptr().add(write - 1);
                let b = &*self.buf.ptr().add(read);
                same_bucket(a, b)
            };

            if !is_dup {
                if write != read {
                    unsafe {
                        let src = self.buf.ptr().add(read);
                        let dst = self.buf.ptr().add(write);
                        ptr::copy_nonoverlapping(src, dst, 1);
                    }
                }
                write += 1;
            } else {
                unsafe {
                    ptr::drop_in_place(self.buf.ptr().add(read));
                }
            }
        }

        self.len = write;
    }

    /// Join string slices with a separator.
    pub fn join(&self, sep: &str) -> super::string::String
    where
        T: AsRef<str>,
    {
        let mut result = super::string::String::new();
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                result.push_str(sep);
            }
            result.push_str(item.as_ref());
        }
        result
    }

    /// Returns true if the vec contains the given value.
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|e| e == x)
    }

    /// Set the length of the vector.
    ///
    /// # Safety
    /// The caller must ensure that `new_len` elements are properly initialized.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    /// Reserve capacity for at least `additional` more elements.
    pub fn reserve(&mut self, additional: usize) {
        let required = self.len + additional;
        if required > self.buf.cap() {
            self.buf.grow(required);
        }
    }

    /// Swap remove — O(1) removal by swapping with last element.
    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        let last = self.len - 1;
        if index != last {
            unsafe {
                let a = self.buf.ptr().add(index);
                let b = self.buf.ptr().add(last);
                ptr::swap(a, b);
            }
        }
        self.pop().unwrap()
    }

    /// Remove the specified range and return an iterator over removed items.
    pub fn drain<R>(&mut self, range: R) -> IntoIter<T>
    where
        R: RangeBounds<usize>,
    {
        let len = self.len;

        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n
                .checked_add(1)
                .expect("range start overflow in Vec::drain"),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n.checked_add(1).expect("range end overflow in Vec::drain"),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        assert!(start <= end, "invalid range in Vec::drain");
        assert!(end <= len, "range end out of bounds in Vec::drain");

        let drain_len = end - start;
        let mut removed = Vec::with_capacity(drain_len);

        unsafe {
            for i in 0..drain_len {
                removed.push(ptr::read(self.buf.ptr().add(start + i)));
            }

            let tail_len = len - end;
            if tail_len > 0 {
                ptr::copy(self.buf.ptr().add(end), self.buf.ptr().add(start), tail_len);
            }

            self.len = len - drain_len;
        }

        removed.into_iter()
    }

    /// Replace the specified range with `replace_with`, returning removed items.
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> IntoIter<T>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        let len = self.len;
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n
                .checked_add(1)
                .expect("range start overflow in Vec::splice"),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n.checked_add(1).expect("range end overflow in Vec::splice"),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        assert!(start <= end, "invalid range in Vec::splice");
        assert!(end <= len, "range end out of bounds in Vec::splice");

        let removed: Vec<T> = self.drain(start..end).collect();

        let mut idx = start;
        for item in replace_with {
            self.insert(idx, item);
            idx += 1;
        }

        removed.into_iter()
    }
}

/// Merge sort for slices.
fn merge_sort<T, F: FnMut(&T, &T) -> core::cmp::Ordering>(slice: &mut [T], compare: &mut F) {
    let len = slice.len();
    if len <= 1 {
        return;
    }
    if len <= 32 {
        // Insertion sort for small
        for i in 1..len {
            let mut j = i;
            while j > 0 && compare(&slice[j - 1], &slice[j]) == core::cmp::Ordering::Greater {
                slice.swap(j - 1, j);
                j -= 1;
            }
        }
        return;
    }
    let mid = len / 2;
    merge_sort(&mut slice[..mid], compare);
    merge_sort(&mut slice[mid..], compare);

    // Merge in-place using rotation
    let mut left = 0;
    let mut right = mid;
    while left < right && right < len {
        if compare(&slice[left], &slice[right]) != core::cmp::Ordering::Greater {
            left += 1;
        } else {
            // Rotate slice[left..=right] so that slice[right] moves to slice[left]
            let val_right = right;
            let mut j = right;
            while j > left {
                slice.swap(j, j - 1);
                j -= 1;
            }
            left += 1;
            right += 1;
            let _ = val_right;
        }
    }
}

// ── Trait implementations ───────────────────────────────────────────────────

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.buf.ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.buf.ptr(), self.len) }
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        assert!(index < self.len, "index out of bounds");
        unsafe { &*self.buf.ptr().add(index) }
    }
}

impl<T> Index<Range<usize>> for Vec<T> {
    type Output = [T];
    fn index(&self, index: Range<usize>) -> &[T] {
        &self.as_slice()[index]
    }
}

impl<T> Index<RangeFrom<usize>> for Vec<T> {
    type Output = [T];
    fn index(&self, index: RangeFrom<usize>) -> &[T] {
        &self.as_slice()[index]
    }
}

impl<T> Index<RangeTo<usize>> for Vec<T> {
    type Output = [T];
    fn index(&self, index: RangeTo<usize>) -> &[T] {
        &self.as_slice()[index]
    }
}

impl<T> Index<RangeInclusive<usize>> for Vec<T> {
    type Output = [T];
    fn index(&self, index: RangeInclusive<usize>) -> &[T] {
        &self.as_slice()[index]
    }
}

impl<T> Index<RangeToInclusive<usize>> for Vec<T> {
    type Output = [T];
    fn index(&self, index: RangeToInclusive<usize>) -> &[T] {
        &self.as_slice()[index]
    }
}

impl<T> Index<RangeFull> for Vec<T> {
    type Output = [T];
    fn index(&self, index: RangeFull) -> &[T] {
        &self.as_slice()[index]
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.len, "index out of bounds");
        unsafe { &mut *self.buf.ptr().add(index) }
    }
}

impl<T> IndexMut<Range<usize>> for Vec<T> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut [T] {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> IndexMut<RangeFrom<usize>> for Vec<T> {
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut [T] {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> IndexMut<RangeTo<usize>> for Vec<T> {
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut [T] {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> IndexMut<RangeInclusive<usize>> for Vec<T> {
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut [T] {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> IndexMut<RangeToInclusive<usize>> for Vec<T> {
    fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut [T] {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> IndexMut<RangeFull> for Vec<T> {
    fn index_mut(&mut self, index: RangeFull) -> &mut [T] {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        while self.len > 0 {
            self.pop();
        }
        // RawVec handles dealloc
    }
}

impl<T: Clone> Clone for Vec<T> {
    fn clone(&self) -> Self {
        let mut new = Vec::with_capacity(self.len);
        for item in self.iter() {
            new.push(item.clone());
        }
        new
    }
}

impl<T: fmt::Debug> fmt::Debug for Vec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: PartialEq> PartialEq for Vec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq> Eq for Vec<T> {}

impl<T: core::hash::Hash> core::hash::Hash for Vec<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v = Vec::new();
        for item in iter {
            v.push(item);
        }
        v
    }
}

impl<T> Extend<T> for Vec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

// ── IntoIterator ────────────────────────────────────────────────────────────

/// Owning iterator over Vec<T>.
pub struct IntoIter<T> {
    buf: RawVec<T>,
    start: usize,
    end: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start >= self.end {
            None
        } else {
            let val = unsafe { ptr::read(self.buf.ptr().add(self.start)) };
            self.start += 1;
            Some(val)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end - self.start;
        (remaining, Some(remaining))
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // Drop remaining elements
        while self.start < self.end {
            unsafe {
                ptr::drop_in_place(self.buf.ptr().add(self.start));
            }
            self.start += 1;
        }
        // RawVec handles dealloc on drop
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        let len = self.len;
        // We need to take ownership of buf without running Vec's Drop
        let buf = unsafe { ptr::read(&self.buf) };
        mem::forget(self);
        IntoIter {
            buf,
            start: 0,
            end: len,
        }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

// ── Default ────────────────────────────────────────────────────────────────

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn test_iter() {
        let mut v = Vec::new();
        v.push(10);
        v.push(20);
        v.push(30);
        let collected: Vec<_> = v.iter().copied().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], 10);
        assert_eq!(collected[1], 20);
        assert_eq!(collected[2], 30);
    }

    #[test]
    fn test_sort() {
        let mut v = Vec::new();
        v.push(3);
        v.push(1);
        v.push(2);
        v.sort();
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn test_from_iterator() {
        let v: Vec<i32> = [1, 2, 3].iter().copied().collect();
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
    }

    #[test]
    fn test_insert_remove() {
        let mut v = Vec::new();
        v.push(1);
        v.push(3);
        v.insert(1, 2);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
        let removed = v.remove(1);
        assert_eq!(removed, 2);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_into_iter() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        let sum: i32 = v.into_iter().sum();
        assert_eq!(sum, 3);
    }

    #[test]
    fn test_drain_full() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        let drained: Vec<_> = v.drain(..).collect();
        assert_eq!(drained, [1, 2, 3].iter().copied().collect());
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_drain_middle() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        let drained: Vec<_> = v.drain(1..3).collect();
        assert_eq!(drained, [2, 3].iter().copied().collect());
        assert_eq!(v, [1, 4].iter().copied().collect());
    }
}
