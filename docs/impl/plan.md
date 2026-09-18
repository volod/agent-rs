# agent-rs Implementation Plan

Forward-only: this file holds only work that remains. Product behavior, boundaries and
evaluation belong in the [specification](../design/spec.md); task shape, statuses, ordering and
records in the [planning workflow](../guide/planning-workflow.md); available behavior in
[current state](current.md). Run `cargo xtask plan-status` for counts and the next eligible task.

Every task keeps `cargo xtask ci` green, adds deterministic, network-free tests, and adds each
new crate to the [dependency table](../design/spec.md#dependencies) in the same change.

## Agent Implementation Tasks

### Project identity -- `project-identity`

#### personalize-template-project

Give a repository created from this template its own identity before any product work starts.

- Serves: `project-identity` -- [Project identity](../design/spec.md#project-identity)
- Agent status: CLEAR
- Dependencies: none.
- User-visible outcome: The package, binary, README and specification name and describe the new
  product; `<name> version` prints the new name and repository URL.
- Scope boundary: Rename the package, `description` and `repository` in `Cargo.toml`, the
  library path in `src/main.rs`, `APP` in `xtask/src/dist.rs`, `CARGO_BIN_EXE_agent-rs` and the
  expected names in tests. Rewrite the README and the purpose, scope and identity sections of
  the specification. Keep the version, planning tooling, gates and documentation lifecycle. Do
  not add capabilities, crates or architecture.
- Data and artifact paths: `Cargo.toml`, `Cargo.lock`, `src/`, `tests/`, `xtask/src/dist.rs`,
  `README.md`, `AGENTS.md`, `docs/`.
- Execution path: Take the product name, repository URL and one-line description from the owner,
  or derive the URL from `git remote get-url origin`. Apply the renames, run `cargo check` to
  refresh `Cargo.lock`, then `cargo xtask ci` and `cargo run -- version`.
- Acceptance gates: `git grep -n -e agent-rs -e agent_rs` finds no reference that denotes the
  active project; `cargo xtask ci` passes; `cargo run -- version` prints the new name and
  repository URL; `project-identity` is `shipped` with a current-state page. A negative result
  names the missing owner input instead of inventing it.
- Documentation target: new `docs/impl/current/project-identity.md`, linked from
  [current state](current.md) and the registry.
- Review checkpoint: none.

## Human-Assisted Tasks

None open. Add a task here when acceptance needs human judgment, authorization, private access or
spending authority.
