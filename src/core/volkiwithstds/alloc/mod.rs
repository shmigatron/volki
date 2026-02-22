//! Custom allocator — mmap-backed size-class free-list.

pub mod free_list;
pub mod page;

pub use free_list::{alloc, dealloc, realloc};
