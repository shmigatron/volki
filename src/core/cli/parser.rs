use crate::core::volkiwithstds::collections::{HashMap, String, Vec};

use super::command::OptionSpec;
use super::error::CliError;

/// Phase 1: raw extraction from env args. Always succeeds.
pub struct RawArgs {
    pub subcommand: Option<String>,
    pub tokens: Vec<String>,
}

impl RawArgs {
    pub fn from_env() -> Self {
        let args: Vec<String> = crate::core::volkiwithstds::env::args()
            .into_iter()
            .skip(1)
            .map(|s| String::from(s.as_str()))
            .collect();
        Self::from_vec(args)
    }

    pub fn from_vec(args: Vec<String>) -> Self {
        let mut subcommand = None;
        let mut tokens = Vec::new();

        for arg in &args {
            if subcommand.is_none() && !arg.starts_with("-") {
                subcommand = Some(arg.clone());
            } else {
                tokens.push(arg.clone());
            }
        }

        RawArgs { subcommand, tokens }
    }
}

/// Phase 2: resolved against the command's option specs.
#[allow(dead_code)]
pub struct ParsedArgs {
    options: HashMap<String, String>,
    flags: HashMap<String, bool>,
    positional: Vec<String>,
}

impl ParsedArgs {
    pub fn resolve(raw: &RawArgs, specs: &[OptionSpec]) -> Result<Self, CliError> {
        let mut options: HashMap<String, String> = HashMap::new();
        let mut flags: HashMap<String, bool> = HashMap::new();
        let mut positional: Vec<String> = Vec::new();

        let value_opts: HashMap<&str, &OptionSpec> = specs
            .iter()
            .filter(|s| s.takes_value)
            .map(|s| (s.name, s))
            .collect();
        let flag_opts: HashMap<&str, &OptionSpec> = specs
            .iter()
            .filter(|s| !s.takes_value)
            .map(|s| (s.name, s))
            .collect();

        let tokens = &raw.tokens;
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            if let Some(name) = token.strip_prefix("--") {
                if name.is_empty() {
                    // bare `--`, treat rest as positional
                    i += 1;
                    while i < tokens.len() {
                        positional.push(tokens[i].clone());
                        i += 1;
                    }
                    break;
                }

                if value_opts.contains_key(&name) {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(CliError::MissingValue(String::from(name)));
                    }
                    options.insert(String::from(name), tokens[i].clone());
                } else if flag_opts.contains_key(&name) {
                    flags.insert(String::from(name), true);
                } else {
                    return Err(CliError::UnknownFlag(String::from(name)));
                }
            } else {
                positional.push(token.clone());
            }

            i += 1;
        }

        for spec in specs {
            if spec.takes_value && !options.contains_key(&String::from(spec.name)) {
                if let Some(default) = spec.default_value {
                    options.insert(String::from(spec.name), String::from(default));
                }
            }
        }

        Ok(ParsedArgs {
            options,
            flags,
            positional,
        })
    }

    pub fn get_option(&self, name: &str) -> Option<&str> {
        self.options.get(&String::from(name)).map(|s| s.as_str())
    }

    pub fn get_flag(&self, name: &str) -> bool {
        self.flags
            .get(&String::from(name))
            .copied()
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn positional(&self) -> &[String] {
        &self.positional
    }

    /// Check if --help was in the raw tokens (before resolution).
    pub fn has_help_flag(tokens: &[String]) -> bool {
        tokens.iter().any(|t| t == "--help" || t == "-h")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vvec;

    fn s(v: &str) -> String {
        String::from(v)
    }
    fn sv(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| s(x)).collect()
    }

    fn spec_value(name: &'static str) -> OptionSpec {
        OptionSpec {
            name,
            description: "",
            takes_value: true,
            required: false,
            default_value: None,
            short: None,
        }
    }

    fn spec_flag(name: &'static str) -> OptionSpec {
        OptionSpec {
            name,
            description: "",
            takes_value: false,
            required: false,
            default_value: None,
            short: None,
        }
    }

    fn spec_value_default(name: &'static str, default: &'static str) -> OptionSpec {
        OptionSpec {
            name,
            description: "",
            takes_value: true,
            required: false,
            default_value: Some(default),
            short: None,
        }
    }

    // --- RawArgs::from_vec ---

    #[test]
    fn raw_args_empty() {
        let raw = RawArgs::from_vec(vvec![]);
        assert!(raw.subcommand.is_none());
        assert!(raw.tokens.is_empty());
    }

    #[test]
    fn raw_args_subcommand_only() {
        let raw = RawArgs::from_vec(sv(&["license"]));
        assert_eq!(raw.subcommand, Some(s("license")));
        assert!(raw.tokens.is_empty());
    }

    #[test]
    fn raw_args_subcommand_with_flags() {
        let raw = RawArgs::from_vec(sv(&["license", "--path", "/tmp"]));
        assert_eq!(raw.subcommand, Some(s("license")));
        assert_eq!(raw.tokens, sv(&["--path", "/tmp"]));
    }

    #[test]
    fn raw_args_flag_before_subcommand() {
        let raw = RawArgs::from_vec(sv(&["--help", "license"]));
        assert_eq!(raw.subcommand, Some(s("license")));
        assert_eq!(raw.tokens, sv(&["--help"]));
    }

    // --- ParsedArgs::resolve ---

    #[test]
    fn resolve_value_option() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["--path", "/tmp"]),
        };
        let specs = [spec_value("path")];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert_eq!(parsed.get_option("path"), Some("/tmp"));
    }

    #[test]
    fn resolve_flag() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["--group"]),
        };
        let specs = [spec_flag("group")];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert!(parsed.get_flag("group"));
    }

    #[test]
    fn resolve_absent_flag() {
        let raw = RawArgs {
            subcommand: None,
            tokens: vvec![],
        };
        let specs = [spec_flag("group")];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert!(!parsed.get_flag("group"));
    }

    #[test]
    fn resolve_default_value() {
        let raw = RawArgs {
            subcommand: None,
            tokens: vvec![],
        };
        let specs = [spec_value_default("path", ".")];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert_eq!(parsed.get_option("path"), Some("."));
    }

    #[test]
    fn resolve_override_default() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["--path", "/home"]),
        };
        let specs = [spec_value_default("path", ".")];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert_eq!(parsed.get_option("path"), Some("/home"));
    }

    #[test]
    fn resolve_unknown_flag_error() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["--unknown"]),
        };
        let specs: [OptionSpec; 0] = [];
        let result = ParsedArgs::resolve(&raw, &specs);
        assert!(matches!(result, Err(CliError::UnknownFlag(_))));
    }

    #[test]
    fn resolve_missing_value_error() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["--path"]),
        };
        let specs = [spec_value("path")];
        let result = ParsedArgs::resolve(&raw, &specs);
        assert!(matches!(result, Err(CliError::MissingValue(_))));
    }

    #[test]
    fn resolve_bare_double_dash_positional() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["--", "foo", "bar"]),
        };
        let specs: [OptionSpec; 0] = [];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert_eq!(parsed.positional(), sv(&["foo", "bar"]).as_slice());
    }

    #[test]
    fn resolve_positional_args() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["arg1", "arg2"]),
        };
        let specs: [OptionSpec; 0] = [];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert_eq!(parsed.positional(), sv(&["arg1", "arg2"]).as_slice());
    }

    #[test]
    fn resolve_mixed_flags_and_positional() {
        let raw = RawArgs {
            subcommand: None,
            tokens: sv(&["--group", "pos1", "--path", "/tmp"]),
        };
        let specs = [spec_flag("group"), spec_value("path")];
        let parsed = ParsedArgs::resolve(&raw, &specs).unwrap();
        assert!(parsed.get_flag("group"));
        assert_eq!(parsed.get_option("path"), Some("/tmp"));
        assert_eq!(parsed.positional(), sv(&["pos1"]).as_slice());
    }

    // --- has_help_flag ---

    #[test]
    fn has_help_long() {
        assert!(ParsedArgs::has_help_flag(&sv(&["--help"])));
    }

    #[test]
    fn has_help_short() {
        assert!(ParsedArgs::has_help_flag(&sv(&["-h"])));
    }

    #[test]
    fn has_help_not_present() {
        assert!(!ParsedArgs::has_help_flag(&sv(&["--version"])));
    }

    #[test]
    fn has_help_among_other_args() {
        assert!(ParsedArgs::has_help_flag(&sv(&[
            "--path", "/tmp", "--help"
        ])));
    }
}
