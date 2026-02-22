//! Read, Write, BufRead traits.

use super::error::{IoError, IoErrorKind, Result};

/// A trait for reading bytes.
pub trait Read {
    /// Read some bytes into `buf`. Returns the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Read exactly `buf.len()` bytes.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut filled = 0;
        while filled < buf.len() {
            match self.read(&mut buf[filled..]) {
                Ok(0) => {
                    return Err(IoError::new(IoErrorKind::UnexpectedEof, "unexpected EOF"));
                }
                Ok(n) => filled += n,
                Err(ref e) if e.kind() == IoErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Read all bytes until EOF into a Vec.
    fn read_to_end(
        &mut self,
        buf: &mut crate::core::volkiwithstds::collections::Vec<u8>,
    ) -> Result<usize> {
        let mut total = 0;
        let mut tmp = [0u8; 4096];
        loop {
            match self.read(&mut tmp) {
                Ok(0) => return Ok(total),
                Ok(n) => {
                    buf.extend_from_slice(&tmp[..n]);
                    total += n;
                }
                Err(ref e) if e.kind() == IoErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }

    /// Read all bytes until EOF into a String.
    fn read_to_string(
        &mut self,
        buf: &mut crate::core::volkiwithstds::collections::String,
    ) -> Result<usize> {
        let mut bytes = crate::core::volkiwithstds::collections::Vec::new();
        let n = self.read_to_end(&mut bytes)?;
        match core::str::from_utf8(bytes.as_slice()) {
            Ok(s) => {
                buf.push_str(s);
                Ok(n)
            }
            Err(_) => Err(IoError::new(
                IoErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            )),
        }
    }
}

/// A trait for writing bytes.
pub trait Write {
    /// Write bytes from `buf`. Returns the number of bytes written.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Flush the output stream.
    fn flush(&mut self) -> Result<()>;

    /// Write all bytes from `buf`.
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut written = 0;
        while written < buf.len() {
            match self.write(&buf[written..]) {
                Ok(0) => {
                    return Err(IoError::new(IoErrorKind::Other, "write returned 0 bytes"));
                }
                Ok(n) => written += n,
                Err(ref e) if e.kind() == IoErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Write a formatted string.
    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<()> {
        // Use a small adapter to bridge core::fmt::Write to our Write
        struct FmtAdapter<'a, W: ?Sized + Write> {
            inner: &'a mut W,
            error: Option<IoError>,
        }

        impl<W: ?Sized + Write> core::fmt::Write for FmtAdapter<'_, W> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Some(e);
                        Err(core::fmt::Error)
                    }
                }
            }
        }

        let mut adapter = FmtAdapter {
            inner: self,
            error: None,
        };

        match core::fmt::write(&mut adapter, fmt) {
            Ok(()) => Ok(()),
            Err(_) => {
                if let Some(e) = adapter.error {
                    Err(e)
                } else {
                    Err(IoError::new(IoErrorKind::Other, "formatter error"))
                }
            }
        }
    }
}

/// A trait for buffered reading.
pub trait BufRead: Read {
    /// Fill the internal buffer and return its contents.
    fn fill_buf(&mut self) -> Result<&[u8]>;

    /// Mark `amt` bytes as consumed.
    fn consume(&mut self, amt: usize);

    /// Read a line into the given string. Returns the number of bytes read.
    fn read_line(
        &mut self,
        buf: &mut crate::core::volkiwithstds::collections::String,
    ) -> Result<usize> {
        let mut total = 0;
        loop {
            let available = self.fill_buf()?;
            if available.is_empty() {
                return Ok(total);
            }
            // Find newline
            let mut consumed = available.len();
            let mut found_newline = false;
            for (i, &b) in available.iter().enumerate() {
                if b == b'\n' {
                    consumed = i + 1;
                    found_newline = true;
                    break;
                }
            }
            match core::str::from_utf8(&available[..consumed]) {
                Ok(s) => buf.push_str(s),
                Err(_) => {
                    return Err(IoError::new(
                        IoErrorKind::InvalidData,
                        "stream did not contain valid UTF-8",
                    ));
                }
            }
            total += consumed;
            self.consume(consumed);
            if found_newline {
                return Ok(total);
            }
        }
    }
}

impl Write for crate::core::volkiwithstds::collections::Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
