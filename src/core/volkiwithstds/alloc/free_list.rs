//! Size-class free-list allocator backed by mmap.
//!
//! Each size class maintains a free list of reusable chunks. Chunks are carved
//! from 64 KiB slabs allocated via mmap. Slab occupancy is tracked so that
//! fully-freed slabs are returned to the OS via munmap.

use super::page::*;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

// ── Layout ──────────────────────────────────────────────────────────────────

/// Allocation header prepended to every allocation.
#[repr(C)]
struct AllocHeader {
    /// Requested size (used for dealloc class dispatch).
    size: usize,
    /// For slab-managed chunks: pointer to the owning `SlabMeta` (as usize).
    /// For large (direct-mmap) chunks: the total mmap region size.
    region_or_meta: usize,
}

/// Header size — derived from the struct so it stays in sync.
const HEADER_SIZE: usize = core::mem::size_of::<AllocHeader>();

// ── Size classes ────────────────────────────────────────────────────────────

const NUM_CLASSES: usize = 9;
const SIZE_CLASSES: [usize; NUM_CLASSES] = [16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// 64 KiB slab.
const SLAB_SIZE: usize = 65536;

// ── Slab tracking ───────────────────────────────────────────────────────────

/// Per-slab metadata, stored at the beginning of each slab.
///
/// `alloc_count` tracks how many chunks from this slab are currently handed out
/// (i.e. not in the free list). When it drops to zero, every chunk has been
/// returned and the slab can be released.
struct SlabMeta {
    base: *mut u8,
    slab_size: usize,
    total_chunks: usize,
    /// Number of chunks currently allocated (not in the free list).
    /// Only accessed while the owning size-class lock is held.
    alloc_count: usize,
}

// ── Free list node ──────────────────────────────────────────────────────────

/// Intrusive free-list node. Overlaid on the user portion of a free chunk.
struct FreeNode {
    next: *mut FreeNode,
}

// ── Per-class state ─────────────────────────────────────────────────────────

/// Per-class free list with spin-lock.
struct SizeClass {
    head: AtomicPtr<FreeNode>,
    lock: AtomicBool,
}

impl SizeClass {
    const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
            lock: AtomicBool::new(false),
        }
    }

    fn acquire(&self) {
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    fn release(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

/// Global free lists — one per size class.
static FREE_LISTS: [SizeClass; NUM_CLASSES] = [
    SizeClass::new(),
    SizeClass::new(),
    SizeClass::new(),
    SizeClass::new(),
    SizeClass::new(),
    SizeClass::new(),
    SizeClass::new(),
    SizeClass::new(),
    SizeClass::new(),
];

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Find the size-class index for `size`, or `None` if too large.
fn class_index(size: usize) -> Option<usize> {
    for (i, &class_size) in SIZE_CLASSES.iter().enumerate() {
        if size <= class_size {
            return Some(i);
        }
    }
    None
}

/// Allocate a new slab and carve it into free chunks.
///
/// Returns the head of a **local** linked list of `FreeNode`s — no global
/// state is touched. The first `chunk_size` bytes of the slab are reserved
/// for `SlabMeta`.
fn refill_class(idx: usize) -> *mut FreeNode {
    let chunk_size = SIZE_CLASSES[idx] + HEADER_SIZE;
    let slab = page_alloc(SLAB_SIZE);
    if slab.is_null() {
        return core::ptr::null_mut();
    }

    // Reserve the first chunk_size bytes for SlabMeta.
    let chunks_start = chunk_size;

    // Count how many usable chunks fit after the meta region.
    let mut total_chunks = 0;
    {
        let mut off = chunks_start;
        while off + chunk_size <= SLAB_SIZE {
            total_chunks += 1;
            off += chunk_size;
        }
    }

    // Write SlabMeta at the start of the slab.
    let meta = slab as *mut SlabMeta;
    unsafe {
        core::ptr::write(
            meta,
            SlabMeta {
                base: slab,
                slab_size: SLAB_SIZE,
                total_chunks,
                alloc_count: 0,
            },
        );
    }

    // Build a local free list from the carved chunks.
    let mut head: *mut FreeNode = core::ptr::null_mut();
    let mut offset = chunks_start;
    while offset + chunk_size <= SLAB_SIZE {
        let ptr = unsafe { slab.add(offset) };

        // Write the allocation header.
        let header = ptr as *mut AllocHeader;
        unsafe {
            (*header).size = SIZE_CLASSES[idx];
            (*header).region_or_meta = meta as usize;
        }

        // Link as a free node (user area starts after the header).
        let user_ptr = unsafe { ptr.add(HEADER_SIZE) } as *mut FreeNode;
        unsafe {
            (*user_ptr).next = head;
        }
        head = user_ptr;
        offset += chunk_size;
    }

    head
}

/// Remove all free-list entries that belong to the address range
/// `[slab_base, slab_base + slab_size)`.
///
/// Must be called while the class lock is held.
unsafe fn remove_slab_entries(class: &SizeClass, slab_base: *mut u8, slab_size: usize) {
    let slab_end = unsafe { slab_base.add(slab_size) };

    // Remove matching entries from the head.
    loop {
        let head = class.head.load(Ordering::Relaxed);
        if head.is_null() {
            return;
        }
        let head_addr = head as *mut u8;
        if head_addr >= slab_base && head_addr < slab_end {
            let next = unsafe { (*head).next };
            class.head.store(next, Ordering::Relaxed);
        } else {
            break;
        }
    }

    // Walk the rest of the list.
    let mut prev = class.head.load(Ordering::Relaxed);
    if prev.is_null() {
        return;
    }
    let mut current = unsafe { (*prev).next };
    while !current.is_null() {
        let addr = current as *mut u8;
        if addr >= slab_base && addr < slab_end {
            let next = unsafe { (*current).next };
            unsafe {
                (*prev).next = next;
            }
            current = next;
        } else {
            prev = current;
            current = unsafe { (*current).next };
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Allocate `size` bytes. Returns null on failure.
///
/// The returned pointer is aligned to at least 16 bytes.
pub fn alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::NonNull::dangling().as_ptr();
    }

    match class_index(size) {
        Some(idx) => alloc_from_class(idx),
        None => alloc_large(size),
    }
}

fn alloc_from_class(idx: usize) -> *mut u8 {
    let class = &FREE_LISTS[idx];
    class.acquire();

    let mut head = class.head.load(Ordering::Relaxed);

    if head.is_null() {
        // Release lock before the potentially slow mmap.
        class.release();
        let new_list = refill_class(idx);
        class.acquire();

        if !new_list.is_null() {
            // Splice the freshly carved list onto the existing head.
            let mut tail = new_list;
            unsafe {
                while !(*tail).next.is_null() {
                    tail = (*tail).next;
                }
                (*tail).next = class.head.load(Ordering::Relaxed);
            }
            class.head.store(new_list, Ordering::Relaxed);
        }

        head = class.head.load(Ordering::Relaxed);
        if head.is_null() {
            class.release();
            return core::ptr::null_mut();
        }
    }

    // Pop the head entry.
    let next = unsafe { (*head).next };
    class.head.store(next, Ordering::Relaxed);

    // Increment the owning slab's alloc_count.
    let header = unsafe { (head as *mut u8).sub(HEADER_SIZE) } as *const AllocHeader;
    let meta = unsafe { (*header).region_or_meta } as *mut SlabMeta;
    unsafe {
        (*meta).alloc_count += 1;
    }

    class.release();
    head as *mut u8
}

fn alloc_large(size: usize) -> *mut u8 {
    let total = size + HEADER_SIZE;
    let page_size = 4096;
    let region_size = (total + page_size - 1) & !(page_size - 1);
    let ptr = page_alloc(region_size);
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let header = ptr as *mut AllocHeader;
    unsafe {
        (*header).size = size;
        (*header).region_or_meta = region_size;
    }
    unsafe { ptr.add(HEADER_SIZE) }
}

/// Deallocate memory previously returned by `alloc`.
///
/// # Safety
/// `ptr` must have been returned by `alloc` with a compatible `size`.
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    if size == 0 || ptr == core::ptr::NonNull::<u8>::dangling().as_ptr() {
        return;
    }

    match class_index(size) {
        Some(idx) => unsafe { dealloc_to_class(ptr, idx) },
        None => unsafe { dealloc_large(ptr) },
    }
}

unsafe fn dealloc_to_class(ptr: *mut u8, idx: usize) {
    let class = &FREE_LISTS[idx];
    let node = ptr as *mut FreeNode;

    // Read slab metadata from the chunk header.
    let header = unsafe { ptr.sub(HEADER_SIZE) } as *const AllocHeader;
    let meta = unsafe { (*header).region_or_meta } as *mut SlabMeta;

    class.acquire();

    // Push the chunk back onto the free list.
    unsafe {
        (*node).next = class.head.load(Ordering::Relaxed);
    }
    class.head.store(node, Ordering::Relaxed);

    // Decrement the slab's alloc_count.
    unsafe {
        (*meta).alloc_count -= 1;

        if (*meta).alloc_count == 0 {
            // Every chunk from this slab is back in the free list.
            // Remove them and return the slab to the OS.
            let base = (*meta).base;
            let slab_size = (*meta).slab_size;
            remove_slab_entries(class, base, slab_size);
            class.release();
            page_free(base, slab_size);
            return;
        }
    }

    class.release();
}

unsafe fn dealloc_large(ptr: *mut u8) {
    let base = unsafe { ptr.sub(HEADER_SIZE) };
    let header = base as *const AllocHeader;
    let region_size = unsafe { (*header).region_or_meta };
    unsafe {
        page_free(base, region_size);
    }
}

/// Reallocate: allocate new, copy, dealloc old.
///
/// # Safety
/// `ptr` must have been returned by `alloc` with `old_size`.
pub unsafe fn realloc(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    if old_size == 0 {
        return alloc(new_size);
    }
    if new_size == 0 {
        unsafe {
            dealloc(ptr, old_size);
        }
        return core::ptr::NonNull::dangling().as_ptr();
    }
    let new_ptr = alloc(new_size);
    if new_ptr.is_null() {
        return core::ptr::null_mut();
    }
    let copy_len = if old_size < new_size {
        old_size
    } else {
        new_size
    };
    unsafe {
        core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_len);
        dealloc(ptr, old_size);
    }
    new_ptr
}
