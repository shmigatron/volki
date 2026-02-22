//! All extern "C" libc declarations and platform-specific constants.

#![allow(non_camel_case_types, dead_code)]

// ── Platform-specific type aliases ──────────────────────────────────────────

pub type c_int = i32;
pub type c_uint = u32;
pub type c_long = i64;
pub type c_ulong = u64;
pub type c_char = i8;
pub type c_void = core::ffi::c_void;
pub type size_t = usize;
pub type ssize_t = isize;
pub type off_t = i64;
pub type mode_t = u16;
pub type pid_t = i32;

#[cfg(target_os = "macos")]
pub type pthread_t = *mut u8;
#[cfg(target_os = "linux")]
pub type pthread_t = u64;

// ── Opaque directory types ──────────────────────────────────────────────────

#[repr(C)]
pub struct DIR {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct addrinfo {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: u32,
    #[cfg(target_os = "macos")]
    pub ai_canonname: *mut c_char,
    #[cfg(target_os = "macos")]
    pub ai_addr: *mut sockaddr,
    #[cfg(target_os = "linux")]
    pub ai_addr: *mut sockaddr,
    #[cfg(target_os = "linux")]
    pub ai_canonname: *mut c_char,
    pub ai_next: *mut addrinfo,
}

#[repr(C)]
pub struct sockaddr {
    #[cfg(target_os = "macos")]
    pub sa_len: u8,
    pub sa_family: u16,
    pub sa_data: [u8; 14],
}

#[repr(C)]
pub struct sockaddr_in {
    #[cfg(target_os = "macos")]
    pub sin_len: u8,
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub sin_zero: [u8; 8],
}

#[repr(C)]
pub struct timespec {
    pub tv_sec: c_long,
    pub tv_nsec: c_long,
}

// ── stat struct ─────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct stat_buf {
    pub st_dev: i32,
    pub st_mode: u16,
    pub st_nlink: u16,
    pub st_ino: u64,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: i32,
    pub st_atime: c_long,
    pub st_atime_nsec: c_long,
    pub st_mtime: c_long,
    pub st_mtime_nsec: c_long,
    pub st_ctime: c_long,
    pub st_ctime_nsec: c_long,
    pub st_birthtime: c_long,
    pub st_birthtime_nsec: c_long,
    pub st_size: off_t,
    pub st_blocks: i64,
    pub st_blksize: i32,
    pub st_flags: u32,
    pub st_gen: u32,
    pub st_lspare: i32,
    pub st_qspare: [i64; 2],
}

#[cfg(target_os = "linux")]
#[repr(C)]
pub struct stat_buf {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub __pad0: u32,
    pub st_rdev: u64,
    pub st_size: off_t,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: c_long,
    pub st_atime_nsec: c_long,
    pub st_mtime: c_long,
    pub st_mtime_nsec: c_long,
    pub st_ctime: c_long,
    pub st_ctime_nsec: c_long,
    pub __unused: [c_long; 3],
}

// ── dirent struct ───────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct dirent {
    pub d_ino: u64,
    pub d_seekoff: u64,
    pub d_reclen: u16,
    pub d_namlen: u16,
    pub d_type: u8,
    pub d_name: [c_char; 1024],
}

#[cfg(target_os = "linux")]
#[repr(C)]
pub struct dirent {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [c_char; 256],
}

// ── Platform-specific constants ─────────────────────────────────────────────

// mmap
pub const PROT_READ: c_int = 0x1;
pub const PROT_WRITE: c_int = 0x2;
pub const MAP_PRIVATE: c_int = 0x02;
pub const MAP_FAILED: *mut c_void = !0usize as *mut c_void;

#[cfg(target_os = "macos")]
pub const MAP_ANONYMOUS: c_int = 0x1000;
#[cfg(target_os = "linux")]
pub const MAP_ANONYMOUS: c_int = 0x20;

// open flags
pub const O_RDONLY: c_int = 0x0;
pub const O_WRONLY: c_int = 0x1;
pub const O_RDWR: c_int = 0x2;

#[cfg(target_os = "macos")]
pub const O_CREAT: c_int = 0x200;
#[cfg(target_os = "linux")]
pub const O_CREAT: c_int = 0x40;

#[cfg(target_os = "macos")]
pub const O_TRUNC: c_int = 0x400;
#[cfg(target_os = "linux")]
pub const O_TRUNC: c_int = 0x200;

#[cfg(target_os = "macos")]
pub const O_APPEND: c_int = 0x8;
#[cfg(target_os = "linux")]
pub const O_APPEND: c_int = 0x400;

// clock
#[cfg(target_os = "macos")]
pub const CLOCK_MONOTONIC: c_int = 6;
#[cfg(target_os = "linux")]
pub const CLOCK_MONOTONIC: c_int = 1;

// stat mode bits
pub const S_IFMT: u32 = 0o170000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFREG: u32 = 0o100000;

// socket
pub const AF_INET: c_int = 2;
pub const AI_PASSIVE: c_int = 1;
pub const SOCK_STREAM: c_int = 1;
pub const SOL_SOCKET: c_int = {
    #[cfg(target_os = "macos")]
    { 0xffff }
    #[cfg(target_os = "linux")]
    { 1 }
};
pub const SO_REUSEADDR: c_int = {
    #[cfg(target_os = "macos")]
    { 0x0004 }
    #[cfg(target_os = "linux")]
    { 2 }
};
pub const SOMAXCONN: c_int = 128;

// fcntl
pub const F_GETFL: c_int = 3;
pub const F_SETFL: c_int = 4;

#[cfg(target_os = "macos")]
pub const O_NONBLOCK: c_int = 0x0004;
#[cfg(target_os = "linux")]
pub const O_NONBLOCK: c_int = 0o4000;

// shutdown
pub const SHUT_RD: c_int = 0;
pub const SHUT_WR: c_int = 1;
pub const SHUT_RDWR: c_int = 2;

// file permissions
pub const S_IRWXU: mode_t = 0o700;
pub const S_IRWXG: mode_t = 0o070;
pub const S_IRWXO: mode_t = 0o007;

// waitpid
pub const WNOHANG: c_int = 1;

// lseek
pub const SEEK_SET: c_int = 0;
pub const SEEK_END: c_int = 2;

// pipe/dup
pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

// ── kqueue (macOS) ─────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub const EVFILT_READ: i16 = -1;
#[cfg(target_os = "macos")]
pub const EVFILT_WRITE: i16 = -2;
#[cfg(target_os = "macos")]
pub const EV_ADD: u16 = 0x0001;
#[cfg(target_os = "macos")]
pub const EV_DELETE: u16 = 0x0002;
#[cfg(target_os = "macos")]
pub const EV_ENABLE: u16 = 0x0004;
#[cfg(target_os = "macos")]
pub const EV_CLEAR: u16 = 0x0020;
#[cfg(target_os = "macos")]
pub const EV_EOF: u16 = 0x8000;
#[cfg(target_os = "macos")]
pub const EV_ERROR: u16 = 0x4000;

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct kevent {
    pub ident: usize,
    pub filter: i16,
    pub flags: u16,
    pub fflags: u32,
    pub data: isize,
    pub udata: *mut c_void,
}

// ── epoll (Linux) ──────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub const EPOLL_CTL_ADD: c_int = 1;
#[cfg(target_os = "linux")]
pub const EPOLL_CTL_DEL: c_int = 2;
#[cfg(target_os = "linux")]
pub const EPOLL_CTL_MOD: c_int = 3;
#[cfg(target_os = "linux")]
pub const EPOLLIN: u32 = 0x001;
#[cfg(target_os = "linux")]
pub const EPOLLOUT: u32 = 0x004;
#[cfg(target_os = "linux")]
pub const EPOLLET: u32 = 1 << 31;
#[cfg(target_os = "linux")]
pub const EPOLLHUP: u32 = 0x010;
#[cfg(target_os = "linux")]
pub const EPOLLERR: u32 = 0x008;
#[cfg(target_os = "linux")]
pub const EPOLLRDHUP: u32 = 0x2000;

#[cfg(target_os = "linux")]
#[repr(C, packed)]
pub struct epoll_event {
    pub events: u32,
    pub data: u64,
}

// ── extern "C" declarations ────────────────────────────────────────────────

unsafe extern "C" {
    // Memory
    pub fn mmap(
        addr: *mut c_void,
        len: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> *mut c_void;
    pub fn munmap(addr: *mut c_void, len: size_t) -> c_int;

    // File I/O
    pub fn open(path: *const c_char, flags: c_int, ...) -> c_int;
    pub fn close(fd: c_int) -> c_int;
    pub fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t;
    pub fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t;
    pub fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t;

    // Metadata
    pub fn stat(path: *const c_char, buf: *mut stat_buf) -> c_int;
    pub fn fstat(fd: c_int, buf: *mut stat_buf) -> c_int;

    // Directories
    pub fn opendir(path: *const c_char) -> *mut DIR;
    pub fn readdir(dirp: *mut DIR) -> *mut dirent;
    pub fn closedir(dirp: *mut DIR) -> c_int;
    pub fn mkdir(path: *const c_char, mode: mode_t) -> c_int;
    pub fn rmdir(path: *const c_char) -> c_int;
    pub fn unlink(path: *const c_char) -> c_int;
    pub fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char;
    pub fn realpath(path: *const c_char, resolved: *mut c_char) -> *mut c_char;
    pub fn chdir(path: *const c_char) -> c_int;

    // Networking
    pub fn socket(domain: c_int, socktype: c_int, protocol: c_int) -> c_int;
    pub fn connect(fd: c_int, addr: *const sockaddr, addrlen: u32) -> c_int;
    pub fn bind(fd: c_int, addr: *const sockaddr, addrlen: u32) -> c_int;
    pub fn listen(fd: c_int, backlog: c_int) -> c_int;
    pub fn accept(fd: c_int, addr: *mut sockaddr, addrlen: *mut u32) -> c_int;
    pub fn setsockopt(
        fd: c_int,
        level: c_int,
        optname: c_int,
        optval: *const c_void,
        optlen: u32,
    ) -> c_int;
    pub fn shutdown(fd: c_int, how: c_int) -> c_int;
    pub fn getpeername(fd: c_int, addr: *mut sockaddr, addrlen: *mut u32) -> c_int;
    pub fn getaddrinfo(
        node: *const c_char,
        service: *const c_char,
        hints: *const addrinfo,
        res: *mut *mut addrinfo,
    ) -> c_int;
    pub fn freeaddrinfo(res: *mut addrinfo);
    pub fn htons(hostshort: u16) -> u16;
    pub fn ntohs(netshort: u16) -> u16;

    // fcntl
    pub fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;

    // Threading
    pub fn pthread_create(
        thread: *mut pthread_t,
        attr: *const c_void,
        start_routine: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> c_int;
    pub fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int;
    pub fn pthread_detach(thread: pthread_t) -> c_int;

    // Time
    pub fn clock_gettime(clk_id: c_int, tp: *mut timespec) -> c_int;
    pub fn nanosleep(req: *const timespec, rem: *mut timespec) -> c_int;

    // Process
    pub fn fork() -> pid_t;
    pub fn execvp(file: *const c_char, argv: *const *const c_char) -> c_int;
    pub fn waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> pid_t;
    pub fn pipe(pipefd: *mut c_int) -> c_int;
    pub fn dup2(oldfd: c_int, newfd: c_int) -> c_int;
    pub fn _exit(status: c_int) -> !;

    // Process info
    pub fn getpid() -> pid_t;

    // Environment
    pub fn getenv(name: *const c_char) -> *const c_char;
    pub fn strlen(s: *const c_char) -> size_t;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: size_t) -> *mut c_void;

    // macOS-specific for args
    #[cfg(target_os = "macos")]
    pub fn _NSGetArgc() -> *mut c_int;
    #[cfg(target_os = "macos")]
    pub fn _NSGetArgv() -> *mut *mut *mut c_char;

    // kqueue (macOS)
    #[cfg(target_os = "macos")]
    pub fn kqueue() -> c_int;
    #[cfg(target_os = "macos")]
    pub fn kevent(
        kq: c_int,
        changelist: *const kevent,
        nchanges: c_int,
        eventlist: *mut kevent,
        nevents: c_int,
        timeout: *const timespec,
    ) -> c_int;

    // epoll (Linux)
    #[cfg(target_os = "linux")]
    pub fn epoll_create1(flags: c_int) -> c_int;
    #[cfg(target_os = "linux")]
    pub fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int;
    #[cfg(target_os = "linux")]
    pub fn epoll_wait(epfd: c_int, events: *mut epoll_event, maxevents: c_int, timeout: c_int) -> c_int;
}

/// Helper: compute the length of a C string.
///
/// # Safety
/// `s` must point to a valid null-terminated C string.
pub unsafe fn c_strlen(s: *const c_char) -> usize {
    unsafe { strlen(s) }
}
