# Release Distribution

## Packaging

`cargo xtask dist [--target TRIPLE] [--tag vX.Y.Z]` produces the archive the
[specification](../../design/spec.md#release-distribution) requires.

- `xtask/src/dist.rs` checks `--tag` against the version before building, resolves the host
  target from `rustc -vV` when `--target` is absent, and runs
  `cargo build --release --locked --package agent-rs --target <T>`.
- It stages `dist/agent-rs-<version>-<target>/` with the binary, `README.md` and `LICENSE`,
  archives it with the system `tar` (`-czf` to `.tar.gz`; `-acf` to `.zip` for Windows targets)
  and removes the staging directory. The version is `CARGO_PKG_VERSION`, which `xtask` shares
  with the product through `workspace.package`.
- `[profile.release]` sets `lto = true`, `codegen-units = 1` and `strip = true`.
  `.cargo/config.toml` links the MSVC C runtime statically and uses `rust-lld` for
  `aarch64-unknown-linux-musl`, so both musl targets build on an x86_64 Linux host without extra
  packages.

## Release workflow

`.github/workflows/release.yml` runs on pushed tags matching `v*.*.*`. The `gate` job runs
`cargo xtask ci`. The `build` matrix runs `cargo xtask dist --target <T> --tag <tag>` for
`x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` on `ubuntu-latest`,
`aarch64-apple-darwin` on `macos-latest` and `x86_64-pc-windows-msvc` on `windows-latest`, and
uploads each archive. The `publish` job writes and verifies `SHA256SUMS` and runs
`gh release create <tag> dist/* --verify-tag --generate-notes`. `ci.yml` runs
`cargo xtask dist` for the host on every push and pull request.

## Tests

`xtask/src/dist.rs` covers argument parsing and usage errors, the tag check and host-target
detection. The macOS and Windows archives are built only by the release workflow.
