//! Process wiring only: the command line, the standard streams and the exit code. Behavior lives
//! in the library crate.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let code =
        agent_rs::cli::run(std::env::args_os(), &mut io::stdout().lock(), &mut io::stderr().lock());
    ExitCode::from(code)
}
