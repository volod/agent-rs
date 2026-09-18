//! Relative Markdown links and heading anchors must resolve inside the repository.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use crate::markdown::{anchors, is_markdown, link_targets, prose_lines};

/// Directories never scanned for Markdown: build output, vendored code and product samples.
/// Hidden directories other than `.github` are skipped too.
const SKIPPED_DIRS: [&str; 5] = ["target", "dist", "vendor", "node_modules", "testdata"];

/// Returns one problem per broken relative link or anchor in the repository's Markdown files.
///
/// # Errors
///
/// Fails when the tree or a Markdown file cannot be read.
pub fn check(root: &Path) -> io::Result<Vec<String>> {
    let mut docs = BTreeMap::new();
    collect_markdown(root, "", &mut docs)?;
    let anchors: BTreeMap<&str, HashSet<String>> =
        docs.iter().map(|(path, text)| (path.as_str(), anchors(text))).collect();
    let mut problems = Vec::new();
    for (path, text) in &docs {
        for (n, line) in prose_lines(text).into_iter().enumerate() {
            for target in link_targets(line) {
                if let Some(problem) = check_link(root, path, target, &anchors) {
                    problems.push(format!("{path}:{}: {problem}", n + 1));
                }
            }
        }
    }
    Ok(problems)
}

/// Reads every Markdown file below `root/dir` into `docs`, keyed by `/`-separated relative path.
fn collect_markdown(root: &Path, dir: &str, docs: &mut BTreeMap<String, String>) -> io::Result<()> {
    for entry in fs::read_dir(root.join(dir))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if dir.is_empty() { name.clone() } else { format!("{dir}/{name}") };
        if entry.file_type()?.is_dir() {
            let hidden = name.starts_with('.') && name != ".github";
            if !hidden && !SKIPPED_DIRS.contains(&name.as_str()) {
                collect_markdown(root, &path, docs)?;
            }
        } else if is_markdown(&name) {
            let text = fs::read_to_string(entry.path())?;
            docs.insert(path, text);
        }
    }
    Ok(())
}

fn check_link(
    root: &Path,
    from: &str,
    target: &str,
    anchors: &BTreeMap<&str, HashSet<String>>,
) -> Option<String> {
    if target.contains("://") || target.starts_with("mailto:") {
        return None;
    }
    let (file, anchor) = target.split_once('#').unwrap_or((target, ""));
    let resolved = if file.is_empty() {
        from.to_owned()
    } else {
        let Some(file) = percent_decode(file) else {
            return Some(format!("bad link {target:?}"));
        };
        let dir = from.rsplit_once('/').map_or("", |(dir, _)| dir);
        let Some(resolved) = normalize(&format!("{dir}/{file}")) else {
            return Some(format!("link {target:?} leaves the repository"));
        };
        if !root.join(&resolved).exists() {
            return Some(format!("broken link {target:?}"));
        }
        resolved
    };
    // Anchors into files that are not scanned Markdown are not checked.
    let known = anchors.get(resolved.as_str())?;
    (!anchor.is_empty() && !known.contains(anchor)).then(|| format!("missing anchor {target:?}"))
}

/// Resolves `.` and `..` in a `/`-separated path; `None` when it climbs above the root.
fn normalize(path: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

/// Decodes `%XX` escapes; `None` for a malformed escape or invalid UTF-8.
fn percent_decode(s: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(s.len());
    let mut rest = s.as_bytes();
    while let Some((&b, tail)) = rest.split_first() {
        if b == b'%' {
            let hex = std::str::from_utf8(tail.get(..2)?).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &tail[2..];
        } else {
            bytes.push(b);
            rest = tail;
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::TempRepo;

    #[test]
    fn check_reports_broken_links_and_anchors() {
        let repo = TempRepo::with(&[
            (
                "README.md",
                "[ok](docs/a.md#archive----core) [web](https://x.y) [self](#top)\n# Top\n",
            ),
            (
                "docs/a.md",
                "# A\n## Archive -- `core`\n[bad](missing.md)\n[anchor](#nope)\n[up](../../x.md)\n\
                 [space](my%20file.md) [dir](../src)\n```\n[fenced](ignored.md)\n```\n",
            ),
            ("docs/my file.md", "# Spaced\n"),
            ("src/lib.rs", ""),
            (".git/notes.md", "[x](nowhere.md)"),
            ("target/doc.md", "[x](nowhere.md)"),
            ("tests/testdata/out.md", "[x](nowhere.md)"),
        ]);
        let problems = check(repo.root()).expect("check links");
        assert_eq!(
            problems,
            [
                r#"docs/a.md:3: broken link "missing.md""#,
                r##"docs/a.md:4: missing anchor "#nope""##,
                r#"docs/a.md:5: link "../../x.md" leaves the repository"#,
            ]
        );
    }

    #[test]
    fn paths_normalize_and_decode() {
        assert_eq!(
            normalize("docs/impl/../design/spec.md").as_deref(),
            Some("docs/design/spec.md")
        );
        assert_eq!(normalize("/./a//b").as_deref(), Some("a/b"));
        assert_eq!(normalize("a/../../b"), None);
        assert_eq!(percent_decode("my%20file.md").as_deref(), Some("my file.md"));
        assert_eq!(percent_decode("bad%2"), None);
        assert_eq!(percent_decode("bad%zz"), None);
    }
}
