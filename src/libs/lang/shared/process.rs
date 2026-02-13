use std::fmt;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

#[derive(Debug)]
pub enum ProcessError {
    Io(io::Error),
    NotFound(String),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::Io(e) => write!(f, "process error: {e}"),
            ProcessError::NotFound(prog) => write!(f, "command not found: {prog}"),
        }
    }
}

impl From<io::Error> for ProcessError {
    fn from(e: io::Error) -> Self {
        ProcessError::Io(e)
    }
}

pub fn run_command(program: &str, args: &[&str], dir: &Path) -> Result<String, ProcessError> {
    let output = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                ProcessError::NotFound(program.to_string())
            } else {
                ProcessError::Io(e)
            }
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(ProcessError::Io(io::Error::new(
            io::ErrorKind::Other,
            stderr,
        )))
    }
}

pub fn run_command_allow_failure(
    program: &str,
    args: &[&str],
    dir: &Path,
) -> Result<CommandOutput, ProcessError> {
    let output = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                ProcessError::NotFound(program.to_string())
            } else {
                ProcessError::Io(e)
            }
        })?;

    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn run_echo() {
        let result = run_command("echo", &["hello"], Path::new("."));
        assert!(result.is_ok());
        assert!(result.unwrap().trim() == "hello");
    }

    #[test]
    fn run_nonexistent_command() {
        let result = run_command("volki_nonexistent_cmd_xyz", &[], Path::new("."));
        assert!(matches!(result, Err(ProcessError::NotFound(_))));
    }

    #[test]
    fn run_allow_failure_false_returns_output() {
        let result = run_command_allow_failure("ls", &["--nonexistent-flag-xyz"], Path::new("."));
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.success);
    }

    #[test]
    fn run_allow_failure_success() {
        let result = run_command_allow_failure("echo", &["test"], Path::new("."));
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.stdout.trim() == "test");
    }

    #[test]
    fn process_error_display() {
        let err = ProcessError::NotFound("npm".to_string());
        assert!(format!("{err}").contains("npm"));

        let err = ProcessError::Io(io::Error::new(io::ErrorKind::Other, "fail"));
        assert!(format!("{err}").contains("process error"));
    }
}
