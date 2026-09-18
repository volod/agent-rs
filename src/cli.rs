//! Command-line contract: argument parsing, usage, the standard streams and exit codes.
//!
//! This is the only module that writes to stdout or stderr or decides an exit code. Commands
//! return values and errors; [`run`] renders them.

use std::ffi::OsString;
use std::io::{self, Write};

use clap::{Parser, Subcommand};

use crate::build_info::BuildInfo;

/// Exit code for success.
pub const EXIT_OK: u8 = 0;
/// Exit code for a command that failed.
pub const EXIT_FAILURE: u8 = 1;
/// Exit code for invalid arguments.
pub const EXIT_USAGE: u8 = 2;

/// Command-line arguments; clap derives parsing, help and usage errors from this type.
#[derive(Debug, Parser)]
#[command(about, arg_required_else_help = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

/// Subcommands in help order.
#[derive(Debug, Subcommand)]
enum Command {
    /// Print the build identity
    Version,
}

/// Runs one command line and returns the process exit code. `args` starts with the program name,
/// as [`std::env::args_os`] yields it.
pub fn run<I, T>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(err) => {
            // Requested help goes to stdout and succeeds; usage errors go to stderr. A failed
            // write has nowhere left to be reported; the exit code still is.
            let rendered = err.render();
            let _ = if err.use_stderr() {
                write!(stderr, "{rendered}")
            } else {
                write!(stdout, "{rendered}")
            };
            return u8::try_from(err.exit_code()).unwrap_or(EXIT_USAGE);
        }
    };
    match execute(&args.command, stdout) {
        Ok(()) => EXIT_OK,
        Err(err) => {
            let _ = writeln!(stderr, "{}: {err}", env!("CARGO_PKG_NAME"));
            EXIT_FAILURE
        }
    }
}

fn execute(command: &Command, stdout: &mut dyn Write) -> io::Result<()> {
    match command {
        Command::Version => writeln!(stdout, "{}", BuildInfo::current()),
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    /// Asserts that `got` contains `want`, or is empty when `want` is.
    fn assert_stream(case: &str, stream: &str, got: &str, want: &str) {
        if want.is_empty() {
            assert!(got.is_empty(), "{case}: {stream} = {got:?}, want empty");
        } else {
            assert!(got.contains(want), "{case}: {stream} = {got:?}, want {want:?}");
        }
    }

    #[test]
    fn arguments_are_consistent() {
        Args::command().debug_assert();
    }

    #[test]
    fn run_follows_the_contract() {
        // (case, arguments, exit code, stdout substring, stderr substring)
        let cases: &[(&str, &[&str], u8, &str, &str)] = &[
            ("version", &["version"], EXIT_OK, "agent-rs ", ""),
            ("help command", &["help"], EXIT_OK, "version", ""),
            ("help flag", &["--help"], EXIT_OK, "Usage:", ""),
            ("no command", &[], EXIT_USAGE, "", "Usage:"),
            ("unknown command", &["nope"], EXIT_USAGE, "", "'nope'"),
            ("unknown flag", &["--nope"], EXIT_USAGE, "", "'--nope'"),
            ("extra argument", &["version", "x"], EXIT_USAGE, "", "'x'"),
        ];
        for &(case, args, want_code, want_stdout, want_stderr) in cases {
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            let code = run(
                std::iter::once("agent-rs").chain(args.iter().copied()),
                &mut stdout,
                &mut stderr,
            );
            let stdout = String::from_utf8_lossy(&stdout);
            let stderr = String::from_utf8_lossy(&stderr);
            assert_eq!(code, want_code, "{case}: exit code; stderr:\n{stderr}");
            assert_stream(case, "stdout", &stdout, want_stdout);
            assert_stream(case, "stderr", &stderr, want_stderr);
        }
    }

    #[test]
    fn failed_output_is_a_failure() {
        struct Closed;
        impl Write for Closed {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut stderr = Vec::new();
        assert_eq!(run(["agent-rs", "version"], &mut Closed, &mut stderr), EXIT_FAILURE);
        assert!(String::from_utf8_lossy(&stderr).starts_with("agent-rs: "));
    }
}
