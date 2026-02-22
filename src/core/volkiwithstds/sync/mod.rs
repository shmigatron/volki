//! Synchronization primitives — Arc<T>.

use crate::core::volkiwithstds::alloc;
use core::mem;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{self, AtomicUsize, Ordering};

/// Inner data for Arc — ref count + value.
#[repr(C)]
struct ArcInner<T> {
    strong: AtomicUsize,
    value: T,
}

/// Atomically reference-counted smart pointer.
pub struct Arc<T> {
    ptr: ptr::NonNull<ArcInner<T>>,
}

impl<T> Arc<T> {
    /// Create a new Arc.
    pub fn new(value: T) -> Self {
        let inner = ArcInner {
            strong: AtomicUsize::new(1),
            value,
        };
        let size = mem::size_of::<ArcInner<T>>();
        let raw = if size == 0 {
            ptr::NonNull::dangling().as_ptr() as *mut u8
        } else {
            alloc::alloc(size)
        };
        assert!(!raw.is_null(), "allocation failed");
        let ptr = raw as *mut ArcInner<T>;
        unsafe {
            ptr::write(ptr, inner);
        }
        Self {
            ptr: unsafe { ptr::NonNull::new_unchecked(ptr) },
        }
    }

    /// Returns the current strong reference count.
    pub fn strong_count(this: &Arc<T>) -> usize {
        unsafe { this.ptr.as_ref().strong.load(Ordering::Relaxed) }
    }

    fn inner(&self) -> &ArcInner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner().value
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        self.inner().strong.fetch_add(1, Ordering::Relaxed);
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.inner().strong.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        // Ensure all accesses to the data happen-before we drop it
        atomic::fence(Ordering::Acquire);
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            let size = mem::size_of::<ArcInner<T>>();
            if size != 0 {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, size);
            }
        }
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&**self, f)
    }
}

impl<T: core::fmt::Display> core::fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&**self, f)
    }
}

impl<T: PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq> Eq for Arc<T> {}

impl<T: core::hash::Hash> core::hash::Hash for Arc<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

// Safety: Arc<T> is Send + Sync when T is Send + Sync
unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

/// A mutex using atomic spin-lock (simple, for internal use).
pub struct Mutex<T> {
    locked: AtomicUsize,
    data: core::cell::UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicUsize::new(0),
            data: core::cell::UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        MutexGuard { mutex: self }
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_basic() {
        let a = Arc::new(42);
        let b = a.clone();
        assert_eq!(*a, 42);
        assert_eq!(*b, 42);
        assert_eq!(Arc::strong_count(&a), 2);
        drop(b);
        assert_eq!(Arc::strong_count(&a), 1);
    }

    #[test]
    fn test_arc_drop() {
        let a = Arc::new(crate::core::volkiwithstds::collections::String::from("hello"));
        let b = a.clone();
        assert_eq!(a.as_str(), "hello");
        drop(a);
        assert_eq!(b.as_str(), "hello");
    }
}
