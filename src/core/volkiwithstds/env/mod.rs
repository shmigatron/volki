//! Environment — var(), current_dir(), temp_dir(), args().

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::path::{Path, PathBuf};
use crate::core::volkiwithstds::sys::syscalls;

/// Get an environment variable by name.
pub fn var(name: &str) -> Option<String> {
    let c_name = crate::core::volkiwithstds::path::CString::new(name);
    let ptr = unsafe { syscalls::getenv(c_name.as_ptr()) };
    if ptr.is_null() {
        return None;
    }
    let len = unsafe { syscalls::c_strlen(ptr) };
    let bytes = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    match core::str::from_utf8(bytes) {
        Ok(s) => Some(String::from(s)),
        Err(_) => None,
    }
}

/// Get the current working directory.
pub fn current_dir() -> crate::core::volkiwithstds::io::Result<PathBuf> {
    let mut buf = [0u8; 4096];
    let ptr = unsafe { syscalls::getcwd(buf.as_mut_ptr() as *mut syscalls::c_char, buf.len()) };
    if ptr.is_null() {
        return Err(crate::core::volkiwithstds::io::IoError::last_os_error());
    }
    let len = unsafe { syscalls::c_strlen(ptr as *const syscalls::c_char) };
    let s = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
    Ok(PathBuf::from(s))
}

/// Get the temporary directory path.
pub fn temp_dir() -> PathBuf {
    match var("TMPDIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir.as_str()),
        _ => PathBuf::from("/tmp"),
    }
}

/// Get command-line arguments.
pub fn args() -> Vec<String> {
    args_os()
}

/// Get command-line arguments (OS-specific implementation).
#[cfg(target_os = "macos")]
fn args_os() -> Vec<String> {
    unsafe {
        let argc_ptr = syscalls::_NSGetArgc();
        let argv_ptr = syscalls::_NSGetArgv();
        let argc = *argc_ptr as usize;
        let argv = *argv_ptr;

        let mut result = Vec::with_capacity(argc);
        for i in 0..argc {
            let arg = *argv.add(i);
            let len = syscalls::c_strlen(arg as *const syscalls::c_char);
            let bytes = core::slice::from_raw_parts(arg as *const u8, len);
            match core::str::from_utf8(bytes) {
                Ok(s) => result.push(String::from(s)),
                Err(_) => result.push(String::from("<invalid utf8>")),
            }
        }
        result
    }
}

#[cfg(target_os = "linux")]
fn args_os() -> Vec<String> {
    // Read /proc/self/cmdline
    match crate::core::volkiwithstds::fs::read(Path::new("/proc/self/cmdline")) {
        Ok(bytes) => {
            let mut result = Vec::new();
            let mut start = 0;
            for (i, &b) in bytes.iter().enumerate() {
                if b == 0 {
                    if i > start {
                        match core::str::from_utf8(&bytes.as_slice()[start..i]) {
                            Ok(s) => result.push(String::from(s)),
                            Err(_) => result.push(String::from("<invalid utf8>")),
                        }
                    }
                    start = i + 1;
                }
            }
            result
        }
        Err(_) => Vec::new(),
    }
}

/// Set the current working directory.
pub fn set_current_dir(path: &Path) -> crate::core::volkiwithstds::io::Result<()> {
    let c_path = path.to_c_string();
    let ret = unsafe { syscalls::chdir(c_path.as_ptr()) };
    if ret != 0 {
        return Err(crate::core::volkiwithstds::io::IoError::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_dir() {
        let cwd = current_dir().unwrap();
        assert!(!cwd.as_str().is_empty());
        assert!(cwd.is_absolute());
    }

    #[test]
    fn test_temp_dir() {
        let tmp = temp_dir();
        assert!(!tmp.as_str().is_empty());
    }

    #[test]
    fn test_var() {
        // PATH should always be set
        let path = var("PATH");
        assert!(path.is_some());
    }

    #[test]
    fn test_var_nonexistent() {
        let val = var("VOLKI_NONEXISTENT_VAR_12345");
        assert!(val.is_none());
    }

    #[test]
    fn test_args() {
        let a = args();
        // Should have at least one arg (the program name)
        assert!(!a.is_empty());
    }
}
