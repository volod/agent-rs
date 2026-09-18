# Project Foundation

## Layout and command

A Cargo workspace on edition 2024 whose root package `agent-rs` has one runtime dependency,
`clap`, plus the dependency-free `xtask` member. The [architecture](../../design/architecture.md)
lists the tree and the dependency direction.

- `src/main.rs` passes `args_os` and locked stdout and stderr to `cli::run` and returns its code
  as `ExitCode`.
- `src/cli.rs` derives the argument parser with clap (`version`, `help`, `-h`, `--help`), renders
  requested help on stdout and usage errors on stderr, and maps results to the exit codes of the
  [command-line contract](../../design/spec.md#command-line-contract).
- `src/build_info.rs` reads the name, version and repository from the manifest at compile time
  and the platform from `std::env::consts`.

## Gates

`cargo xtask ci` runs `cargo fmt --all --check`, clippy on every target with `-D warnings`,
`cargo test --workspace`, `cargo doc` with `RUSTDOCFLAGS="-D warnings"`, `lint-spec-plan` and
`lint-doc-links`; every cargo step and the alias itself run `--locked`. `[workspace.lints]`
forbids `unsafe_code` and enables `clippy::pedantic`. `rust-toolchain.toml` pins the toolchain,
and `rustfmt.toml` sets `use_small_heuristics = "Max"`. `.github/workflows/ci.yml` runs
`cargo xtask ci` and `cargo xtask dist` on `ubuntu-latest` for pushes to `main` and for pull
requests. The [development guide](../../guide/development.md) lists every command.

## Planning tooling

`xtask/src/plan.rs` and its `parse`, `lint` and `status` submodules sit behind
`cargo xtask lint-spec-plan` and `cargo xtask plan-status`; `xtask/src/links.rs` sits behind
`cargo xtask lint-doc-links`, and `xtask/src/markdown.rs` holds the Markdown reading they share.
The checks cover registry statuses, current-page links for shipped rows, plan groups in registry
order, the fields and statuses of each lane, dependencies that resolve to open tasks or existing
records without cycles, and records that are named, indexed and unique. Relative Markdown links
and heading anchors outside fenced code must resolve. The status report names the next eligible
task.

## Agent rules

`AGENTS.md` is the only rule source. `CLAUDE.md` and `GEMINI.md` import it with `@`, and
`.cursor/rules/project-rules.mdc` links it, so rules change in one place.

## Tests

- `src/cli.rs`: commands, help, usage errors, stream separation, exit codes, a failed write, and
  clap's `debug_assert` of the argument definitions.
- `src/build_info.rs`: manifest identity and the one-line format.
- `tests/cli.rs`: runs the built binary and checks its output and exit codes.
- `xtask/src/plan/tests.rs`: each lint defect on temporary repository trees, multi-line fields,
  record-name parsing and the status report.
- `xtask/src/links.rs`, `xtask/src/markdown.rs`: broken links and anchors, skipped directories,
  path normalization, percent-decoding, GitHub slugs and fence handling.
