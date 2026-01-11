#![forbid(unsafe_code)]

//! Game-agnostic command-line shell for save patchers.
//!
//! This crate provides a reusable Clap-based CLI that delegates game-specific behavior to a
//! [`GameCli`] implementation.
//!
//! ## Public API surface
//!
//! The stable integration surface is:
//! - [`GameCli`]: implemented by a game crate
//! - [`PatchOutcome`]: the structured result passed across the boundary
//! - [`run`] / [`run_with_args`]: the generic CLI runner
//!
//! Everything else in this crate is considered internal and may change.
//!
//! ## Versioning
//!
//! This crate follows semantic versioning.
//!
//! While the major version is `0`, minor version bumps (`0.x`) may include breaking changes.
//! Patch releases (`0.x.y`) should remain backwards compatible.
//!
//! # Example
//! ```no_run
//! use anyhow::Result;
//!
//! struct MyGame;
//!
//! impl gb_save_cli::GameCli for MyGame {
//!     fn detect_version(_bytes: &[u8]) -> Result<u16> {
//!         Ok(1)
//!     }
//!
//!     fn patch(bytes: Vec<u8>, _target: u16, _dev_type: u8) -> Result<Vec<u8>> {
//!         Ok(bytes)
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     gb_save_cli::run::<MyGame>()
//! }
//! ```

use std::ffi::OsString;
use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use gb_save_core::{PatchLogEntry, PatchLogLevel};

/// Game-specific glue for the generic CLI.
///
/// A game crate implements this trait and then calls [`run`].
///
/// # Example
/// ```no_run
/// use anyhow::Result;
///
/// struct MyGame;
///
/// impl gb_save_cli::GameCli for MyGame {
///     fn detect_version(_bytes: &[u8]) -> Result<u16> {
///         Ok(1)
///     }
///
///     fn patch(bytes: Vec<u8>, _target: u16, _dev_type: u8) -> Result<Vec<u8>> {
///         Ok(bytes)
///     }
/// }
///
/// fn main() -> Result<()> {
///     gb_save_cli::run::<MyGame>()
/// }
/// ```
pub trait GameCli {
    /// Detects the save version from raw bytes.
    ///
    /// # Example
    /// ```
    /// use gb_save_cli::GameCli;
    /// # use anyhow::Result;
    /// # struct MyGame;
    /// # impl gb_save_cli::GameCli for MyGame {
    /// #     fn detect_version(_bytes: &[u8]) -> Result<u16> { Ok(7) }
    /// #     fn patch(bytes: Vec<u8>, _target: u16, _dev_type: u8) -> Result<Vec<u8>> { Ok(bytes) }
    /// # }
    /// let version = MyGame::detect_version(&[0u8; 4])?;
    /// assert_eq!(version, 7);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    fn detect_version(bytes: &[u8]) -> Result<u16>;

    /// Applies either a migration (`dev_type == 0`) or a fix patch (`dev_type != 0`).
    ///
    /// # Example
    /// ```
    /// use gb_save_cli::GameCli;
    /// # use anyhow::Result;
    /// # struct MyGame;
    /// # impl gb_save_cli::GameCli for MyGame {
    /// #     fn detect_version(_bytes: &[u8]) -> Result<u16> { Ok(1) }
    /// #     fn patch(bytes: Vec<u8>, _target: u16, _dev_type: u8) -> Result<Vec<u8>> { Ok(bytes) }
    /// # }
    /// let out = MyGame::patch(vec![1, 2, 3], 1, 0)?;
    /// assert_eq!(out, vec![1, 2, 3]);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    fn patch(bytes: Vec<u8>, target: u16, dev_type: u8) -> Result<Vec<u8>>;

    /// Applies a patch and returns a structured outcome.
    ///
    /// The default implementation calls [`patch`] and returns an outcome with no logs.
    ///
    /// # Example
    /// ```
    /// use gb_save_cli::GameCli;
    /// # use anyhow::Result;
    /// # struct MyGame;
    /// # impl gb_save_cli::GameCli for MyGame {
    /// #     fn detect_version(_bytes: &[u8]) -> Result<u16> { Ok(1) }
    /// #     fn patch(bytes: Vec<u8>, _target: u16, _dev_type: u8) -> Result<Vec<u8>> { Ok(bytes) }
    /// # }
    /// let outcome = MyGame::patch_with_log(vec![0u8; 8], 1, 0);
    /// assert!(outcome.ok);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    fn patch_with_log(bytes: Vec<u8>, target: u16, dev_type: u8) -> PatchOutcome {
        match Self::patch(bytes, target, dev_type) {
            Ok(bytes) => PatchOutcome {
                ok: true,
                bytes: Some(bytes),
                error: None,
                logs: Vec::new(),
            },
            Err(e) => PatchOutcome {
                ok: false,
                bytes: None,
                error: Some(e.to_string()),
                logs: Vec::new(),
            },
        }
    }
}

/// Result of a patch operation.
///
/// # Example
/// ```
/// use gb_save_cli::PatchOutcome;
///
/// let outcome = PatchOutcome {
///     ok: true,
///     bytes: Some(vec![1, 2, 3]),
///     error: None,
///     logs: Vec::new(),
/// };
/// assert!(outcome.ok);
/// ```
#[derive(Debug, Clone)]
pub struct PatchOutcome {
    /// Whether the patch succeeded.
    pub ok: bool,
    /// Patched save bytes, if patching succeeded.
    pub bytes: Option<Vec<u8>>,
    /// Human-readable error message, if patching failed.
    pub error: Option<String>,
    /// Structured logs emitted during patching.
    pub logs: Vec<PatchLogEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Parser)]
#[command(name = "gb-save-patcher")]
#[command(version)]
#[command(about = "Save patcher CLI", long_about = None)]
struct Cli {
    /// Reduce output; only errors are printed.
    #[arg(long, global = true)]
    quiet: bool,

    /// Increase output verbosity (-v for info, -vv for extra details).
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Output format for logs and errors.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human, global = true)]
    format: OutputFormat,

    /// Colored output policy (human format only).
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true)]
    color: ColorMode,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Prints the detected save version.
    Version { path: PathBuf },

    /// Applies a patch and writes the output.
    Patch {
        #[arg(long = "in")]
        input: PathBuf,

        #[arg(long = "out")]
        output: PathBuf,

        #[arg(long)]
        target: u16,

        #[arg(long, default_value_t = 0)]
        dev_type: u8,
    },
}

fn should_print(level: PatchLogLevel, quiet: bool, verbose: u8) -> bool {
    if quiet {
        return level == PatchLogLevel::Error;
    }

    match (verbose, level) {
        (0, PatchLogLevel::Info) => false,
        (0, PatchLogLevel::Warning | PatchLogLevel::Error) => true,
        _ => true,
    }
}

fn should_color(mode: ColorMode) -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }

    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => std::io::stderr().is_terminal(),
    }
}

fn render_level(level: PatchLogLevel, color: bool) -> &'static str {
    match (level, color) {
        (PatchLogLevel::Info, false) => "info",
        (PatchLogLevel::Warning, false) => "warn",
        (PatchLogLevel::Error, false) => "error",
        (PatchLogLevel::Info, true) => "\x1b[90minfo\x1b[0m",
        (PatchLogLevel::Warning, true) => "\x1b[33mwarn\x1b[0m",
        (PatchLogLevel::Error, true) => "\x1b[31merror\x1b[0m",
    }
}

fn print_logs_human(logs: &[PatchLogEntry], quiet: bool, verbose: u8, color_mode: ColorMode) {
    let color = should_color(color_mode);

    for entry in logs {
        if !should_print(entry.level, quiet, verbose) {
            continue;
        }

        eprintln!(
            "[{level}] {source}: {message}",
            level = render_level(entry.level, color),
            source = entry.source,
            message = entry.message
        );
    }
}

fn print_outcome_json(outcome: &PatchOutcome) {
    let logs = outcome
        .logs
        .iter()
        .map(|entry| {
            let level = match entry.level {
                PatchLogLevel::Info => "info",
                PatchLogLevel::Warning => "warn",
                PatchLogLevel::Error => "error",
            };

            serde_json::json!({
                "level": level,
                "source": entry.source,
                "message": entry.message,
            })
        })
        .collect::<Vec<_>>();

    let mut obj = serde_json::Map::new();
    obj.insert("ok".to_string(), serde_json::Value::Bool(outcome.ok));
    obj.insert("logs".to_string(), serde_json::Value::Array(logs));

    if let Some(bytes) = &outcome.bytes {
        obj.insert(
            "bytes_len".to_string(),
            serde_json::Value::Number(bytes.len().into()),
        );
    }

    if let Some(error) = &outcome.error {
        obj.insert(
            "error".to_string(),
            serde_json::Value::String(error.clone()),
        );
    }

    println!("{}", serde_json::Value::Object(obj));
}

/// Runs the CLI using the game-specific implementation `G`.
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// # struct MyGame;
/// # impl gb_save_cli::GameCli for MyGame {
/// #     fn detect_version(_bytes: &[u8]) -> Result<u16> { Ok(1) }
/// #     fn patch(bytes: Vec<u8>, _target: u16, _dev_type: u8) -> Result<Vec<u8>> { Ok(bytes) }
/// # }
/// # fn main() -> Result<()> {
/// gb_save_cli::run::<MyGame>()
/// # }
/// ```
pub fn run<G: GameCli>() -> Result<()> {
    run_with_args::<G, _, _>(std::env::args_os())
}

/// Runs the CLI using the provided argument iterator.
///
/// This is primarily useful for tests and embedding.
///
/// # Example
/// ```no_run
/// # use anyhow::Result;
/// # struct MyGame;
/// # impl gb_save_cli::GameCli for MyGame {
/// #     fn detect_version(_bytes: &[u8]) -> Result<u16> { Ok(1) }
/// #     fn patch(bytes: Vec<u8>, _target: u16, _dev_type: u8) -> Result<Vec<u8>> { Ok(bytes) }
/// # }
/// # fn main() -> Result<()> {
/// let args = ["gb-save-patcher", "version", "path/to/save.sav"];
/// gb_save_cli::run_with_args::<MyGame, _, _>(args);
/// # Ok(())
/// # }
/// ```
pub fn run_with_args<G: GameCli, I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);

    match cli.command {
        Command::Version { path } => {
            let bytes =
                fs::read(&path).with_context(|| format!("read input: {}", path.display()))?;
            let version = G::detect_version(&bytes)?;

            match cli.format {
                OutputFormat::Human => println!("{version}"),
                OutputFormat::Json => {
                    let mut obj = serde_json::Map::new();
                    obj.insert("ok".to_string(), serde_json::Value::Bool(true));
                    obj.insert(
                        "version".to_string(),
                        serde_json::Value::Number(version.into()),
                    );
                    println!("{}", serde_json::Value::Object(obj));
                }
            }
        }
        Command::Patch {
            input,
            output,
            target,
            dev_type,
        } => {
            let bytes =
                fs::read(&input).with_context(|| format!("read input: {}", input.display()))?;

            let outcome = G::patch_with_log(bytes, target, dev_type);

            match cli.format {
                OutputFormat::Human => {
                    print_logs_human(&outcome.logs, cli.quiet, cli.verbose, cli.color);
                    if let Some(error) = &outcome.error {
                        anyhow::bail!(error.clone());
                    }
                }
                OutputFormat::Json => {
                    print_outcome_json(&outcome);
                    if let Some(error) = &outcome.error {
                        anyhow::bail!(error.clone());
                    }
                }
            }

            let patched = outcome
                .bytes
                .with_context(|| "patch outcome did not include output bytes")?;

            fs::write(&output, patched)
                .with_context(|| format!("write output: {}", output.display()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_only_prints_errors() {
        assert!(!should_print(PatchLogLevel::Info, true, 0));
        assert!(!should_print(PatchLogLevel::Warning, true, 2));
        assert!(should_print(PatchLogLevel::Error, true, 0));
    }

    #[test]
    fn default_prints_warnings_and_errors() {
        assert!(!should_print(PatchLogLevel::Info, false, 0));
        assert!(should_print(PatchLogLevel::Warning, false, 0));
        assert!(should_print(PatchLogLevel::Error, false, 0));
    }

    #[test]
    fn verbose_prints_everything() {
        assert!(should_print(PatchLogLevel::Info, false, 1));
        assert!(should_print(PatchLogLevel::Warning, false, 1));
        assert!(should_print(PatchLogLevel::Error, false, 1));
    }
}
