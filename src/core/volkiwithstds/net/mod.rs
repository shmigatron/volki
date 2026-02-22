//! Networking — TcpStream.

use crate::core::volkiwithstds::io::error::{IoError, IoErrorKind, Result};
use crate::core::volkiwithstds::io::traits::{Read, Write};
use crate::core::volkiwithstds::path::CString;
use crate::core::volkiwithstds::sys::{errno, syscalls};

/// A TCP stream connected to a remote host.
pub struct TcpStream {
    fd: i32,
}

impl TcpStream {
    /// Connect to a remote host.
    pub fn connect(addr: (&str, u16)) -> Result<Self> {
        let (host, port) = addr;

        let c_host = CString::new(host);
        // Convert port to string for getaddrinfo
        let mut port_buf = [0u8; 8];
        let port_str = port_to_str(port, &mut port_buf);
        let c_port = CString::new(port_str);

        let mut hints: syscalls::addrinfo = unsafe { core::mem::zeroed() };
        hints.ai_family = syscalls::AF_INET;
        hints.ai_socktype = syscalls::SOCK_STREAM;

        let mut result: *mut syscalls::addrinfo = core::ptr::null_mut();
        let ret = unsafe {
            syscalls::getaddrinfo(
                c_host.as_ptr(),
                c_port.as_ptr(),
                &hints,
                &mut result,
            )
        };

        if ret != 0 || result.is_null() {
            return Err(IoError::new(
                IoErrorKind::Other,
                "failed to resolve address",
            ));
        }

        let ai = unsafe { &*result };
        let fd = unsafe { syscalls::socket(ai.ai_family, ai.ai_socktype, ai.ai_protocol) };
        if fd < 0 {
            unsafe { syscalls::freeaddrinfo(result); }
            return Err(IoError::last_os_error());
        }

        let connect_ret = unsafe {
            syscalls::connect(fd, ai.ai_addr as *const syscalls::sockaddr, ai.ai_addrlen)
        };

        unsafe { syscalls::freeaddrinfo(result); }

        if connect_ret < 0 {
            unsafe { syscalls::close(fd); }
            return Err(IoError::last_os_error());
        }

        Ok(Self { fd })
    }

    /// Set non-blocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
        let flags = unsafe { syscalls::fcntl(self.fd, syscalls::F_GETFL) };
        if flags < 0 {
            return Err(IoError::last_os_error());
        }
        let new_flags = if nonblocking {
            flags | syscalls::O_NONBLOCK
        } else {
            flags & !syscalls::O_NONBLOCK
        };
        let ret = unsafe { syscalls::fcntl(self.fd, syscalls::F_SETFL, new_flags) };
        if ret < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(())
    }

    /// Returns the raw file descriptor.
    pub fn as_raw_fd(&self) -> i32 {
        self.fd
    }
}

impl Read for TcpStream {
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

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        loop {
            let ret = unsafe {
                syscalls::write(
                    self.fd,
                    buf.as_ptr() as *const syscalls::c_void,
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

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe { syscalls::close(self.fd); }
        }
    }
}

/// A TCP listener bound to an address.
pub struct TcpListener {
    fd: i32,
}

impl TcpListener {
    /// Bind to the given address.
    pub fn bind(addr: (&str, u16)) -> Result<Self> {
        let (host, port) = addr;

        let c_host = CString::new(host);
        let mut port_buf = [0u8; 8];
        let port_str = port_to_str(port, &mut port_buf);
        let c_port = CString::new(port_str);

        let mut hints: syscalls::addrinfo = unsafe { core::mem::zeroed() };
        hints.ai_family = syscalls::AF_INET;
        hints.ai_socktype = syscalls::SOCK_STREAM;
        hints.ai_flags = syscalls::AI_PASSIVE;

        let mut result: *mut syscalls::addrinfo = core::ptr::null_mut();
        let ret = unsafe {
            syscalls::getaddrinfo(
                c_host.as_ptr(),
                c_port.as_ptr(),
                &hints,
                &mut result,
            )
        };

        if ret != 0 || result.is_null() {
            return Err(IoError::new(IoErrorKind::Other, "failed to resolve address"));
        }

        let ai = unsafe { &*result };
        let fd = unsafe { syscalls::socket(ai.ai_family, ai.ai_socktype, ai.ai_protocol) };
        if fd < 0 {
            unsafe { syscalls::freeaddrinfo(result); }
            return Err(IoError::last_os_error());
        }

        // Set SO_REUSEADDR
        let one: i32 = 1;
        unsafe {
            syscalls::setsockopt(
                fd,
                syscalls::SOL_SOCKET,
                syscalls::SO_REUSEADDR,
                &one as *const i32 as *const syscalls::c_void,
                core::mem::size_of::<i32>() as u32,
            );
        }

        let bind_ret = unsafe {
            syscalls::bind(fd, ai.ai_addr as *const syscalls::sockaddr, ai.ai_addrlen)
        };

        unsafe { syscalls::freeaddrinfo(result); }

        if bind_ret < 0 {
            unsafe { syscalls::close(fd); }
            return Err(IoError::last_os_error());
        }

        let listen_ret = unsafe { syscalls::listen(fd, 128) };
        if listen_ret < 0 {
            unsafe { syscalls::close(fd); }
            return Err(IoError::last_os_error());
        }

        Ok(Self { fd })
    }

    /// Accept a new connection.
    pub fn accept(&self) -> Result<TcpStream> {
        let client_fd = unsafe {
            syscalls::accept(self.fd, core::ptr::null_mut(), core::ptr::null_mut())
        };
        if client_fd < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(TcpStream { fd: client_fd })
    }

    /// Set non-blocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
        let flags = unsafe { syscalls::fcntl(self.fd, syscalls::F_GETFL) };
        if flags < 0 {
            return Err(IoError::last_os_error());
        }
        let new_flags = if nonblocking {
            flags | syscalls::O_NONBLOCK
        } else {
            flags & !syscalls::O_NONBLOCK
        };
        let ret = unsafe { syscalls::fcntl(self.fd, syscalls::F_SETFL, new_flags) };
        if ret < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(())
    }

    /// Returns the raw file descriptor.
    pub fn as_raw_fd(&self) -> i32 {
        self.fd
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe { syscalls::close(self.fd); }
        }
    }
}

/// Extract the peer's IPv4 address from a connected socket fd.
/// Returns the IPv4 address as a network-order u32, or None on failure.
pub fn peer_ip_from_fd(fd: i32) -> Option<u32> {
    let mut addr: syscalls::sockaddr_in = unsafe { core::mem::zeroed() };
    let mut addrlen = core::mem::size_of::<syscalls::sockaddr_in>() as u32;
    let ret = unsafe {
        syscalls::getpeername(
            fd,
            &mut addr as *mut syscalls::sockaddr_in as *mut syscalls::sockaddr,
            &mut addrlen,
        )
    };
    if ret == 0 && addr.sin_family == syscalls::AF_INET as u16 {
        Some(addr.sin_addr)
    } else {
        None
    }
}

fn port_to_str(port: u16, buf: &mut [u8; 8]) -> &str {
    let mut val = port as u32;
    let mut pos = 8;
    if val == 0 {
        pos -= 1;
        buf[pos] = b'0';
    } else {
        while val > 0 {
            pos -= 1;
            buf[pos] = b'0' + (val % 10) as u8;
            val /= 10;
        }
    }
    unsafe { core::str::from_utf8_unchecked(&buf[pos..]) }
}
