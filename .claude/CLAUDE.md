# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                          # Build the project
cargo test                           # Run all tests (unit + integration)
cargo test --lib                     # Run unit tests only
cargo test --test cli_integration    # Run CLI integration tests only
cargo test --test scanner_integration # Run scanner integration tests only
cargo test <test_name>               # Run a single test by name
cargo run -- license --path .        # Run the CLI locally
```

Uses Rust edition 2024. No external dependencies (zero deps in Cargo.toml).

## Architecture

**volki** is a multi-ecosystem CLI tool for scanning project dependencies and reporting license information. It auto-detects the project ecosystem (Node.js, Python, Ruby, Rust, Go, Java, .NET, PHP, Elixir, Swift, Dart) and reads installed dependency metadata to extract license data.

### Two-layer structure: `core` and `libs`

- **`src/core/`** — Framework layer: CLI parsing, command dispatch, package detection
- **`src/libs/`** — Ecosystem-specific implementations + shared utilities

### CLI framework (hand-rolled, no clap)

The CLI uses a custom `Command` trait + `CommandRegistry` pattern:
- `Command` trait (`core/cli/command.rs`): defines `name()`, `description()`, `options()`, `execute()`
- `CommandRegistry` (`core/cli/registry.rs`): collects commands, handles arg parsing and dispatch
- Commands are registered in `core/cli/mod.rs::build_cli()`
- Arg parsing is two-phase: `RawArgs` (raw extraction) → `ParsedArgs` (resolved against `OptionSpec`)

To add a new CLI command: create a struct implementing `Command` in `core/cli/commands/`, then register it in `build_cli()`.

### License scanning pipeline

The `license` command is the main feature. The flow:
1. `commands/license.rs` — entry point, determines ecosystem (auto-detect or `--ecosystem` flag)
2. Dispatches to `libs/<ecosystem>/license/scanner.rs::scan()` for ecosystem-specific scanning
3. Each scanner reads the ecosystem's manifest/lockfile/dependency directory, extracts per-package license info
4. All scanners return `Vec<PackageLicense>` → passed to `libs/shared/license/scan_util.rs::finalize_scan()` for filtering, sorting, and grouping
5. Output via `libs/shared/license/display.rs` (list, grouped, or summary mode)

### Shared license infrastructure (`libs/shared/license/`)

- `types.rs` — Core types: `ScanConfig`, `ScanResult`, `PackageLicense`, `LicenseCategory`, `RiskLevel`
- `scan_util.rs` — `finalize_scan()` applies filters/sorting/grouping shared by all scanners
- `parsers/` — Reusable parsers: `json.rs` (hand-rolled JSON), `toml_simple.rs`, `key_value.rs`, `xml_extract.rs`
- `heuristic.rs` — Fallback license detection from LICENSE files
- `display.rs` — Terminal output formatting

### Adding a new ecosystem scanner

Each ecosystem follows the same pattern under `libs/<shortname>/license/`:
- `mod.rs` — re-exports `scanner::scan`
- `scanner.rs` — implements `pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError>`

The scanner reads ecosystem-specific files, builds `Vec<PackageLicense>`, calls `finalize_scan()`.

### Package detection (`core/package/detect/`)

- `detector.rs` — checks for manifest files (package.json, Cargo.toml, etc.) to identify ecosystems
- `types.rs` — `Ecosystem`, `PackageManager`, `DetectedProject` enums

### Entry point

`main.rs` calls `volki::core::cli::build_cli().run()`. The library root is `lib.rs` (re-exports `core` and `libs`).
