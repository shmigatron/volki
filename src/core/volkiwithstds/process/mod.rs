//! Process management — Command, Output, ExitStatus.

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::io::error::{IoError, Result};
use crate::core::volkiwithstds::path::CString;
use crate::core::volkiwithstds::sys::{errno, syscalls};

/// What to do with a stdio stream.
pub enum Stdio {
    /// Inherit from parent.
    Inherit,
    /// Pipe to capture output.
    Piped,
    /// Discard (redirect to /dev/null).
    Null,
}

/// The result of a process execution.
pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// The exit status of a process.
#[derive(Debug, Clone, Copy)]
pub struct ExitStatus {
    raw: i32,
}

impl ExitStatus {
    /// Returns true if the process exited successfully (code 0).
    pub fn success(&self) -> bool {
        self.code() == Some(0)
    }

    /// Returns the exit code, if the process exited normally.
    pub fn code(&self) -> Option<i32> {
        // WIFEXITED: (status & 0x7f) == 0
        if (self.raw & 0x7f) == 0 {
            // WEXITSTATUS: (status >> 8) & 0xff
            Some((self.raw >> 8) & 0xff)
        } else {
            None
        }
    }
}

impl core::fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.code() {
            Some(code) => write!(f, "exit code: {}", code),
            None => write!(f, "signal: {}", self.raw & 0x7f),
        }
    }
}

/// A builder for spawning a child process.
pub struct Command {
    program: String,
    args: Vec<String>,
    current_dir: Option<String>,
    stdout_cfg: Stdio,
    stderr_cfg: Stdio,
    stdin_data: Option<Vec<u8>>,
}

impl Command {
    /// Create a new Command for the given program.
    pub fn new(program: &str) -> Self {
        Self {
            program: String::from(program),
            args: Vec::new(),
            current_dir: None,
            stdout_cfg: Stdio::Piped,
            stderr_cfg: Stdio::Piped,
            stdin_data: None,
        }
    }

    /// Add an argument.
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(String::from(arg));
        self
    }

    /// Add multiple arguments.
    pub fn args(&mut self, args: &[&str]) -> &mut Self {
        for arg in args {
            self.args.push(String::from(*arg));
        }
        self
    }

    /// Set the working directory for the child process.
    pub fn current_dir(&mut self, dir: &str) -> &mut Self {
        self.current_dir = Some(String::from(dir));
        self
    }

    /// Configure stdout handling.
    pub fn stdout(&mut self, cfg: Stdio) -> &mut Self {
        self.stdout_cfg = cfg;
        self
    }

    /// Configure stderr handling.
    pub fn stderr(&mut self, cfg: Stdio) -> &mut Self {
        self.stderr_cfg = cfg;
        self
    }

    /// Set data to write to the child's stdin.
    pub fn stdin_data(&mut self, data: Vec<u8>) -> &mut Self {
        self.stdin_data = Some(data);
        self
    }

    /// Execute the command and collect its output.
    pub fn output(&mut self) -> Result<Output> {
        // Create pipes for stdout and stderr
        let mut stdout_pipe = [0i32; 2];
        let mut stderr_pipe = [0i32; 2];
        let mut stdin_pipe = [0i32; 2];

        let capture_stdout = matches!(self.stdout_cfg, Stdio::Piped);
        let capture_stderr = matches!(self.stderr_cfg, Stdio::Piped);
        let has_stdin_data = self.stdin_data.is_some();

        if has_stdin_data {
            if unsafe { syscalls::pipe(stdin_pipe.as_mut_ptr()) } != 0 {
                return Err(IoError::last_os_error());
            }
        }

        if capture_stdout {
            if unsafe { syscalls::pipe(stdout_pipe.as_mut_ptr()) } != 0 {
                if has_stdin_data {
                    unsafe {
                        syscalls::close(stdin_pipe[0]);
                        syscalls::close(stdin_pipe[1]);
                    }
                }
                return Err(IoError::last_os_error());
            }
        }

        if capture_stderr {
            if unsafe { syscalls::pipe(stderr_pipe.as_mut_ptr()) } != 0 {
                if capture_stdout {
                    unsafe {
                        syscalls::close(stdout_pipe[0]);
                        syscalls::close(stdout_pipe[1]);
                    }
                }
                if has_stdin_data {
                    unsafe {
                        syscalls::close(stdin_pipe[0]);
                        syscalls::close(stdin_pipe[1]);
                    }
                }
                return Err(IoError::last_os_error());
            }
        }

        let pid = unsafe { syscalls::fork() };
        if pid < 0 {
            return Err(IoError::last_os_error());
        }

        if pid == 0 {
            // ── Child process ──

            // Redirect stdin if piping data
            if has_stdin_data {
                unsafe {
                    syscalls::close(stdin_pipe[1]); // close write end in child
                    syscalls::dup2(stdin_pipe[0], syscalls::STDIN_FILENO);
                    syscalls::close(stdin_pipe[0]);
                }
            }

            // Redirect stdout
            match &self.stdout_cfg {
                Stdio::Piped => unsafe {
                    syscalls::close(stdout_pipe[0]);
                    syscalls::dup2(stdout_pipe[1], syscalls::STDOUT_FILENO);
                    syscalls::close(stdout_pipe[1]);
                },
                Stdio::Null => {
                    let dev_null = CString::new("/dev/null");
                    let fd = unsafe { syscalls::open(dev_null.as_ptr(), syscalls::O_WRONLY) };
                    if fd >= 0 {
                        unsafe {
                            syscalls::dup2(fd, syscalls::STDOUT_FILENO);
                            syscalls::close(fd);
                        }
                    }
                }
                Stdio::Inherit => {}
            }

            // Redirect stderr
            match &self.stderr_cfg {
                Stdio::Piped => unsafe {
                    syscalls::close(stderr_pipe[0]);
                    syscalls::dup2(stderr_pipe[1], syscalls::STDERR_FILENO);
                    syscalls::close(stderr_pipe[1]);
                },
                Stdio::Null => {
                    let dev_null = CString::new("/dev/null");
                    let fd = unsafe { syscalls::open(dev_null.as_ptr(), syscalls::O_WRONLY) };
                    if fd >= 0 {
                        unsafe {
                            syscalls::dup2(fd, syscalls::STDERR_FILENO);
                            syscalls::close(fd);
                        }
                    }
                }
                Stdio::Inherit => {}
            }

            // Change directory if requested
            if let Some(ref dir) = self.current_dir {
                let c_dir = CString::new(dir.as_str());
                unsafe {
                    syscalls::chdir(c_dir.as_ptr());
                }
            }

            // Build argv
            let c_program = CString::new(self.program.as_str());
            let mut c_args: Vec<CString> = Vec::with_capacity(self.args.len());
            for arg in self.args.iter() {
                c_args.push(CString::new(arg.as_str()));
            }

            // argv: [program, arg1, arg2, ..., NULL]
            let mut argv: Vec<*const syscalls::c_char> =
                Vec::with_capacity(self.args.len() + 2);
            argv.push(c_program.as_ptr());
            for c_arg in c_args.iter() {
                argv.push(c_arg.as_ptr());
            }
            argv.push(core::ptr::null());

            unsafe {
                syscalls::execvp(c_program.as_ptr(), argv.as_ptr());
                // If execvp returns, it failed
                syscalls::_exit(127);
            }
        }

        // ── Parent process ──

        // Close read end of stdin pipe, write data, then close write end
        if has_stdin_data {
            unsafe {
                syscalls::close(stdin_pipe[0]); // close read end in parent
            }
            if let Some(ref data) = self.stdin_data {
                let mut offset = 0;
                while offset < data.len() {
                    let remaining = &data.as_slice()[offset..];
                    let n = unsafe {
                        syscalls::write(
                            stdin_pipe[1],
                            remaining.as_ptr() as *const syscalls::c_void,
                            remaining.len(),
                        )
                    };
                    if n <= 0 {
                        break;
                    }
                    offset += n as usize;
                }
            }
            unsafe {
                syscalls::close(stdin_pipe[1]);
            }
        }

        // Close write ends of pipes
        if capture_stdout {
            unsafe {
                syscalls::close(stdout_pipe[1]);
            }
        }
        if capture_stderr {
            unsafe {
                syscalls::close(stderr_pipe[1]);
            }
        }

        // Read stdout
        let stdout_bytes = if capture_stdout {
            read_pipe(stdout_pipe[0])
        } else {
            Vec::new()
        };

        // Read stderr
        let stderr_bytes = if capture_stderr {
            read_pipe(stderr_pipe[0])
        } else {
            Vec::new()
        };

        // Close read ends
        if capture_stdout {
            unsafe {
                syscalls::close(stdout_pipe[0]);
            }
        }
        if capture_stderr {
            unsafe {
                syscalls::close(stderr_pipe[0]);
            }
        }

        // Wait for child
        let mut status: i32 = 0;
        loop {
            let ret = unsafe { syscalls::waitpid(pid, &mut status, 0) };
            if ret < 0 {
                let err = errno::get_errno();
                if err == errno::EINTR {
                    continue;
                }
                return Err(IoError::from_errno(err));
            }
            break;
        }

        Ok(Output {
            status: ExitStatus { raw: status },
            stdout: stdout_bytes,
            stderr: stderr_bytes,
        })
    }

    /// Run the command and check for success.
    pub fn status(&mut self) -> Result<ExitStatus> {
        self.stdout(Stdio::Inherit).stderr(Stdio::Inherit);
        let output = self.output()?;
        Ok(output.status)
    }
}

/// Read all data from a pipe fd.
fn read_pipe(fd: i32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe {
            syscalls::read(fd, buf.as_mut_ptr() as *mut syscalls::c_void, buf.len())
        };
        if n <= 0 {
            break;
        }
        result.extend_from_slice(&buf[..n as usize]);
    }
    result
}

/// Returns the current process ID.
pub fn id() -> u32 {
    unsafe { syscalls::getpid() as u32 }
}

/// Exit the current process with the given code.
pub fn exit(code: i32) -> ! {
    unsafe { syscalls::_exit(code) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_echo() {
        let output = Command::new("echo").args(&["hello"]).output().unwrap();
        assert!(output.status.success());
        let stdout_str = core::str::from_utf8(output.stdout.as_slice()).unwrap();
        assert_eq!(stdout_str.trim(), "hello");
    }

    #[test]
    fn test_command_false() {
        let output = Command::new("false").output().unwrap();
        assert!(!output.status.success());
    }

    #[test]
    fn test_exit_status_code() {
        let output = Command::new("sh").args(&["-c", "exit 42"]).output().unwrap();
        assert_eq!(output.status.code(), Some(42));
    }
}
