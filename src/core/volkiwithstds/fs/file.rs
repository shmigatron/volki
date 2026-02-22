//! File operations — read_to_string, write.

use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::io::error::{IoError, IoErrorKind, Result};
use crate::core::volkiwithstds::path::Path;
use crate::core::volkiwithstds::sys::{errno, syscalls};

/// Read an entire file into a String.
///
/// Reads the complete file into bytes first, then validates UTF-8 on the
/// full buffer. This avoids false rejections when multi-byte characters
/// span read chunk boundaries.
pub fn read_to_string(path: &Path) -> Result<String> {
    let bytes = read(path)?;
    match core::str::from_utf8(bytes.as_slice()) {
        Ok(s) => Ok(String::from(s)),
        Err(_) => Err(IoError::new(
            IoErrorKind::InvalidData,
            "file is not valid UTF-8",
        )),
    }
}

/// Read an entire file into bytes.
pub fn read(path: &Path) -> Result<crate::core::volkiwithstds::collections::Vec<u8>> {
    let c_path = path.to_c_string();
    let fd = unsafe { syscalls::open(c_path.as_ptr(), syscalls::O_RDONLY) };
    if fd < 0 {
        return Err(IoError::last_os_error());
    }

    let mut stat_buf: syscalls::stat_buf = unsafe { core::mem::zeroed() };
    let ret = unsafe { syscalls::fstat(fd, &mut stat_buf) };
    let file_size = if ret == 0 && stat_buf.st_size > 0 {
        stat_buf.st_size as usize
    } else {
        4096
    };

    let mut result = crate::core::volkiwithstds::collections::Vec::with_capacity(file_size);
    let mut buf = [0u8; 8192];

    loop {
        let n = unsafe { syscalls::read(fd, buf.as_mut_ptr() as *mut syscalls::c_void, buf.len()) };
        if n < 0 {
            let err = errno::get_errno();
            if err == errno::EINTR {
                continue;
            }
            unsafe {
                syscalls::close(fd);
            }
            return Err(IoError::from_errno(err));
        }
        if n == 0 {
            break;
        }
        result.extend_from_slice(&buf[..n as usize]);
    }

    unsafe {
        syscalls::close(fd);
    }
    Ok(result)
}

/// Write bytes to a file (create or truncate).
pub fn write<P, C>(path: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let path = path.as_ref();
    let contents = contents.as_ref();
    let c_path = path.to_c_string();
    let fd = unsafe {
        syscalls::open(
            c_path.as_ptr(),
            syscalls::O_WRONLY | syscalls::O_CREAT | syscalls::O_TRUNC,
            0o644 as syscalls::c_int,
        )
    };
    if fd < 0 {
        return Err(IoError::last_os_error());
    }

    let mut written = 0;
    while written < contents.len() {
        let n = unsafe {
            syscalls::write(
                fd,
                contents[written..].as_ptr() as *const syscalls::c_void,
                contents.len() - written,
            )
        };
        if n < 0 {
            let err = errno::get_errno();
            if err == errno::EINTR {
                continue;
            }
            unsafe {
                syscalls::close(fd);
            }
            return Err(IoError::from_errno(err));
        }
        written += n as usize;
    }

    unsafe {
        syscalls::close(fd);
    }
    Ok(())
}

/// Write a string to a file.
pub fn write_str(path: &Path, contents: &str) -> Result<()> {
    write(path, contents.as_bytes())
}

/// An open file handle.
pub struct File {
    fd: i32,
}

impl File {
    /// Open a file for reading.
    pub fn open(path: &Path) -> Result<Self> {
        let c_path = path.to_c_string();
        let fd = unsafe { syscalls::open(c_path.as_ptr(), syscalls::O_RDONLY) };
        if fd < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(Self { fd })
    }

    /// Create/truncate a file for writing.
    pub fn create(path: &Path) -> Result<Self> {
        let c_path = path.to_c_string();
        let fd = unsafe {
            syscalls::open(
                c_path.as_ptr(),
                syscalls::O_WRONLY | syscalls::O_CREAT | syscalls::O_TRUNC,
                0o644 as syscalls::c_int,
            )
        };
        if fd < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(Self { fd })
    }
}

impl crate::core::volkiwithstds::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            let ret = unsafe {
                syscalls::read(
                    self.fd,
                    buf.as_mut_ptr() as *mut syscalls::c_void,
                    buf.len(),
                )
            };
            if ret < 0 {
                let err = errno::get_errno();
                if err == errno::EINTR {
                    continue;
                }
                return Err(IoError::from_errno(err));
            }
            return Ok(ret as usize);
        }
    }
}

impl crate::core::volkiwithstds::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        loop {
            let ret = unsafe {
                syscalls::write(self.fd, buf.as_ptr() as *const syscalls::c_void, buf.len())
            };
            if ret < 0 {
                let err = errno::get_errno();
                if err == errno::EINTR {
                    continue;
                }
                return Err(IoError::from_errno(err));
            }
            return Ok(ret as usize);
        }
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe {
                syscalls::close(self.fd);
            }
        }
    }
}
