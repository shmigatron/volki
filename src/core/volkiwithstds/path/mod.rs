//! Path (borrowed) and PathBuf (owned) — filesystem path types.

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::sys::syscalls;
use core::fmt;
use core::ops::Deref;

/// A null-terminated byte string for passing to C functions.
pub struct CString {
    bytes: Vec<u8>,
}

impl CString {
    /// Create a CString from a &str (appends null byte).
    pub fn new(s: &str) -> Self {
        let mut bytes = Vec::with_capacity(s.len() + 1);
        bytes.extend_from_slice(s.as_bytes());
        bytes.push(0);
        Self { bytes }
    }

    /// Returns a pointer to the null-terminated bytes.
    pub fn as_ptr(&self) -> *const syscalls::c_char {
        self.bytes.as_ptr() as *const syscalls::c_char
    }
}

/// A borrowed filesystem path (wrapper around str).
#[repr(transparent)]
pub struct Path {
    inner: str,
}

impl Path {
    /// Wrap a &str as a &Path.
    pub fn new(s: &str) -> &Path {
        // Safety: Path is #[repr(transparent)] over str
        unsafe { &*(s as *const str as *const Path) }
    }

    /// Returns the inner string.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Join this path with another component.
    pub fn join(&self, other: &str) -> PathBuf {
        let mut buf = PathBuf::from(self.as_str());
        if !self.inner.is_empty() && !self.inner.ends_with("/") {
            buf.inner.push('/');
        }
        buf.inner.push_str(other);
        buf
    }

    /// Returns the parent directory, if any.
    pub fn parent(&self) -> Option<&Path> {
        let s = self.inner.trim_end_matches('/');
        if s.is_empty() {
            return None;
        }
        match s.rfind('/') {
            Some(0) => Some(Path::new("/")),
            Some(idx) => Some(Path::new(&self.inner[..idx])),
            None => Some(Path::new("")),
        }
    }

    /// Returns the final component of the path.
    pub fn file_name(&self) -> Option<&str> {
        let s = self.inner.trim_end_matches('/');
        if s.is_empty() {
            return None;
        }
        match s.rfind('/') {
            Some(idx) => Some(&s[idx + 1..]),
            None => Some(s),
        }
    }

    /// Returns the extension (after the last '.'), if any.
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name()?;
        match name.rfind('.') {
            Some(0) | None => None,
            Some(idx) => Some(&name[idx + 1..]),
        }
    }

    /// Returns a new PathBuf with the extension replaced.
    pub fn with_extension(&self, ext: &str) -> PathBuf {
        let mut new = self.to_path_buf();
        new.set_extension(ext);
        new
    }

    /// Check if the path string contains a substring.
    pub fn contains(&self, pattern: &str) -> bool {
        self.inner.contains(pattern)
    }

    /// Returns the file stem (before the last '.').
    pub fn file_stem(&self) -> Option<&str> {
        let name = self.file_name()?;
        match name.rfind('.') {
            Some(0) | None => Some(name),
            Some(idx) => Some(&name[..idx]),
        }
    }

    /// Check if the path exists.
    pub fn exists(&self) -> bool {
        crate::core::volkiwithstds::fs::metadata::exists(self)
    }

    /// Check if the path is a directory.
    pub fn is_dir(&self) -> bool {
        crate::core::volkiwithstds::fs::metadata::is_dir(self)
    }

    /// Check if the path is a file.
    pub fn is_file(&self) -> bool {
        crate::core::volkiwithstds::fs::metadata::is_file(self)
    }

    /// Returns a Display-able wrapper.
    pub fn display(&self) -> &str {
        &self.inner
    }

    /// Canonicalize the path (resolve symlinks, make absolute).
    pub fn canonicalize(&self) -> crate::core::volkiwithstds::io::Result<PathBuf> {
        let c_path = self.to_c_string();
        let mut buf = [0u8; 4096];
        let result = unsafe {
            syscalls::realpath(c_path.as_ptr(), buf.as_mut_ptr() as *mut syscalls::c_char)
        };
        if result.is_null() {
            return Err(crate::core::volkiwithstds::io::IoError::last_os_error());
        }
        let len = unsafe { syscalls::c_strlen(result as *const syscalls::c_char) };
        let s = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
        Ok(PathBuf::from(s))
    }

    /// Strip a prefix from this path.
    pub fn strip_prefix<'a>(&'a self, prefix: &str) -> Option<&'a str> {
        let s = self.as_str();
        if s.starts_with(prefix) {
            let rest = &s[prefix.len()..];
            if rest.is_empty() {
                Some("")
            } else if rest.starts_with('/') {
                Some(&rest[1..])
            } else if prefix.ends_with('/') {
                Some(rest)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns true if this path starts with the given prefix.
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.inner.starts_with(prefix)
    }

    /// Returns true if this path ends with the given suffix.
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.inner.ends_with(suffix)
    }

    /// Convert to a CString for C function calls.
    pub fn to_c_string(&self) -> CString {
        CString::new(&self.inner)
    }

    /// Convert to an owned PathBuf.
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.as_str())
    }

    /// Returns true if the path is absolute.
    pub fn is_absolute(&self) -> bool {
        self.inner.starts_with('/')
    }

    /// Returns true if the path is relative.
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    /// Returns an iterator over the components of the path.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.inner.split('/').filter(|s| !s.is_empty())
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", &self.inner)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner)
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Path {}

impl core::hash::Hash for Path {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

// ── PathBuf ─────────────────────────────────────────────────────────────────

/// An owned filesystem path.
pub struct PathBuf {
    inner: String,
}

impl PathBuf {
    /// Create an empty PathBuf.
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    /// Create a PathBuf from a string.
    pub fn from(s: &str) -> Self {
        Self {
            inner: String::from(s),
        }
    }

    /// Push a path component.
    pub fn push(&mut self, component: &str) {
        if !self.inner.is_empty() && !self.inner.ends_with("/") {
            self.inner.push('/');
        }
        self.inner.push_str(component);
    }

    /// Pop the last component.
    pub fn pop(&mut self) -> bool {
        if let Some(parent) = self.as_path().parent() {
            let parent_len = parent.as_str().len();
            self.inner.truncate(parent_len);
            true
        } else {
            false
        }
    }

    /// Set the extension.
    pub fn set_extension(&mut self, ext: &str) {
        let s = self.inner.as_str();
        if let Some(dot_pos) = s.rfind('.') {
            let slash_pos = s.rfind('/').unwrap_or(0);
            if dot_pos > slash_pos {
                self.inner.truncate(dot_pos);
            }
        }
        if !ext.is_empty() {
            self.inner.push('.');
            self.inner.push_str(ext);
        }
    }

    /// Returns a new PathBuf with the extension replaced.
    pub fn with_extension(&self, ext: &str) -> PathBuf {
        let mut new = self.clone();
        new.set_extension(ext);
        new
    }

    /// Returns a &Path view.
    pub fn as_path(&self) -> &Path {
        Path::new(self.inner.as_str())
    }
}

impl Deref for PathBuf {
    type Target = Path;
    fn deref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl Clone for PathBuf {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl fmt::Debug for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.inner)
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.inner.as_str())
    }
}

impl PartialEq for PathBuf {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for PathBuf {}

impl PartialOrd for PathBuf {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathBuf {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl core::hash::Hash for PathBuf {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl Default for PathBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for PathBuf {
    fn from(s: &str) -> Self {
        PathBuf::from(s)
    }
}

impl From<String> for PathBuf {
    fn from(s: String) -> Self {
        PathBuf {
            inner: s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_join() {
        let p = Path::new("/usr");
        let joined = p.join("bin");
        assert_eq!(joined.as_str(), "/usr/bin");
    }

    #[test]
    fn test_path_parent() {
        let p = Path::new("/usr/bin/ls");
        let parent = p.parent().unwrap();
        assert_eq!(parent.as_str(), "/usr/bin");
    }

    #[test]
    fn test_path_file_name() {
        let p = Path::new("/usr/bin/ls");
        assert_eq!(p.file_name(), Some("ls"));
    }

    #[test]
    fn test_path_extension() {
        let p = Path::new("/home/user/file.txt");
        assert_eq!(p.extension(), Some("txt"));
    }

    #[test]
    fn test_pathbuf_push() {
        let mut p = PathBuf::from("/usr");
        p.push("local");
        p.push("bin");
        assert_eq!(p.as_str(), "/usr/local/bin");
    }

    #[test]
    fn test_path_exists() {
        assert!(Path::new("/").exists());
        assert!(!Path::new("/nonexistent_path_12345").exists());
    }
}
