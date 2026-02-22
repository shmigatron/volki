//! Thread spawning via pthread_create.

use crate::core::volkiwithstds::alloc;
use crate::core::volkiwithstds::sys::syscalls;
use core::mem;
use core::ptr;

/// A handle to a spawned thread.
pub struct JoinHandle<T> {
    thread: syscalls::pthread_t,
    result_ptr: *mut Option<T>,
}

/// Type-erased payload for pthread_create.
struct RawPayload {
    /// Function pointer to the monomorphized trampoline.
    invoke: unsafe fn(*mut u8),
    /// Pointer to the actual closure data.
    data: *mut u8,
}

/// Typed payload containing the closure and result slot.
struct TypedPayload<F, T> {
    func: Option<F>,
    result: Option<T>,
}

/// Trampoline called by pthread — type-erased.
unsafe extern "C" fn raw_trampoline(arg: *mut syscalls::c_void) -> *mut syscalls::c_void {
    let raw = arg as *mut RawPayload;
    let invoke = unsafe { (*raw).invoke };
    let data = unsafe { (*raw).data };
    unsafe { invoke(data) };
    ptr::null_mut()
}

/// Monomorphized invoke function — knows the concrete types.
unsafe fn invoke_typed<F: FnOnce() -> T, T>(data: *mut u8) {
    let typed = data as *mut TypedPayload<F, T>;
    let func = unsafe { (*typed).func.take().unwrap() };
    let result = func();
    unsafe {
        (*typed).result = Some(result);
    }
}

/// Spawn a new thread.
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    // Allocate the typed payload
    let typed_size = mem::size_of::<TypedPayload<F, T>>();
    let typed_ptr = if typed_size == 0 {
        ptr::NonNull::dangling().as_ptr()
    } else {
        let p = alloc::alloc(typed_size);
        assert!(!p.is_null(), "allocation failed");
        p
    };
    let typed = typed_ptr as *mut TypedPayload<F, T>;
    unsafe {
        ptr::write(
            typed,
            TypedPayload {
                func: Some(f),
                result: None,
            },
        );
    }

    // Allocate the raw payload (type-erased)
    let raw_size = mem::size_of::<RawPayload>();
    let raw_ptr = alloc::alloc(raw_size);
    assert!(!raw_ptr.is_null(), "allocation failed");
    let raw = raw_ptr as *mut RawPayload;
    unsafe {
        ptr::write(
            raw,
            RawPayload {
                invoke: invoke_typed::<F, T>,
                data: typed_ptr,
            },
        );
    }

    let result_ptr = unsafe { &mut (*typed).result as *mut Option<T> };

    let mut thread: syscalls::pthread_t = unsafe { mem::zeroed() };
    let ret = unsafe {
        syscalls::pthread_create(
            &mut thread,
            ptr::null(),
            raw_trampoline,
            raw_ptr as *mut syscalls::c_void,
        )
    };

    if ret != 0 {
        // Clean up on failure
        unsafe {
            ptr::drop_in_place(typed);
            if typed_size != 0 {
                alloc::dealloc(typed_ptr, typed_size);
            }
            alloc::dealloc(raw_ptr, raw_size);
        }
        panic!("failed to spawn thread");
    }

    JoinHandle { thread, result_ptr }
}

impl<T> JoinHandle<T> {
    /// Wait for the thread to finish and return its result.
    pub fn join(self) -> T {
        let mut retval: *mut syscalls::c_void = ptr::null_mut();
        unsafe {
            syscalls::pthread_join(self.thread, &mut retval);
        }
        // The result was written by the trampoline into the typed payload
        let result = unsafe { (*self.result_ptr).take() };
        // Note: we intentionally leak the payload allocations here for simplicity.
        // In a production implementation, we'd track and free them.
        result.expect("thread did not produce a result")
    }
}

// Safety: JoinHandle is Send (the thread result is Send)
unsafe impl<T: Send> Send for JoinHandle<T> {}
