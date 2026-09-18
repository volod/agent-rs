# Project rules

Canonical rules for every agent and contributor. `CLAUDE.md`, `GEMINI.md` and `.cursor/rules/`
only point here; never put rules in them. Read the selected task, the code it touches and the
design sections it links. Load other guidance only when a condition under
[Read when needed](#read-when-needed) applies.

## Project

`agent-rs` is a template for Rust command-line projects built by people and coding agents. The
[specification](docs/design/spec.md) says what the product must do and how each capability is
evaluated, the [architecture](docs/design/architecture.md) owns modules and dependency direction,
the [plan](docs/impl/plan.md) holds only remaining work, [task records](docs/impl/records/README.md)
keep evidence and decisions, and [current state](docs/impl/current.md) describes what exists.

## Guardrails

- Preserve unrelated work. Diagnostic or review requests do not authorize code changes.
- Do not commit, push, rewrite history or revert user changes unless explicitly asked.
- Rust edition 2024 on the toolchain pinned in `rust-toolchain.toml`. Run gates through
  `cargo xtask`; each step is a plain `cargo` command listed in the
  [development guide](docs/guide/development.md#commands).
- Prefer the standard library. Add a crate only after adding it to the
  [dependency table](docs/design/spec.md#dependencies); commit `Cargo.toml` and `Cargo.lock`
  together.
- `unsafe` is forbidden workspace-wide; allowing it is a specification change.
- Never hardcode machine-specific paths. Never put secrets in code, logs, fixtures or docs.
- Use ASCII in code, logs and docs.

## Rust conventions

- `src/main.rs` only wires the process and calls `cli::run`. Product code lives in library
  modules under `src/`, black-box tests in `tests/`, repository tooling in `xtask/`. Move a module
  into a workspace crate under `crates/` only when it needs its own dependencies, build unit or
  consumers.
- Name modules for what they provide; no `utils`, `common` or `helpers`. Use `name.rs` beside a
  `name/` directory, not `mod.rs`. Follow the
  [dependency direction](docs/design/architecture.md#dependency-direction); put platform code
  behind `#[cfg(...)]` in dedicated modules.
- Only `cli` parses arguments, reads the environment, writes to stdout or stderr and maps errors
  to exit codes. Other modules take typed input, return `Result` and never print, exit or panic
  on expected failures. `unwrap` and `expect` assert invariants; the `expect` message names it.
- A fallible module exposes its own error enum implementing `std::error::Error`, with the cause
  in `source()`. Callers match variants, never message text; library signatures never return
  `Box<dyn Error>`.
- Borrow in parameters (`&str`, `&[T]`, `&Path`), return owned values, and never `clone` only to
  quiet the borrow checker. No global mutable state: pass writers (`&mut dyn Write`), clocks and
  paths in.
- Stay synchronous until a capability specifies concurrency; then prefer `std::thread::scope` and
  channels, and add an async runtime only through the dependency table.
- Public items carry `///` docs, with `# Errors` and `# Panics` sections where they apply. Fix
  clippy findings; silence one only with `#[expect(lint, reason = "...")]` at the narrowest item.
- Unit tests sit in a `#[cfg(test)] mod tests` at the end of their module; black-box tests in
  `tests/` run the binary through `env!("CARGO_BIN_EXE_<name>")`. Tests are table-driven,
  deterministic and network-free, build fixtures in temporary directories, and assert behavior,
  not incidental implementation details. A bug fix starts with a failing regression test.
- Keep files at about 300 lines or less; split at real seams.

## Task cycle

1. Run `cargo xtask plan-status`, select one eligible task and note the counts. Check its
   dependencies and the records they link. Do not start blocked work.
2. Create the task record from the [template](docs/impl/records/template.md) using the
   [naming rules](docs/guide/planning-workflow.md#record-file-naming), paste the full task text
   and index it. Identify the affected modules and reusable code; do not silently broaden scope.
3. Implement and self-review. Tests cover the happy path, the main edge cases and a regression
   for every bug fixed.
4. Verify with the relevant tests and `cargo xtask ci`. Record failures and unrun checks
   honestly. Fix causes; never weaken a gate. Coverage is diagnostic, never a gate. Failed
   acceptance keeps the task open.
5. Before stopping, update the record with evidence, decisions, audit notes and the next action.
   On acceptance: update the narrow `docs/impl/current/` page and link the record; remove the
   task from the plan and replace references to its id with the record link; mark the capability
   `shipped` when its last task is done. Run `cargo xtask lint-spec-plan` and
   `cargo xtask lint-doc-links`, report the task counts before and after and the next eligible
   task, inspect `git status` and remove temporary files.

## Read when needed

- **Adding or changing capabilities or tasks:** the
  [planning workflow](docs/guide/planning-workflow.md). Specify behavior and evaluation before
  planning or coding.
- **Concerns outside the task, or a checkpoint task:** the
  [audit rules](docs/guide/planning-workflow.md#audit-notes-and-checkpoints). Route each concern
  to exactly one owner.
- **Human-assisted tasks:** agents prepare inputs and report what is needed; they never mark a
  human task done. See [task lanes](docs/guide/planning-workflow.md#task-lanes).
- **Toolchain, commands, CI, releases or test layout:** the
  [development guide](docs/guide/development.md).
