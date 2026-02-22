#![cfg_attr(not(test), no_std)]
extern crate alloc;

pub mod core;
pub mod libs;

// ── Global allocator (mmap-backed free-list) ────────────────────────────────

#[cfg(not(test))]
struct VolkiAllocator;

#[cfg(not(test))]
unsafe impl ::core::alloc::GlobalAlloc for VolkiAllocator {
    unsafe fn alloc(&self, layout: ::core::alloc::Layout) -> *mut u8 {
        crate::core::volkiwithstds::alloc::alloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: ::core::alloc::Layout) {
        unsafe { crate::core::volkiwithstds::alloc::dealloc(ptr, layout.size()) }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: ::core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        unsafe { crate::core::volkiwithstds::alloc::realloc(ptr, layout.size(), new_size) }
    }
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: VolkiAllocator = VolkiAllocator;
