# agent-rs

A copy-ready Rust project skeleton for teams that want coding agents to work from one set of
rules, one product specification and one forward plan.

A fresh copy builds an `agent-rs` command, passes its quality gates, and has a machine-checked
plan whose first task gives the new repository its own identity. The Rust layout, gates and
planning workflow are meant to stay; the name and the product are yours.

## Quick start

Requirements: [rustup](https://rustup.rs) and Git. The pinned toolchain installs itself on the
first `cargo` command.

```bash
cargo xtask ci           # fmt, clippy, tests, docs, plan and link checks
cargo run -- version     # agent-rs 0.1.0 (https://github.com/volod/agent-rs, linux/x86_64)
cargo xtask plan-status  # next agent task: personalize-template-project
```

## Start a project from the template

1. Create a repository from this template and clone it.
2. Ask your agent to take the next task from `cargo xtask plan-status`. The first one,
   `personalize-template-project`, renames the package, binary and docs; give it the product
   name, repository URL and a one-line description.
3. Describe the product's first capability in the [specification](docs/design/spec.md), then
   plan and implement it as the [planning workflow](docs/guide/planning-workflow.md) describes.

## Daily commands

| Command | Purpose |
| --- | --- |
| `cargo run -- <args>` | Run the command from source |
| `cargo test --workspace` | Run all tests |
| `cargo xtask ci` | Run the required gate (what CI runs) |
| `cargo xtask dist` | Build a release archive for this host into `dist/` |
| `cargo xtask plan-status` | Count open tasks and show the next eligible one |
| `cargo xtask help` | List every repository command |

The [development guide](docs/guide/development.md) lists the `cargo` command behind each one.

## Releases

Set `version` in `Cargo.toml`, merge, then push the matching tag; the release workflow checks,
builds static Linux (x86_64, arm64), macOS (arm64) and Windows (x86_64) archives and publishes
them with `SHA256SUMS`:

```bash
git tag v0.1.0 && git push origin v0.1.0
```

## How work flows

```text
docs/design/spec.md       what the product must do and how each capability is evaluated
        |
docs/impl/plan.md         only work that remains, ordered by the capability registry
        |
docs/impl/records/        one record per task: full scope, decisions, evidence
        |
docs/impl/current.md      what exists now, linking the records that prove it
```

`cargo xtask lint-spec-plan` fails when these documents disagree, and
`cargo xtask lint-doc-links` fails on a broken relative link or anchor.

## Layout

```text
src/main.rs         process wiring: arguments, streams, exit code
src/cli.rs          command-line contract: clap parsing, help, exit codes
src/build_info.rs   build identity from the package manifest
tests/              black-box tests of the built binary
xtask/              dev-only tooling: ci gate, planning and link checks, release archives
docs/               specification, plan, records, current state, guides
```

## Agent support

[AGENTS.md](AGENTS.md) is the only rule file. `CLAUDE.md` and `GEMINI.md` import it, and the
Cursor rule in `.cursor/rules/` points to it; Codex and other agents read `AGENTS.md` directly.

## License

MIT. See [LICENSE](LICENSE).
