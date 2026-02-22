use crate::core::volkiwithstds::path::Path;

use crate::vvec;

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::package::detect::detector::detect;
use crate::core::package::detect::types::Ecosystem;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::libs::lang::shared::license::display;
use crate::libs::lang::shared::license::types::{RiskLevel, ScanConfig};

pub struct LicenseCommand;

impl Command for LicenseCommand {
    fn name(&self) -> &str {
        "license"
    }

    fn description(&self) -> &str {
        "Scan project dependencies and display license information"
    }

    fn long_description(&self) -> &str {
        "Scan a project's dependencies, extract license info from each package, \
         and display results with colors and filtering options.\n\n\
         Supports 11 ecosystems: Node.js, Python, Ruby, Rust, Go, Java, .NET, \
         PHP, Elixir, Swift, and Dart. Auto-detects the ecosystem from project \
         files, or use --ecosystem to specify manually."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vvec![
            OptionSpec {
                name: "path",
                description: "Directory containing the project",
                takes_value: true,
                required: false,
                default_value: Some("."),
                short: Some('p'),
            },
            OptionSpec {
                name: "ecosystem",
                description: "Force ecosystem (node, python, ruby, rust, go, java, dotnet, php, elixir, swift, dart)",
                takes_value: true,
                required: false,
                default_value: None,
                short: None,
            },
            OptionSpec {
                name: "filter",
                description: "Show only packages matching this license",
                takes_value: true,
                required: false,
                default_value: None,
                short: Some('f'),
            },
            OptionSpec {
                name: "exclude",
                description: "Hide packages matching this license",
                takes_value: true,
                required: false,
                default_value: None,
                short: Some('e'),
            },
            OptionSpec {
                name: "risk",
                description: "Risk filter: low/medium/high",
                takes_value: true,
                required: false,
                default_value: Some("high"),
                short: Some('r'),
            },
            OptionSpec {
                name: "group",
                description: "Group output by license type",
                takes_value: false,
                required: false,
                default_value: None,
                short: Some('g'),
            },
            OptionSpec {
                name: "dev",
                description: "Include dev dependencies",
                takes_value: false,
                required: false,
                default_value: None,
                short: Some('d'),
            },
            OptionSpec {
                name: "summary",
                description: "Show summary counts only",
                takes_value: false,
                required: false,
                default_value: None,
                short: Some('s'),
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let path = args.get_option("path").unwrap_or(".");
        let filter = args.get_option("filter").map(|s| String::from(s));
        let exclude = args.get_option("exclude").map(|s| String::from(s));
        let risk_str = args.get_option("risk").unwrap_or("high");
        let group = args.get_flag("group");
        let dev = args.get_flag("dev");
        let summary = args.get_flag("summary");

        let risk_level = RiskLevel::from_str(risk_str).ok_or_else(|| {
            CliError::InvalidUsage(crate::vformat!(
                "Invalid risk level '{}'. Use: low, medium, high",
                risk_str
            ))
        })?;

        let config = ScanConfig {
            path: String::from(path),
            include_dev: dev,
            filter,
            exclude,
            risk_level,
        };

        let ecosystem = match args.get_option("ecosystem") {
            Some(s) => parse_ecosystem_flag(s)?,
            None => auto_detect(path)?,
        };

        let result = match ecosystem {
            Ecosystem::Node => crate::libs::lang::js::license::scan(&config),
            Ecosystem::Python => crate::libs::lang::py::license::scan(&config),
            Ecosystem::Rust => crate::libs::lang::rs::license::scan(&config),
            Ecosystem::Ruby => crate::libs::lang::rb::license::scan(&config),
            Ecosystem::Go => crate::libs::lang::go::license::scan(&config),
            Ecosystem::Java => crate::libs::lang::java::license::scan(&config),
            Ecosystem::DotNet => crate::libs::lang::dotnet::license::scan(&config),
            Ecosystem::Php => crate::libs::lang::php::license::scan(&config),
            Ecosystem::Elixir => crate::libs::lang::ex::license::scan(&config),
            Ecosystem::Swift => crate::libs::lang::swift::license::scan(&config),
            Ecosystem::Dart => crate::libs::lang::dart::license::scan(&config),
        }
        .map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

        let mut out = crate::core::volkiwithstds::io::stdout();

        if summary {
            display::print_summary(&mut out, &result);
        } else if group {
            display::print_grouped(&mut out, &result);
        } else {
            display::print_list(&mut out, &result);
        }

        Ok(())
    }
}

fn auto_detect(path: &str) -> Result<Ecosystem, CliError> {
    let dir = Path::new(path);
    let projects = detect(dir).map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

    match projects.len() {
        0 => Err(CliError::InvalidUsage(crate::vformat!(
            "No supported ecosystem detected. Use --ecosystem to specify manually."
        ))),
        1 => Ok(projects[0].ecosystem.clone()),
        _ => {
            let names: Vec<String> = projects
                .iter()
                .map(|p| crate::vformat!("{}", p.ecosystem))
                .collect();
            Err(CliError::InvalidUsage(crate::vformat!(
                "Multiple ecosystems detected: {}. Use --ecosystem to specify which to scan.",
                names.join(", ")
            )))
        }
    }
}

fn parse_ecosystem_flag(s: &str) -> Result<Ecosystem, CliError> {
    match String::from(s).to_lowercase().as_str() {
        "node" | "js" | "javascript" => Ok(Ecosystem::Node),
        "python" | "py" => Ok(Ecosystem::Python),
        "ruby" | "rb" => Ok(Ecosystem::Ruby),
        "rust" | "rs" => Ok(Ecosystem::Rust),
        "go" | "golang" => Ok(Ecosystem::Go),
        "java" => Ok(Ecosystem::Java),
        "dotnet" | ".net" | "csharp" | "cs" => Ok(Ecosystem::DotNet),
        "php" => Ok(Ecosystem::Php),
        "elixir" | "ex" => Ok(Ecosystem::Elixir),
        "swift" => Ok(Ecosystem::Swift),
        "dart" | "flutter" => Ok(Ecosystem::Dart),
        _ => Err(CliError::InvalidUsage(crate::vformat!(
            "Unknown ecosystem '{}'. Supported: node, python, ruby, rust, go, java, dotnet, php, elixir, swift, dart",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_node_aliases() {
        assert_eq!(parse_ecosystem_flag("node").unwrap(), Ecosystem::Node);
        assert_eq!(parse_ecosystem_flag("js").unwrap(), Ecosystem::Node);
        assert_eq!(parse_ecosystem_flag("javascript").unwrap(), Ecosystem::Node);
    }

    #[test]
    fn parse_python_aliases() {
        assert_eq!(parse_ecosystem_flag("python").unwrap(), Ecosystem::Python);
        assert_eq!(parse_ecosystem_flag("py").unwrap(), Ecosystem::Python);
    }

    #[test]
    fn parse_ruby_aliases() {
        assert_eq!(parse_ecosystem_flag("ruby").unwrap(), Ecosystem::Ruby);
        assert_eq!(parse_ecosystem_flag("rb").unwrap(), Ecosystem::Ruby);
    }

    #[test]
    fn parse_rust_aliases() {
        assert_eq!(parse_ecosystem_flag("rust").unwrap(), Ecosystem::Rust);
        assert_eq!(parse_ecosystem_flag("rs").unwrap(), Ecosystem::Rust);
    }

    #[test]
    fn parse_go_aliases() {
        assert_eq!(parse_ecosystem_flag("go").unwrap(), Ecosystem::Go);
        assert_eq!(parse_ecosystem_flag("golang").unwrap(), Ecosystem::Go);
    }

    #[test]
    fn parse_java() {
        assert_eq!(parse_ecosystem_flag("java").unwrap(), Ecosystem::Java);
    }

    #[test]
    fn parse_dotnet_aliases() {
        assert_eq!(parse_ecosystem_flag("dotnet").unwrap(), Ecosystem::DotNet);
        assert_eq!(parse_ecosystem_flag(".net").unwrap(), Ecosystem::DotNet);
        assert_eq!(parse_ecosystem_flag("csharp").unwrap(), Ecosystem::DotNet);
        assert_eq!(parse_ecosystem_flag("cs").unwrap(), Ecosystem::DotNet);
    }

    #[test]
    fn parse_php() {
        assert_eq!(parse_ecosystem_flag("php").unwrap(), Ecosystem::Php);
    }

    #[test]
    fn parse_elixir_aliases() {
        assert_eq!(parse_ecosystem_flag("elixir").unwrap(), Ecosystem::Elixir);
        assert_eq!(parse_ecosystem_flag("ex").unwrap(), Ecosystem::Elixir);
    }

    #[test]
    fn parse_swift() {
        assert_eq!(parse_ecosystem_flag("swift").unwrap(), Ecosystem::Swift);
    }

    #[test]
    fn parse_dart_aliases() {
        assert_eq!(parse_ecosystem_flag("dart").unwrap(), Ecosystem::Dart);
        assert_eq!(parse_ecosystem_flag("flutter").unwrap(), Ecosystem::Dart);
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(parse_ecosystem_flag("NODE").unwrap(), Ecosystem::Node);
        assert_eq!(parse_ecosystem_flag("Python").unwrap(), Ecosystem::Python);
        assert_eq!(parse_ecosystem_flag("RUST").unwrap(), Ecosystem::Rust);
    }

    #[test]
    fn parse_unknown_returns_err() {
        assert!(parse_ecosystem_flag("cobol").is_err());
        assert!(parse_ecosystem_flag("").is_err());
    }
}
