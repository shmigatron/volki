//! Cursor<T> — in-memory I/O for testing.

use super::error::Result;
use super::traits::{BufRead, Read, Write};
use crate::core::volkiwithstds::collections::Vec;

/// A Cursor wraps an in-memory buffer and provides Read/Write.
pub struct Cursor<T> {
    inner: T,
    pos: usize,
}

impl<T> Cursor<T> {
    /// Create a new Cursor wrapping the given buffer.
    pub fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }

    /// Returns the current position.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Set the position.
    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// Get a reference to the inner value.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume the cursor and return the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl Read for Cursor<&[u8]> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let available = &self.inner[self.pos..];
        let n = buf.len().min(available.len());
        buf[..n].copy_from_slice(&available[..n]);
        self.pos += n;
        Ok(n)
    }
}

impl Read for Cursor<Vec<u8>> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let available = &self.inner.as_slice()[self.pos..];
        let n = buf.len().min(available.len());
        buf[..n].copy_from_slice(&available[..n]);
        self.pos += n;
        Ok(n)
    }
}

impl Write for Cursor<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // Extend or overwrite
        let end = self.pos + buf.len();
        while self.inner.len() < end {
            self.inner.push(0);
        }
        for (i, &b) in buf.iter().enumerate() {
            self.inner[self.pos + i] = b;
        }
        self.pos = end;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl BufRead for Cursor<&[u8]> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        Ok(&self.inner[self.pos..])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = (self.pos + amt).min(self.inner.len());
    }
}

impl BufRead for Cursor<Vec<u8>> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        Ok(&self.inner.as_slice()[self.pos..])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = (self.pos + amt).min(self.inner.len());
    }
}
