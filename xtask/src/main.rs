//! Repository tooling, run as `cargo xtask <command>` from anywhere in the workspace. It is never
//! shipped; see `cargo xtask help` and `docs/guide/development.md`.

mod dist;
#[cfg(test)]
mod fixture;
mod links;
mod markdown;
mod plan;

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <command>

Commands:
  ci              Required gate: fmt, clippy, tests, docs, lint-spec-plan, lint-doc-links
  plan-status     Count open tasks and show the next eligible one
  lint-spec-plan  Check that the capability registry, plan and task records agree
  lint-doc-links  Check relative Markdown links and anchors
  dist            Build and archive a release binary into dist/
                  [--target TRIPLE] (default: host) [--tag vX.Y.Z] (must equal the version)
  help            Show this help
";

/// Why a command did not succeed.
#[derive(Debug)]
pub enum Error {
    /// Invalid arguments: exit code 2.
    Usage(String),
    /// A step or check failed: exit code 1.
    Failed(String),
}

impl Error {
    /// Maps an I/O error on `path` to a failure that names the path.
    fn io(path: impl AsRef<Path>) -> impl FnOnce(io::Error) -> Self {
        let path = path.as_ref().display().to_string();
        move |err| Self::Failed(format!("{path}: {err}"))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Failed(message) => f.write_str(message),
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&repository_root(), &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err @ Error::Usage(_)) => {
            eprintln!("xtask: {err}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Err(err @ Error::Failed(_)) => {
            eprintln!("xtask: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(root: &Path, args: &[String]) -> Result<(), Error> {
    let Some((command, rest)) = args.split_first() else {
        return Err(Error::Usage("missing command".to_owned()));
    };
    if command != "dist" && !rest.is_empty() {
        return Err(Error::Usage(format!("{command} takes no arguments")));
    }
    match command.as_str() {
        "ci" => ci(root),
        "plan-status" => {
            let (inputs, _) = plan::load(root).map_err(|err| Error::Failed(err.to_string()))?;
            print!("{}", plan::Status::new(&inputs.plan));
            Ok(())
        }
        "lint-spec-plan" => lint_spec_plan(root),
        "lint-doc-links" => lint_doc_links(root),
        "dist" => {
            let archive = dist::dist(root, &dist::Options::parse(rest)?)?;
            println!("{}", archive.display());
            Ok(())
        }
        "help" | "-h" | "--help" => {
            print!("{USAGE}");
            Ok(())
        }
        other => Err(Error::Usage(format!("unknown command {other:?}"))),
    }
}

/// The required gate. CI runs exactly this.
fn ci(root: &Path) -> Result<(), Error> {
    cargo(root, &["fmt", "--all", "--check"], &[])?;
    let clippy = ["clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"];
    cargo(root, &clippy, &[])?;
    cargo(root, &["test", "--workspace", "--locked"], &[])?;
    cargo(
        root,
        &["doc", "--workspace", "--no-deps", "--locked"],
        &[("RUSTDOCFLAGS", "-D warnings")],
    )?;
    lint_spec_plan(root)?;
    lint_doc_links(root)
}

fn lint_spec_plan(root: &Path) -> Result<(), Error> {
    let (inputs, mut problems) = plan::load(root).map_err(|err| Error::Failed(err.to_string()))?;
    problems.extend(plan::lint(&inputs));
    report("lint-spec-plan", &problems)
}

fn lint_doc_links(root: &Path) -> Result<(), Error> {
    let problems = links::check(root).map_err(|err| Error::Failed(err.to_string()))?;
    report("lint-doc-links", &problems)
}

fn report(check: &str, problems: &[String]) -> Result<(), Error> {
    for problem in problems {
        eprintln!("{problem}");
    }
    if problems.is_empty() {
        println!("{check}: ok");
        Ok(())
    } else {
        Err(Error::Failed(format!("{check}: {} problem(s)", problems.len())))
    }
}

/// Runs the cargo that runs xtask, in the repository root, echoing the command line.
fn cargo(root: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<(), Error> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    run_command(Command::new(cargo).args(args).envs(env.iter().copied()).current_dir(root))
}

/// Runs a command with inherited streams and fails unless it exits successfully.
fn run_command(command: &mut Command) -> Result<(), Error> {
    let program =
        Path::new(command.get_program()).file_stem().unwrap_or_default().to_string_lossy();
    let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy()).collect();
    let line = format!("{program} {}", args.join(" "));
    eprintln!("$ {line}");
    let status = command.status().map_err(|err| Error::Failed(format!("{line}: {err}")))?;
    if status.success() { Ok(()) } else { Err(Error::Failed(format!("{line}: {status}"))) }
}

/// The workspace root: xtask's manifest sits one directory below it. `cargo run` and `cargo test`
/// set `CARGO_MANIFEST_DIR` at run time; the compile-time value can name an old checkout, because
/// Cargo reuses a moved or shared `target/` without rebuilding.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from);
    manifest_dir.parent().expect("xtask lives inside the workspace").to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_rejects_bad_usage() {
        let root = repository_root();
        for args in [&[][..], &["bogus"], &["ci", "extra"]] {
            let args: Vec<String> = args.iter().map(|&arg| arg.to_owned()).collect();
            assert!(matches!(run(&root, &args), Err(Error::Usage(_))), "{args:?}");
        }
    }
}
