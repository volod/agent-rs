//! Black-box tests: run the built command as a process and check the command-line contract
//! through its streams and exit code.

use std::process::Command;

#[test]
fn command_line_contract() {
    // (arguments, exit code, stdout substring, stderr substring)
    let cases: &[(&[&str], i32, &str, &str)] = &[
        (&["version"], 0, "agent-rs ", ""),
        (&["version"], 0, env!("CARGO_PKG_REPOSITORY"), ""),
        (&["help"], 0, "version", ""),
        (&["nope"], 2, "", "'nope'"),
    ];
    for &(args, want_code, want_stdout, want_stderr) in cases {
        let out = Command::new(env!("CARGO_BIN_EXE_agent-rs"))
            .args(args)
            .output()
            .expect("run the command");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let case = args.join(" ");
        assert_eq!(out.status.code(), Some(want_code), "{case}: stderr:\n{stderr}");
        assert!(stdout.contains(want_stdout), "{case}: stdout = {stdout:?}");
        assert!(stderr.contains(want_stderr), "{case}: stderr = {stderr:?}");
    }
}
