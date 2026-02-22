//! Box<T> — heap-allocated smart pointer.

use crate::core::volkiwithstds::alloc;
use core::fmt;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr;

/// A heap-allocated value.
pub struct Box<T: ?Sized> {
    ptr: ptr::NonNull<T>,
}

impl<T> Box<T> {
    /// Allocate a value on the heap.
    pub fn new(value: T) -> Self {
        if mem::size_of::<T>() == 0 {
            mem::forget(value);
            return Self {
                ptr: ptr::NonNull::dangling(),
            };
        }
        let raw = alloc::alloc(mem::size_of::<T>());
        assert!(!raw.is_null(), "allocation failed");
        let ptr = raw as *mut T;
        unsafe {
            ptr::write(ptr, value);
        }
        Self {
            ptr: unsafe { ptr::NonNull::new_unchecked(ptr) },
        }
    }

    /// Consume the box and return the contained value.
    pub fn into_inner(b: Box<T>) -> T {
        let val = unsafe { ptr::read(b.ptr.as_ptr()) };
        if mem::size_of::<T>() != 0 {
            unsafe {
                alloc::dealloc(b.ptr.as_ptr() as *mut u8, mem::size_of::<T>());
            }
        }
        mem::forget(b);
        val
    }
}

impl<T: ?Sized> Box<T> {
    /// Convert to a raw pointer, consuming the Box without deallocating.
    pub fn into_raw(b: Box<T>) -> *mut T {
        let ptr = b.ptr.as_ptr();
        mem::forget(b);
        ptr
    }

    /// Reconstruct a Box from a raw pointer.
    ///
    /// # Safety
    /// `raw` must have been obtained from `Box::into_raw` with the same type.
    pub unsafe fn from_raw(raw: *mut T) -> Box<T> {
        Box {
            ptr: unsafe { ptr::NonNull::new_unchecked(raw) },
        }
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop the contained value
            ptr::drop_in_place(self.ptr.as_ptr());
            // Deallocate the memory
            let size = mem::size_of_val(self.ptr.as_ref());
            if size != 0 {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, size);
            }
        }
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: Clone> Clone for Box<T> {
    fn clone(&self) -> Self {
        Box::new((**self).clone())
    }
}

impl<T: PartialEq + ?Sized> PartialEq for Box<T> {
    fn eq(&self, other: &Self) -> bool {
        (**self).eq(&**other)
    }
}

impl<T: Eq + ?Sized> Eq for Box<T> {}

impl<T: core::hash::Hash + ?Sized> core::hash::Hash for Box<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

// Safety: Box<T> can be sent across threads if T can.
unsafe impl<T: Send + ?Sized> Send for Box<T> {}
unsafe impl<T: Sync + ?Sized> Sync for Box<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_basic() {
        let b = Box::new(42);
        assert_eq!(*b, 42);
    }

    #[test]
    fn test_box_into_inner() {
        let b = Box::new(String::from("hello"));
        let s = Box::into_inner(b);
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_box_deref_mut() {
        let mut b = Box::new(10);
        *b = 20;
        assert_eq!(*b, 20);
    }

    #[test]
    fn test_box_raw_roundtrip() {
        let b = Box::new(99u32);
        let raw = Box::into_raw(b);
        let b2 = unsafe { Box::from_raw(raw) };
        assert_eq!(*b2, 99);
    }

    // Note: for Box<dyn Trait>, we use the same struct but rely on
    // Rust's fat pointer support via ?Sized bound. Testing this requires
    // trait objects, which work automatically since we handle ?Sized in Drop.
    use super::super::string::String;
}
