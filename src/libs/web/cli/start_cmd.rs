//! web:start — start the web server.

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::time::Duration;
use crate::libs::web::server::Server;
use crate::veprintln;

pub struct WebStartCommand;

impl Command for WebStartCommand {
    fn name(&self) -> &str {
        "web:start"
    }

    fn description(&self) -> &str {
        "Start the web server"
    }

    fn long_description(&self) -> &str {
        "Starts a web server. Users configure routes programmatically."
    }

    fn options(&self) -> Vec<OptionSpec> {
        let mut opts = Vec::new();
        opts.push(OptionSpec {
            name: "port",
            description: "Port to listen on",
            takes_value: true,
            required: false,
            default_value: Some("3000"),
            short: Some('p'),
        });
        opts.push(OptionSpec {
            name: "host",
            description: "Host to bind to",
            takes_value: true,
            required: false,
            default_value: Some("127.0.0.1"),
            short: None,
        });
        opts.push(OptionSpec {
            name: "tls-cert",
            description: "Path to TLS certificate PEM file",
            takes_value: true,
            required: false,
            default_value: None,
            short: None,
        });
        opts.push(OptionSpec {
            name: "tls-key",
            description: "Path to TLS private key PEM file",
            takes_value: true,
            required: false,
            default_value: None,
            short: None,
        });
        opts.push(OptionSpec {
            name: "max-body-size",
            description: "Max request body size in bytes",
            takes_value: true,
            required: false,
            default_value: None,
            short: None,
        });
        opts.push(OptionSpec {
            name: "read-timeout",
            description: "Read timeout in seconds",
            takes_value: true,
            required: false,
            default_value: None,
            short: None,
        });
        opts.push(OptionSpec {
            name: "rate-limit",
            description: "Global rate limit as requests/seconds (e.g. 100/60)",
            takes_value: true,
            required: false,
            default_value: None,
            short: None,
        });
        opts
    }

    fn requires_config(&self) -> bool {
        true
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        super::require_web_section()?;
        let host = args.get_option("host").unwrap_or("127.0.0.1");
        let port_str = args.get_option("port").unwrap_or("3000");
        let port: u16 = port_str.parse().map_err(|_| {
            CliError::InvalidUsage(String::from("invalid port number"))
        })?;

        let tls_cert = args.get_option("tls-cert");
        let tls_key = args.get_option("tls-key");

        let mut server = Server::new().host(host).port(port);

        // Apply security options
        if let Some(max_body) = args.get_option("max-body-size") {
            let bytes: usize = max_body.parse().map_err(|_| {
                CliError::InvalidUsage(String::from("invalid --max-body-size value"))
            })?;
            server = server.max_body_size(bytes);
        }
        if let Some(timeout_str) = args.get_option("read-timeout") {
            let secs: u64 = timeout_str.parse().map_err(|_| {
                CliError::InvalidUsage(String::from("invalid --read-timeout value"))
            })?;
            server = server.read_timeout(Duration::from_secs(secs));
        }
        if let Some(rate_str) = args.get_option("rate-limit") {
            // Format: requests/seconds
            if let Some(slash) = rate_str.find('/') {
                let reqs: u32 = rate_str[..slash].parse().map_err(|_| {
                    CliError::InvalidUsage(String::from("invalid --rate-limit format, use requests/seconds"))
                })?;
                let secs: u64 = rate_str[slash + 1..].parse().map_err(|_| {
                    CliError::InvalidUsage(String::from("invalid --rate-limit format, use requests/seconds"))
                })?;
                server = server.rate_limit(reqs, Duration::from_secs(secs));
            } else {
                return Err(CliError::InvalidUsage(String::from(
                    "invalid --rate-limit format, use requests/seconds (e.g. 100/60)",
                )));
            }
        }

        match (tls_cert, tls_key) {
            (Some(cert), Some(key)) => {
                server = server.tls(cert, key);
                veprintln!();
                veprintln!("  volki web server (HTTPS)");
                veprintln!("  https://{}:{}", host, port);
                veprintln!();
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(CliError::InvalidUsage(String::from(
                    "--tls-cert and --tls-key must both be provided",
                )));
            }
            (None, None) => {
                veprintln!();
                veprintln!("  volki web server");
                veprintln!("  http://{}:{}", host, port);
                veprintln!();
            }
        }

        server.listen();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_name() {
        assert_eq!(WebStartCommand.name(), "web:start");
    }

    #[test]
    fn test_start_requires_config() {
        assert!(WebStartCommand.requires_config());
    }
}
