//! Identity of the running build, fixed at compile time from the package manifest.

use std::fmt;

/// Identity of one build of the command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInfo {
    /// Package and command name.
    pub name: &'static str,
    /// Package version from `Cargo.toml`, the single source of the release version.
    pub version: &'static str,
    /// Repository URL from `Cargo.toml`; empty when unset.
    pub repository: &'static str,
    /// Operating system, for example `linux`.
    pub os: &'static str,
    /// CPU architecture, for example `x86_64`.
    pub arch: &'static str,
}

impl BuildInfo {
    /// Returns the identity of this build.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            repository: env!("CARGO_PKG_REPOSITORY"),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        }
    }
}

/// One line, for example `agent-rs 0.1.0 (https://github.com/volod/agent-rs, linux/x86_64)`.
impl fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repository =
            if self.repository.is_empty() { "unknown repository" } else { self.repository };
        let Self { name, version, os, arch, .. } = self;
        write!(f, "{name} {version} ({repository}, {os}/{arch})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_reads_the_manifest() {
        let info = BuildInfo::current();
        assert_eq!(info.name, "agent-rs");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.os.is_empty() && !info.arch.is_empty(), "{info:?}");
    }

    #[test]
    fn display_is_one_line() {
        let mut info = BuildInfo {
            name: "app",
            version: "1.2.3",
            repository: "https://example.com/app",
            os: "linux",
            arch: "x86_64",
        };
        assert_eq!(info.to_string(), "app 1.2.3 (https://example.com/app, linux/x86_64)");
        info.repository = "";
        assert_eq!(info.to_string(), "app 1.2.3 (unknown repository, linux/x86_64)");
    }
}
