//! Platform-abstracted poller (kqueue on macOS, epoll on Linux).

use crate::core::volkiwithstds::io::error::{IoError, Result};
use crate::core::volkiwithstds::sys::syscalls;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interest {
    Read,
    Write,
    ReadWrite,
}

#[derive(Clone, Copy)]
pub struct Event {
    pub fd: i32,
    pub readable: bool,
    pub writable: bool,
    pub error: bool,
    pub hangup: bool,
}

pub struct Poller {
    fd: i32,
}

// ── macOS (kqueue) ──────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
impl Poller {
    pub fn new() -> Result<Self> {
        let fd = unsafe { syscalls::kqueue() };
        if fd < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(Self { fd })
    }

    pub fn register(&self, target_fd: i32, interest: Interest) -> Result<()> {
        let mut changes: [syscalls::kevent; 2] = unsafe { core::mem::zeroed() };
        let mut nchanges = 0;

        if matches!(interest, Interest::Read | Interest::ReadWrite) {
            changes[nchanges] = syscalls::kevent {
                ident: target_fd as usize,
                filter: syscalls::EVFILT_READ,
                flags: syscalls::EV_ADD | syscalls::EV_CLEAR,
                fflags: 0,
                data: 0,
                udata: core::ptr::null_mut(),
            };
            nchanges += 1;
        }

        if matches!(interest, Interest::Write | Interest::ReadWrite) {
            changes[nchanges] = syscalls::kevent {
                ident: target_fd as usize,
                filter: syscalls::EVFILT_WRITE,
                flags: syscalls::EV_ADD | syscalls::EV_CLEAR,
                fflags: 0,
                data: 0,
                udata: core::ptr::null_mut(),
            };
            nchanges += 1;
        }

        let ret = unsafe {
            syscalls::kevent(
                self.fd,
                changes.as_ptr(),
                nchanges as i32,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
            )
        };
        if ret < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(())
    }

    pub fn modify(&self, target_fd: i32, interest: Interest) -> Result<()> {
        // kqueue: delete both, re-add desired
        self.deregister(target_fd)?;
        self.register(target_fd, interest)
    }

    pub fn deregister(&self, target_fd: i32) -> Result<()> {
        let changes = [
            syscalls::kevent {
                ident: target_fd as usize,
                filter: syscalls::EVFILT_READ,
                flags: syscalls::EV_DELETE,
                fflags: 0,
                data: 0,
                udata: core::ptr::null_mut(),
            },
            syscalls::kevent {
                ident: target_fd as usize,
                filter: syscalls::EVFILT_WRITE,
                flags: syscalls::EV_DELETE,
                fflags: 0,
                data: 0,
                udata: core::ptr::null_mut(),
            },
        ];

        // Ignore errors — filter may not be registered
        unsafe {
            syscalls::kevent(
                self.fd,
                changes.as_ptr(),
                2,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
            );
        }
        Ok(())
    }

    pub fn poll(&self, events: &mut [Event], timeout_ms: i32) -> Result<usize> {
        let max = events.len();
        let mut raw_events: [syscalls::kevent; 256] = unsafe { core::mem::zeroed() };
        let nevents = max.min(256);

        let timeout = if timeout_ms >= 0 {
            let ts = syscalls::timespec {
                tv_sec: (timeout_ms / 1000) as i64,
                tv_nsec: ((timeout_ms % 1000) as i64) * 1_000_000,
            };
            &ts as *const syscalls::timespec
        } else {
            core::ptr::null()
        };

        let ret = unsafe {
            syscalls::kevent(
                self.fd,
                core::ptr::null(),
                0,
                raw_events.as_mut_ptr(),
                nevents as i32,
                timeout,
            )
        };

        if ret < 0 {
            return Err(IoError::last_os_error());
        }

        let count = ret as usize;
        for i in 0..count {
            let ev = &raw_events[i];
            events[i] = Event {
                fd: ev.ident as i32,
                readable: ev.filter == syscalls::EVFILT_READ,
                writable: ev.filter == syscalls::EVFILT_WRITE,
                error: (ev.flags & syscalls::EV_ERROR) != 0,
                hangup: (ev.flags & syscalls::EV_EOF) != 0,
            };
        }

        Ok(count)
    }
}

// ── Linux (epoll) ───────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
impl Poller {
    pub fn new() -> Result<Self> {
        let fd = unsafe { syscalls::epoll_create1(0) };
        if fd < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(Self { fd })
    }

    fn interest_to_events(interest: Interest) -> u32 {
        let mut events = syscalls::EPOLLET;
        match interest {
            Interest::Read => events |= syscalls::EPOLLIN,
            Interest::Write => events |= syscalls::EPOLLOUT,
            Interest::ReadWrite => events |= syscalls::EPOLLIN | syscalls::EPOLLOUT,
        }
        events
    }

    pub fn register(&self, target_fd: i32, interest: Interest) -> Result<()> {
        let mut ev = syscalls::epoll_event {
            events: Self::interest_to_events(interest),
            data: target_fd as u64,
        };
        let ret = unsafe {
            syscalls::epoll_ctl(self.fd, syscalls::EPOLL_CTL_ADD, target_fd, &mut ev)
        };
        if ret < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(())
    }

    pub fn modify(&self, target_fd: i32, interest: Interest) -> Result<()> {
        let mut ev = syscalls::epoll_event {
            events: Self::interest_to_events(interest),
            data: target_fd as u64,
        };
        let ret = unsafe {
            syscalls::epoll_ctl(self.fd, syscalls::EPOLL_CTL_MOD, target_fd, &mut ev)
        };
        if ret < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(())
    }

    pub fn deregister(&self, target_fd: i32) -> Result<()> {
        let ret = unsafe {
            syscalls::epoll_ctl(
                self.fd,
                syscalls::EPOLL_CTL_DEL,
                target_fd,
                core::ptr::null_mut(),
            )
        };
        if ret < 0 {
            return Err(IoError::last_os_error());
        }
        Ok(())
    }

    pub fn poll(&self, events: &mut [Event], timeout_ms: i32) -> Result<usize> {
        let max = events.len().min(256);
        let mut raw_events: [syscalls::epoll_event; 256] = unsafe { core::mem::zeroed() };

        let ret = unsafe {
            syscalls::epoll_wait(self.fd, raw_events.as_mut_ptr(), max as i32, timeout_ms)
        };

        if ret < 0 {
            return Err(IoError::last_os_error());
        }

        let count = ret as usize;
        for i in 0..count {
            let ev = &raw_events[i];
            events[i] = Event {
                fd: ev.data as i32,
                readable: (ev.events & syscalls::EPOLLIN) != 0,
                writable: (ev.events & syscalls::EPOLLOUT) != 0,
                error: (ev.events & syscalls::EPOLLERR) != 0,
                hangup: (ev.events & (syscalls::EPOLLHUP | syscalls::EPOLLRDHUP)) != 0,
            };
        }

        Ok(count)
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        unsafe {
            syscalls::close(self.fd);
        }
    }
}
