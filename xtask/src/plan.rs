//! The planning documents: the capability registry in the design specification, the forward plan
//! and the task records. [`load`] reads them, [`lint()`] checks that they agree and [`Status`]
//! summarizes open work. The formats are defined in `docs/guide/planning-workflow.md`.

mod lint;
mod parse;
mod status;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use crate::markdown::{backticked_ids, is_markdown};

pub use lint::lint;
pub use parse::{parse_group_table, parse_plan, parse_record_name, parse_registry};
pub use status::Status;

/// Design specification holding the capability registry.
pub const SPEC: &str = "docs/design/spec.md";
/// Forward plan.
pub const PLAN: &str = "docs/impl/plan.md";
/// Planning workflow holding the record group table.
pub const WORKFLOW: &str = "docs/guide/planning-workflow.md";
/// Task records and their index.
pub const RECORDS: &str = "docs/impl/records";

/// Files in the records directory that are not task records.
const NON_RECORDS: [&str; 2] = ["README.md", "template.md"];

/// A top-level section of the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lane {
    Agent,
    Human,
}

impl Lane {
    fn from_heading(line: &str) -> Option<Self> {
        match line.trim() {
            "## Agent Implementation Tasks" => Some(Self::Agent),
            "## Human-Assisted Tasks" => Some(Self::Human),
            _ => None,
        }
    }

    /// Fields every task of the lane carries, in workflow order.
    fn required_fields(self) -> &'static [&'static str] {
        match self {
            Self::Agent => &[
                "Serves",
                "Agent status",
                "Dependencies",
                "User-visible outcome",
                "Scope boundary",
                "Data and artifact paths",
                "Execution path",
                "Acceptance gates",
                "Documentation target",
                "Review checkpoint",
            ],
            Self::Human => &[
                "Serves",
                "Human status",
                "Dependencies",
                "Requested input or decision",
                "Unblocks",
            ],
        }
    }

    fn status_field(self) -> &'static str {
        match self {
            Self::Agent => "Agent status",
            Self::Human => "Human status",
        }
    }

    fn statuses(self) -> &'static [&'static str] {
        match self {
            Self::Agent => &["CLEAR", "RUN NEEDED"],
            Self::Human => &["HUMAN-GATED", "BLOCKED BY HUMAN"],
        }
    }
}

impl fmt::Display for Lane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Agent => "agent",
            Self::Human => "human",
        })
    }
}

/// One `#### task-id` block of the plan.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    /// Capability of the enclosing group heading; empty outside a group.
    pub capability: String,
    pub lane: Lane,
    pub line: usize,
    pub fields: BTreeMap<String, String>,
}

impl Task {
    /// Returns a field value, or `""` when the field is missing.
    pub fn field(&self, name: &str) -> &str {
        self.fields.get(name).map_or("", String::as_str)
    }

    /// Backticked task ids in the `Dependencies` field; record links are not ids.
    pub fn dependencies(&self) -> Vec<&str> {
        backticked_ids(self.field("Dependencies"))
    }
}

/// One ``### Title -- `capability` `` heading inside a lane.
#[derive(Debug)]
pub struct Group {
    pub capability: String,
    pub lane: Lane,
    pub line: usize,
}

/// The parsed forward plan.
#[derive(Debug, Default)]
pub struct Plan {
    pub groups: Vec<Group>,
    pub tasks: Vec<Task>,
}

/// One row of the capability registry.
#[derive(Debug)]
pub struct Capability {
    pub id: String,
    pub status: String,
    pub implementation: String,
}

/// One task record file, named `NNNN-<group>-<task-id>.md`.
#[derive(Debug)]
pub struct Record {
    pub file: String,
    pub sequence: String,
    pub task_id: String,
    pub body: String,
}

/// Everything a lint run reads.
#[derive(Debug)]
pub struct Inputs {
    pub registry: Vec<Capability>,
    pub plan: Plan,
    /// Capability id to record group abbreviation.
    pub groups: BTreeMap<String, String>,
    pub records: Vec<Record>,
    /// Text of the records README.
    pub record_index: String,
}

/// Reads the planning documents under `root`. Badly named records are returned as problems.
///
/// # Errors
///
/// Fails when a planning document or the records directory cannot be read.
pub fn load(root: &Path) -> io::Result<(Inputs, Vec<String>)> {
    let read = |path: &str| {
        fs::read_to_string(root.join(path))
            .map_err(|err| io::Error::new(err.kind(), format!("read {path}: {err}")))
    };
    let mut inputs = Inputs {
        registry: parse_registry(&read(SPEC)?),
        plan: parse_plan(&read(PLAN)?),
        groups: parse_group_table(&read(WORKFLOW)?),
        records: Vec::new(),
        record_index: read(&format!("{RECORDS}/README.md"))?,
    };
    let mut problems = Vec::new();
    for entry in fs::read_dir(root.join(RECORDS))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !entry.file_type()?.is_file()
            || !is_markdown(&name)
            || NON_RECORDS.contains(&name.as_str())
        {
            continue;
        }
        match parse_record_name(&name, &inputs.groups) {
            Ok(mut record) => {
                record.body = read(&format!("{RECORDS}/{name}"))?;
                inputs.records.push(record);
            }
            Err(problem) => problems.push(problem),
        }
    }
    inputs.records.sort_by(|a, b| a.file.cmp(&b.file));
    problems.sort();
    Ok((inputs, problems))
}
