//! mmap/munmap wrappers for page-level allocation.

use crate::core::volkiwithstds::sys::syscalls::{
    self, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, c_void,
};

/// Allocate `size` bytes of anonymous memory via mmap.
/// Returns null pointer on failure.
pub fn page_alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }
    let ptr = unsafe {
        syscalls::mmap(
            core::ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if ptr == MAP_FAILED {
        core::ptr::null_mut()
    } else {
        ptr as *mut u8
    }
}

/// Free `size` bytes of memory previously allocated via `page_alloc`.
///
/// # Safety
/// `ptr` must have been returned by `page_alloc` with the same `size`.
pub unsafe fn page_free(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size > 0 {
        unsafe {
            syscalls::munmap(ptr as *mut c_void, size);
        }
    }
}
