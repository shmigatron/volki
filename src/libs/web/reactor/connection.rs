//! Per-connection state machine and buffers.

use crate::core::volkiwithstds::collections::Vec;
use crate::core::volkiwithstds::io::error::{IoError, IoErrorKind};
use crate::core::volkiwithstds::sys::{errno, syscalls};
use crate::core::volkiwithstds::sys::openssl::SSL;
use crate::core::volkiwithstds::time::Instant;
use crate::core::security::tls::stream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    Handshaking,
    ReadingRequest,
    Processing,
    WritingResponse,
    Done,
}

/// Whether a connection uses plaintext or TLS I/O.
pub enum IoMode {
    Plaintext,
    Tls { ssl: *mut SSL },
}

/// Result of a non-blocking TLS handshake attempt.
pub enum HandshakeResult {
    Complete,
    WantRead,
    WantWrite,
}

pub struct Connection {
    pub fd: i32,
    pub state: ConnState,
    pub read_buf: Vec<u8>,
    pub write_buf: Vec<u8>,
    pub write_pos: usize,
    pub keep_alive: bool,
    pub mode: IoMode,
    pub last_activity: Instant,
    pub client_ip: u32,
    pub max_read_buf: usize,
}

impl Connection {
    pub fn new(fd: i32, client_ip: u32, max_read_buf: usize) -> Self {
        Self {
            fd,
            state: ConnState::ReadingRequest,
            read_buf: Vec::with_capacity(4096),
            write_buf: Vec::new(),
            write_pos: 0,
            keep_alive: true,
            mode: IoMode::Plaintext,
            last_activity: Instant::now(),
            client_ip,
            max_read_buf,
        }
    }

    /// Create a TLS connection — starts in `Handshaking` state.
    pub fn new_tls(fd: i32, ssl: *mut SSL, client_ip: u32, max_read_buf: usize) -> Self {
        Self {
            fd,
            state: ConnState::Handshaking,
            read_buf: Vec::with_capacity(4096),
            write_buf: Vec::new(),
            write_pos: 0,
            keep_alive: true,
            mode: IoMode::Tls { ssl },
            last_activity: Instant::now(),
            client_ip,
            max_read_buf,
        }
    }

    /// Attempt the non-blocking TLS handshake.
    pub fn try_handshake(&mut self) -> Result<HandshakeResult, IoError> {
        let ssl = match &self.mode {
            IoMode::Tls { ssl } => *ssl,
            IoMode::Plaintext => {
                return Err(IoError::new(IoErrorKind::Other, "not a TLS connection"));
            }
        };
        match stream::ssl_accept(ssl) {
            Ok(true) => Ok(HandshakeResult::Complete),
            Ok(false) => Ok(HandshakeResult::WantRead),
            Err(crate::core::security::tls::error::TlsError::WantRead) => {
                Ok(HandshakeResult::WantRead)
            }
            Err(crate::core::security::tls::error::TlsError::WantWrite) => {
                Ok(HandshakeResult::WantWrite)
            }
            Err(_) => Err(IoError::new(IoErrorKind::Other, "TLS handshake failed")),
        }
    }

    /// Try to read data from the socket. Returns Ok(true) if data was read,
    /// Ok(false) if EAGAIN/WouldBlock, Err on real error.
    pub fn try_read(&mut self) -> Result<bool, IoError> {
        match &self.mode {
            IoMode::Plaintext => self.try_read_plaintext(),
            IoMode::Tls { ssl } => self.try_read_tls(*ssl),
        }
    }

    fn try_read_plaintext(&mut self) -> Result<bool, IoError> {
        let mut tmp = [0u8; 8192];
        loop {
            let ret = unsafe {
                syscalls::read(
                    self.fd,
                    tmp.as_mut_ptr() as *mut syscalls::c_void,
                    tmp.len(),
                )
            };
            if ret > 0 {
                self.read_buf.extend_from_slice(&tmp[..ret as usize]);
                self.last_activity = Instant::now();
                if self.max_read_buf > 0 && self.read_buf.len() > self.max_read_buf {
                    self.state = ConnState::Done;
                    return Err(IoError::new(IoErrorKind::Other, "read buffer exceeded"));
                }
                return Ok(true);
            } else if ret == 0 {
                // EOF
                self.state = ConnState::Done;
                return Ok(false);
            } else {
                let err = errno::get_errno();
                if err == errno::EINTR {
                    continue;
                }
                if err == errno::EAGAIN {
                    return Ok(false);
                }
                return Err(IoError::from_errno(err));
            }
        }
    }

    fn try_read_tls(&mut self, ssl: *mut SSL) -> Result<bool, IoError> {
        let mut tmp = [0u8; 8192];
        match stream::ssl_read(ssl, &mut tmp) {
            Ok(n) => {
                self.read_buf.extend_from_slice(&tmp[..n]);
                self.last_activity = Instant::now();
                if self.max_read_buf > 0 && self.read_buf.len() > self.max_read_buf {
                    self.state = ConnState::Done;
                    return Err(IoError::new(IoErrorKind::Other, "read buffer exceeded"));
                }
                Ok(true)
            }
            Err(crate::core::security::tls::error::TlsError::WantRead) => Ok(false),
            Err(crate::core::security::tls::error::TlsError::WantWrite) => Ok(false),
            Err(crate::core::security::tls::error::TlsError::ConnectionClosed) => {
                self.state = ConnState::Done;
                Ok(false)
            }
            Err(_) => Err(IoError::new(IoErrorKind::Other, "TLS read error")),
        }
    }

    /// Try to write response data. Returns Ok(true) if all data written,
    /// Ok(false) if EAGAIN/WouldBlock (partial write), Err on real error.
    pub fn try_write(&mut self) -> Result<bool, IoError> {
        match &self.mode {
            IoMode::Plaintext => self.try_write_plaintext(),
            IoMode::Tls { ssl } => self.try_write_tls(*ssl),
        }
    }

    fn try_write_plaintext(&mut self) -> Result<bool, IoError> {
        while self.write_pos < self.write_buf.len() {
            let remaining = &self.write_buf[self.write_pos..];
            let ret = unsafe {
                syscalls::write(
                    self.fd,
                    remaining.as_ptr() as *const syscalls::c_void,
                    remaining.len(),
                )
            };
            if ret > 0 {
                self.write_pos += ret as usize;
                self.last_activity = Instant::now();
            } else if ret == 0 {
                return Err(IoError::new(IoErrorKind::Other, "write returned 0"));
            } else {
                let err = errno::get_errno();
                if err == errno::EINTR {
                    continue;
                }
                if err == errno::EAGAIN {
                    return Ok(false);
                }
                return Err(IoError::from_errno(err));
            }
        }
        Ok(true)
    }

    fn try_write_tls(&mut self, ssl: *mut SSL) -> Result<bool, IoError> {
        while self.write_pos < self.write_buf.len() {
            let remaining = &self.write_buf[self.write_pos..];
            match stream::ssl_write(ssl, remaining) {
                Ok(n) => {
                    self.write_pos += n;
                    self.last_activity = Instant::now();
                }
                Err(crate::core::security::tls::error::TlsError::WantWrite) => {
                    return Ok(false);
                }
                Err(crate::core::security::tls::error::TlsError::WantRead) => {
                    return Ok(false);
                }
                Err(_) => {
                    return Err(IoError::new(IoErrorKind::Other, "TLS write error"));
                }
            }
        }
        Ok(true)
    }

    pub fn set_response(&mut self, data: Vec<u8>) {
        self.write_buf = data;
        self.write_pos = 0;
        self.state = ConnState::WritingResponse;
    }

    pub fn reset_for_keep_alive(&mut self) {
        self.read_buf.clear();
        self.write_buf.clear();
        self.write_pos = 0;
        self.state = ConnState::ReadingRequest;
        self.last_activity = Instant::now();
    }

    /// Shut down TLS (if active) before closing the fd.
    pub fn shutdown_tls(&mut self) {
        if let IoMode::Tls { ssl } = &self.mode {
            stream::ssl_shutdown(*ssl);
            stream::ssl_free(*ssl);
        }
    }
}
