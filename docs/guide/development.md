# Development Guide

## Requirements

- [rustup](https://rustup.rs). The first `cargo` command in the repository installs the
  toolchain pinned in `rust-toolchain.toml`, with `clippy` and `rustfmt`;
  `rustup toolchain install` does it explicitly.
- A host linker: `cc` (GCC or Clang) on Linux, the Xcode command-line tools on macOS, the MSVC
  build tools on Windows.
- `tar` for `cargo xtask dist`; Linux, macOS and Windows 10+ ship it.
- Network access the first time, to download the toolchain and crates.

## First run

```bash
cargo xtask ci           # every required gate
cargo run -- version     # agent-rs 0.1.0 (https://github.com/volod/agent-rs, linux/x86_64)
cargo xtask plan-status  # the next eligible task
```

## Commands

`cargo xtask` is an alias in `.cargo/config.toml` for `cargo run --locked --package xtask --`.
The `xtask` crate has no dependencies and compiles on first use. Every command is locked: after
editing `Cargo.toml` by hand, run `cargo check` to update `Cargo.lock`.

| Command | Runs | Purpose |
| --- | --- | --- |
| `cargo build` | | Debug build of `target/debug/agent-rs` |
| `cargo run -- <args>` | | Run the command from source |
| `cargo fmt --all` | | Format |
| `cargo test --workspace` | | Unit, black-box and `xtask` tests |
| `cargo xtask ci` | `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`; `lint-spec-plan`; `lint-doc-links` | Required before accepting a task; CI runs it |
| `cargo xtask plan-status` | | Open task counts and the next eligible task |
| `cargo xtask lint-spec-plan` | | Registry, plan and records agree |
| `cargo xtask lint-doc-links` | | Relative Markdown links and anchors resolve |
| `cargo xtask dist [--target T] [--tag vX.Y.Z]` | `cargo build --release --locked --package agent-rs --target T`, then `tar` | Archive in `dist/`; `--tag` requires that version |
| `cargo clean` | | Remove `target/`; delete `dist/` by hand |

## Toolchain and dependencies

- To move to a newer toolchain, change `channel` in `rust-toolchain.toml` and `rust-version` in
  `Cargo.toml` together, then fix what the new clippy reports.
- Add a crate with `cargo add <crate> [--features ...]`, enabling only the features in use, and
  list it in the [dependency table](../design/spec.md#dependencies). Add test-only crates with
  `--dev`. `cargo update` moves locked versions within their SemVer ranges; commit `Cargo.toml`
  and `Cargo.lock` together.
- Lints are set in `[workspace.lints]` in `Cargo.toml`. A crate opts in with
  `[lints] workspace = true`.

## Test layout

| Path | Purpose |
| --- | --- |
| `src/**/*.rs`, `#[cfg(test)] mod tests` | Unit and white-box tests beside the code; they may use private items |
| `tests/*.rs` | Black-box tests. Each file is its own crate that runs the built binary through `env!("CARGO_BIN_EXE_agent-rs")` or uses the public library API |
| `tests/support/mod.rs` | Helpers shared by several test files; `mod.rs` is required here because Cargo compiles every top-level `tests/*.rs` as a test crate. Add it with its first helper |
| `tests/fixtures/` | Committed inputs and golden outputs. Add it with its first file |
| `xtask/src/**/*.rs` | Tooling tests; `fixture::TempRepo` builds throwaway repository trees |

Rules:

- Tests are deterministic and network-free. Build fixtures in temporary directories that are
  removed afterwards; never commit large or binary fixtures when a test can generate them.
- Black-box tests run in `cargo test` and therefore in `cargo xtask ci`. Mark a slow or
  environment-dependent test `#[ignore = "reason"]` and run it as a declared run with
  `cargo test -- --ignored`.
- Reusable test helpers live under `tests/` or in `#[cfg(test)]` modules, never in product code.

## Versioning and releases

Versions follow [Semantic Versioning](https://semver.org) (`0.y.z` before 1.0). The single
source is `version` under `[workspace.package]` in `Cargo.toml`; `agent-rs version` and the
archive names read it at compile time.

To release, bump the version, run `cargo check` to record it in `Cargo.lock` and `cargo xtask ci`,
merge, then tag that commit and push the tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow rebuilds each target from a clean checkout of the tag, fails when the tag
and the version differ, and publishes the archives and `SHA256SUMS` as a GitHub release.
`cargo xtask dist` builds the same archive for one target locally.

## CI

`.github/workflows/ci.yml` runs `cargo xtask ci` and `cargo xtask dist` on `ubuntu-latest` for
pushes to `main` and for pull requests, with the pinned toolchain and a dependency cache.
`.github/workflows/release.yml` runs on `v*.*.*` tags: `cargo xtask ci`, then
`cargo xtask dist --target <T> --tag <tag>` for each release target on a native runner, then
checksums and `gh release create`. Keep workflows thin wrappers over `cargo xtask` so local and
CI results agree.
