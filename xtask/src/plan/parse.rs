//! Parsers for the planning documents. The formats are defined in the planning workflow; these
//! functions never fail, and whatever they cannot parse is left for `lint` to report.

use std::collections::BTreeMap;

use super::{Capability, Group, Lane, Plan, Record, Task};
use crate::markdown::{backticked_ids, is_id, sole_id, table_cells};

/// Parses `plan.md`. Field values may continue on indented lines.
pub fn parse_plan(text: &str) -> Plan {
    let mut plan = Plan::default();
    let mut lane = None;
    let mut capability = String::new();
    let mut task: Option<Task> = None;
    let mut field: Option<String> = None;
    for (i, line) in text.lines().enumerate() {
        if ["## ", "### ", "#### "].iter().any(|prefix| line.starts_with(prefix)) {
            plan.tasks.extend(task.take());
            field = None;
        }
        if line.starts_with("## ") {
            lane = Lane::from_heading(line);
            capability.clear();
        } else if let Some(rest) = line.strip_prefix("### ") {
            capability.clear();
            if let (Some(lane), Some(id)) = (lane, group_capability(rest)) {
                id.clone_into(&mut capability);
                plan.groups.push(Group { capability: capability.clone(), lane, line: i + 1 });
            }
        } else if let Some(rest) = line.strip_prefix("#### ") {
            if let (Some(lane), Some(id)) = (lane, task_id(rest)) {
                let (id, capability) = (id.to_owned(), capability.clone());
                task = Some(Task { id, capability, lane, line: i + 1, fields: BTreeMap::new() });
            }
        } else if let Some(task) = task.as_mut() {
            if let Some((name, value)) = field_line(line) {
                task.fields.insert(name.to_owned(), value.to_owned());
                field = Some(name.to_owned());
            } else if line.trim().is_empty() {
                field = None;
            } else if let Some(value) = field.as_ref().and_then(|f| task.fields.get_mut(f)) {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(line.trim());
            }
        }
    }
    plan.tasks.extend(task);
    plan
}

/// Parses the `## Capability Registry` table of the specification. The id is the first cell that
/// is exactly one backticked id; status and implementation cells are found by their headings.
pub fn parse_registry(text: &str) -> Vec<Capability> {
    let mut registry = Vec::new();
    let mut in_registry = false;
    let mut columns: Option<(Option<usize>, Option<usize>)> = None;
    for line in text.lines() {
        if line.starts_with("## ") {
            in_registry = line.trim() == "## Capability Registry";
            continue;
        }
        let Some(cells) = table_cells(line).filter(|_| in_registry) else {
            continue;
        };
        let Some((status, implementation)) = columns else {
            let column = |name| cells.iter().position(|cell| *cell == name);
            columns = Some((column("Status"), column("Implementation")));
            continue;
        };
        let cell = |i: Option<usize>| i.and_then(|i| cells.get(i)).map_or("", |cell| *cell);
        if let Some(id) = cells.iter().find_map(|cell| sole_id(cell)) {
            registry.push(Capability {
                id: id.to_owned(),
                status: cell(status).to_owned(),
                implementation: cell(implementation).to_owned(),
            });
        }
    }
    registry
}

/// Parses the capability-to-record-group table of the planning workflow: two-cell rows whose
/// cells each hold a backticked id.
pub fn parse_group_table(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(table_cells)
        .filter_map(|cells| match cells.as_slice() {
            [capability, group] => Some((
                (*backticked_ids(capability).first()?).to_owned(),
                (*backticked_ids(group).first()?).to_owned(),
            )),
            _ => None,
        })
        .collect()
}

/// Splits `NNNN-<group>-<task-id>.md` using the known group abbreviations, longest first.
///
/// # Errors
///
/// Returns the lint problem for a malformed name or an unknown group.
pub fn parse_record_name(name: &str, groups: &BTreeMap<String, String>) -> Result<Record, String> {
    let malformed = || format!("record {name}: name must be NNNN-<group>-<task-id>.md");
    let stem = name.strip_suffix(".md").ok_or_else(malformed)?;
    let (sequence, rest) = stem.split_at_checked(4).ok_or_else(malformed)?;
    let rest = rest.strip_prefix('-').filter(|rest| is_id(rest)).ok_or_else(malformed)?;
    if !sequence.bytes().all(|b| b.is_ascii_digit()) {
        return Err(malformed());
    }
    let mut abbrevs: Vec<&str> = groups.values().map(String::as_str).collect();
    abbrevs.sort_by_key(|abbrev| std::cmp::Reverse(abbrev.len()));
    abbrevs
        .into_iter()
        .find_map(|abbrev| rest.strip_prefix(abbrev)?.strip_prefix('-').filter(|id| !id.is_empty()))
        .map(|task_id| Record {
            file: name.to_owned(),
            sequence: sequence.to_owned(),
            task_id: task_id.to_owned(),
            body: String::new(),
        })
        .ok_or_else(|| {
            format!("record {name}: unknown group; add it to the planning workflow table")
        })
}

/// Returns the capability of a ``Title -- `capability` `` group heading.
fn group_capability(heading: &str) -> Option<&str> {
    let (title, id) = heading.trim_end().rsplit_once(" -- ")?;
    sole_id(id).filter(|_| !title.trim().is_empty())
}

/// Returns the id of a `task-id` or `task-id (optional)` task heading.
fn task_id(heading: &str) -> Option<&str> {
    let heading = heading.trim_end();
    let id = match heading.strip_suffix("(optional)") {
        Some(id) if id.ends_with(char::is_whitespace) => id.trim_end(),
        Some(_) => return None,
        None => heading,
    };
    is_id(id).then_some(id)
}

/// Splits a `- Field name: value` line.
fn field_line(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.strip_prefix("- ")?.split_once(':')?;
    let valid = name.starts_with(|c: char| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '-');
    valid.then(|| (name, value.trim()))
}
