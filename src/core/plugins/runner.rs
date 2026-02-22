use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::io::IoErrorKind;
use crate::core::volkiwithstds::process::Command;
use crate::vformat;

use crate::{log_debug, log_error};

use super::protocol::{PluginRequest, PluginResponse, parse_response};
use super::types::{PluginError, ResolvedPlugin};

pub fn invoke(
    plugin: &ResolvedPlugin,
    request: &PluginRequest,
) -> Result<PluginResponse, PluginError> {
    log_debug!(
        "invoking plugin '{}' ({})",
        plugin.name,
        plugin.runtime.command()
    );
    let json = request.to_json();

    let json_bytes = {
        let bytes = json.as_bytes();
        let mut v = Vec::new();
        for &b in bytes {
            v.push(b);
        }
        v
    };

    let output = Command::new(plugin.runtime.command())
        .arg(plugin.entry_point.as_str())
        .stdin_data(json_bytes)
        .output()
        .map_err(|e| {
            if e.kind() == IoErrorKind::NotFound {
                PluginError::RuntimeNotAvailable(String::from(plugin.runtime.command()))
            } else {
                PluginError::SpawnFailed(vformat!("{e}"))
            }
        })?;

    if !output.stderr.is_empty() {
        let stderr_str = match core::str::from_utf8(output.stderr.as_slice()) {
            Ok(s) => String::from(s),
            Err(_) => String::from("(non-utf8 output)"),
        };
        if !output.status.success() {
            log_error!("plugin '{}' stderr: {}", plugin.name, stderr_str.trim());
            return Err(PluginError::PluginStderr(stderr_str));
        }
        log_debug!("plugin '{}' stderr: {}", plugin.name, stderr_str.trim());
    }

    if !output.status.success() {
        return Err(PluginError::SpawnFailed(vformat!(
            "plugin exited with code {}",
            output.status
        )));
    }

    let stdout_str = match core::str::from_utf8(output.stdout.as_slice()) {
        Ok(s) => String::from(s),
        Err(_) => {
            return Err(PluginError::InvalidResponse(String::from(
                "non-utf8 stdout",
            )));
        }
    };
    parse_response(&stdout_str).map_err(|e| PluginError::InvalidResponse(e))
}
