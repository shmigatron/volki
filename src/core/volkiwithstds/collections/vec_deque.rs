//! VecDeque<T> — ring buffer double-ended queue.

use crate::core::volkiwithstds::alloc;
use core::mem;
use core::ptr;

/// A double-ended queue implemented with a ring buffer.
pub struct VecDeque<T> {
    buf: *mut T,
    cap: usize, // always a power of 2
    head: usize,
    len: usize,
}

const INITIAL_CAP: usize = 8;

impl<T> VecDeque<T> {
    /// Creates an empty VecDeque.
    pub fn new() -> Self {
        Self {
            buf: ptr::null_mut(),
            cap: 0,
            head: 0,
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

    fn wrap_index(&self, idx: usize) -> usize {
        idx & (self.cap - 1)
    }

    fn grow(&mut self) {
        let new_cap = if self.cap == 0 {
            INITIAL_CAP
        } else {
            self.cap * 2
        };

        let size = new_cap * mem::size_of::<T>();
        let new_buf = if mem::size_of::<T>() == 0 {
            ptr::NonNull::dangling().as_ptr()
        } else {
            let ptr = alloc::alloc(size);
            assert!(!ptr.is_null(), "allocation failed");
            ptr as *mut T
        };

        // Copy old elements to new buffer in order
        if self.cap > 0 && self.len > 0 {
            let old_cap = self.cap;
            for i in 0..self.len {
                let src_idx = (self.head + i) & (old_cap - 1);
                unsafe {
                    ptr::write(new_buf.add(i), ptr::read(self.buf.add(src_idx)));
                }
            }
            // Free old buffer
            if mem::size_of::<T>() != 0 {
                unsafe {
                    alloc::dealloc(self.buf as *mut u8, old_cap * mem::size_of::<T>());
                }
            }
        }

        self.buf = new_buf;
        self.cap = new_cap;
        self.head = 0;
    }

    /// Push an element to the back.
    pub fn push_back(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        let idx = self.wrap_index(self.head + self.len);
        unsafe {
            ptr::write(self.buf.add(idx), value);
        }
        self.len += 1;
    }

    /// Push an element to the front.
    pub fn push_front(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        self.head = if self.head == 0 {
            self.cap - 1
        } else {
            self.head - 1
        };
        unsafe {
            ptr::write(self.buf.add(self.head), value);
        }
        self.len += 1;
    }

    /// Pop from the front.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let val = unsafe { ptr::read(self.buf.add(self.head)) };
        self.head = self.wrap_index(self.head + 1);
        self.len -= 1;
        Some(val)
    }

    /// Pop from the back.
    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let idx = self.wrap_index(self.head + self.len);
        unsafe { Some(ptr::read(self.buf.add(idx))) }
    }

    /// Peek at the front element.
    pub fn front(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(&*self.buf.add(self.head)) }
        }
    }

    /// Peek at the back element.
    pub fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            let idx = self.wrap_index(self.head + self.len - 1);
            unsafe { Some(&*self.buf.add(idx)) }
        }
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> Drop for VecDeque<T> {
    fn drop(&mut self) {
        // Drop all elements
        self.clear();
        // Free the buffer
        if self.cap > 0 && mem::size_of::<T>() != 0 {
            unsafe {
                alloc::dealloc(self.buf as *mut u8, self.cap * mem::size_of::<T>());
            }
        }
    }
}

impl<T: Clone> Clone for VecDeque<T> {
    fn clone(&self) -> Self {
        let mut new = VecDeque::new();
        for i in 0..self.len {
            let idx = (self.head + i) & (self.cap - 1);
            let val = unsafe { &*self.buf.add(idx) };
            new.push_back(val.clone());
        }
        new
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for VecDeque<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        for i in 0..self.len {
            let idx = (self.head + i) & (self.cap - 1);
            let val = unsafe { &*self.buf.add(idx) };
            list.entry(val);
        }
        list.finish()
    }
}

impl<T> Default for VecDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: VecDeque is safe to send/sync if T is.
unsafe impl<T: Send> Send for VecDeque<T> {}
unsafe impl<T: Sync> Sync for VecDeque<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut q = VecDeque::new();
        q.push_back(1);
        q.push_back(2);
        q.push_back(3);
        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.pop_front(), Some(2));
        assert_eq!(q.pop_front(), Some(3));
        assert_eq!(q.pop_front(), None);
    }

    #[test]
    fn test_wrap_around() {
        let mut q = VecDeque::new();
        for i in 0..20 {
            q.push_back(i);
        }
        for i in 0..10 {
            assert_eq!(q.pop_front(), Some(i));
        }
        for i in 20..30 {
            q.push_back(i);
        }
        assert_eq!(q.len(), 20);
    }

    #[test]
    fn test_push_front() {
        let mut q = VecDeque::new();
        q.push_front(1);
        q.push_front(2);
        q.push_front(3);
        assert_eq!(q.pop_front(), Some(3));
        assert_eq!(q.pop_front(), Some(2));
        assert_eq!(q.pop_front(), Some(1));
    }
}
