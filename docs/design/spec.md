# agent-rs Specification

## Purpose

`agent-rs` is a copy-ready template for Rust command-line projects that people and coding agents
develop together. A fresh copy builds, tests and plans on day one: it ships an `agent-rs`
command, one set of agent rules, quality gates shared by local work and CI, and a planning
workflow in which every change traces from a specified capability to tests and current-state
documentation.

This specification is living. Behavior, boundaries and evaluation belong here, in capability
sections below or in capability pages copied from the [page template](template.md). Remaining
work belongs in the [plan](../impl/plan.md); delivered behavior in
[current state](../impl/current.md); modules and their edges in the
[architecture](architecture.md). A need discovered during implementation enters here first,
through [capability changes](../guide/planning-workflow.md#capability-changes).

## Scope

In scope:

- a Cargo workspace with one binary-and-library package and an `xtask` tooling crate, building
  one executable for Linux, macOS and Windows;
- release archives with checksums, published to GitHub Releases from a version tag;
- `cargo xtask` entry points that wrap plain `cargo` commands and are shared by local work and
  CI;
- gates for formatting, clippy, locked dependencies, tests, documentation and planning
  integrity;
- `AGENTS.md` as the only rule source, with adapters for Claude, Gemini and Cursor;
- a capability registry, forward plan, task records and current-state pages.

Out of scope until a specified capability needs them: a business domain, an async runtime,
configuration files, persistence, network services, containers, crates.io publishing, code
signing and deployment.

## Design principles

1. **Standard library first.** Every crate is justified in the [dependency table](#dependencies).
2. **Thin edges.** `main` wires the process, `cli` owns the command-line contract, and library
   modules return typed results to it.
3. **One workflow.** `cargo xtask` wraps plain `cargo` commands; CI runs the same commands.
4. **Deterministic evidence.** Tests are network-free and repeatable; acceptance is proven by
   commands another contributor can rerun.
5. **Traceable work.** Specification, plan, record and current-state page agree, and
   `cargo xtask lint-spec-plan` checks it.

## Cross-cutting rules

### Toolchain and build

- Rust edition 2024. `rust-toolchain.toml` pins the toolchain with `clippy` and `rustfmt`, and
  `rust-version` in `Cargo.toml` equals it; both change together.
- Package `agent-rs`, library `agent_rs`, binary `agent-rs` (`agent-rs.exe`). `Cargo.lock` is
  committed and every gate runs with `--locked`.
- Lints are configured once in `[workspace.lints]`: `unsafe_code` is forbidden and
  `clippy::pedantic` warns; gates deny warnings.
- The version is `workspace.package.version` in `Cargo.toml`, read at compile time; releases are
  Git tags `vMAJOR.MINOR.PATCH` that must equal it. See
  [release distribution](#release-distribution).
- Release builds use `lto`, one codegen unit and stripped symbols. Linux release binaries target
  musl and are statically linked; Windows binaries link the C runtime statically.
- Gates run on Linux. Other platforms are built by the release workflow; platform-specific code
  lives behind `#[cfg(...)]`.

### Dependencies

The product has one runtime dependency; `xtask` has none. `rustfmt` and `clippy` are toolchain
components, not crates.

| Dependency | Kind | Use |
| --- | --- | --- |
| [`clap`](https://docs.rs/clap) with `derive` | runtime | Argument parsing, help and usage errors in `cli`; hand-written parsing would reimplement the help and error behavior every command needs |

Adding a row is a specification change: name the need, the standard-library alternative that was
rejected and why.

### Command-line contract

- Synopsis: `agent-rs <command> [arguments]`. Commands: `version`, `help`; `-h` and `--help`
  print the same help.
- stdout carries command results and requested help only; usage errors and diagnostics go to
  stderr.
- Exit codes: `0` success, `1` failure, `2` usage error. SIGINT and SIGTERM keep the default
  disposition, so the shell reports `128 + signal`; a capability that must clean up on
  interruption specifies its own handling.

## Project foundation

The repository builds, tests and checks itself from a fresh clone with only `rustup`.
`cargo xtask ci` runs every required gate, and GitHub Actions runs the same command plus
`cargo xtask dist`.

Boundary: the foundation owns layout, tooling and gates, not product behavior.

Evaluation: `cargo xtask ci` passes on a fresh clone; the planning tool's tests reproduce each
defect it detects; the black-box test drives the built binary through its exit codes. A negative
result names the failing gate and never weakens it.

## Release distribution

`cargo xtask dist [--target TRIPLE]` builds the release binary for one target (the host by
default) and writes `dist/agent-rs-<version>-<target>.tar.gz`, or `.zip` for Windows targets.
The archive holds one directory of the same name containing the executable, `README.md` and
`LICENSE`. The version in the name comes from the same manifest field as `agent-rs version`, so
an archive never disagrees with its binary. `--tag vX.Y.Z` fails before building when the tag is
not `v` followed by that version.

Pushing a tag `vMAJOR.MINOR.PATCH` publishes a GitHub release: the release workflow runs
`cargo xtask ci`, then builds archives with `--tag` for `x86_64-unknown-linux-musl` and
`aarch64-unknown-linux-musl` (both on a Linux runner; arm64 links with the bundled `rust-lld`),
`aarch64-apple-darwin` and `x86_64-pc-windows-msvc` (each on its own runner). It writes
`SHA256SUMS` in `sha256sum -c` format, verifies it and uploads the archives and checksums with
generated notes.

Boundary: no signing, SBOM, installers, crates.io or package-manager publishing, and no
byte-for-byte reproducible archives.

Evaluation: `xtask` unit tests cover option parsing, the tag check and host-target detection;
CI runs `cargo xtask dist` on every push and pull request. A negative result keeps the previous
release and names the failing target or the mismatched tag.

## Project identity

`agent-rs version` prints one line naming the command, version, repository and platform, for
example `agent-rs 0.1.0 (https://github.com/volod/agent-rs, linux/x86_64)`. After a repository
is created from the template, its package name, binary, repository URL, README and this
specification name and describe the new product.

Boundary: identity only. It does not choose the product's domain, crates or architecture.

Evaluation: unit and black-box tests agree on the identity; no template name remains where it
denotes the active project; `cargo xtask ci` passes. A negative result names the missing owner
input (product name, repository URL, description) instead of inventing it.

## Capability Registry

Every capability appears exactly once. Status is `planned` while it has open plan tasks and
`shipped` once its current-state page exists and no task remains. Row order is the
implementation line the plan follows.

| # | Capability | Status | How it is evaluated | Implementation |
| --- | --- | --- | --- | --- |
| 1 | `project-foundation` | shipped | `cargo xtask ci` passes on a fresh clone; planning-tool and black-box CLI tests | [Current](../impl/current/project-foundation.md) |
| 2 | `release-distribution` | shipped | `xtask` dist tests; `cargo xtask dist` in CI; checksums verified before publishing | [Current](../impl/current/release-distribution.md) |
| 3 | `project-identity` | planned | Identity tests pass and no stale template identity remains after personalization | [Open work](../impl/plan.md#project-identity----project-identity) |

## Development integrity

[AGENTS.md](../../AGENTS.md) defines the task cycle; the
[planning workflow](../guide/planning-workflow.md) defines task shape, lanes, records and
checkpoints.

- Task records under `docs/impl/records/` keep the full accepted task, amendments, decisions and
  evidence. Current pages describe available behavior and link records. The plan holds only
  unresolved work.
- Tests are deterministic, network-free and build fixtures in temporary directories. Unit tests
  sit in their module; black-box tests live in `tests/`; see the
  [test layout](../guide/development.md#test-layout).
- Coverage percentages are diagnostic, never a gate.
- Tests that need a real external system, credentials or human judgment are human-assisted or
  declared runs, never part of `cargo xtask ci`.

## Success criteria

A team creates a repository from the template, completes the first plan task to give it its own
identity, and then adds capabilities by specifying them here, planning tasks, and closing each
task with a record and a current-state page, while `cargo xtask ci` stays green. A contributor
can tell what the product promises, what remains, what exists and how each capability is proven
from the repository alone.
