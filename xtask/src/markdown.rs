//! The little Markdown reading the planning and link checks share: fenced code, headings, GitHub
//! anchors, inline links, table rows and backticked ids.

use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Reports whether a file name has the `.md` extension, in any case.
pub fn is_markdown(name: &str) -> bool {
    Path::new(name).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// Returns the lines of `text` with fenced code blocks blanked, so line numbers stay aligned.
pub fn prose_lines(text: &str) -> Vec<&str> {
    let mut in_fence = false;
    text.lines()
        .map(|line| {
            if line.trim_start().starts_with("```") {
                in_fence = !in_fence;
                ""
            } else if in_fence {
                ""
            } else {
                line
            }
        })
        .collect()
}

/// Returns the text of an ATX heading (`## Title`), or `None` for any other line.
pub fn heading(line: &str) -> Option<&str> {
    let level = line.bytes().take_while(|&b| b == b'#').count();
    let rest = &line[level..];
    if !(1..=6).contains(&level) || !rest.starts_with([' ', '\t']) {
        return None;
    }
    let text = rest.trim().trim_end_matches('#').trim_end();
    (!text.is_empty()).then_some(text)
}

/// Converts heading text to its GitHub anchor.
pub fn slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

/// Returns the anchor of every heading in `text`, with GitHub's `-1`, `-2` suffixes for repeats.
pub fn anchors(text: &str) -> HashSet<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut anchors = HashSet::new();
    for heading in prose_lines(text).into_iter().filter_map(heading) {
        let slug = slug(heading);
        let repeats = seen.entry(slug.clone()).or_default();
        anchors.insert(if *repeats == 0 { slug } else { format!("{slug}-{repeats}") });
        *repeats += 1;
    }
    anchors
}

/// Returns the targets of the inline links (`[text](target)`) in one line.
pub fn link_targets(line: &str) -> Vec<&str> {
    let mut targets = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find(']') else { break };
        let Some(tail) = rest[close + 1..].strip_prefix('(') else {
            continue;
        };
        let Some(end) = tail.find(')') else { continue };
        let target = &tail[..end];
        if !target.is_empty() && !target.contains(char::is_whitespace) {
            targets.push(target);
            rest = &tail[end + 1..];
        }
    }
    targets
}

/// Splits a table row into trimmed cells; `None` when the line is not a row.
pub fn table_cells(line: &str) -> Option<Vec<&str>> {
    let inner = line.trim().strip_prefix('|')?;
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    Some(inner.split('|').map(str::trim).collect())
}

/// Returns every backticked id in `text`, in order.
pub fn backticked_ids(text: &str) -> Vec<&str> {
    let parts: Vec<&str> = text.split('`').collect();
    (1..parts.len().saturating_sub(1))
        .step_by(2)
        .map(|i| parts[i])
        .filter(|part| is_id(part))
        .collect()
}

/// Returns the id when `cell` is exactly one backticked id.
pub fn sole_id(cell: &str) -> Option<&str> {
    let id = cell.strip_prefix('`')?.strip_suffix('`')?;
    is_id(id).then_some(id)
}

/// Reports whether `s` is an id: lowercase ASCII letters, digits and inner hyphens.
pub fn is_id(s: &str) -> bool {
    s.bytes().next().is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_matches_github() {
        let cases = [
            ("Project identity -- `project-identity`", "project-identity----project-identity"),
            ("Run lock", "run-lock"),
            ("Video -- Previews (v2.0)!", "video----previews-v20"),
            ("snake_case and CAPS", "snake_case-and-caps"),
        ];
        for (heading, want) in cases {
            assert_eq!(slug(heading), want, "slug({heading:?})");
        }
    }

    #[test]
    fn anchors_skip_code_and_number_repeats() {
        let text = "# Top\n## Notes\n```\n# Not a heading\n```\n## Notes ##\n#NoSpace\n";
        let mut got: Vec<_> = anchors(text).into_iter().collect();
        got.sort();
        assert_eq!(got, ["notes", "notes-1", "top"]);
    }

    #[test]
    fn link_targets_find_inline_links() {
        let line = "[a](x.md) [b [c](y.md#z) ![img](i.png) [t](u \"title\") [r][ref] [e]()";
        assert_eq!(link_targets(line), ["x.md", "y.md#z", "i.png"]);
    }

    #[test]
    fn ids_and_cells() {
        assert_eq!(backticked_ids("`a-1` and `B` and `c` `open"), ["a-1", "c"]);
        assert_eq!(sole_id("`core`"), Some("core"));
        assert_eq!(sole_id("`core` x"), None);
        assert_eq!(table_cells("| a | `b` |"), Some(vec!["a", "`b`"]));
        assert_eq!(table_cells("text"), None);
        assert!(!is_id("-lead") && !is_id("") && is_id("0001-x"));
    }
}
