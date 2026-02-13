use std::io::Write;
use std::process::{Command, Stdio};

use crate::{log_debug, log_error};

use super::protocol::{PluginRequest, PluginResponse, parse_response};
use super::types::{PluginError, ResolvedPlugin};

pub fn invoke(
    plugin: &ResolvedPlugin,
    request: &PluginRequest,
) -> Result<PluginResponse, PluginError> {
    log_debug!("invoking plugin '{}' ({})", plugin.name, plugin.runtime.command());
    let json = request.to_json();

    let mut child = Command::new(plugin.runtime.command())
        .arg(&plugin.entry_point)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                PluginError::RuntimeNotAvailable(plugin.runtime.command().to_string())
            } else {
                PluginError::SpawnFailed(e.to_string())
            }
        })?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(json.as_bytes()).map_err(PluginError::IoError)?;
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().map_err(PluginError::IoError)?;

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            log_error!("plugin '{}' stderr: {}", plugin.name, stderr.trim());
            return Err(PluginError::PluginStderr(stderr.to_string()));
        }
        log_debug!("plugin '{}' stderr: {}", plugin.name, stderr.trim());
    }

    if !output.status.success() {
        return Err(PluginError::SpawnFailed(format!(
            "plugin exited with code {}",
            output.status
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_response(&stdout).map_err(|e| PluginError::InvalidResponse(e))
}
