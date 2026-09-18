//! Release archive for one target: `dist/<app>-<version>-<target>.tar.gz`, or `.zip` for Windows,
//! holding one directory of the same name with the release binary, `README.md` and `LICENSE`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{Error, cargo, run_command};

/// The shipped package and binary.
const APP: &str = "agent-rs";
/// Product version, shared with xtask through `workspace.package`.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Files packed next to the binary.
const FILES: [&str; 2] = ["README.md", "LICENSE"];
/// Output directory under the repository root.
const OUT_DIR: &str = "dist";

/// Arguments of `cargo xtask dist`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Target triple; the host when unset.
    pub target: Option<String>,
    /// Release tag that must equal `v<version>`.
    pub tag: Option<String>,
}

impl Options {
    /// Parses `[--target TRIPLE] [--tag vX.Y.Z]`.
    pub fn parse(args: &[String]) -> Result<Self, Error> {
        let mut options = Self::default();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            let slot = match arg.as_str() {
                "--target" => &mut options.target,
                "--tag" => &mut options.tag,
                other => return Err(Error::Usage(format!("dist: unknown argument {other:?}"))),
            };
            let value =
                args.next().ok_or_else(|| Error::Usage(format!("dist: {arg} needs a value")))?;
            *slot = Some(value.clone());
        }
        Ok(options)
    }
}

/// Builds the release binary and writes its archive; returns the archive path.
pub fn dist(root: &Path, options: &Options) -> Result<PathBuf, Error> {
    check_tag(options.tag.as_deref())?;
    let target = match &options.target {
        Some(target) => target.clone(),
        None => host_target(root)?,
    };
    cargo(root, &["build", "--release", "--locked", "--package", APP, "--target", &target], &[])?;

    let windows = target.contains("windows");
    let exe = if windows { format!("{APP}.exe") } else { APP.to_owned() };
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| root.join("target"), |dir| root.join(dir));
    let name = format!("{APP}-{VERSION}-{target}");
    let out = root.join(OUT_DIR);
    let stage = out.join(&name);
    if stage.exists() {
        fs::remove_dir_all(&stage).map_err(Error::io(&stage))?;
    }
    fs::create_dir_all(&stage).map_err(Error::io(&stage))?;
    let binary = target_dir.join(&target).join("release").join(&exe);
    fs::copy(&binary, stage.join(&exe)).map_err(Error::io(&binary))?;
    for file in FILES {
        fs::copy(root.join(file), stage.join(file)).map_err(Error::io(root.join(file)))?;
    }

    // `tar` ships with Linux, macOS and Windows 10+; the Windows one (bsdtar) writes zip with -a.
    let (archive, flags) =
        if windows { (format!("{name}.zip"), "-acf") } else { (format!("{name}.tar.gz"), "-czf") };
    run_command(Command::new("tar").args([flags, &archive, &name]).current_dir(&out))?;
    fs::remove_dir_all(&stage).map_err(Error::io(&stage))?;
    Ok(out.join(archive))
}

fn check_tag(tag: Option<&str>) -> Result<(), Error> {
    match tag {
        Some(tag) if tag != format!("v{VERSION}") => {
            Err(Error::Failed(format!("tag {tag} does not match version {VERSION} in Cargo.toml")))
        }
        _ => Ok(()),
    }
}

/// Returns the host target triple reported by `rustc -vV`.
fn host_target(root: &Path) -> Result<String, Error> {
    let output = Command::new("rustc")
        .arg("-vV")
        .current_dir(root)
        .output()
        .map_err(|err| Error::Failed(format!("run rustc: {err}")))?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| Error::Failed("rustc -vV reported no host target".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|&arg| arg.to_owned()).collect()
    }

    #[test]
    fn options_parse() {
        let parsed =
            Options::parse(&args(&["--target", "x86_64-pc-windows-msvc", "--tag", "v1.0.0"]))
                .expect("valid arguments");
        assert_eq!(parsed.target.as_deref(), Some("x86_64-pc-windows-msvc"));
        assert_eq!(parsed.tag.as_deref(), Some("v1.0.0"));
        assert_eq!(Options::parse(&[]).expect("no arguments"), Options::default());
        for bad in [&["--target"][..], &["--bogus", "x"]] {
            assert!(matches!(Options::parse(&args(bad)), Err(Error::Usage(_))), "{bad:?}");
        }
    }

    #[test]
    fn tag_must_match_the_version() {
        assert!(check_tag(None).is_ok());
        assert!(check_tag(Some(&format!("v{VERSION}"))).is_ok());
        assert!(matches!(check_tag(Some(VERSION)), Err(Error::Failed(_))));
        assert!(matches!(check_tag(Some("v999.0.0")), Err(Error::Failed(_))));
    }

    #[test]
    fn host_target_is_a_triple() {
        let target = host_target(&crate::repository_root()).expect("rustc reports its host");
        assert!(target.split('-').count() >= 3, "{target}");
    }
}
