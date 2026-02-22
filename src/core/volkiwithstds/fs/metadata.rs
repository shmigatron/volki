//! Filesystem metadata — exists, is_dir, is_file via stat().

use crate::core::volkiwithstds::path::Path;
use crate::core::volkiwithstds::sys::syscalls;

/// Metadata about a filesystem entry.
pub struct Metadata {
    mode: u32,
    size: u64,
}

impl Metadata {
    /// Returns the file size in bytes.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if this is a directory.
    pub fn is_dir(&self) -> bool {
        (self.mode & syscalls::S_IFMT) == syscalls::S_IFDIR
    }

    /// Returns true if this is a regular file.
    pub fn is_file(&self) -> bool {
        (self.mode & syscalls::S_IFMT) == syscalls::S_IFREG
    }
}

/// Get metadata for a path.
pub fn metadata(path: &Path) -> crate::core::volkiwithstds::io::Result<Metadata> {
    let c_path = path.to_c_string();
    let mut stat_buf: syscalls::stat_buf = unsafe { core::mem::zeroed() };
    let ret = unsafe { syscalls::stat(c_path.as_ptr(), &mut stat_buf) };
    if ret != 0 {
        return Err(crate::core::volkiwithstds::io::IoError::last_os_error());
    }

    #[cfg(target_os = "macos")]
    let mode = stat_buf.st_mode as u32;
    #[cfg(target_os = "linux")]
    let mode = stat_buf.st_mode;

    Ok(Metadata {
        mode,
        size: stat_buf.st_size as u64,
    })
}

/// Check if a path exists.
pub fn exists(path: &Path) -> bool {
    metadata(path).is_ok()
}

/// Check if a path is a directory.
pub fn is_dir(path: &Path) -> bool {
    metadata(path).map(|m| m.is_dir()).unwrap_or(false)
}

/// Check if a path is a regular file.
pub fn is_file(path: &Path) -> bool {
    metadata(path).map(|m| m.is_file()).unwrap_or(false)
}
