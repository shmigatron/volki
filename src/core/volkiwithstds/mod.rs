//! volkiwithstds — pure no_std replacement module.
//!
//! Provides native implementations of collections, I/O, filesystem, networking,
//! threading, time, process management, and path handling without using `std` or
//! `alloc`. Only `core` language primitives and `extern "C"` libc linkage are used.

#![allow(dead_code)]

pub mod sys;
pub mod alloc;
pub mod collections;
pub mod io;
pub mod fs;
pub mod path;
pub mod net;
pub mod sync;
pub mod thread;
pub mod time;
pub mod process;
pub mod env;
pub mod fmt;

/// Prelude — convenient imports for common types and traits.
pub mod prelude {
    pub use super::collections::{Box, HashMap, HashSet, String, Vec, VecDeque};
    pub use super::io::{BufRead, Read, Write, IoError, IoErrorKind, Result as IoResult};
    pub use super::path::{Path, PathBuf, CString};
    pub use super::sync::Arc;
    pub use super::time::{Duration, Instant};
}
