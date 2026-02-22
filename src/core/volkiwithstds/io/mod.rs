//! I/O primitives — error types, Read/Write traits, file descriptors, stdio.

pub mod cursor;
pub mod error;
pub mod fd;
pub mod stdio;
pub mod traits;

pub use cursor::Cursor;
pub use error::{IoError, IoErrorKind, Result};
pub use fd::Fd;
pub use stdio::{stderr, stdin, stdout, Stderr, Stdin, StdinLock, Stdout};
pub use traits::{BufRead, Read, Write};
