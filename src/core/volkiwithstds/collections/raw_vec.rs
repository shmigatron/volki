//! RawVec<T> — backing store with growth logic for Vec<T>.

use crate::core::volkiwithstds::alloc;
use core::mem;
use core::ptr::NonNull;

/// Raw vector storage — manages allocation but not initialization.
pub struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

// SAFETY: RawVec is just a pointer + capacity (like std's RawVec).
// Sending it across threads is safe when T is Send.
unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    /// Create a new empty RawVec (no allocation).
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    /// Create a RawVec with the given capacity.
    pub fn with_capacity(cap: usize) -> Self {
        if cap == 0 || mem::size_of::<T>() == 0 {
            return Self::new();
        }
        let size = cap
            .checked_mul(mem::size_of::<T>())
            .expect("capacity overflow");
        let ptr = alloc::alloc(size);
        assert!(!ptr.is_null(), "allocation failed");
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut T) },
            cap,
        }
    }

    /// Returns a raw pointer to the allocation.
    pub fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Returns the current capacity.
    pub fn cap(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.cap
        }
    }

    /// Grow to hold at least `min_cap` elements.
    pub fn grow(&mut self, min_cap: usize) {
        if mem::size_of::<T>() == 0 {
            return;
        }
        let new_cap = if self.cap == 0 {
            let initial = if min_cap > 4 { min_cap } else { 4 };
            initial
        } else {
            let doubled = self.cap * 2;
            if doubled >= min_cap { doubled } else { min_cap }
        };

        let new_size = new_cap
            .checked_mul(mem::size_of::<T>())
            .expect("capacity overflow");

        let new_ptr = if self.cap == 0 {
            alloc::alloc(new_size)
        } else {
            let old_size = self.cap * mem::size_of::<T>();
            unsafe { alloc::realloc(self.ptr.as_ptr() as *mut u8, old_size, new_size) }
        };

        assert!(!new_ptr.is_null(), "allocation failed");
        self.ptr = unsafe { NonNull::new_unchecked(new_ptr as *mut T) };
        self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 && mem::size_of::<T>() != 0 {
            let size = self.cap * mem::size_of::<T>();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, size);
            }
        }
    }
}
