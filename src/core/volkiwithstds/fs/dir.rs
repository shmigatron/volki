//! Directory operations — read_dir, create_dir_all, remove_dir_all.

use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::io::error::{IoError, Result};
use crate::core::volkiwithstds::path::{Path, PathBuf};
use crate::core::volkiwithstds::sys::syscalls;

/// An entry in a directory listing.
pub struct DirEntry {
    path: PathBuf,
    name: String,
    file_type: FileType,
}

/// File type from dirent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Other,
}

impl DirEntry {
    /// Returns the full path of this entry.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Returns the file name.
    pub fn file_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the file type.
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
}

/// Iterator over directory entries.
pub struct ReadDir {
    dir: *mut syscalls::DIR,
    parent: PathBuf,
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = unsafe { syscalls::readdir(self.dir) };
            if entry.is_null() {
                return None;
            }

            let name = unsafe {
                let name_ptr = &(*entry).d_name as *const syscalls::c_char;
                let len = syscalls::c_strlen(name_ptr);
                let bytes = core::slice::from_raw_parts(name_ptr as *const u8, len);
                match core::str::from_utf8(bytes) {
                    Ok(s) => String::from(s),
                    Err(_) => continue,
                }
            };

            // Skip . and ..
            if name.as_str() == "." || name.as_str() == ".." {
                continue;
            }

            let file_type = unsafe {
                match (*entry).d_type {
                    4 => FileType::Directory, // DT_DIR
                    8 => FileType::File,      // DT_REG
                    10 => FileType::Symlink,  // DT_LNK
                    _ => FileType::Other,
                }
            };

            let path = self.parent.join(name.as_str());

            return Some(Ok(DirEntry {
                path: PathBuf::from(path.as_str()),
                name,
                file_type,
            }));
        }
    }
}

impl Drop for ReadDir {
    fn drop(&mut self) {
        if !self.dir.is_null() {
            unsafe {
                syscalls::closedir(self.dir);
            }
        }
    }
}

/// Read a directory, returning an iterator over entries.
pub fn read_dir(path: &Path) -> Result<ReadDir> {
    let c_path = path.to_c_string();
    let dir = unsafe { syscalls::opendir(c_path.as_ptr()) };
    if dir.is_null() {
        return Err(IoError::last_os_error());
    }
    Ok(ReadDir {
        dir,
        parent: path.to_path_buf(),
    })
}

/// Create a directory and all parent directories.
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let s = path.as_str();
    if s.is_empty() {
        return Ok(());
    }

    // Try to create the directory directly first
    let c_path = path.to_c_string();
    let ret = unsafe { syscalls::mkdir(c_path.as_ptr(), 0o755) };
    if ret == 0 {
        return Ok(());
    }

    let err = crate::core::volkiwithstds::sys::errno::get_errno();
    if err == crate::core::volkiwithstds::sys::errno::EEXIST {
        return Ok(());
    }

    // Walk each component and create missing directories
    let mut current = String::new();
    let starts_with_slash = s.starts_with("/");
    if starts_with_slash {
        current.push('/');
    }

    for component in s.split("/") {
        if component.is_empty() {
            continue;
        }
        if !current.is_empty() && !current.ends_with("/") {
            current.push('/');
        }
        current.push_str(component);

        let c_dir = crate::core::volkiwithstds::path::CString::new(current.as_str());
        let ret = unsafe { syscalls::mkdir(c_dir.as_ptr(), 0o755) };
        if ret != 0 {
            let err = crate::core::volkiwithstds::sys::errno::get_errno();
            if err != crate::core::volkiwithstds::sys::errno::EEXIST {
                return Err(IoError::from_errno(err));
            }
        }
    }

    Ok(())
}

/// Create a single directory.
pub fn create_dir(path: &Path) -> Result<()> {
    let c_path = path.to_c_string();
    let ret = unsafe { syscalls::mkdir(c_path.as_ptr(), 0o755) };
    if ret != 0 {
        return Err(IoError::last_os_error());
    }
    Ok(())
}

/// Remove a directory and all its contents recursively.
pub fn remove_dir_all(path: &Path) -> Result<()> {
    let entries = read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        if entry.file_type() == FileType::Directory {
            remove_dir_all(entry.path())?;
        } else {
            let c_path = entry.path().to_c_string();
            let ret = unsafe { syscalls::unlink(c_path.as_ptr()) };
            if ret != 0 {
                return Err(IoError::last_os_error());
            }
        }
    }
    let c_path = path.to_c_string();
    let ret = unsafe { syscalls::rmdir(c_path.as_ptr()) };
    if ret != 0 {
        return Err(IoError::last_os_error());
    }
    Ok(())
}

/// Remove a single empty directory.
pub fn remove_dir(path: &Path) -> Result<()> {
    let c_path = path.to_c_string();
    let ret = unsafe { syscalls::rmdir(c_path.as_ptr()) };
    if ret != 0 {
        return Err(IoError::last_os_error());
    }
    Ok(())
}

/// Remove a file.
pub fn remove_file(path: &Path) -> Result<()> {
    let c_path = path.to_c_string();
    let ret = unsafe { syscalls::unlink(c_path.as_ptr()) };
    if ret != 0 {
        return Err(IoError::last_os_error());
    }
    Ok(())
}
